use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{CONTENT_TYPE, COOKIE},
    },
    routing::{get, post},
};
use create_game::create_game;
use join_game::join_game;
use list_games::list_games;
use state::ApiState;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tutorial::create_tutorial_game;
use ws::handle_ws_connection;

use crate::AppConfig;

mod create_game;
mod join_game;
mod list_games;
pub mod state;
mod tutorial;
pub mod ws;

pub fn build_router(state: ApiState, config: AppConfig) -> Router {
    Router::new()
        .route("/game", post(create_game))
        .route("/games", get(list_games))
        .route("/game/join", post(join_game))
        .route("/tutorial", post(create_tutorial_game))
        .route("/ws", get(handle_ws_connection))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE, COOKIE])
                .allow_origin([config.allow_origin.parse::<HeaderValue>().unwrap()])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_credentials(true),
        )
}
