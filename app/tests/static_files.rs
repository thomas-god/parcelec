//! Regression tests for the static-file serving fallback.
//!
//! These tests need a built client (`client/build`). When it is missing (e.g.
//! the Rust-only CI job) they are skipped instead of failing.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use parcelec_app::{AppConfig, build_router, new_api_state};
use tower::ServiceExt;

fn static_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap().join("../client/build")
}

fn build_app(static_dir: std::path::PathBuf) -> Router {
    let config = AppConfig {
        allow_origin: "http://localhost".to_string(),
        port: 0,
        static_dir,
    };
    let state = new_api_state(&config);
    build_router(state, config)
}

async fn get(app: &Router, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body_of(response: axum::response::Response) -> Vec<u8> {
    use http_body_util::BodyExt;
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

#[tokio::test]
async fn maps_routes_to_prerendered_html_like_nginx() {
    let dir = static_dir();
    if !dir.join("index.html").is_file() {
        eprintln!("skipping: client not built at {}", dir.display());
        return;
    }
    let app = build_app(dir.clone());

    // `$uri.html` resolution, including the `/game` directory/file conflict.
    for (url, file) in [
        ("/", "index.html"),
        ("/games", "games.html"),
        ("/game", "game.html"),
        ("/game/new", "game/new.html"),
        ("/tutorial", "tutorial.html"),
        // unknown routes fall back to the SPA shell
        ("/unknown/deep/link", "index.html"),
    ] {
        let response = get(&app, url).await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "unexpected status for {url}"
        );
        assert_eq!(
            body_of(response).await,
            std::fs::read(dir.join(file)).unwrap(),
            "{url} should map to {file}"
        );
    }
}

#[tokio::test]
async fn serves_assets_with_content_type() {
    let dir = static_dir();
    if !dir.join("index.html").is_file() {
        eprintln!("skipping: client not built at {}", dir.display());
        return;
    }
    let app = build_app(dir.clone());

    let css = std::fs::read_dir(dir.join("_app/immutable/assets"))
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "css"))
        .expect("a built css asset");

    let url = format!(
        "/_app/immutable/assets/{}",
        css.file_name().to_string_lossy()
    );
    let response = get(&app, &url).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/css");
}

#[tokio::test]
async fn unknown_api_routes_do_not_serve_the_spa() {
    let dir = static_dir();
    let app = build_app(dir);

    let response = get(&app, "/api/does-not-exist").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_path_traversal() {
    let dir = static_dir();
    let app = build_app(dir);

    let response = get(&app, "/../Cargo.toml").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
