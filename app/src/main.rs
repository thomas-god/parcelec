use std::sync::Arc;

use api::{start_server, AppState};
use game::game_repository::GameRepository;
use player::repository::PlayerConnectionRepository;

pub mod api;
pub mod bots;
pub mod game;
pub mod market;
pub mod plants;
pub mod player;

#[tokio::main]
async fn main() {
    let connections_repo = PlayerConnectionRepository::start();
    let games_repo = GameRepository::start(&connections_repo);

    let app_state = Arc::new(AppState {
        game_repository: games_repo,
        player_connections_repository: connections_repo,
    });

    start_server(app_state).await;
}
