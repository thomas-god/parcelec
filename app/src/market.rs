use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::order_book::{Bid, Direction, Offer, OrderBook, OrderRequest, TradeLeg};

#[derive(Debug, Clone, Serialize)]
pub struct PublicOrder {
    pub direction: Direction,
    pub volume: usize,
    pub price: usize,
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
pub enum ClientMessage {
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
    NewClient(Client),
}

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub tx: Sender<ClientMessage>,
}

pub struct Market {
    rx: Receiver<MarketMessage>,
    tx: Sender<MarketMessage>,
    order_book: OrderBook,
    clients: Vec<Client>,
}

impl Market {
    pub fn new() -> Market {
        let (tx, rx) = channel::<MarketMessage>(128);

        Market {
            rx,
            tx,
            clients: Vec::new(),
            order_book: OrderBook::new(),
        }
    }

    pub fn get_tx(&self) -> Sender<MarketMessage> {
        self.tx.clone()
    }

    pub async fn process(&mut self) {
        while let Some(message) = self.rx.recv().await {
            println!("Received message: {message:?}");
            match message {
                MarketMessage::OrderRequest(request) => self.process_new_offer(request).await,
                MarketMessage::NewClient(client) => {
                    self.clients.push(client);
                }
                MarketMessage::OrderDeletionRequest { order_id } => {
                    self.order_book.remove_offer(order_id);
                    self.send_order_book_snapshot().await;
                }
            }
        }
    }

    async fn send_to_clients(&self, message: ClientMessage) {
        for client in self.clients.iter() {
            let _ = client.tx.send(message.clone()).await;
        }
    }

    async fn send_to_client(&self, client: String, message: ClientMessage) {
        let Some(Client { tx, .. }) = self.clients.iter().find(|c| c.id == client) else {
            return;
        };
        let _ = tx.send(message).await;
    }

    async fn send_order_book_snapshot(&self) {
        let snapshot = self.order_book.snapshot();

        let message = ClientMessage::OrderBookSnapshot {
            bids: snapshot.bids.iter().map(PublicOrder::from).collect(),
            offers: snapshot.offers.iter().map(PublicOrder::from).collect(),
        };
        let _ = self.send_to_clients(message).await;
    }

    async fn process_new_offer(&mut self, request: OrderRequest) {
        let trades = self.order_book.register_order_request(request);
        println!("New trades: {trades:?}");

        self.send_order_book_snapshot().await;
        if !trades.is_empty() {
            for (leg_1, leg_2) in trades.iter().map(|trade| trade.split()) {
                self.send_to_client(leg_1.owner.clone(), ClientMessage::NewTrade(leg_1))
                    .await;
                self.send_to_client(leg_2.owner.clone(), ClientMessage::NewTrade(leg_2))
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

    use crate::market::Client;
    use crate::order_book::{Direction, OrderRequest};

    use super::{ClientMessage, Market, MarketMessage};

    #[tokio::test]
    async fn test_process_request_order() {
        // Start market actor
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register new client to market actor
        let (tx, mut rx) = channel::<ClientMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewClient(Client {
                id: "toto".to_string(),
                tx: tx.clone(),
            }))
            .await;

        // Send order request to market actor
        let order_request = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: 50_00,
            volume: 10,
            owner: "toto".to_owned(),
        });
        market_tx.send(order_request).await.unwrap();

        // The list of offers has been updated to contains our new offer
        let Some(ClientMessage::OrderBookSnapshot { bids, offers }) = rx.recv().await else {
            unreachable!("Expected ClientMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);

        // // Delete our offer
        // let request = MarketMessage::OrderDeletionRequest {
        //     order_id: new_offer_id.clone(),
        //     owner: "toto".to_string(),
        // };
        // market_tx.send(request).await.unwrap();
        // let Some(ClientMessage::OrderBookSnapshot(updated_offers)) = rx.recv().await else {
        //     unreachable!("Expected ClientMessage::PublicOffers")
        // };
        // assert!(updated_offers.iter().all(|offer| offer.id != new_offer_id));
    }

    #[tokio::test]
    async fn test_match_offers() {
        // Start market actor
        let mut market = Market::new();
        let market_tx = market.get_tx();
        tokio::spawn(async move {
            market.process().await;
        });

        // Register buyer client to market actor and send BUY order
        let (tx_buyer, mut rx_buyer) = channel::<ClientMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewClient(Client {
                id: "buyer".to_string(),
                tx: tx_buyer.clone(),
            }))
            .await;
        let buy_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            volume: 10,
            price: 50_00,
            owner: "buyer".to_owned(),
        });
        market_tx.send(buy_order).await.unwrap();
        rx_buyer.recv().await.unwrap();

        // Register seller client to market actor and send SELL order
        let (tx_seller, mut rx_seller) = channel::<ClientMessage>(1);
        let _ = market_tx
            .send(MarketMessage::NewClient(Client {
                id: "seller".to_string(),
                tx: tx_seller.clone(),
            }))
            .await;
        let sell_order = MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Sell,
            volume: 10,
            price: 50_00,
            owner: "seller".to_owned(),
        });
        market_tx.send(sell_order).await.unwrap();

        // The order book snapshot should be empty for both clients
        let Some(ClientMessage::OrderBookSnapshot { bids, offers }) = rx_buyer.recv().await else {
            unreachable!("Expected ClientMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        let Some(ClientMessage::OrderBookSnapshot { bids, offers }) = rx_seller.recv().await else {
            unreachable!("Expected ClientMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        // Each client should receive its own trade leg
        let Some(ClientMessage::NewTrade(trade_buyer)) = rx_buyer.recv().await else {
            unreachable!("Should have received a trade")
        };
        assert_eq!(trade_buyer.direction, Direction::Buy);
        let Some(ClientMessage::NewTrade(trade_seller)) = rx_seller.recv().await else {
            unreachable!("Should have received a trade")
        };
        assert_eq!(trade_seller.direction, Direction::Sell);
    }
}
