use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use models::Direction;
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{mpsc, oneshot, watch};

use order_book::{Bid, Offer, OrderBook, OrderRequest, Trade, TradeLeg};

use crate::{
    game::delivery_period::DeliveryPeriodId,
    player::{PlayerConnection, PlayerMessage},
};

pub mod models;
pub mod order_book;

#[derive(Debug, Clone, Serialize)]
pub struct OrderRepr {
    pub order_id: String,
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub created_at: DateTime<Utc>,
    pub owned: bool,
}

impl OrderRepr {
    fn from_offer(offer: &Offer, player_id: Option<&String>) -> Self {
        OrderRepr {
            order_id: offer.0.id.clone(),
            direction: offer.0.direction.clone(),
            price: offer.0.price,
            volume: offer.0.volume,
            created_at: offer.0.timestamp,
            owned: player_id.map(|id| *id == offer.0.owner).unwrap_or(false),
        }
    }
    fn from_bid(bid: &Bid, player_id: Option<&String>) -> Self {
        OrderRepr {
            order_id: bid.0.id.clone(),
            direction: bid.0.direction.clone(),
            price: bid.0.price,
            volume: bid.0.volume,
            created_at: bid.0.timestamp,
            owned: player_id.map(|id| *id == bid.0.owner).unwrap_or(false),
        }
    }
}

#[derive(Debug)]
pub enum MarketMessage {
    OpenMarket(DeliveryPeriodId),
    CloseMarket {
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<Vec<Trade>>,
    },
    OrderRequest(OrderRequest),
    OrderDeletionRequest {
        order_id: String,
    },
    NewPlayerConnection(PlayerConnection),
    PlayerDisconnection {
        connection_id: String,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MarketState {
    Open,
    Closed,
}

impl Serialize for MarketState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("MarketState", 2)?;
        state.serialize_field("type", "MarketState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Closed => "Closed",
                Self::Open => "Open",
            },
        )?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct MarketContext {
    pub tx: mpsc::Sender<MarketMessage>,
    pub state_rx: watch::Receiver<MarketState>,
}

pub struct Market {
    state: MarketState,
    state_sender: watch::Sender<MarketState>,
    delivery_period: DeliveryPeriodId,
    rx: mpsc::Receiver<MarketMessage>,
    tx: mpsc::Sender<MarketMessage>,
    order_book: OrderBook,
    players: Vec<PlayerConnection>,
    past_trades: HashMap<DeliveryPeriodId, Vec<Trade>>,
}

impl Market {
    pub fn new(state: MarketState, delivery_period: DeliveryPeriodId) -> Market {
        let (state_tx, _) = watch::channel(state);
        let (tx, rx) = mpsc::channel::<MarketMessage>(128);

        Market {
            state,
            delivery_period,
            state_sender: state_tx,
            rx,
            tx,
            players: Vec::new(),
            order_book: OrderBook::new(),
            past_trades: HashMap::new(),
        }
    }

    pub fn get_context(&self) -> MarketContext {
        MarketContext {
            tx: self.tx.clone(),
            state_rx: self.state_sender.subscribe(),
        }
    }

    pub async fn process(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match (&self.state, message) {
                (_, MarketMessage::NewPlayerConnection(conn)) => {
                    println!("New player: {conn:?}");
                    self.players.push(conn.clone());
                    self.send_order_book_snapshot_to_conn(&conn).await;
                    self.send_trade_list_to_conn(&conn).await
                }
                (MarketState::Open, MarketMessage::OrderRequest(request)) => {
                    self.process_new_offer(request).await
                }
                (MarketState::Open, MarketMessage::OrderDeletionRequest { order_id }) => {
                    println!("Order deletion request for order: {order_id:?}");
                    self.order_book.remove_offer(order_id);
                    self.send_order_book_snapshot_to_all().await;
                }
                (_, MarketMessage::PlayerDisconnection { connection_id }) => {
                    self.players.retain(|conn| conn.id != connection_id);
                }
                (MarketState::Closed, MarketMessage::OpenMarket(period_id)) => {
                    if period_id == self.delivery_period {
                        self.state = MarketState::Open;
                        self.delivery_period = self.delivery_period.next();
                        let _ = self.state_sender.send(MarketState::Open);
                    }
                }
                (MarketState::Open, MarketMessage::CloseMarket { tx_back, period_id }) => {
                    if period_id == self.delivery_period {
                        let trades = self.order_book.drain();
                        self.past_trades.insert(period_id, trades.clone());
                        self.state = MarketState::Closed;
                        let _ = tx_back.send(trades);
                        let _ = self.state_sender.send(MarketState::Closed);
                    }
                }
                (MarketState::Closed, MarketMessage::CloseMarket { period_id, tx_back }) => {
                    if let Some(trades) = self.past_trades.get(&period_id) {
                        let _ = tx_back.send(trades.clone());
                    }
                }
                (state, msg) => {
                    println!("Msg {msg:?} unsupported in state: {state:?}")
                }
            }
        }
    }

    async fn send_to_connection(&self, connection_id: &str, message: PlayerMessage) {
        join_all(
            self.players
                .iter()
                .filter(|conn| conn.id == *connection_id)
                .map(|conn| conn.tx.send(message.clone()))
                .collect::<Vec<_>>(),
        )
        .await;
    }

    async fn send_to_player(&self, player_id: String, message: PlayerMessage) {
        join_all(
            self.players
                .iter()
                .filter(|conn| conn.player_id == *player_id)
                .map(|conn| conn.tx.send(message.clone()))
                .collect::<Vec<_>>(),
        )
        .await;
    }

    async fn send_order_book_snapshot_to_all(&self) {
        join_all(
            self.players
                .iter()
                .map(|conn| self.send_order_book_snapshot_to_conn(conn))
                .collect::<Vec<_>>(),
        )
        .await;
    }

    async fn send_order_book_snapshot_to_conn(&self, conn: &PlayerConnection) {
        let snapshot = self.order_book.snapshot();

        let message = PlayerMessage::OrderBookSnapshot {
            bids: snapshot
                .bids
                .iter()
                .map(|bid| OrderRepr::from_bid(bid, Some(&conn.player_id)))
                .collect(),
            offers: snapshot
                .offers
                .iter()
                .map(|offer| OrderRepr::from_offer(offer, Some(&conn.player_id)))
                .collect(),
        };
        let _ = self.send_to_connection(&conn.id, message).await;
    }

    async fn send_trade_list_to_conn(&self, conn: &PlayerConnection) {
        let trade_legs: Vec<TradeLeg> = self
            .order_book
            .trades
            .iter()
            .flat_map(|trade| trade.for_player(&conn.player_id))
            .collect();
        let _ = self
            .send_to_player(
                conn.player_id.clone(),
                PlayerMessage::TradeList { trades: trade_legs },
            )
            .await;
    }

    async fn process_new_offer(&mut self, request: OrderRequest) {
        let trades = self.order_book.register_order_request(request);
        println!("New trades: {trades:?}");

        self.send_order_book_snapshot_to_all().await;

        if !trades.is_empty() {
            join_all(trades.iter().flat_map(|trade| {
                trade.split().map(|leg| {
                    self.send_to_player(leg.owner.clone(), PlayerMessage::NewTrade(leg.clone()))
                })
            }))
            .await;
        }
    }
}

impl Default for Market {
    fn default() -> Self {
        Self::new(MarketState::Open, DeliveryPeriodId::from(0))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    };
    use uuid::Uuid;

    use crate::{
        game::delivery_period::DeliveryPeriodId,
        market::{models::Direction, order_book::OrderRequest, MarketState, PlayerConnection},
    };

    use super::{Market, MarketContext, MarketMessage, PlayerMessage};

    fn start_market_actor() -> MarketContext {
        let mut market = Market::default();
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });
        context
    }

    struct PlayerConn {
        player_id: String,
        rx: Receiver<PlayerMessage>,
    }
    async fn register_player(market_tx: Sender<MarketMessage>) -> PlayerConn {
        let player_id = Uuid::new_v4().to_string();
        let conn_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = channel::<PlayerMessage>(16);
        let _ = market_tx
            .send(MarketMessage::NewPlayerConnection(PlayerConnection {
                id: conn_id.clone(),
                player_id: player_id.clone(),
                tx: tx.clone(),
            }))
            .await;
        // Consume first Order Book snapshot
        let _ = rx.recv().await;
        // Consume first trade list
        let _ = rx.recv().await;

        PlayerConn { player_id, rx }
    }

    #[tokio::test]
    async fn test_register_player() {
        let market = start_market_actor();

        // Register a new player
        let player_id = Uuid::new_v4().to_string();
        let connection_id = Uuid::new_v4().to_string();
        let (tx, mut player_rx) = channel::<PlayerMessage>(1);
        let _ = market
            .tx
            .send(MarketMessage::NewPlayerConnection(PlayerConnection {
                id: connection_id.clone(),
                player_id: player_id.clone(),
                tx: tx.clone(),
            }))
            .await;

        // We shoudl receive an initial snapshot of the current order book
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = player_rx.recv().await
        else {
            unreachable!("Should have received an order book snapshot");
        };

        // We should receive a list of the player's previous trades
        let Some(PlayerMessage::TradeList { trades: _ }) = player_rx.recv().await else {
            unreachable!("Should have received a list of past trades")
        };
    }

    #[tokio::test]
    async fn test_register_another_player() {
        let market = start_market_actor();
        // Register a new player
        let PlayerConn { mut rx, .. } = register_player(market.tx.clone()).await;

        // Registering another player
        register_player(market.tx.clone()).await;

        // It should not send a snapshot to the already connected player(s)
        let Err(_) = rx.try_recv() else {
            unreachable!("Should not have received a message")
        };
    }

    #[tokio::test]
    async fn test_process_request_order() {
        let market = start_market_actor();

        // Register new player to market actor
        let PlayerConn {
            player_id, mut rx, ..
        } = register_player(market.tx.clone()).await;

        // Send order request to market actor
        let order_request = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: 50_00,
            volume: 10,
            owner: player_id.clone(),
        });
        market.tx.clone().send(order_request).await.unwrap();

        // The list of offers has been updated to contains our new offer
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
    }

    #[tokio::test]
    async fn test_match_offers() {
        let market = start_market_actor();

        // Register buyer player to market actor and send BUY order
        let PlayerConn {
            player_id: buyer_id,
            rx: mut rx_buyer,
            ..
        } = register_player(market.tx.clone()).await;
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: buyer_id.to_owned(),
            }))
            .await
            .unwrap();
        // Flush the corresponding OBS
        rx_buyer.recv().await.unwrap();

        // Register seller player to market actor and send SELL order
        let PlayerConn {
            player_id: seller_id,
            rx: mut rx_seller,
            ..
        } = register_player(market.tx.clone()).await;
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: 10,
                price: 50_00,
                owner: seller_id.to_owned(),
            }))
            .await
            .unwrap();

        // The order book snapshot should be empty for both players
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx_buyer.recv().await else {
            unreachable!("Expected PlayerMessage::OrderBookSnapshot")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx_seller.recv().await else {
            unreachable!("Expected PlayerMessage::OrderBookSnapshot")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        // Each player should receive its own trade leg
        let Some(PlayerMessage::NewTrade(trade_buyer)) = rx_buyer.recv().await else {
            unreachable!("Should have received a trade")
        };
        assert_eq!(trade_buyer.direction, Direction::Buy);
        let Some(PlayerMessage::NewTrade(trade_seller)) = rx_seller.recv().await else {
            unreachable!("Should have received a trade")
        };
        assert_eq!(trade_seller.direction, Direction::Sell);
    }

    #[tokio::test]
    async fn same_player_multiple_connections() {
        let market = start_market_actor();

        // register the same player id, over two distincts connections
        let PlayerConn {
            player_id,
            rx: mut rx_1,
            ..
        } = register_player(market.tx.clone()).await;
        let (tx_2, mut rx_2) = channel::<PlayerMessage>(16);
        let _ = market
            .tx
            .send(MarketMessage::NewPlayerConnection(PlayerConnection {
                id: Uuid::new_v4().to_string(),
                player_id: "same_player".to_string(),
                tx: tx_2.clone(),
            }))
            .await;
        // Flush first OBS and trade list messages
        let _ = rx_2.recv().await;
        let _ = rx_2.recv().await;

        // Generate some trades for the player, both connections should received them
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await
            .unwrap();
        let _ = rx_2.recv().await;
        let _ = rx_1.recv().await;
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: "same_player".to_owned(),
            }))
            .await
            .unwrap();

        // First connection: 1 OBS + 1 trade
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = rx_1.recv().await
        else {
            unreachable!("Should have received an OBS")
        };
        let Some(PlayerMessage::NewTrade(_)) = rx_1.recv().await else {
            unreachable!("Should have received a trade")
        };

        // Second connection: 1 OBS + 1 trade
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = rx_2.recv().await
        else {
            unreachable!("Should have received an OBS")
        };
        let Some(PlayerMessage::NewTrade(_)) = rx_2.recv().await else {
            unreachable!("Should have received a trade")
        };
    }

    #[tokio::test]
    async fn test_order_book_snapshot_entry_owner() {
        let market = start_market_actor();

        // Register first player to market actor
        let PlayerConn {
            player_id,
            rx: mut rx_buyer,
            ..
        } = register_player(market.tx.clone()).await;

        // Register second player to market actor
        let PlayerConn {
            rx: mut rx_seller, ..
        } = register_player(market.tx.clone()).await;

        // Send an order with the first player
        let buy_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            volume: 10,
            price: 50_00,
            owner: player_id.to_owned(),
        });
        market.tx.send(buy_order).await.unwrap();

        // OBS for first player says it owns the order
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers: _ }) = rx_buyer.recv().await
        else {
            unreachable!("Should have received an OBS")
        };
        assert!(bids[0].owned);

        // OBS for second player says it does not own the order
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers: _ }) = rx_seller.recv().await
        else {
            unreachable!("Should have received an OBS")
        };
        assert!(!bids[0].owned);
    }

    #[tokio::test]
    async fn test_closed_market_does_not_process_order_request() {
        let mut market = Market::new(MarketState::Closed, DeliveryPeriodId::from(0));
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register a player to market actor
        let PlayerConn {
            player_id, mut rx, ..
        } = register_player(context.tx.clone()).await;

        // Send an OrderRequest to the market
        let _ = context
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;

        // We should not receive an order book snapshot
        tokio::select! {
        _ = rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }

    #[tokio::test]
    async fn test_close_market_and_reopen() {
        let market = start_market_actor();

        // Register player to market actor
        let PlayerConn {
            player_id, mut rx, ..
        } = register_player(market.tx.clone()).await;

        // Close the market
        let (tx_back, _) = oneshot::channel();
        let _ = market
            .tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        // Reopen the market
        let _ = market
            .tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;

        // Send an OrderRequest to the market
        let _ = market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;

        // We should receive an obs
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = rx.recv().await else {
            unreachable!("Should have received an OBS")
        };
    }

    #[tokio::test]
    async fn test_register_player_during_market_closed() {
        let mut market = Market::new(MarketState::Closed, DeliveryPeriodId::from(0));
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register player to market actor while market is closed
        let player_id = Uuid::new_v4().to_string();
        let conn_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = channel::<PlayerMessage>(16);
        let _ = context
            .tx
            .send(MarketMessage::NewPlayerConnection(PlayerConnection {
                id: conn_id.clone(),
                player_id: player_id.clone(),
                tx: tx.clone(),
            }))
            .await;

        // Should still received empty OBS and trade list
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx.recv().await else {
            unreachable!("Should have received an initial OBS")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);
        let Some(PlayerMessage::TradeList { trades }) = rx.recv().await else {
            unreachable!("Should have received an initial trade list")
        };
        assert_eq!(trades.len(), 0);

        // Reopen the market
        let _ = context
            .tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;

        // Send an OrderRequest to the market
        let _ = context
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;

        // We should receive an obs
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = rx.recv().await else {
            unreachable!("Should have received an OBS")
        };
    }

    #[tokio::test]
    async fn test_close_market_receive_trades() {
        let market = start_market_actor();

        // Register player to market actor
        let PlayerConn { player_id, .. } = register_player(market.tx.clone()).await;

        // Make a trade with ourself
        let _ = market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;
        let _ = market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;

        // Close the market and receive the trade list back
        let (tx_back, rx_back) = oneshot::channel();
        let _ = market
            .tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades.len(), 1);
    }

    #[tokio::test]
    async fn test_market_state_watch() {
        let mut market = Market::default();
        let MarketContext {
            tx: market_tx,
            mut state_rx,
        } = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        assert_eq!(*state_rx.borrow(), MarketState::Open);

        // Close the market
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Reopen the market
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);
    }

    #[tokio::test]
    async fn test_try_closing_market_wrong_period_id_does_not_close_it() {
        let mut market = Market::new(MarketState::Open, DeliveryPeriodId::from(1));
        let MarketContext {
            tx: market_tx,
            mut state_rx,
        } = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Try closing the market with period ID greater than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Try closing the market with period ID smaller than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);
    }

    #[tokio::test]
    async fn test_opening_market_wrong_period_id_does_not_open_it() {
        let mut market = Market::new(MarketState::Closed, DeliveryPeriodId::from(1));
        let MarketContext {
            tx: market_tx,
            mut state_rx,
        } = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Try openning the market with period ID greater than the actual one
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(2)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Try closing the market with period ID smaller than the actual one
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);
    }

    #[tokio::test]
    async fn test_open_market_then_close_next_period() {
        let mut market = Market::new(MarketState::Closed, DeliveryPeriodId::from(1));
        let MarketContext {
            tx: market_tx,
            mut state_rx,
        } = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });

        // Open the market
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(1)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Close the market with next period id
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);
    }

    #[tokio::test]
    async fn test_closing_twice_should_return_the_same_trades() {
        let mut market = start_market_actor();

        // Register player to market actor
        let PlayerConn { player_id, .. } = register_player(market.tx.clone()).await;

        // Make a trade with ourself
        let _ = market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;
        let _ = market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: 10,
                price: 50_00,
                owner: player_id.to_owned(),
            }))
            .await;

        // Close the market and receive the trade list back
        let (tx_back, rx_back) = oneshot::channel();
        let _ = market
            .tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades.len(), 1);
        assert!(market.state_rx.changed().await.is_ok());
        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Closed);

        // Close the market again and receive the same trades
        let (tx_back, rx_back) = oneshot::channel();
        let _ = market
            .tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let same_trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades, same_trades);
    }
}
