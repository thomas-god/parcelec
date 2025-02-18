use std::{env, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use tokio::sync::oneshot;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::game::{
    game_repository::{GameId, GameRepositoryMessage, GetGameResponse},
    game_service::AuthPlayerToGame,
    GameMessage, RegisterPlayerResponse,
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct JoinGame {
    game_id: String,
    name: String,
}

pub async fn join_game<GS: AuthPlayerToGame>(
    cookies: Cookies,
    State(state): State<Arc<AppState<GS>>>,
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

    let (tx, rx) = oneshot::channel();

    let _ = game
        .tx
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
        Ok(RegisterPlayerResponse::GameIsRunning) => {
            println!("Cannot register a player to a running game");
            return StatusCode::CONFLICT;
        }
        Err(err) => {
            println!("{err:?}");
            println!("Error while sending message to game instance");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let player_id_cookie = Cookie::build(("player_id", player_id.to_string()))
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
