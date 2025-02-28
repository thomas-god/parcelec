use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    bots::start_bots,
    game::{delivery_period::DeliveryPeriodId, scores::GameRankings, Game, GameId, GameName},
    market::MarketActor,
};

use super::ApiState;

#[derive(Debug, Deserialize)]
pub struct NewGameRequest {
    game_name: String,
}

#[derive(Debug, Serialize)]
struct NewGameSuccess {
    game_id: GameId,
    game_name: GameName,
}

pub async fn create_game(
    State(state): State<ApiState>,
    Json(input): Json<NewGameRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let Ok(game_name) = GameName::new(input.game_name) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let mut state = state.write().await;
    let game_id = GameId::default();
    let market_context = MarketActor::start(&game_id, state.player_connections_repository.clone());
    let last_delivery_period = DeliveryPeriodId::from(4);
    let game_context = Game::start(
        &game_id,
        Some(game_name.clone()),
        state.player_connections_repository.clone(),
        market_context.clone(),
        Some(last_delivery_period),
        GameRankings { tier_limits: None },
    );

    state
        .market_services
        .insert(game_id.clone(), market_context.clone());
    state
        .game_services
        .insert(game_id.clone(), game_context.clone());

    // Start the bots
    let cloned_market_context = market_context.clone();
    tokio::spawn(async move {
        start_bots(cloned_market_context).await;
    });

    tracing::info!("Game {game_name:?} created");
    Ok((
        StatusCode::CREATED,
        Json(NewGameSuccess { game_id, game_name }),
    ))
}
