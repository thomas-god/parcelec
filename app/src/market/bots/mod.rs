use extreme_orders::ExtremeOrdersBot;
use tutorial_initial_orders::TutorialInitialOrdersBot;

use crate::market::{Market, MarketContext};

pub mod extreme_orders;
pub mod tutorial_initial_orders;

pub async fn start_bots<MS: Market>(market: MarketContext<MS>) {
    ExtremeOrdersBot::start(market.clone());
}

pub async fn start_bots_tutorial<MS: Market>(market: MarketContext<MS>) {
    TutorialInitialOrdersBot::start(market.clone());
    ExtremeOrdersBot::start(market.clone());
}
