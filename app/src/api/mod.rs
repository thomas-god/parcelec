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
    game::{
        game_repository::{CreateNewGameResponse, GameId, GameRepositoryMessage, GetGameResponse},
        ConnectPlayerResponse, GameMessage, RegisterPlayerResponse,
    },
    market::{MarketMessage, MarketState},
    plants::stack::{StackMessage, StackState},
    player::PlayerConnectionActor,
};

pub struct AppState {
    pub game_repository: mpsc::Sender<GameRepositoryMessage>,
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
    let Some(id) = cookies
        .get("player_id")
        .map(|c| c.value_trimmed().to_string())
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(_) = cookies
        .get("player_name")
        .map(|c| c.value_trimmed().to_string())
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(game_id) = cookies
        .get("game_id")
        .map(|c| GameId::from(c.value_trimmed()))
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let (tx_back, rx) = oneshot::channel();
    let _ = state
        .game_repository
        .send(GameRepositoryMessage::GetGame {
            game_id: game_id.clone(),
            tx_back,
        })
        .await;
    let Ok(GetGameResponse::Found(game)) = rx.await else {
        println!("No game found for ID: {game_id:?}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let (tx, rx) = oneshot::channel();
    let _ = game
        .clone()
        .send(GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await;

    let (player_stack, stack_state, market, market_state) = match rx.await {
        Ok(ConnectPlayerResponse::OK {
            market,
            market_state,
            player_stack,
            stack_state,
        }) => {
            println!("Player is connected, continuing with processing WS");
            (player_stack, stack_state, market, market_state)
        }
        Ok(ConnectPlayerResponse::DoesNotExist) => {
            println!("Player does not exist, invalidating its cookies");
            let game_id_cookie = Cookie::build(("game_id", "".to_string()))
                .max_age(Duration::seconds(0))
                .same_site(SameSite::Strict)
                .path("/")
                .build();
            cookies.add(game_id_cookie);
            let player_id_cookie = Cookie::build(("player_id", "".to_string()))
                .max_age(Duration::seconds(0))
                .same_site(SameSite::Strict)
                .path("/")
                .build();
            cookies.add(player_id_cookie);
            let name_cookie = Cookie::build(("player_name", "".to_string()))
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
    let market = market.clone();
    let market_state = market_state.clone();
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
    game_id: String,
    name: String,
}

pub async fn join_game(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Json(input): Json<JoinGame>,
) -> impl IntoResponse {
    println!("{input:?}");
    if input.name.is_empty() || input.game_id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    let game_id = GameId::from(input.game_id);

    let Ok(domain) = env::var("DOMAIN") else {
        println!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let (tx_back, rx) = oneshot::channel();
    let _ = state
        .game_repository
        .send(GameRepositoryMessage::GetGame {
            game_id: game_id.clone(),
            tx_back,
        })
        .await;
    let Ok(GetGameResponse::Found(game)) = rx.await else {
        println!("No game found for ID: {game_id:?}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let (tx, rx) = oneshot::channel::<RegisterPlayerResponse>();

    let _ = game
        .send(GameMessage::RegisterPlayer {
            name: input.name.clone(),
            tx_back: tx,
        })
        .await;

    let player_id = match rx.await {
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

    let player_id_cookie = Cookie::build(("player_id", player_id.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain.clone())
        .path("/")
        .build();
    cookies.add(player_id_cookie);
    let game_id_cookie = Cookie::build(("game_id", game_id.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain.clone())
        .path("/")
        .build();
    cookies.add(game_id_cookie);
    let player_name_cookie = Cookie::build(("player_name", input.name.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain)
        .path("/")
        .build();
    cookies.add(player_name_cookie);
    println!("Registered player {} with id {player_id}", input.name);
    StatusCode::CREATED
}


