use initial_orders::InitialOrdersBot;

use crate::market::{MarketContext, MarketService, MarketState};

pub mod initial_orders;

pub async fn start_bots(mut market: MarketContext) {
    while *market.state_rx.borrow_and_update() != MarketState::Open {
        let _ = market.state_rx.changed().await;
    }

    let market_service = MarketService::new(market.tx);
    let mut initial_orders_bots = InitialOrdersBot::new(market_service);

    tokio::spawn(async move { initial_orders_bots.start().await });
}
