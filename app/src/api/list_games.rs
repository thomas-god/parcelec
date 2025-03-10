use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::game::GameState;

use super::ApiState;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
struct GameView {
    id: String,
    name: String,
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
        api::state::AppState,
        game::{GameContext, GameId, GameName, delivery_period::DeliveryPeriodId},
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
        }))
    }

    fn start_game(id: GameId, name: GameName, state: GameState) -> GameContext {
        let (tx, _) = mpsc::channel(16);
        let (_, state_rx) = watch::channel(state);
        GameContext {
            id,
            name,
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
        let mut body: ListGamesResponse = serde_json::from_slice(&body).unwrap();
        let games = body.games.sort();
        let expected_games = vec![
            GameView {
                id: 0.to_string(),
                name: 0.to_string(),
            },
            GameView {
                id: 1.to_string(),
                name: 1.to_string(),
            },
        ]
        .sort();
        assert_eq!(games, expected_games);
    }

    #[tokio::test]
    async fn test_list_games_filters_non_open_games() {
        let state = init_state();

        // Add games for each variant of GameState
        for game_state in [
            GameState::Open,
            GameState::Running(DeliveryPeriodId::from(0)),
            GameState::PostDelivery(DeliveryPeriodId::from(0)),
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
                    name: GameState::Open.to_string()
                },]
            }
        );
    }
}
