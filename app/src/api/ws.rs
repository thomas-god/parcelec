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
    game::GameId,
    market::Market,
    player::{
        connection::{start_player_connection, PlayerConnectionContext},
        PlayerId,
    },
};

use super::ApiState;

pub async fn handle_ws_connection(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let Some((player_id, game_id)) = extract_cookies(&cookies) else {
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

async fn handle_socket<MS: Market>(socket: WebSocket, context: PlayerConnectionContext<MS>) {
    tokio::spawn(async move {
        start_player_connection(socket, context).await;
    });
}

#[cfg(test)]
mod tests {
    use crate::api::{build_app, AppState};
    use axum::http::{Request, StatusCode};
    use std::{
        collections::HashMap,
        future::IntoFuture,
        net::{Ipv4Addr, SocketAddr},
        sync::Arc,
    };
    use tokio::sync::{mpsc, RwLock};
    use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Error};

    async fn build_server() -> SocketAddr {
        let (tx_conn, _) = mpsc::channel(1024);
        let state = Arc::new(RwLock::new(AppState {
            player_connections_repository: tx_conn,
            market_services: HashMap::new(),
            game_services: HashMap::new(),
            stack_services: HashMap::new(),
        }));
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
        let addr = build_server().await;
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
        let addr = build_server().await;
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
        let addr = build_server().await;
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
