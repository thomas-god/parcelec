use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{
    constants::DEFAULT_PERIOD_DURATION_SECONDS,
    forecast::generate_random_forecasts_shape,
    game::{
        GameActor, GameId, GameName,
        infra::{
            GameActorConfig,
            stack_config::{GameStackConfig, GameStackFixedConfig, GameStackPerPlayerBaseConfig},
        },
    },
    infra::api::state::cleanup_state,
    market::{MarketActor, bots::start_bots},
    player::infra::PlayerConnectionsService,
    utils::{
        program_actors_termination,
        units::{Energy, EnergyCost, Power},
    },
};

use super::ApiState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameStackPerPlayerBaseConfigRequest {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub consumers_revenues: EnergyCost,
    pub gas_max_capacity: Power,
    pub nuclear_max_capacity: Power,
    pub battery_max_capacity: Energy,
    pub consumers_max_capacity: Power,
    pub consumers_forecasts_range: usize,
    pub renewable_max_capacity: Power,
    pub renewable_forecasts_range: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameStackConfigRequest {
    Fixed(GameStackFixedConfig),
    PerPlayer(GameStackPerPlayerBaseConfigRequest),
}

impl From<GameStackConfigRequest> for GameStackConfig {
    fn from(value: GameStackConfigRequest) -> Self {
        match value {
            GameStackConfigRequest::Fixed(config) => GameStackConfig::Fixed(config),
            GameStackConfigRequest::PerPlayer(config) => {
                GameStackConfig::PerPlayer(GameStackPerPlayerBaseConfig {
                    gas_cost: config.gas_cost,
                    nuclear_cost: config.nuclear_cost,
                    consumers_revenues: config.consumers_revenues,
                    gas_max_capacity: config.gas_max_capacity,
                    nuclear_max_capacity: config.nuclear_max_capacity,
                    battery_max_capacity: config.battery_max_capacity,
                    consumers_max_abs_capacity: config.consumers_max_capacity,
                    renewable_max_capacity: config.renewable_max_capacity,
                    consumers_forecasts: generate_random_forecasts_shape(10),
                    consumers_forecasts_range: config.consumers_forecasts_range,
                    renewable_forecasts: generate_random_forecasts_shape(10),
                    renewable_forecasts_range: config.renewable_forecasts_range,
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewGameRequest {
    game_name: String,
    period_duration_seconds: Option<u64>,
    number_of_periods: usize,
    stack: GameStackConfigRequest,
}

#[derive(Debug, Serialize)]
struct NewGameSuccess {
    game_id: GameId,
    game_name: GameName,
}

pub async fn create_game(
    State(state): State<ApiState>,
    Json(request): Json<NewGameRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let Ok(game_name) = GameName::new(request.game_name) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut state = state.write().await;
    let game_id = GameId::default();
    let cancellation_token = program_actors_termination(Duration::from_secs(3600 * 24));
    cleanup_state(
        game_id.clone(),
        cancellation_token.clone(),
        state.cleanup_tx.clone(),
    );
    let connections_service =
        PlayerConnectionsService::new(state.player_connections_repository.clone());
    let market_context = MarketActor::start(
        &game_id,
        connections_service.clone(),
        cancellation_token.clone(),
    );
    let period_duration = request
        .period_duration_seconds
        .unwrap_or(DEFAULT_PERIOD_DURATION_SECONDS);

    let game_config = GameActorConfig {
        id: game_id.clone(),
        name: game_name.clone(),
        delivery_period_duration: Some(Duration::from_secs(period_duration)),
        number_of_delivery_periods: request.number_of_periods,
        stack_config: request.stack.into(),
    };
    let game_context = GameActor::start(
        game_config,
        connections_service.clone(),
        market_context.clone(),
        cancellation_token.clone(),
    );

    state
        .market_services
        .insert(game_id.clone(), market_context.clone());
    state
        .game_services
        .insert(game_id.clone(), game_context.clone());

    // Start the bots
    let cloned_market_context = market_context.clone();
    let cloned_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        start_bots(cloned_market_context, cloned_cancellation_token).await;
    });

    tracing::info!("Game {game_name:?} created");
    Ok((
        StatusCode::CREATED,
        Json(NewGameSuccess { game_id, game_name }),
    ))
}
