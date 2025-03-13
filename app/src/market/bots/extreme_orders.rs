use tokio::sync::mpsc::{Receiver, channel};

use crate::{
    constants,
    market::{Direction, Market, MarketContext, MarketState, order_book::OrderRequest},
    player::{PlayerId, PlayerMessage},
};

pub struct ExtremeOrdersBot<MS: Market> {
    bot_id: PlayerId,
    market: MarketContext<MS>,
    _rx: Receiver<PlayerMessage>,
}

impl<MS: Market> ExtremeOrdersBot<MS> {
    fn new(market: MarketContext<MS>) -> ExtremeOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        ExtremeOrdersBot {
            bot_id,
            market,
            _rx: rx,
        }
    }

    pub fn start(market: MarketContext<MS>) {
        let mut bot = ExtremeOrdersBot::new(market);

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
            .get_market_snapshot(self.bot_id.clone())
            .await;

        loop {
            self.wait_for_market_to_open().await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Buy,
                    price: constants::MARKET_EXTREME_BUY_OFFER_PRICE,
                    volume: constants::MARKET_EXTREME_OFFERS_VOLUME,
                    owner: self.bot_id.clone(),
                })
                .await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Sell,
                    price: constants::MARKET_EXTREME_SELL_OFFER_PRICE,
                    volume: constants::MARKET_EXTREME_OFFERS_VOLUME,
                    owner: self.bot_id.clone(),
                })
                .await;
            self.wait_for_market_to_close().await;
        }
    }
}
