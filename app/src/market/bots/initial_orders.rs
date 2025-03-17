use tokio::sync::mpsc::{Receiver, channel};

use crate::{
    game::delivery_period::DeliveryPeriodId,
    market::{Direction, Market, MarketContext, MarketState, order_book::OrderRequest},
    player::{PlayerId, PlayerMessage},
};

pub struct InitialOrdersBot<MS: Market> {
    id: PlayerId,
    market: MarketContext<MS>,
    _rx: Receiver<PlayerMessage>,
}

impl<MS: Market> InitialOrdersBot<MS> {
    fn new(market: MarketContext<MS>) -> InitialOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        InitialOrdersBot {
            id: bot_id,
            market,
            _rx: rx,
        }
    }

    pub fn start(market: MarketContext<MS>) {
        let mut bot = InitialOrdersBot::new(market);

        tokio::spawn(async move {
            bot.run().await;
        });
    }

    async fn wait_for_market_to_open(&mut self) {
        while *self.market.state_rx.borrow_and_update() != MarketState::Open {
            let _ = self.market.state_rx.changed().await;
        }
    }

    async fn wait_for_market_to_close(&mut self) {
        while *self.market.state_rx.borrow_and_update() != MarketState::Closed {
            let _ = self.market.state_rx.changed().await;
        }
    }

    pub async fn run(&mut self) {
        let _ = self
            .market
            .service
            .get_market_snapshot(self.id.clone())
            .await;

        let mut period = DeliveryPeriodId::default();
        loop {
            // 1st period: do nothing
            period = period.next();
            self.wait_for_market_to_open().await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Buy,
                    price: 55_00,
                    volume: 200,
                    owner: self.id.clone(),
                })
                .await;
            self.wait_for_market_to_close().await;

            // 2nd period: do nothing
            period = period.next();
            self.wait_for_market_to_open().await;
            self.wait_for_market_to_close().await;

            // 3rd period: buy 200MW
            period = period.next();
            self.wait_for_market_to_open().await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Buy,
                    price: 55_00,
                    volume: 200,
                    owner: self.id.clone(),
                })
                .await;
            self.wait_for_market_to_close().await;

            // 4rd period: sell 200MW
            period = period.next();
            self.wait_for_market_to_open().await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Sell,
                    price: 85_00,
                    volume: 200,
                    owner: self.id.clone(),
                })
                .await;
            self.wait_for_market_to_close().await;
        }
    }
}
