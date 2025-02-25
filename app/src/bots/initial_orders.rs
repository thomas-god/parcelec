use tokio::sync::mpsc::{self, channel, Receiver};

use crate::{
    game::{delivery_period::DeliveryPeriodId, GameId},
    market::{
        order_book::OrderRequest, Direction, ForecastLevel, Market, MarketContext, MarketForecast,
        MarketState,
    },
    player::{connection::PlayerMessage, repository::ConnectionRepositoryMessage, PlayerId},
};

pub struct InitialOrdersBot<MS: Market> {
    id: PlayerId,
    game: GameId,
    market: MarketContext<MS>,
    players_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    _rx: Receiver<PlayerMessage>,
}

impl<MS: Market> InitialOrdersBot<MS> {
    fn new(
        game: GameId,
        market: MarketContext<MS>,
        players_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> InitialOrdersBot<MS> {
        let bot_id = PlayerId::default();
        let (_, rx) = channel(16);

        InitialOrdersBot {
            id: bot_id,
            game,
            market,
            players_repository,
            _rx: rx,
        }
    }

    pub fn start(
        game: GameId,
        market: MarketContext<MS>,
        players_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    ) {
        let mut bot = InitialOrdersBot::new(game, market, players_repository);

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
            self.wait_for_market_to_close().await;

            // 2nd period: do nothing
            period = period.next();
            self.wait_for_market_to_open().await;
            self.wait_for_market_to_close().await;
            // Send forecast for next period (will buy)
            let _ = self
                .players_repository
                .send(ConnectionRepositoryMessage::SendToAllPlayers(
                    self.game.clone(),
                    PlayerMessage::MarketForecast(MarketForecast {
                        direction: Direction::Buy,
                        volume: ForecastLevel::Medium,
                        issuer: self.id.clone(),
                        period,
                        price: None,
                    }),
                ))
                .await;

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
            // Send forecast for next period (will sell)
            let _ = self
                .players_repository
                .send(ConnectionRepositoryMessage::SendToAllPlayers(
                    self.game.clone(),
                    PlayerMessage::MarketForecast(MarketForecast {
                        direction: Direction::Sell,
                        volume: ForecastLevel::Medium,
                        issuer: self.id.clone(),
                        period,
                        price: None,
                    }),
                ))
                .await;

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
