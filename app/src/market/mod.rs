use std::fmt::Debug;

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use models::Direction;
use serde::Serialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use order_book::{Bid, Offer, OrderBook, OrderRequest, TradeLeg};

use crate::player::{PlayerConnection, PlayerMessage};

pub mod models;
pub mod order_book;

#[derive(Debug, Clone, Serialize)]
pub struct PlayerOrder {
    pub order_id: String,
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub created_at: DateTime<Utc>,
    pub owned: bool,
}

impl PlayerOrder {
    fn from_offer(offer: &Offer, player_id: Option<&String>) -> Self {
        PlayerOrder {
            order_id: offer.0.id.clone(),
            direction: offer.0.direction.clone(),
            price: offer.0.price,
            volume: offer.0.volume,
            created_at: offer.0.timestamp,
            owned: player_id.map(|id| *id == offer.0.owner).unwrap_or(false),
        }
    }
    fn from_bid(bid: &Bid, player_id: Option<&String>) -> Self {
        PlayerOrder {
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
    OrderRequest(OrderRequest),
    OrderDeletionRequest { order_id: String },
    NewPlayerConnection(PlayerConnection),
    PlayerDisconnection { connection_id: String },
}

pub struct Market {
    rx: Receiver<MarketMessage>,
    tx: Sender<MarketMessage>,
    order_book: OrderBook,
    players: Vec<PlayerConnection>,
}

impl Market {
    pub fn new() -> Market {
        let (tx, rx) = channel::<MarketMessage>(128);

        Market {
            rx,
            tx,
            players: Vec::new(),
            order_book: OrderBook::new(),
        }
    }

    pub fn get_tx(&self) -> Sender<MarketMessage> {
        self.tx.clone()
    }

    pub async fn process(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                MarketMessage::NewPlayerConnection(conn) => {
                    println!("New player: {conn:?}");
                    self.players.push(conn.clone());
                    self.send_order_book_snapshot_to_conn(&conn).await;
                    self.send_trade_list_to_conn(&conn).await
                }
                MarketMessage::OrderRequest(request) => self.process_new_offer(request).await,
                MarketMessage::OrderDeletionRequest { order_id } => {
                    println!("Order deletion request for order: {order_id:?}");
                    self.order_book.remove_offer(order_id);
                    self.send_order_book_snapshot_to_all().await;
                }
                MarketMessage::PlayerDisconnection { connection_id } => {
                    self.players.retain(|conn| conn.id != connection_id);
                }
            }
        }
    }

    async fn send_to_players(&self, message: PlayerMessage) {
        for player in self.players.iter() {
            let _ = player.tx.send(message.clone()).await;
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
                .map(|bid| PlayerOrder::from_bid(bid, Some(&conn.player_id)))
                .collect(),
            offers: snapshot
                .offers
                .iter()
                .map(|offer| PlayerOrder::from_offer(offer, Some(&conn.player_id)))
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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use uuid::Uuid;

    use crate::market::{models::Direction, order_book::OrderRequest, PlayerConnection};

    use super::{Market, MarketMessage, PlayerMessage};

    fn start_market_actor() -> Sender<MarketMessage> {
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });
        market_tx
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
        let market_tx = start_market_actor();

        // Register a new player
        let player_id = Uuid::new_v4().to_string();
        let connection_id = Uuid::new_v4().to_string();
        let (tx, mut player_rx) = channel::<PlayerMessage>(1);
        let _ = market_tx
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
        let market_tx = start_market_actor();
        // Register a new player
        let PlayerConn { mut rx, .. } = register_player(market_tx.clone()).await;

        // Registering another player
        register_player(market_tx.clone()).await;

        // It should not send a snapshot to the already connected player(s)
        let Err(_) = rx.try_recv() else {
            unreachable!("Should not have received a message")
        };
    }

    #[tokio::test]
    async fn test_process_request_order() {
        let market_tx = start_market_actor();

        // Register new player to market actor
        let PlayerConn {
            player_id, mut rx, ..
        } = register_player(market_tx.clone()).await;

        // Send order request to market actor
        let order_request = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: 50_00,
            volume: 10,
            owner: player_id.clone(),
        });
        market_tx.clone().send(order_request).await.unwrap();

        // The list of offers has been updated to contains our new offer
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
    }

    #[tokio::test]
    async fn test_match_offers() {
        let market_tx = start_market_actor();

        // Register buyer player to market actor and send BUY order
        let PlayerConn {
            player_id: buyer_id,
            rx: mut rx_buyer,
            ..
        } = register_player(market_tx.clone()).await;
        market_tx
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
        } = register_player(market_tx.clone()).await;
        market_tx
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
        let market_tx = start_market_actor();

        // register the same player id, over two distincts connections
        let PlayerConn {
            player_id,
            rx: mut rx_1,
            ..
        } = register_player(market_tx.clone()).await;
        let (tx_2, mut rx_2) = channel::<PlayerMessage>(16);
        let _ = market_tx
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
        market_tx
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
        market_tx
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
        let market_tx = start_market_actor();

        // Register first player to market actor
        let PlayerConn {
            player_id,
            rx: mut rx_buyer,
            ..
        } = register_player(market_tx.clone()).await;

        // Register second player to market actor
        let PlayerConn {
            rx: mut rx_seller, ..
        } = register_player(market_tx.clone()).await;

        // Send an order with the first player
        let buy_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            volume: 10,
            price: 50_00,
            owner: player_id.to_owned(),
        });
        market_tx.send(buy_order).await.unwrap();

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
}
