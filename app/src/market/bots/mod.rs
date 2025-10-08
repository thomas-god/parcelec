use extreme_orders::ExtremeOrdersBot;
use tokio_util::sync::CancellationToken;
use tutorial_initial_orders::TutorialInitialOrdersBot;

use crate::market::{Market, MarketContext};

pub mod extreme_orders;
pub mod tutorial_initial_orders;

pub async fn start_bots<MS: Market>(
    market: MarketContext<MS>,
    cancellation_token: CancellationToken,
) {
    ExtremeOrdersBot::start(market.clone(), cancellation_token);
}

pub async fn start_bots_tutorial<MS: Market>(
    market: MarketContext<MS>,
    cancellation_token: CancellationToken,
) {
    TutorialInitialOrdersBot::start(market.clone(), cancellation_token.clone());
    ExtremeOrdersBot::start(market.clone(), cancellation_token);
}
