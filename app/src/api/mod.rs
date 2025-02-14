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
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot, watch};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, CookieManagerLayer, Cookies,
};
use tower_http::cors::CorsLayer;

use crate::{
    game::{ConnectPlayerResponse, GameContext, GameMessage, RegisterPlayerResponse},
    market::{MarketMessage, MarketState},
    plants::stack::{StackMessage, StackState},
    player::PlayerConnectionActor,
};

pub struct AppState {
    pub context: GameContext,
}

pub fn build_router(app_state: Arc<AppState>) -> Option<Router> {
    let Ok(origin) = env::var("ALLOW_ORIGIN") else {
        println!("No ALLOW_ORIGIN environnement variable");
        return None;
    };
    Some(
        Router::new()
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
            ),
    )
}
pub async fn handle_ws_connection(
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
    let (tx, rx) = oneshot::channel::<ConnectPlayerResponse>();
    let _ = state
        .context
        .game
        .clone()
        .send(GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await;

    let (player_stack, stack_state) = match rx.await {
        Ok(ConnectPlayerResponse::OK {
            player_stack,
            stack_state,
        }) => {
            println!("Player is connected, continuing with processing WS");
            (player_stack, stack_state)
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
        Ok(ConnectPlayerResponse::NoStackFound) => {
            println!("Player exists but has no matching stack");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(err) => {
            println!("Something went wrong");
            println!("{err:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let market = state.context.market.clone();
    let market_state = state.context.market_state.clone();
    ws.on_upgrade(move |socket| {
        handle_socket(socket, id, market, market_state, player_stack, stack_state)
    })
}

async fn handle_socket(
    socket: WebSocket,
    player_id: String,
    market: mpsc::Sender<MarketMessage>,
    market_state: watch::Receiver<MarketState>,
    stack: mpsc::Sender<StackMessage>,
    stack_state: watch::Receiver<StackState>,
) {
    tokio::spawn(async move {
        PlayerConnectionActor::start(socket, player_id, market, market_state, stack, stack_state)
            .await;
    });
}

#[derive(Debug, Deserialize)]
pub struct JoinGame {
    name: String,
}

pub async fn join_game(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Json(input): Json<JoinGame>,
) -> impl IntoResponse {
    println!("{input:?}");
    if input.name.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let Ok(domain) = env::var("DOMAIN") else {
        println!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let (tx, rx) = oneshot::channel::<RegisterPlayerResponse>();
    let game = state.context.game.clone();

    let _ = game
        .send(GameMessage::RegisterPlayer {
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
        .domain(domain.clone())
        .path("/")
        .build();
    cookies.add(id_cookie);

    let name_cookie = Cookie::build(("name", input.name.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain)
        .path("/")
        .build();
    cookies.add(name_cookie);
    println!("Registered player {} with id {id}", input.name);
    StatusCode::CREATED
}
