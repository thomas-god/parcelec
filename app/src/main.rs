use std::sync::Arc;

use api::{build_router, AppState};
// use bots::start_bots;
use game::game_repository::GameRepository;
use tokio::net::TcpListener;

pub mod api;
pub mod bots;
pub mod game;
pub mod market;
pub mod plants;
pub mod player;

#[tokio::main]
async fn main() {
    let mut game_repository = GameRepository::new();
    let game_repository_tx = game_repository.get_tx();

    tokio::spawn(async move { game_repository.start().await });

    // start_bots(context.market.clone()).await;

    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Unable to start TCP listener");
    println!("Listenning on {addr}");

    let app_state = Arc::new(AppState {
        game_repository: game_repository_tx,
    });
    let app = build_router(app_state).unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
