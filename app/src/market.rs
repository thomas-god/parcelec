use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum Direction {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Offer {
    pub direction: Direction,
    pub volume: usize,
    pub price: usize,
}

#[derive(Debug, Clone)]
pub struct PublicOffer {
    offer: Offer,
    id: String,
}

pub struct InternalOffer {
    pub offer: Offer,
    pub id: String,
    pub owner: String,
}

impl From<&InternalOffer> for PublicOffer {
    fn from(offer: &InternalOffer) -> Self {
        PublicOffer {
            offer: offer.offer.clone(),
            id: (*offer.id).to_string(),
        }
    }
}

#[derive(Clone)]
pub enum ClientMessage {
    OfferRequestAccepted { offer_id: String },
    PublicOffers(Vec<PublicOffer>),
}

#[derive(Debug)]
pub struct OfferRequest {
    offer: Offer,
    owner: String,
    tx_back: Sender<ClientMessage>,
}

#[derive(Debug)]
pub enum MarketMessage {
    OfferRequest(OfferRequest),
    OfferDeletionRequest { offer_id: String, owner: String },
    NewClient(Client),
}

#[derive(Debug)]
pub struct Client {
    id: String,
    tx: Sender<ClientMessage>,
}

pub struct Market {
    rx: Receiver<MarketMessage>,
    tx: Sender<MarketMessage>,
    offers: Vec<InternalOffer>,
    clients: Vec<Client>,
}

impl Market {
    pub fn new() -> Market {
        let (tx, rx) = channel::<MarketMessage>(128);

        Market {
            rx,
            tx,
            clients: Vec::new(),
            offers: Vec::new(),
        }
    }

    pub fn get_tx(&self) -> Sender<MarketMessage> {
        self.tx.clone()
    }

    async fn process(&mut self) {
        while let Some(message) = self.rx.recv().await {
            println!("Received message: {message:?}");
            match message {
                MarketMessage::OfferRequest(request) => self.process_new_offer(request).await,
                MarketMessage::NewClient(client) => {
                    self.clients.push(client);
                }
                MarketMessage::OfferDeletionRequest { offer_id, owner } => {
                    self.offers
                        .retain(|offer| !(offer.id == offer_id && offer.owner == owner));
                    self.send_public_offers().await;
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

    async fn send_public_offers(&self) {
        let message =
            ClientMessage::PublicOffers(self.offers.iter().map(PublicOffer::from).collect());
        let _ = self.send_to_clients(message).await;
    }

    async fn process_new_offer(&mut self, request: OfferRequest) {
        {
            let new_offer = InternalOffer {
                offer: request.offer,
                owner: request.owner,
                id: "toto".to_string(),
            };
            self.offers.push(new_offer);

            request
                .tx_back
                .send(ClientMessage::OfferRequestAccepted {
                    offer_id: "toto".to_owned(),
                })
                .await
                .unwrap();
            self.send_public_offers().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::channel;

    use crate::market::{Client, Offer};

    use super::{ClientMessage, Direction, Market, MarketMessage, OfferRequest};

    #[tokio::test]
    async fn test_send_new_offer() {
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

        // Send offer request to market actor
        let offer = MarketMessage::OfferRequest(OfferRequest {
            offer: Offer {
                direction: Direction::Buy,
                volume: 1,
                price: 100,
            },
            owner: "toto".to_owned(),
            tx_back: tx,
        });
        market_tx.send(offer).await.unwrap();

        // Our offer has been accepted
        let Some(ClientMessage::OfferRequestAccepted {
            offer_id: new_offer_id,
        }) = rx.recv().await
        else {
            unreachable!("Expected ClientMessage::OfferAccepted")
        };

        // The list of offers has been updated to contains our new offer
        let Some(ClientMessage::PublicOffers(offers)) = rx.recv().await else {
            unreachable!("Expected ClientMessage::PublicOffers")
        };
        assert!(offers.iter().any(|offer| offer.id == new_offer_id));

        // Delete our offer
        let request = MarketMessage::OfferDeletionRequest {
            offer_id: new_offer_id.clone(),
            owner: "toto".to_string(),
        };
        market_tx.send(request).await.unwrap();
        let Some(ClientMessage::PublicOffers(updated_offers)) = rx.recv().await else {
            unreachable!("Expected ClientMessage::PublicOffers")
        };
        assert!(updated_offers.iter().all(|offer| offer.id != new_offer_id));
    }

    #[tokio::test]
    async fn test_match_offers() {
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

        // Send offer request to market actor
        let buy_offer = MarketMessage::OfferRequest(OfferRequest {
            offer: Offer {
                direction: Direction::Buy,
                volume: 1,
                price: 100,
            },
            owner: "toto".to_owned(),
            tx_back: tx.clone(),
        });
        market_tx.send(buy_offer).await.unwrap();
        rx.recv().await.unwrap();
        rx.recv().await.unwrap();

        // Send second offer matching the first one
        let sell_offer = MarketMessage::OfferRequest(OfferRequest {
            offer: Offer {
                direction: Direction::Sell,
                volume: 1,
                price: 100,
            },
            owner: "toto".to_owned(),
            tx_back: tx.clone(),
        });
        market_tx.send(sell_offer).await.unwrap();
    }
}
