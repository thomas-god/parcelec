use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use tokio::sync::oneshot;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    game::{
        game_repository::{GameId, GameRepositoryMessage, GetGameResponse},
        ConnectPlayerResponse, GameMessage,
    },
    player::{start_player_connection, PlayerConnectionContext},
};

use super::AppState;

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
        .tx
        .send(GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await;

    let (game, market, player_stack) = match rx.await {
        Ok(ConnectPlayerResponse::OK {
            game,
            market,
            player_stack,
        }) => {
            println!("Player is connected, continuing with processing WS");
            (game, market, player_stack)
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
    let context = PlayerConnectionContext {
        player_id: id,
        game,
        market,
        stack: player_stack,
    };
    ws.on_upgrade(move |socket| handle_socket(socket, context))
}

async fn handle_socket(socket: WebSocket, context: PlayerConnectionContext) {
    tokio::spawn(async move {
        start_player_connection(socket, context).await;
    });
}
