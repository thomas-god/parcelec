use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{game::GameState, player::GameStackConfigView};

use super::ApiState;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
struct GameView {
    id: String,
    name: String,
    stack: GameStackConfigView,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ListGamesResponse {
    games: Vec<GameView>,
}

pub async fn list_games(State(state): State<ApiState>) -> impl IntoResponse {
    let state = state.read().await;

    Json(ListGamesResponse {
        games: state
            .game_services
            .iter()
            .filter_map(|(_, game)| {
                if *game.state_rx.borrow() == GameState::Open {
                    return Some(GameView {
                        id: game.id.to_string(),
                        name: game.name.to_string(),
                        stack: (&game.stack).into(),
                    });
                }
                None
            })
            .collect(),
    })
}

#[cfg(test)]
mod test_api_list_games {
    use crate::{
        game::{
            GameContext, GameId, GameName,
            delivery_period::DeliveryPeriodId,
            infra::stack_config::{GameStackConfig, GameStackFixedConfig},
        },
        infra::api::state::AppState,
        utils::{
            config::AppConfig,
            units::{Energy, EnergyCost, Power},
        },
    };

    use super::*;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{RwLock, mpsc, watch};

    fn init_state() -> ApiState {
        let (tx, _) = mpsc::channel(16);
        let (cleanup_tx, _) = mpsc::channel(16);
        Arc::new(RwLock::new(AppState {
            game_services: HashMap::new(),
            market_services: HashMap::new(),
            stack_services: HashMap::new(),
            player_connections_repository: tx,
            cleanup_tx,
            config: AppConfig::default(),
        }))
    }

    fn stack_config() -> GameStackConfig {
        GameStackConfig::Fixed(GameStackFixedConfig {
            battery_capacity: Energy::from(300),
            consumers_forecasts: vec![],
            consumers_forecasts_range: 2,
            consumers_revenues: EnergyCost::from(50),
            gas_capacity: Power::from(500),
            gas_cost: EnergyCost::from(80),
            nuclear_capacity: Power::from(1200),
            nuclear_cost: EnergyCost::from(35),
            renewable_forecasts: vec![],
            renewable_forecasts_range: 2,
        })
    }

    fn stack_config_view() -> GameStackConfigView {
        let config = stack_config();
        (&config).into()
    }

    fn start_game(id: GameId, name: GameName, state: GameState) -> GameContext {
        let (tx, _) = mpsc::channel(16);
        let (_, state_rx) = watch::channel(state);
        GameContext {
            id,
            name,
            stack: stack_config(),
            last_delivery_period: DeliveryPeriodId::from(3),
            tx,
            state_rx,
        }
    }

    #[tokio::test]
    async fn test_list_games_empty() {
        let state = init_state();

        let response = list_games(State(state)).await;
        let body = response
            .into_response()
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({"games": []}));
    }

    #[tokio::test]
    async fn test_list_games_with_open_games() {
        let state = init_state();

        // Add two open games
        for i in 0..2 {
            let game_id = GameId::from(i.to_string());
            let game_name = GameName::from(i.to_string());
            let ctx = start_game(game_id.clone(), game_name, GameState::Open);
            let mut s = state.write().await;
            s.game_services.insert(game_id, ctx);
        }

        let response = list_games(State(state)).await;
        let body = response
            .into_response()
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body: ListGamesResponse = serde_json::from_slice(&body).unwrap();
        assert!(body.games.contains(&GameView {
            id: 0.to_string(),
            name: 0.to_string(),
            stack: stack_config_view(),
        }));
        assert!(body.games.contains(&GameView {
            id: 1.to_string(),
            name: 1.to_string(),
            stack: stack_config_view(),
        },));
    }

    #[tokio::test]
    async fn test_list_games_filters_non_open_games() {
        let state = init_state();

        // Add games for each variant of GameState
        for game_state in [
            GameState::Open,
            GameState::Running {
                period: DeliveryPeriodId::from(0),
                end_at: None,
            },
            GameState::PostDelivery {
                period: DeliveryPeriodId::from(0),
                end_at: None,
            },
            GameState::Ended(DeliveryPeriodId::from(0)),
        ] {
            let game_id = GameId::from(game_state.to_string());
            let game_name = GameName::from(game_state.to_string());
            let ctx = start_game(game_id.clone(), game_name, game_state);
            let mut s = state.write().await;
            s.game_services.insert(game_id, ctx);
        }

        let response = list_games(State(state)).await;
        let body = response
            .into_response()
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        let body: ListGamesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            ListGamesResponse {
                games: vec![GameView {
                    id: GameState::Open.to_string(),
                    name: GameState::Open.to_string(),
                    stack: stack_config_view()
                },]
            }
        );
    }
}
