use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use tower_cookies::Cookies;
use tracing::info;

use crate::{
    game::{GameName, GameState},
    infra::api::{
        cookies::{extract_game_cookies, invalidate_game_cookies},
        state::ApiState,
    },
};

#[derive(Debug, Serialize)]
pub struct ActiveGame {
    name: GameName,
    state: String,
}

fn state_name(state: &GameState) -> String {
    match state {
        GameState::Open => "Open".to_string(),
        GameState::Running { .. } => "Running".to_string(),
        GameState::PostDelivery { .. } => "Running".to_string(),
        GameState::Ended(_) => "Ended".to_string(),
    }
}

pub async fn current_game(State(state): State<ApiState>, cookies: Cookies) -> impl IntoResponse {
    let Some((_player_id, _player_name, game_id)) = extract_game_cookies(&cookies) else {
        invalidate_game_cookies(&cookies);
        info!("No cookies");
        return StatusCode::NO_CONTENT.into_response();
    };

    let state = state.read().await;
    let Some(game_context) = state.game_services.get(&game_id) else {
        invalidate_game_cookies(&cookies);
        info!("No game found");
        return StatusCode::NO_CONTENT.into_response();
    };

    let state = game_context.state_rx.borrow();
    if let GameState::Ended(_) = *state {
        invalidate_game_cookies(&cookies);
        info!("Game ended");
        return StatusCode::NO_CONTENT.into_response();
    }

    Json(ActiveGame {
        name: game_context.name.clone(),
        state: state_name(&state),
    })
    .into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tokio::sync::{mpsc, watch};
    use tower::ServiceExt;

    use crate::{
        AppConfig,
        game::{GameContext, GameId, GameName, GameState, delivery_period::DeliveryPeriodId},
        infra::api::{build_router, state::new_api_state},
    };

    fn make_game_context(id: &str, game_state: GameState) -> (GameId, GameContext) {
        let game_id = GameId::from(id);
        let (tx, _rx) = mpsc::channel(1);
        let (_, state_rx) = watch::channel(game_state);
        let ctx = GameContext {
            id: game_id.clone(),
            name: GameName::new("test-game".to_string()).unwrap(),
            last_delivery_period: DeliveryPeriodId::from(4),
            tx,
            state_rx,
        };
        (game_id, ctx)
    }

    fn test_config() -> AppConfig {
        AppConfig {
            port: 0,
            allow_origin: "http://localhost:5173".to_string(),
        }
    }

    #[tokio::test]
    async fn returns_204_without_cookies() {
        let config = test_config();
        let state = new_api_state(&config);
        let app = build_router(state, config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/game")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn returns_204_when_game_not_in_state() {
        let config = test_config();
        let state = new_api_state(&config);
        let app = build_router(state, config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/game")
                    .header(
                        "cookie",
                        "player_id=player-1; player_name=Alice; game_id=unknown-game",
                    )
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn returns_204_when_game_is_ended() {
        let config = test_config();
        let state = new_api_state(&config);
        let (game_id, ctx) =
            make_game_context("ended-game", GameState::Ended(DeliveryPeriodId::from(4)));
        state
            .write()
            .await
            .game_services
            .insert(game_id.clone(), ctx);
        let app = build_router(state, config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/game")
                    .header(
                        "cookie",
                        format!("player_id=player-1; player_name=Alice; game_id={game_id}"),
                    )
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn returns_200_with_active_game_when_open() {
        let config = test_config();
        let state = new_api_state(&config);
        let (game_id, ctx) = make_game_context("open-game", GameState::Open);
        state
            .write()
            .await
            .game_services
            .insert(game_id.clone(), ctx);
        let app = build_router(state, config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/game")
                    .header(
                        "cookie",
                        format!("player_id=player-1; player_name=Alice; game_id={game_id}"),
                    )
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "test-game");
        assert_eq!(json["state"], "Open");
    }

    #[tokio::test]
    async fn returns_200_with_active_game_when_running() {
        let config = test_config();
        let state = new_api_state(&config);
        let (game_id, ctx) = make_game_context(
            "running-game",
            GameState::Running {
                period: DeliveryPeriodId::from(2),
                end_at: None,
            },
        );
        state
            .write()
            .await
            .game_services
            .insert(game_id.clone(), ctx);
        let app = build_router(state, config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/game")
                    .header(
                        "cookie",
                        format!("player_id=player-1; player_name=Alice; game_id={game_id}"),
                    )
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "test-game");
        assert_eq!(json["state"], "Running");
    }
}
