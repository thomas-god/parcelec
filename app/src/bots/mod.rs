use initial_orders::InitialOrdersBot;
use tokio::sync::mpsc;

use crate::{
    game::GameId,
    market::{Market, MarketContext},
    player::repository::ConnectionRepositoryMessage,
};

pub mod initial_orders;

pub async fn start_bots<MS: Market>(
    game: GameId,
    market: MarketContext<MS>,
    players_repository: mpsc::Sender<ConnectionRepositoryMessage>,
) {
    InitialOrdersBot::start(game, market, players_repository);
}
