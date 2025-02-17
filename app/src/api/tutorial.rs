use std::{env, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tokio::sync::oneshot;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    bots::start_bots,
    game::{
        game_repository::{CreateNewGameResponse, GameRepositoryMessage},
        GameMessage, RegisterPlayerResponse,
    },
};

use super::AppState;

pub async fn create_tutorial_game(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Create a new game
    let (tx_back, rx) = oneshot::channel();
    let _ = state
        .game_repository
        .send(GameRepositoryMessage::CreateNewGame { tx_back })
        .await;
    let Ok(CreateNewGameResponse {
        game_id,
        game_context,
    }) = rx.await
    else {
        println!("Unable to create a game");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    // Start the bots
    let cloned_game_context = game_context.clone();
    tokio::spawn(async move {
        start_bots(cloned_game_context).await;
    });

    // Register a player for this game
    let player_name = "tutorial_player".to_string();
    let (tx_back, rx) = oneshot::channel();
    let _ = game_context
        .tx
        .send(GameMessage::RegisterPlayer {
            name: player_name.clone(),
            tx_back,
        })
        .await;
    let Ok(RegisterPlayerResponse::Success { id: player_id }) = rx.await else {
        println!("Unable to register tutorial player");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    // Start the game
    let _ = game_context
        .tx
        .send(GameMessage::PlayerIsReady(player_id.clone()))
        .await;

    // Write cookies back
    let Ok(domain) = env::var("DOMAIN") else {
        println!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
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
    let name_cookie = Cookie::build(("player_name", player_name.clone()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain)
        .path("/")
        .build();
    cookies.add(name_cookie);
    println!("Tutorial game created");
    StatusCode::CREATED
}
