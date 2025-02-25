use std::{collections::HashMap, sync::Arc};

use api::{start_server, AppState};
use player::repository::PlayerConnectionRepository;
use tokio::sync::RwLock;

pub mod api;
pub mod bots;
pub mod forecast;
pub mod game;
pub mod market;
pub mod plants;
pub mod player;

#[tokio::main]
async fn main() {
    let connections_repo = PlayerConnectionRepository::start();
    let market_services = HashMap::new();
    let game_services = HashMap::new();
    let stack_services = HashMap::new();

    let app_state = Arc::new(RwLock::new(AppState {
        player_connections_repository: connections_repo,
        market_services,
        game_services,
        stack_services,
    }));

    start_server(app_state).await;
}
