use initial_orders::InitialOrdersBot;
use tokio::sync::oneshot;

use crate::{
    game::{GameContext, GameMessage, GameState},
    market::MarketService,
};

pub mod initial_orders;

pub async fn start_bots(mut game: GameContext) {
    while *game.state_rx.borrow_and_update() != GameState::Running {
        let _ = game.state_rx.changed().await;
    }

    let (tx_back, rx) = oneshot::channel();
    let _ = game.tx.send(GameMessage::GetMarketTx { tx_back }).await;
    let Ok(market) = rx.await else {
        println!("Unable to get market tx for starting bots");
        return;
    };

    let market_service = MarketService::new(market);
    let mut initial_orders_bots = InitialOrdersBot::new(market_service);

    tokio::spawn(async move { initial_orders_bots.start().await });
}
