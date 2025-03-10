use std::env;

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
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tutorial::create_tutorial_game;
use ws::handle_ws_connection;

mod create_game;
mod join_game;
mod list_games;
pub mod state;
mod tutorial;
pub mod ws;

pub async fn start_server(app_state: ApiState) {
    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Unable to start TCP listener");
    tracing::info!("Listenning on {addr}");

    let origin = env::var("ALLOW_ORIGIN").expect("No ALLOW_ORIGIN environnement variable");

    let app = build_app(app_state, origin);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn build_app(app_state: ApiState, origin: String) -> Router {
    Router::new()
        .route("/game", post(create_game))
        .route("/games", get(list_games))
        .route("/game/join", post(join_game))
        .route("/tutorial", post(create_tutorial_game))
        .route("/ws", get(handle_ws_connection))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE, COOKIE])
                .allow_origin([origin.parse::<HeaderValue>().unwrap()])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_credentials(true),
        )
}
