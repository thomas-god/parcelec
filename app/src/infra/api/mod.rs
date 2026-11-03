use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{
        HeaderValue, Method, Request, StatusCode,
        header::{CONTENT_TYPE, COOKIE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use create_game::create_game;
use current_game::current_game;
use join_game::join_game;
use list_games::list_games;
use state::ApiState;
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tutorial::create_tutorial_game;
use ws::handle_ws_connection;

use crate::AppConfig;

mod cookies;
mod create_game;
mod current_game;
mod join_game;
mod list_games;
pub mod state;
mod tutorial;
pub mod ws;

/// State used by the static files fallback handler.
#[derive(Clone)]
struct StaticFiles {
    dir: Arc<PathBuf>,
}

pub fn build_router(state: ApiState, config: AppConfig) -> Router {
    let static_files = StaticFiles {
        dir: Arc::new(config.static_dir.clone()),
    };

    Router::new()
        .route("/api/game", post(create_game).get(current_game))
        .route("/api/games", get(list_games))
        .route("/api/game/join", post(join_game))
        .route("/api/tutorial", post(create_tutorial_game))
        .route("/api/ws", get(handle_ws_connection))
        .with_state(state)
        .fallback_service(
            // Serve existing files directly (assets, favicons, ...). Directory
            // redirects are disabled so that a path that is both a directory and
            // an `.html` page (e.g. `/game` vs `game.html`) is resolved by the
            // fallback below instead of being redirected to `/game/`.
            ServeDir::new(config.static_dir.clone())
                .redirect_to_trailing_slash(false)
                .fallback(get(serve_static).with_state(static_files)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_headers([CONTENT_TYPE, COOKIE])
                .allow_origin(
                    config
                        .allow_origin
                        .parse::<HeaderValue>()
                        .unwrap_or_else(|_| {
                            panic!(
                                "Could not parse allow_config ({}) into a HeaderValue",
                                config.allow_origin
                            )
                        }),
                )
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_credentials(true),
        )
}

/// Reproduces nginx's `try_files $uri $uri.html $uri/index.html /index.html`
/// behavior for the SPA: try the exact path, then the `.html` variant, then the
/// directory index, and finally fall back to the application shell.
async fn serve_static(State(files): State<StaticFiles>, req: Request<Body>) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    // Never return the SPA shell for unknown API routes.
    if path == "api" || path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Refuse path traversal attempts (`ServeDir` already rejects these, but the
    // fallback must not reintroduce them when building candidate paths).
    if !is_safe_relative_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let candidates = [
        files.dir.join(path),
        files.dir.join(format!("{path}.html")),
        files.dir.join(path).join("index.html"),
    ];

    let mut selected = None;
    for candidate in &candidates {
        if tokio::fs::metadata(candidate)
            .await
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
        {
            selected = Some(candidate.clone());
            break;
        }
    }

    let file = selected.unwrap_or_else(|| files.dir.join("index.html"));

    ServeFile::new(file)
        .oneshot(req)
        .await
        .map(|response| response.map(Body::new))
        .unwrap_or_else(|never| match never {})
}

/// Only allow paths made of normal components, mirroring `ServeDir`'s validation.
fn is_safe_relative_path(path: &str) -> bool {
    Path::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
