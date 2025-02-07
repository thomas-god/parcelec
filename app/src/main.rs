use std::{env, sync::Arc};

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::{
        header::{CONTENT_TYPE, COOKIE},
        HeaderValue, Method,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use market::{Market, MarketMessage};
use player::PlayerActor;
use serde::Deserialize;
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, CookieManagerLayer, Cookies,
};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

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

    let Ok(origin) = env::var("ALLOW_ORIGIN") else {
        println!("No ALLOW_ORIGIN environnement variable");
        return;
    };
    let app_state = Arc::new(AppState { market_tx });
    let app = Router::new()
        .route("/game/join", post(post_join_game))
        .route("/player", get(get_player_info))
        .route("/ws", get(handle_ws_connection))
        .with_state(app_state)
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE, COOKIE])
                .allow_origin([origin.parse::<HeaderValue>().unwrap()])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_credentials(true),
        );

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handle_ws_connection(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> impl IntoResponse {
    let id = cookies
        .get("id")
        .map(|c| c.value_trimmed().to_string())
        .unwrap();
    let name = cookies
        .get("name")
        .map(|c| c.value_trimmed().to_string())
        .unwrap();
    println!("{id:?}, {name:?}");
    let tx = state.market_tx.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: Sender<MarketMessage>) {
    tokio::spawn(async move {
        PlayerActor::start(socket, tx).await;
    });
}

#[derive(Debug, Deserialize)]
struct JoinGame {
    name: String,
}

async fn post_join_game(cookies: Cookies, Json(input): Json<JoinGame>) {
    println!("{input:?}");
    if input.name.is_empty() {
        return;
    }

    let id = Uuid::new_v4().to_string();
    let id_cookie = Cookie::build(("id", id.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(id_cookie);

    let name_cookie = Cookie::build(("name", input.name.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(name_cookie);
    println!("Registered player {} with id {id}", input.name);
}

async fn get_player_info(cookies: Cookies) {
    println!("{:?}", cookies.list());
}
