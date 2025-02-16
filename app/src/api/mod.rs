use std::{env, sync::Arc};

use axum::{
    http::{
        header::{CONTENT_TYPE, COOKIE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use join_game::join_game;
use tokio::sync::mpsc;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tutorial::create_tutorial_game;
use ws::handle_ws_connection;

use crate::{
    game::game_repository::GameRepositoryMessage, player::repository::ConnectionRepositoryMessage,
};

mod join_game;
mod tutorial;
mod ws;

pub struct AppState {
    pub game_repository: mpsc::Sender<GameRepositoryMessage>,
    pub player_connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
}

pub fn build_router(app_state: Arc<AppState>) -> Option<Router> {
    let Ok(origin) = env::var("ALLOW_ORIGIN") else {
        println!("No ALLOW_ORIGIN environnement variable");
        return None;
    };
    Some(
        Router::new()
            .route("/game/join", post(join_game))
            .route("/tutorial", post(create_tutorial_game))
            .route("/ws", get(handle_ws_connection))
            .with_state(app_state)
            .layer(CookieManagerLayer::new())
            .layer(
                CorsLayer::new()
                    .allow_headers([CONTENT_TYPE, COOKIE])
                    .allow_origin([origin.parse::<HeaderValue>().unwrap()])
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_credentials(true),
            ),
    )
}
