use initial_orders::InitialOrdersBot;

use crate::market::{Market, MarketContext, MarketState};

pub mod initial_orders;

pub async fn start_bots<MS: Market>(mut market: MarketContext<MS>) {
    while *market.state_rx.borrow_and_update() != MarketState::Open {
        let _ = market.state_rx.changed().await;
    }

    let mut initial_orders_bots = InitialOrdersBot::new(market.service.clone());

    tokio::spawn(async move { initial_orders_bots.start().await });
}
