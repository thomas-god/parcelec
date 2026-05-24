use std::time::Duration;

use rand::random_range;
use tokio::{
    sync::mpsc::{Receiver, channel},
    time::sleep,
};
use tokio_util::sync::CancellationToken;

use crate::{
    game::{GameContext, GameState, delivery_period::DeliveryPeriodId},
    market::{Direction, Market, MarketContext, MarketState, order_book::OrderRequest},
    player::{PlayerId, PlayerMessage},
    utils::units::{Energy, EnergyCost},
};

pub struct TutorialInitialOrdersBot<MS: Market> {
    id: PlayerId,
    market: MarketContext<MS>,
    game: GameContext,
    cancellation_token: CancellationToken,
    _rx: Receiver<PlayerMessage>,
}

impl<MS: Market> TutorialInitialOrdersBot<MS> {
    fn new(
        game: GameContext,
        market: MarketContext<MS>,
        cancellation_token: CancellationToken,
    ) -> TutorialInitialOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        TutorialInitialOrdersBot {
            id: bot_id,
            game,
            market,
            cancellation_token,
            _rx: rx,
        }
    }

    pub fn start(
        game: GameContext,
        market: MarketContext<MS>,
        cancellation_token: CancellationToken,
    ) {
        let mut bot = TutorialInitialOrdersBot::new(game, market, cancellation_token);

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

        let cancellation_token = self.cancellation_token.clone();
        loop {
            tokio::select! {
                _ = self.post_orders() => {}
                _ = cancellation_token.cancelled() => {
                    break;
                }
            }
        }
    }

    async fn post_orders(&mut self) {
        self.wait_for_market_to_open().await;
        let state = self.game.state_rx.borrow().clone();
        let GameState::Running { period, .. } = state else {
            return;
        };

        sleep(Duration::from_secs_f32(random_range(2.0..10.0))).await;

        if period == DeliveryPeriodId::from(1) {
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
    }
}
