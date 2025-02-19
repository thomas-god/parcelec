use std::{collections::HashMap, sync::Arc};

use api::{start_server, AppState};
use game::game_repository::GameRepository;
use player::repository::PlayerConnectionRepository;
use tokio::sync::RwLock;

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
    let market_services = HashMap::new();
    let game_services = HashMap::new();
    let stack_services = HashMap::new();

    let app_state = Arc::new(RwLock::new(AppState {
        game_repository: games_repo,
        player_connections_repository: connections_repo,
        market_services,
        game_services,
        stack_services,
    }));

    start_server(app_state).await;
}
