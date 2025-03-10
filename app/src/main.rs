use std::{collections::HashMap, sync::Arc};

use api::{AppState, start_server};
use player::repository::PlayerConnectionRepository;
use tokio::sync::RwLock;

pub mod api;
pub mod forecast;
pub mod game;
pub mod market;
pub mod plants;
pub mod player;
pub mod utils;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        tracing::error!("Error while setting up tracing subscriber: {err:?}");
    };
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
