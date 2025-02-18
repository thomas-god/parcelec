use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{mpsc, oneshot, watch};

use order_book::{Bid, Offer, OrderBook, OrderRequest, Trade, TradeLeg};

use crate::{
    game::{delivery_period::DeliveryPeriodId, game_repository::GameId},
    player::{connection::PlayerMessage, repository::ConnectionRepositoryMessage, PlayerId},
};

pub mod models;
pub mod order_book;

pub use models::{Direction, Market, MarketService};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OrderRepr {
    pub order_id: String,
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub created_at: DateTime<Utc>,
    pub owned: bool,
}

impl OrderRepr {
    fn from_offer(offer: &Offer, player_id: Option<&PlayerId>) -> Self {
        OrderRepr {
            order_id: offer.0.id.clone(),
            direction: offer.0.direction.clone(),
            price: offer.0.price,
            volume: offer.0.volume,
            created_at: offer.0.timestamp,
            owned: player_id.map(|id| *id == offer.0.owner).unwrap_or(false),
        }
    }
    fn from_bid(bid: &Bid, player_id: Option<&PlayerId>) -> Self {
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
pub struct OBS {
    pub bids: Vec<OrderRepr>,
    pub offers: Vec<OrderRepr>,
}

#[derive(Debug)]
pub enum MarketMessage {
    GetMarketSnapshot {
        player_id: PlayerId,
        tx_back: oneshot::Sender<(Vec<TradeLeg>, OBS)>,
    },
    OpenMarket(DeliveryPeriodId),
    CloseMarket {
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<Vec<Trade>>,
    },
    OrderRequest(OrderRequest),
    OrderDeletionRequest {
        order_id: String,
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

pub struct MarketActor {
    game_id: GameId,
    delivery_period: DeliveryPeriodId,
    state: MarketState,
    state_sender: watch::Sender<MarketState>,
    rx: mpsc::Receiver<MarketMessage>,
    tx: mpsc::Sender<MarketMessage>,
    order_book: OrderBook,
    players: Vec<PlayerId>,
    players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    past_trades: HashMap<DeliveryPeriodId, Vec<Trade>>,
}

impl MarketActor {
    pub fn new(
        game_id: GameId,
        state: MarketState,
        delivery_period: DeliveryPeriodId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> MarketActor {
        let (state_tx, _) = watch::channel(state);
        let (tx, rx) = mpsc::channel::<MarketMessage>(128);

        MarketActor {
            game_id,
            state,
            delivery_period,
            state_sender: state_tx,
            rx,
            tx,
            players: Vec::new(),
            players_connections,
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
                (_, MarketMessage::GetMarketSnapshot { player_id, tx_back }) => {
                    println!("New player: {player_id:?}");
                    self.players.push(player_id.clone());
                    let _ =
                        tx_back.send((self.player_trades(&player_id), self.player_obs(&player_id)));
                }
                (MarketState::Open, MarketMessage::OrderRequest(request)) => {
                    self.process_new_offer(request).await
                }
                (MarketState::Open, MarketMessage::OrderDeletionRequest { order_id }) => {
                    println!("Order deletion request for order: {order_id:?}");
                    self.order_book.remove_offer(order_id);
                    self.send_order_book_snapshot_to_all().await;
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
                        println!("Closing market for period: {period_id:?}");
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

    async fn send_order_book_snapshot_to_all(&self) {
        join_all(
            self.players
                .iter()
                .map(|player_id| self.send_order_book_snapshot_to_player(player_id))
                .collect::<Vec<_>>(),
        )
        .await;
    }

    fn player_obs(&self, player_id: &PlayerId) -> OBS {
        let snapshot = self.order_book.snapshot();
        OBS {
            bids: snapshot
                .bids
                .iter()
                .map(|bid| OrderRepr::from_bid(bid, Some(player_id)))
                .collect(),
            offers: snapshot
                .offers
                .iter()
                .map(|offer| OrderRepr::from_offer(offer, Some(player_id)))
                .collect(),
        }
    }

    async fn send_order_book_snapshot_to_player(&self, player_id: &PlayerId) {
        let obs = self.player_obs(player_id);
        let message = PlayerMessage::OrderBookSnapshot {
            bids: obs.bids,
            offers: obs.offers,
        };
        let _ = self
            .players_connections
            .send(ConnectionRepositoryMessage::SendToPlayer(
                self.game_id.clone(),
                player_id.clone(),
                message,
            ))
            .await;
    }

    fn player_trades(&self, player_id: &PlayerId) -> Vec<TradeLeg> {
        self.order_book
            .trades
            .iter()
            .flat_map(|trade| trade.for_player(player_id))
            .collect()
    }

    async fn process_new_offer(&mut self, request: OrderRequest) {
        let trades = self.order_book.register_order_request(request);
        println!("New trades: {trades:?}");

        self.send_order_book_snapshot_to_all().await;

        if !trades.is_empty() {
            join_all(trades.iter().flat_map(|trade| {
                trade.split().map(|leg| {
                    self.players_connections
                        .send(ConnectionRepositoryMessage::SendToPlayer(
                            self.game_id.clone(),
                            leg.owner.clone(),
                            PlayerMessage::NewTrade(leg.clone()),
                        ))
                })
            }))
            .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{
        mpsc::{self},
        oneshot,
    };

    use crate::{
        game::{delivery_period::DeliveryPeriodId, game_repository::GameId},
        market::{models::Direction, order_book::OrderRequest, MarketState},
        player::{repository::ConnectionRepositoryMessage, PlayerId},
    };

    use super::{MarketActor, MarketContext, MarketMessage, PlayerMessage};

    fn start_market_actor(
        game_id: &GameId,
        connections: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> MarketContext {
        let mut market = MarketActor::new(
            game_id.clone(),
            MarketState::Open,
            DeliveryPeriodId::from(0),
            connections,
        );
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });
        context
    }

    async fn register_player(market: mpsc::Sender<MarketMessage>) -> PlayerId {
        let player_id = PlayerId::default();
        let (tx, _) = oneshot::channel();
        let _ = market
            .send(MarketMessage::GetMarketSnapshot {
                player_id: player_id.clone(),
                tx_back: tx,
            })
            .await;
        player_id
    }

    #[tokio::test]
    async fn test_process_request_order() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let market = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(market.tx.clone()).await;

        // Send order request to market actor
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                price: 50_00,
                volume: 10,
                owner: player_id.clone(),
            }))
            .await
            .unwrap();

        // The list of offers has been updated to contains our new offer (that we own)
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            target_game_id,
            target_player_id,
            PlayerMessage::OrderBookSnapshot { bids, offers },
        )) = conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(target_game_id, game_id);
        assert_eq!(target_player_id, player_id);
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
        assert!(bids.first().unwrap().owned);
    }

    #[tokio::test]
    async fn test_process_delete_order() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let market = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(market.tx.clone()).await;

        // Send order request to market actor
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                price: 50_00,
                volume: 10,
                owner: player_id.clone(),
            }))
            .await
            .unwrap();

        // The list of offers has been updated to contains our new offer
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::OrderBookSnapshot { bids, offers },
        )) = conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
        let order_id = bids.first().unwrap().order_id.clone();

        // Send request to delete order
        market
            .tx
            .send(MarketMessage::OrderDeletionRequest { order_id })
            .await
            .unwrap();
        // The list of offers should be empty
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::OrderBookSnapshot { bids, offers },
        )) = conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);
    }

    #[tokio::test]
    async fn test_match_offers() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let market = start_market_actor(&game_id, conn_tx.clone());
        let buyer_id = register_player(market.tx.clone()).await;
        let seller_id = register_player(market.tx.clone()).await;

        // Send BUY order
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
                owner: buyer_id.clone(),
            }))
            .await
            .unwrap();
        // Flush the corresponding OBS updates (1 per player)
        conn_rx.recv().await.unwrap();
        conn_rx.recv().await.unwrap();

        // Send SELL order
        market
            .tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: 10,
                price: 50_00,
                owner: seller_id.clone(),
            }))
            .await
            .unwrap();

        // The order book snapshots should be empty
        for _ in 0..2 {
            let Some(ConnectionRepositoryMessage::SendToPlayer(
                target_game_id,
                _,
                PlayerMessage::OrderBookSnapshot { bids, offers },
            )) = conn_rx.recv().await
            else {
                unreachable!("Expected PlayerMessage::PublicOffers")
            };
            assert_eq!(target_game_id, game_id);
            assert_eq!(bids.len(), 0);
            assert_eq!(offers.len(), 0);
        }

        // Each player should receive its own trade leg
        for _ in 0..2 {
            let Some(ConnectionRepositoryMessage::SendToPlayer(
                _,
                player_id,
                PlayerMessage::NewTrade(trade),
            )) = conn_rx.recv().await
            else {
                unreachable!("Expected PlayerMessage::NewTrade")
            };
            if player_id == buyer_id {
                assert_eq!(trade.direction, Direction::Buy)
            } else if player_id == seller_id {
                assert_eq!(trade.direction, Direction::Sell)
            } else {
                unreachable!()
            }
        }
    }

    #[tokio::test]
    async fn test_closed_market_does_not_process_order_request() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(0),
            conn_tx,
        );
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });
        let player_id = register_player(context.tx.clone()).await;

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
        _ = conn_rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }

    #[tokio::test]
    async fn test_close_market_and_reopen() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let market = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(market.tx.clone()).await;

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
        let Some(ConnectionRepositoryMessage::SendToPlayer(..)) = conn_rx.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
    }

    #[tokio::test]
    async fn test_register_player_during_market_closed() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(0),
            conn_tx,
        );
        let context = market.get_context();
        tokio::spawn(async move {
            market.process().await;
        });
        let player_id = register_player(context.tx.clone()).await;

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
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::OrderBookSnapshot { .. },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received an OBS")
        };
    }

    #[tokio::test]
    async fn test_close_market_receive_trades() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let market = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(market.tx.clone()).await;

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
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let mut market = start_market_actor(&game_id, conn_tx.clone());
        register_player(market.tx.clone()).await;

        assert_eq!(*market.state_rx.borrow(), MarketState::Open);

        // Close the market
        let (tx_back, _) = oneshot::channel();
        let _ = market
            .tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        assert!(market.state_rx.changed().await.is_ok());
        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Closed);

        // Reopen the market
        let _ = market
            .tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;
        assert!(market.state_rx.changed().await.is_ok());
        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Open);
    }

    #[tokio::test]
    async fn test_try_closing_market_wrong_period_id_does_not_close_it() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let mut market = MarketActor::new(
            game_id,
            MarketState::Open,
            DeliveryPeriodId::from(1),
            conn_tx,
        );
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
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
        );
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
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
        );
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
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let mut market = start_market_actor(&game_id, conn_tx);
        let player_id = register_player(market.tx.clone()).await;

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

#[cfg(test)]
mod test_order_repr {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        market::{order_book::Bid, OrderRepr},
        player::PlayerId,
    };

    use super::{
        models::Direction,
        order_book::{Offer, Order},
    };

    #[test]
    fn test_order_repr_ownership_from_offer() {
        let offer = Offer(Order {
            direction: Direction::Sell,
            id: Uuid::new_v4().to_string(),
            owner: PlayerId::from("toto"),
            price: 10_00,
            timestamp: Utc::now(),
            volume: 100,
        });

        assert!(!OrderRepr::from_offer(&offer, None).owned);
        assert!(OrderRepr::from_offer(&offer, Some(&PlayerId::from("toto"))).owned);
        assert!(!OrderRepr::from_offer(&offer, Some(&PlayerId::from("not_toto"))).owned);
    }

    #[test]
    fn test_order_repr_ownership_from_bid() {
        let bid = Bid(Order {
            direction: Direction::Buy,
            id: Uuid::new_v4().to_string(),
            owner: PlayerId::from("toto"),
            price: 10_00,
            timestamp: Utc::now(),
            volume: 100,
        });

        assert!(!OrderRepr::from_bid(&bid, None).owned);
        assert!(OrderRepr::from_bid(&bid, Some(&PlayerId::from("toto"))).owned);
        assert!(!OrderRepr::from_bid(&bid, Some(&PlayerId::from("not_toto"))).owned);
    }
}

#[cfg(test)]
mod test_market_state {
    use crate::market::MarketState;

    #[test]
    fn test_game_state_serialize() {
        assert_eq!(
            serde_json::to_string(&MarketState::Open).unwrap(),
            "{\"type\":\"MarketState\",\"state\":\"Open\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&MarketState::Closed).unwrap(),
            "{\"type\":\"MarketState\",\"state\":\"Closed\"}".to_string()
        );
    }
}
