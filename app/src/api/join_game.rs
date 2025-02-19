use std::{collections::HashMap, env};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use tokio::sync::oneshot;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::game::{GameId, GameMessage, RegisterPlayerResponse};

use super::ApiState;

#[derive(Debug, Deserialize)]
pub struct JoinGame {
    game_id: String,
    name: String,
}

pub async fn join_game(
    cookies: Cookies,
    State(state): State<ApiState>,
    Json(input): Json<JoinGame>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    println!("{input:?}");
    if input.name.is_empty() || input.game_id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    let game_id = GameId::from(input.game_id);

    let Ok(domain) = env::var("DOMAIN") else {
        println!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    // let (tx_back, rx) = oneshot::channel();
    // let _ = state
    //     .game_repository
    //     .send(GameRepositoryMessage::GetGame {
    //         game_id: game_id.clone(),
    //         tx_back,
    //     })
    //     .await;
    // let Ok(GetGameResponse::Found(game)) = rx.await else {
    //     println!("No game found for ID: {game_id:?}");
    //     return StatusCode::INTERNAL_SERVER_ERROR;
    // };
    let Some(game) = state.game_services.get(&game_id) else {
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

    let (player_id, player_stack) = match rx.await {
        Ok(RegisterPlayerResponse::Success { id, stack }) => (id, stack),
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
    match state.stack_services.get_mut(&game_id) {
        Some(game_stacks) => {
            if game_stacks.get(&player_id).is_some() {
                println!("A stack already exist for player {player_id:?} in game {game_id:?}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            game_stacks.insert(player_id.clone(), player_stack.clone());
        }
        None => {
            let game_stacks = HashMap::from([(player_id.clone(), player_stack)]);
            state.stack_services.insert(game_id.clone(), game_stacks);
        }
    }

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
