use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::IntoResponse,
};
use connection::{PlayerConnectionContext, start_player_connection};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use crate::{
    game::GameId,
    market::Market,
    plants::Stack,
    player::{PlayerId, PlayerName},
};

use super::ApiState;

pub mod connection;

pub async fn handle_ws_connection(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let Some((player_id, player_name, game_id)) = extract_cookies(&cookies) else {
        invalidate_cookies(cookies);
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let state = state.read().await;
    let Some(game_context) = state.game_services.get(&game_id) else {
        invalidate_cookies(cookies);
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(market_context) = state.market_services.get(&game_id) else {
        invalidate_cookies(cookies);
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(stack_context) = state
        .stack_services
        .get(&game_id)
        .and_then(|game_stacks| game_stacks.get(&player_id))
    else {
        invalidate_cookies(cookies);
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let player_context = PlayerConnectionContext {
        game_id: game_id.clone(),
        player_id: player_id.clone(),
        player_name,
        game: game_context.clone(),
        market: market_context.clone(),
        stack: stack_context.clone(),
        connections_repository: state.player_connections_repository.clone(),
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

fn extract_cookies(cookies: &Cookies) -> Option<(PlayerId, PlayerName, GameId)> {
    let id = cookies
        .get("player_id")
        .map(|c| PlayerId::from(c.value_trimmed()))?;
    let name = cookies
        .get("player_name")
        .map(|c| PlayerName::from(c.value_trimmed()))?;
    let game_id = cookies
        .get("game_id")
        .map(|c| GameId::from(c.value_trimmed()))?;
    Some((id, name, game_id))
}

async fn handle_socket<MS: Market, PS: Stack>(
    socket: WebSocket,
    context: PlayerConnectionContext<MS, PS>,
) {
    tokio::spawn(async move {
        if let Err(err) = start_player_connection(socket, context).await {
            tracing::error!("Player connection ended: {err:?}");
        };
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        infra::api::{build_router, state::AppState},
        utils::config::AppConfig,
    };
    use axum::http::{Request, StatusCode};
    use std::{
        collections::HashMap,
        future::IntoFuture,
        net::{Ipv4Addr, SocketAddr},
        sync::Arc,
    };
    use tokio::sync::{RwLock, mpsc};
    use tokio_tungstenite::tungstenite::{Error, client::IntoClientRequest};

    async fn build_server() -> SocketAddr {
        let (tx_conn, _) = mpsc::channel(1024);
        let (cleanup_tx, _) = mpsc::channel(1024);
        let config = AppConfig::default();
        let state = Arc::new(RwLock::new(AppState {
            player_connections_repository: tx_conn,
            market_services: HashMap::new(),
            game_services: HashMap::new(),
            stack_services: HashMap::new(),
            cleanup_tx,
            config: config.clone(),
        }));
        let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(axum::serve(listener, build_router(state, config)).into_future());
        addr
    }

    fn default_request(addr: SocketAddr) -> Request<()> {
        let mut ws_request = format!("ws://{addr}/api/ws").into_client_request().unwrap();
        ws_request.headers_mut().insert("Cookie", "player_id=c083c29f-291d-4701-a9f0-50b9ed26b120; game_id=970b9e72-89be-4ba2-978a-4be4eb53ff51; player_name=tutorial_player".parse().unwrap());
        ws_request
    }

    #[tokio::test]
    async fn test_reject_ws_connection_game_cookie_not_found() {
        let addr = build_server().await;
        let mut ws_request = format!("ws://{addr}/api/ws").into_client_request().unwrap();
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
        let addr = build_server().await;
        let mut ws_request = format!("ws://{addr}/api/ws").into_client_request().unwrap();
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
        let addr = build_server().await;
        let mut ws_request = format!("ws://{addr}/api/ws").into_client_request().unwrap();
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
        let addr = build_server().await;
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
        let addr = build_server().await;
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
        let addr = build_server().await;
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
}
