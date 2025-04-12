use std::{collections::HashMap, time::Duration};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    constants::DEFAULT_PERIOD_DURATION_SECONDS,
    forecast::Forecast,
    game::{
        GameActor, GameId, GameName,
        delivery_period::DeliveryPeriodTimers,
        infra::GameActorConfig,
        scores::{GameRankings, TierLimits},
    },
    infra::api::state::cleanup_state,
    market::{MarketActor, bots::start_bots},
    plants::{
        PlantId,
        infra::actor::StackPlants,
        technologies::{
            battery::Battery, consumers::Consumers, gas_plant::GasPlant, nuclear::NuclearPlant,
            renewable::RenewablePlant,
        },
    },
    player::infra::PlayerConnectionsService,
    utils::program_actors_termination,
};

use super::ApiState;

#[derive(Debug, Deserialize, Clone, Copy)]
struct GasPlantConfig {
    max_power: isize,
    cost: isize,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct NuclearPlantConfig {
    max_power: isize,
    cost: isize,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct BatteryConfig {
    max_charge: usize,
}

#[derive(Debug, Deserialize, Clone)]
struct StackConfig {
    gas: GasPlantConfig,
    nuclear: NuclearPlantConfig,
    battery: BatteryConfig,
    consumers_forecasts: Vec<Forecast>,
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

    if request.stack.consumers_forecasts.len() != request.number_of_periods
        || request.stack.renewable_forecasts.len() != request.number_of_periods
    {
        warn!(
            "Invalid number of forecasts, expected {}, got ({}, {}) for consumers and renewable",
            request.number_of_periods,
            request.stack.consumers_forecasts.len(),
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
        name: Some(game_name.clone()),
        delivery_period_timers: Some(DeliveryPeriodTimers {
            market: Duration::from_secs(period_duration),
            stacks: Duration::from_secs(period_duration),
        }),
        number_of_delivery_periods: request.number_of_periods,
        ranking_calculator: GameRankings {
            tier_limits: Some(TierLimits {
                gold: 80_000,
                silver: 50_000,
                bronze: 25_000,
            }),
        },
        build_stack: stack_plants_builder(request.stack),
    };
    let game_context = GameActor::start(
        game_config,
        connections_service.clone(),
        market_context.clone(),
        cancellation_token,
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

fn stack_plants_builder(
    config: StackConfig,
) -> impl Fn() -> StackPlants + Clone + Send + Sync + 'static {
    move || {
        let StackConfig {
            gas,
            nuclear,
            battery,
            consumers_forecasts,
            renewable_forecasts,
        } = &config;
        let mut map: StackPlants = HashMap::new();
        map.insert(
            PlantId::default(),
            Box::new(Battery::new(battery.max_charge, 0)),
        );
        map.insert(
            PlantId::default(),
            Box::new(GasPlant::new(gas.cost, gas.max_power)),
        );
        map.insert(
            PlantId::default(),
            Box::new(NuclearPlant::new(nuclear.max_power, nuclear.cost)),
        );
        map.insert(
            PlantId::default(),
            Box::new(RenewablePlant::new(renewable_forecasts.clone())),
        );
        map.insert(
            PlantId::default(),
            Box::new(Consumers::new(56, consumers_forecasts.clone())),
        );
        map
    }
}
