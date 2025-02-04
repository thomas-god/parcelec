use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::order_book::{Bid, Direction, Offer, OrderBook, OrderRequest, TradeLeg};

#[derive(Debug, Clone, Serialize)]
pub struct PublicOrder {
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub created_at: DateTime<Utc>,
}

impl From<&Bid> for PublicOrder {
    fn from(bid: &Bid) -> Self {
        PublicOrder {
            direction: bid.0.direction.clone(),
            price: bid.0.price,
            volume: bid.0.volume,
            created_at: bid.0.timestamp,
        }
    }
}
impl From<&Offer> for PublicOrder {
    fn from(offer: &Offer) -> Self {
        PublicOrder {
            direction: offer.0.direction.clone(),
            price: offer.0.price,
            volume: offer.0.volume,
            created_at: offer.0.timestamp,
        }
    }
}

#[derive(Clone, Serialize)]
pub enum PlayerMessage {
    // OfferRequestAccepted { offer_id: String },
    NewTrade(TradeLeg),
    OrderBookSnapshot {
        bids: Vec<PublicOrder>,
        offers: Vec<PublicOrder>,
    },
}

#[derive(Debug)]
pub enum MarketMessage {
    OrderRequest(OrderRequest),
    OrderDeletionRequest { order_id: String },
    NewPlayer(Player),
}

pub struct Player {
    pub id: String,
    pub tx: Sender<PlayerMessage>,
}

impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player").field("id", &self.id).finish()
    }
}

pub struct Market {
    rx: Receiver<MarketMessage>,
    tx: Sender<MarketMessage>,
    order_book: OrderBook,
    players: Vec<Player>,
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
                MarketMessage::OrderRequest(request) => self.process_new_offer(request).await,
                MarketMessage::NewPlayer(player) => {
                    println!("New player: {player:?}");
                    let player_id = player.id.clone();
                    self.players.push(player);
                    self.send_order_book_snapshot_to_player(player_id).await;
                }
                MarketMessage::OrderDeletionRequest { order_id } => {
                    println!("Order deletion request for order: {order_id:?}");
                    self.order_book.remove_offer(order_id);
                    self.send_order_book_snapshot_to_all().await;
                }
            }
        }
    }

    async fn send_to_players(&self, message: PlayerMessage) {
        for player in self.players.iter() {
            let _ = player.tx.send(message.clone()).await;
        }
    }

    async fn send_to_player(&self, player: String, message: PlayerMessage) {
        let Some(Player { tx, .. }) = self.players.iter().find(|p| p.id == player) else {
            return;
        };
        let _ = tx.send(message).await;
    }

    async fn send_order_book_snapshot_to_all(&self) {
        let snapshot = self.order_book.snapshot();

        let message = PlayerMessage::OrderBookSnapshot {
            bids: snapshot.bids.iter().map(PublicOrder::from).collect(),
            offers: snapshot.offers.iter().map(PublicOrder::from).collect(),
        };
        let _ = self.send_to_players(message).await;
    }

    async fn send_order_book_snapshot_to_player(&self, player: String) {
        let snapshot = self.order_book.snapshot();

        let message = PlayerMessage::OrderBookSnapshot {
            bids: snapshot.bids.iter().map(PublicOrder::from).collect(),
            offers: snapshot.offers.iter().map(PublicOrder::from).collect(),
        };
        let _ = self.send_to_player(player, message).await;
    }

    async fn process_new_offer(&mut self, request: OrderRequest) {
        let trades = self.order_book.register_order_request(request);
        println!("New trades: {trades:?}");

        self.send_order_book_snapshot_to_all().await;
        if !trades.is_empty() {
            for (leg_1, leg_2) in trades.iter().map(|trade| trade.split()) {
                self.send_to_player(leg_1.owner.clone(), PlayerMessage::NewTrade(leg_1))
                    .await;
                self.send_to_player(leg_2.owner.clone(), PlayerMessage::NewTrade(leg_2))
                    .await;
            }
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
    use tokio::sync::mpsc::channel;

    use crate::market::Player;
    use crate::order_book::{Direction, OrderRequest};

    use super::{Market, MarketMessage, PlayerMessage};

    #[tokio::test]
    async fn test_register_player() {
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register new player
        let (tx, mut rx) = channel::<PlayerMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: "toto".to_string(),
                tx: tx.clone(),
            }))
            .await;

        // We shoudl receive an initial snapshot of the current order book
        let Some(PlayerMessage::OrderBookSnapshot { bids: _, offers: _ }) = rx.recv().await else {
            unreachable!("Should have received an order book snapshot");
        };

        // Registering anoter player should not send a snapshot to the already connected player(s)
        let (second_tx, mut second_rx) = channel::<PlayerMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: "tutu".to_string(),
                tx: second_tx.clone(),
            }))
            .await;
        let _ = second_rx.recv().await;
        let Err(_) = rx.try_recv() else {
            unreachable!("Should not have received a message")
        };
    }

    #[tokio::test]
    async fn test_process_request_order() {
        // Start market actor
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register new player to market actor
        let (tx, mut rx) = channel::<PlayerMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: "toto".to_string(),
                tx: tx.clone(),
            }))
            .await;
        let _ = rx.recv().await;

        // Send order request to market actor
        let order_request = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: 50_00,
            volume: 10,
            owner: "toto".to_owned(),
        });
        market_tx.send(order_request).await.unwrap();

        // The list of offers has been updated to contains our new offer
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
    }

    #[tokio::test]
    async fn test_match_offers() {
        // Start market actor
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register buyer player to market actor and send BUY order
        let (tx_buyer, mut rx_buyer) = channel::<PlayerMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: "buyer".to_string(),
                tx: tx_buyer.clone(),
            }))
            .await;
        let _ = rx_buyer.recv().await;
        let buy_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            volume: 10,
            price: 50_00,
            owner: "buyer".to_owned(),
        });
        market_tx.send(buy_order).await.unwrap();
        rx_buyer.recv().await.unwrap();

        // Register seller player to market actor and send SELL order
        let (tx_seller, mut rx_seller) = channel::<PlayerMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: "seller".to_string(),
                tx: tx_seller.clone(),
            }))
            .await;
        let _ = rx_seller.recv().await;
        let sell_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Sell,
            volume: 10,
            price: 50_00,
            owner: "seller".to_owned(),
        });
        market_tx.send(sell_order).await.unwrap();

        // The order book snapshot should be empty for both players
        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx_buyer.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = rx_seller.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
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
}
