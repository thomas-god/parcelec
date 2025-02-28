use std::{collections::HashMap, env};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    game::{GameId, GameMessage, GameState, RegisterPlayerResponse},
    player::PlayerName,
};

use super::ApiState;

#[derive(Debug, Deserialize, Serialize)]
pub struct JoinGame {
    game_id: String,
    player_name: String,
}

pub async fn join_game(
    cookies: Cookies,
    State(state): State<ApiState>,
    Json(input): Json<JoinGame>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    let (Some(game_id), Some(player_name)) = (
        GameId::parse(&input.game_id),
        PlayerName::parse(&input.player_name),
    ) else {
        return StatusCode::BAD_REQUEST;
    };

    let Ok(domain) = env::var("DOMAIN") else {
        tracing::error!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let Some(game) = state.game_services.get(&game_id) else {
        return StatusCode::NOT_FOUND;
    };

    if *game.state_rx.borrow() != GameState::Open {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    let (tx, rx) = oneshot::channel();

    let _ = game
        .tx
        .send(GameMessage::RegisterPlayer {
            name: player_name.clone(),
            tx_back: tx,
        })
        .await;

    let (player_id, player_stack) = match rx.await {
        Ok(RegisterPlayerResponse::Success { id, stack }) => (id, stack),
        Ok(RegisterPlayerResponse::PlayerAlreadyExist) => {
            tracing::warn!("Player with name {} already exist", player_name);
            return StatusCode::CONFLICT;
        }
        Ok(RegisterPlayerResponse::GameStarted) => {
            tracing::warn!("Cannot register a player to a running game");
            return StatusCode::CONFLICT;
        }
        Err(err) => {
            tracing::error!("Error while sending message to game instance: {err:?}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    match state.stack_services.get_mut(&game_id) {
        Some(game_stacks) => {
            if game_stacks.get(&player_id).is_some() {
                tracing::error!(
                    "A stack already exist for player {player_id:?} in game {game_id:?}"
                );
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
    let player_name_cookie = Cookie::build(("player_name", player_name.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain)
        .path("/")
        .build();
    cookies.add(player_name_cookie);
    tracing::info!("Registered player {} with id {player_id}", player_name);
    StatusCode::CREATED
}

#[cfg(test)]
mod test_api_join_game {
    use crate::api::join_game::{join_game, JoinGame};
    use crate::api::{ApiState, AppState};
    use crate::game::{
        GameContext, GameId, GameMessage, GameName, GameState, RegisterPlayerResponse,
    };
    use crate::plants::actor::{StackContext, StackState};
    use crate::plants::StackService;
    use crate::player::PlayerId;
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Arc;
    use tokio::sync::{mpsc, watch, RwLock};
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;

    fn init_state() -> ApiState {
        let (tx, _) = mpsc::channel(16);
        Arc::new(RwLock::new(AppState {
            game_services: HashMap::new(),
            market_services: HashMap::new(),
            stack_services: HashMap::new(),
            player_connections_repository: tx,
        }))
    }

    fn start_game(id: GameId, state: GameState) -> (GameContext, mpsc::Receiver<GameMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let (_, state_rx) = watch::channel(state);
        (
            GameContext {
                id,
                name: GameName::default(),
                tx,
                state_rx,
            },
            rx,
        )
    }

    #[tokio::test]
    async fn test_join_game_success() {
        env::set_var("DOMAIN", "test.example.com");

        let state = init_state();
        let game_id = GameId::default();
        let (game, mut rx) = start_game(game_id.clone(), GameState::Open);
        {
            let mut s = state.write().await;
            s.game_services.insert(game_id.clone(), game);
        }

        let app = Router::new()
            .route("/join", post(join_game))
            .layer(CookieManagerLayer::new())
            .with_state(state);

        let request_body = JoinGame {
            game_id: game_id.to_string(),
            player_name: "TestPlayer".to_string(),
        };

        tokio::spawn(async move {
            if let Some(GameMessage::RegisterPlayer { name: _, tx_back }) = rx.recv().await {
                let player_id = PlayerId::default();
                let (_, state_rx) = watch::channel(StackState::Open);
                let (tx, _) = mpsc::channel(16);
                let service = StackService::new(tx);
                let player_stack = StackContext { service, state_rx };
                tx_back
                    .send(RegisterPlayerResponse::Success {
                        id: player_id,
                        stack: player_stack,
                    })
                    .unwrap();
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/join")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        // Check cookies
        let cookies = response
            .headers()
            .get_all(http::header::SET_COOKIE)
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(cookies.len(), 3);
    }

    #[tokio::test]
    async fn test_join_game_player_already_exists() {
        env::set_var("DOMAIN", "test.example.com");

        let state = init_state();
        let game_id = GameId::default();
        let (game, mut rx) = start_game(game_id.clone(), GameState::Open);
        {
            let mut s = state.write().await;
            s.game_services.insert(game_id.clone(), game);
        }

        let app = Router::new()
            .route("/join", post(join_game))
            .layer(CookieManagerLayer::new())
            .with_state(state);

        let request_body = JoinGame {
            game_id: game_id.to_string(),
            player_name: "TestPlayer".to_string(),
        };

        tokio::spawn(async move {
            if let Some(GameMessage::RegisterPlayer { name: _, tx_back }) = rx.recv().await {
                tx_back
                    .send(RegisterPlayerResponse::PlayerAlreadyExist)
                    .unwrap();
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/join")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_join_game_already_started() {
        env::set_var("DOMAIN", "test.example.com");

        let state = init_state();
        let game_id = GameId::default();
        let (game, mut rx) = start_game(game_id.clone(), GameState::Open);
        {
            let mut s = state.write().await;
            s.game_services.insert(game_id.clone(), game);
        }

        let app = Router::new()
            .route("/join", post(join_game))
            .layer(CookieManagerLayer::new())
            .with_state(state);

        let request_body = JoinGame {
            game_id: game_id.to_string(),
            player_name: "TestPlayer".to_string(),
        };

        tokio::spawn(async move {
            if let Some(GameMessage::RegisterPlayer { name: _, tx_back }) = rx.recv().await {
                tx_back.send(RegisterPlayerResponse::GameStarted).unwrap();
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/join")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_join_game_invalid_game_id() {
        env::set_var("DOMAIN", "test.example.com");

        let state = init_state();

        let app = Router::new()
            .route("/join", post(join_game))
            .layer(CookieManagerLayer::new())
            .with_state(state);

        let request_body = JoinGame {
            game_id: GameId::default().to_string(),
            player_name: "TestPlayer".to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/join")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
