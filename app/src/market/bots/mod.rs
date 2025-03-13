use extreme_orders::ExtremeOrdersBot;
use initial_orders::InitialOrdersBot;

use crate::market::{Market, MarketContext};

pub mod extreme_orders;
pub mod initial_orders;

pub async fn start_bots<MS: Market>(market: MarketContext<MS>) {
    InitialOrdersBot::start(market.clone());
    ExtremeOrdersBot::start(market.clone());
}
