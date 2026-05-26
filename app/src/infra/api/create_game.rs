use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    constants::DEFAULT_PERIOD_DURATION_SECONDS,
    forecast::Forecast,
    game::{
        GameActor, GameId, GameName,
        infra::{
            GameActorConfig,
            stack_config::{GameStackBaseConfig, GameStackCapacitiesConfig, GameStackConfig},
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

#[derive(Debug, Deserialize, Clone, Copy)]
struct GasPlantConfig {
    max_power: Power,
    cost: EnergyCost,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct NuclearPlantConfig {
    max_power: Power,
    cost: EnergyCost,
}

#[derive(Debug, Deserialize, Clone)]
struct ConsumersConfig {
    revenues: EnergyCost,
    forecasts: Vec<Forecast>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct BatteryConfig {
    max_charge: Energy,
}

#[derive(Debug, Deserialize, Clone)]
struct StackConfig {
    gas: GasPlantConfig,
    nuclear: NuclearPlantConfig,
    battery: BatteryConfig,
    consumers: ConsumersConfig,
    renewable_forecasts: Vec<Forecast>,
}

#[derive(Debug, Deserialize)]
pub struct NewGameRequest {
    game_name: String,
    period_duration_seconds: Option<u64>,
    number_of_periods: usize,
    stack: StackConfig,
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

    if request.stack.consumers.forecasts.len() != request.number_of_periods
        || request.stack.renewable_forecasts.len() != request.number_of_periods
    {
        warn!(
            "Invalid number of forecasts, expected {}, got ({}, {}) for consumers and renewable",
            request.number_of_periods,
            request.stack.consumers.forecasts.len(),
            request.stack.renewable_forecasts.len()
        );
        return Err(StatusCode::BAD_REQUEST);
    }

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
        stack_config: game_stack_config(request.stack),
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

fn game_stack_config(config: StackConfig) -> GameStackConfig {
    GameStackConfig::Fixed(
        GameStackBaseConfig {
            consumers_revenues: config.consumers.revenues,
            gas_cost: config.gas.cost,
            nuclear_cost: config.nuclear.cost,
        },
        GameStackCapacitiesConfig {
            gas_capacity: config.gas.max_power,
            nuclear_capcity: config.nuclear.max_power,
            battery_capacity: config.battery.max_charge,
            renewable_forecasts: config.renewable_forecasts,
            consumers_forecasts: config.consumers.forecasts,
        },
    )
}
