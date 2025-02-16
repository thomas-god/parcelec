use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

use crate::{
    market::{models::Direction, order_book::OrderRequest, MarketMessage},
    player::connection::PlayerMessage,
};

pub struct InitialOrdersBot {
    id: String,
    market_tx: Sender<MarketMessage>,
    rx: Receiver<PlayerMessage>,
}

impl InitialOrdersBot {
    pub fn new(market_tx: Sender<MarketMessage>) -> InitialOrdersBot {
        let bot_id = Uuid::new_v4().to_string();
        let (_, rx) = channel(16);

        InitialOrdersBot {
            id: bot_id,
            market_tx,
            rx,
        }
    }

    pub async fn start(&mut self) {
        if self
            .market_tx
            .clone()
            .send(MarketMessage::NewPlayerConnection(self.id.clone()))
            .await
            .is_err()
        {
            println!("Unable to connect bot to market");
            return;
        }

        self.send_orders().await;

        while let Some(msg) = self.rx.recv().await {
            println!("Bot {} received msg: {msg:?}", self.id);
        }
    }

    async fn send_orders(&self) {
        let _ = self
            .market_tx
            .clone()
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                price: 20_00,
                volume: 250,
                owner: self.id.clone(),
            }))
            .await;

        let _ = self
            .market_tx
            .clone()
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                price: 90_00,
                volume: 250,
                owner: self.id.clone(),
            }))
            .await;
    }
}
