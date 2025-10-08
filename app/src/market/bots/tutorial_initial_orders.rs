use tokio::sync::mpsc::{Receiver, channel};
use tokio_util::sync::CancellationToken;

use crate::{
    game::delivery_period::DeliveryPeriodId,
    market::{Direction, Market, MarketContext, MarketState, order_book::OrderRequest},
    player::{PlayerId, PlayerMessage},
    utils::units::{Energy, EnergyCost},
};

pub struct TutorialInitialOrdersBot<MS: Market> {
    id: PlayerId,
    market: MarketContext<MS>,
    cancellation_token: CancellationToken,
    _rx: Receiver<PlayerMessage>,
}

impl<MS: Market> TutorialInitialOrdersBot<MS> {
    fn new(
        market: MarketContext<MS>,
        cancellation_token: CancellationToken,
    ) -> TutorialInitialOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        TutorialInitialOrdersBot {
            id: bot_id,
            market,
            cancellation_token,
            _rx: rx,
        }
    }

    pub fn start(market: MarketContext<MS>, cancellation_token: CancellationToken) {
        let mut bot = TutorialInitialOrdersBot::new(market, cancellation_token);

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

        let cancellation_token = self.cancellation_token.clone();
        loop {
            tokio::select! {
                next_period = self.post_orders(&period) => {
                    period = next_period;
                }
                _ = cancellation_token.cancelled() => {
                    break;
                }
            }
        }
    }

    async fn post_orders(&mut self, period: &DeliveryPeriodId) -> DeliveryPeriodId {
        self.wait_for_market_to_open().await;
        let next_period = period.next();

        if *period == DeliveryPeriodId::from(1) {
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Sell,
                    price: EnergyCost::from(60),
                    volume: Energy::from(300),
                    owner: self.id.clone(),
                })
                .await;
            self.market
                .service
                .new_order(OrderRequest {
                    direction: Direction::Buy,
                    price: EnergyCost::from(50),
                    volume: Energy::from(300),
                    owner: self.id.clone(),
                })
                .await;
        }

        self.wait_for_market_to_close().await;
        next_period
    }
}
