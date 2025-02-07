use std::{env, sync::Arc};

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::{
        header::{CONTENT_TYPE, COOKIE},
        HeaderValue, Method, StatusCode,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use game::{ConnectPlayerResponse, Game, GameContext, RegisterPlayerResponse};
use market::MarketMessage;
use player::PlayerConnection;
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{mpsc::Sender, oneshot::channel},
};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, CookieManagerLayer, Cookies,
};
use tower_http::cors::CorsLayer;

pub mod game;
pub mod market;
pub mod player;

struct AppState {
    context: GameContext,
}

#[tokio::main]
async fn main() {
    let mut game = Game::new().await;
    let context = game.get_context();
    tokio::spawn(async move {
        game.run().await;
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
    let app_state = Arc::new(AppState { context });
    let app = Router::new()
        .route("/game/join", post(join_game))
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
    let Some(id) = cookies.get("id").map(|c| c.value_trimmed().to_string()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(_) = cookies.get("name").map(|c| c.value_trimmed().to_string()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let (tx, rx) = channel::<ConnectPlayerResponse>();
    let _ = state
        .context
        .game
        .clone()
        .send(game::GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await;

    match rx.await {
        Ok(ConnectPlayerResponse::OK) => {
            println!("Player is connected, continuing with processing WS");
        }
        Ok(ConnectPlayerResponse::DoesNotExist) => {
            println!("Player does not exist, invalidating its cookies");
            let id_cookie = Cookie::build(("id", "".to_string()))
                .max_age(Duration::seconds(0))
                .same_site(SameSite::Strict)
                .path("/")
                .build();
            cookies.add(id_cookie);

            let name_cookie = Cookie::build(("name", "".to_string()))
                .max_age(Duration::seconds(0))
                .same_site(SameSite::Strict)
                .path("/")
                .build();
            cookies.add(name_cookie);
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(err) => {
            println!("Something went wrong");
            println!("{err:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let market = state.context.market.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, id, market))
}

async fn handle_socket(socket: WebSocket, player_id: String, market: Sender<MarketMessage>) {
    tokio::spawn(async move {
        PlayerConnection::start(socket, player_id, market).await;
    });
}

#[derive(Debug, Deserialize)]
struct JoinGame {
    name: String,
}

async fn join_game(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Json(input): Json<JoinGame>,
) -> impl IntoResponse {
    println!("{input:?}");
    if input.name.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let (tx, rx) = channel::<RegisterPlayerResponse>();
    let game = state.context.game.clone();

    let _ = game
        .send(game::GameMessage::RegisterPlayer {
            name: input.name.clone(),
            tx_back: tx,
        })
        .await;

    let id = match rx.await {
        Ok(RegisterPlayerResponse::Success { id }) => id,
        Ok(RegisterPlayerResponse::PlayerAlreadyExist) => {
            println!("Player with name {} already exist", input.name);
            return StatusCode::CONFLICT;
        }
        Err(err) => {
            println!("{err:?}");
            println!("Error while sending message to game instance");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

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
    StatusCode::CREATED
}
