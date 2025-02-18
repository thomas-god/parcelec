use tokio::sync::mpsc::{channel, Receiver};

use crate::{
    market::{order_book::OrderRequest, Direction, Market},
    player::{connection::PlayerMessage, PlayerId},
};

pub struct InitialOrdersBot<MS: Market> {
    id: PlayerId,
    market: MS,
    rx: Receiver<PlayerMessage>,
}

impl<MS: Market> InitialOrdersBot<MS> {
    pub fn new(market: MS) -> InitialOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        InitialOrdersBot {
            id: bot_id,
            market,
            rx,
        }
    }

    pub async fn start(&mut self) {
        let _ = self.market.get_market_snapshot(self.id.clone()).await;

        self.send_orders().await;

        while let Some(msg) = self.rx.recv().await {
            println!("Bot {} received msg: {msg:?}", self.id);
        }
    }

    async fn send_orders(&self) {
        self.market
            .new_order(OrderRequest {
                direction: Direction::Buy,
                price: 20_00,
                volume: 250,
                owner: self.id.clone(),
            })
            .await;
        self.market
            .new_order(OrderRequest {
                direction: Direction::Sell,
                price: 90_00,
                volume: 250,
                owner: self.id.clone(),
            })
            .await;
    }
}
