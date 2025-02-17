use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::{
    game::game_repository::GameId,
    models::{AuthPlayerToGame, AuthPlayerToGameError},
    player::{
        connection::{start_player_connection, PlayerConnectionContext},
        PlayerId,
    },
};

use super::AppState;

pub async fn handle_ws_connection<GS: AuthPlayerToGame>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<GS>>>,
    cookies: Cookies,
) -> impl IntoResponse {
    let Some((player_id, game_id)) = extract_cookies(&cookies) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let player_context = match state.game_service.auth_player(&game_id, &player_id).await {
        Ok(context) => context,
        Err(AuthPlayerToGameError::NoGameFound)
        | Err(AuthPlayerToGameError::NoPlayerFound)
        | Err(AuthPlayerToGameError::NoStackFound) => {
            invalidate_cookies(cookies);
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(_) => {
            invalidate_cookies(cookies);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, player_context))
}

fn invalidate_cookies(cookies: Cookies) {
    let game_id_cookie = Cookie::build(("game_id", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(game_id_cookie);
    let player_id_cookie = Cookie::build(("player_id", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(player_id_cookie);
    let name_cookie = Cookie::build(("player_name", "".to_string()))
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Strict)
        .path("/")
        .build();
    cookies.add(name_cookie);
}

fn extract_cookies(cookies: &Cookies) -> Option<(PlayerId, GameId)> {
    let id = cookies
        .get("player_id")
        .map(|c| PlayerId::from(c.value_trimmed()))?;
    let _ = cookies
        .get("player_name")
        .map(|c| c.value_trimmed().to_string())?;
    let game_id = cookies
        .get("game_id")
        .map(|c| GameId::from(c.value_trimmed()))?;
    Some((id, game_id))
}

async fn handle_socket(socket: WebSocket, context: PlayerConnectionContext) {
    tokio::spawn(async move {
        start_player_connection(socket, context).await;
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{build_app, AppState},
        models::{AuthPlayerToGame, AuthPlayerToGameError},
    };
    use axum::http::{Request, StatusCode};
    use std::{
        future::IntoFuture,
        net::{Ipv4Addr, SocketAddr},
        sync::Arc,
    };
    use tokio::sync::mpsc;
    use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Error};

    #[derive(Debug, Clone)]
    struct MockedGameService {
        res: Result<
            crate::player::connection::PlayerConnectionContext,
            crate::models::AuthPlayerToGameError,
        >,
    }
    impl AuthPlayerToGame for MockedGameService {
        async fn auth_player(
            &self,
            _game_id: &crate::game::game_repository::GameId,
            _player_id: &crate::player::PlayerId,
        ) -> Result<
            crate::player::connection::PlayerConnectionContext,
            crate::models::AuthPlayerToGameError,
        > {
            self.res.clone()
        }
    }

    async fn build_server(gs: MockedGameService) -> SocketAddr {
        let (tx_conn, _) = mpsc::channel(1024);
        let (tx_games, _) = mpsc::channel(1024);
        let state = Arc::new(AppState {
            game_service: gs,
            game_repository: tx_games,
            player_connections_repository: tx_conn,
        });
        let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(axum::serve(listener, build_app(state, "origin".to_string())).into_future());
        addr
    }

    fn default_request(addr: SocketAddr) -> Request<()> {
        let mut ws_request = format!("ws://{addr}/ws").into_client_request().unwrap();
        ws_request.headers_mut().insert("Cookie", "player_id=c083c29f-291d-4701-a9f0-50b9ed26b120; game_id=970b9e72-89be-4ba2-978a-4be4eb53ff51; player_name=tutorial_player".parse().unwrap());
        ws_request
    }

    #[tokio::test]
    async fn test_reject_ws_connection_game_cookie_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoGameFound),
        };
        let addr = build_server(game_service).await;
        let mut ws_request = format!("ws://{addr}/ws").into_client_request().unwrap();
        ws_request.headers_mut().insert(
            "Cookie",
            "player_id=c083c29f-291d-4701-a9f0-50b9ed26b120; player_name=tutorial_player"
                .parse()
                .unwrap(),
        );

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_player_id_cookie_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoGameFound),
        };
        let addr = build_server(game_service).await;
        let mut ws_request = format!("ws://{addr}/ws").into_client_request().unwrap();
        ws_request.headers_mut().insert(
            "Cookie",
            "game_id=c083c29f-291d-4701-a9f0-50b9ed26b120; player_name=tutorial_player"
                .parse()
                .unwrap(),
        );

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_player_name_cookie_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoGameFound),
        };
        let addr = build_server(game_service).await;
        let mut ws_request = format!("ws://{addr}/ws").into_client_request().unwrap();
        ws_request.headers_mut().insert(
            "Cookie",
            "game_id=c083c29f-291d-4701-a9f0-50b9ed26b120; player_id=tutorial_player"
                .parse()
                .unwrap(),
        );

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_if_game_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoGameFound),
        };
        let addr = build_server(game_service).await;
        let ws_request = default_request(addr);

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_if_player_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoPlayerFound),
        };
        let addr = build_server(game_service).await;
        let ws_request = default_request(addr);

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_if_player_stack_not_found() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::NoStackFound),
        };
        let addr = build_server(game_service).await;
        let ws_request = default_request(addr);

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_reject_ws_connection_if_player_auth_fails() {
        let game_service = MockedGameService {
            res: Err(AuthPlayerToGameError::Unknown),
        };
        let addr = build_server(game_service).await;
        let ws_request = default_request(addr);

        let res = tokio_tungstenite::connect_async(ws_request)
            .await
            .expect_err("Should have rejected the WS connection");
        match res {
            Error::Http(resp) => {
                assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
            }
            _ => unreachable!(),
        }
    }
}
