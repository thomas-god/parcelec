use std::{collections::HashMap, env, sync::Arc};

use axum::{
    http::{
        header::{CONTENT_TYPE, COOKIE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use join_game::join_game;
use tokio::{
    net::TcpListener,
    sync::{mpsc, RwLock},
};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tutorial::create_tutorial_game;
use ws::handle_ws_connection;

use crate::{
    game::{GameContext, GameId},
    market::{MarketContext, MarketService},
    plants::stack::StackContext,
    player::{repository::ConnectionRepositoryMessage, PlayerId},
};

mod join_game;
mod tutorial;
mod ws;

pub type ApiState = Arc<RwLock<AppState>>;
pub struct AppState {
    pub market_services: HashMap<GameId, MarketContext<MarketService>>,
    pub game_services: HashMap<GameId, GameContext>,
    pub stack_services: HashMap<GameId, HashMap<PlayerId, StackContext>>,
    pub player_connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
}

pub async fn start_server(app_state: ApiState) {
    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Unable to start TCP listener");
    println!("Listenning on {addr}");

    let origin = env::var("ALLOW_ORIGIN").expect("No ALLOW_ORIGIN environnement variable");

    let app = build_app(app_state, origin);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn build_app(app_state: ApiState, origin: String) -> Router {
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
        )
}
