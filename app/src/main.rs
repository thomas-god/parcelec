use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use market::{Market, MarketMessage};
use player::PlayerActor;
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_cookies::{cookie::time::Duration, Cookie, CookieManagerLayer, Cookies};

pub mod market;
pub mod order_book;
pub mod player;

struct AppState {
    market_tx: Sender<MarketMessage>,
}

#[tokio::main]
async fn main() {
    let mut market = Market::new();
    let market_tx = market.get_tx();
    tokio::spawn(async move {
        println!("Starting market actor");
        market.process().await;
    });

    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Unable to start TCP listener");
    println!("Listenning on {addr}");

    let app_state = Arc::new(AppState { market_tx });
    let app = Router::new()
        .route("/game/join", post(post_join_game))
        .route("/player", get(get_player_info))
        .route("/ws", get(handle_ws_connection))
        .with_state(app_state)
        .layer(CookieManagerLayer::new());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handle_ws_connection(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let tx = state.market_tx.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: Sender<MarketMessage>) {
    tokio::spawn(async move {
        PlayerActor::start(socket, tx).await;
    });
}

async fn post_join_game(cookies: Cookies) {
    println!("hello");
    let cookie = Cookie::build(("toto", "tutu"))
        .max_age(Duration::days(1))
        .domain("127.0.0.1")
        .build();
    cookies.add(cookie);
}

async fn get_player_info(cookies: Cookies) {
    println!("{:?}", cookies.list());
}
