#![allow(dead_code)]
use std::{io, time::Duration};

use fantoccini::{Client, ClientBuilder};
use parcelec_app::{AppConfig, new_api_state};
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;

pub const TESTRUN_SETUP_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
pub const WEBDRIVER_ADDRESS: &str = "http://localhost:4444";

pub async fn init_webdriver_client() -> Client {
    let mut firefox_args = Vec::new();
    firefox_args.extend(["--headless", "--disable-gpu", "--disable-dev-shm-usage"]);

    let mut caps = serde_json::map::Map::new();
    caps.insert(
        "moz:firefoxOptions".to_string(),
        serde_json::json!({
            "args": firefox_args,
        }),
    );
    ClientBuilder::native()
        .capabilities(caps)
        .connect(WEBDRIVER_ADDRESS)
        .await
        .expect("web driver to be available")
}

type ServerTaskHandle = JoinHandle<Result<(), io::Error>>;

pub async fn init() -> (String, ServerTaskHandle) {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let handle = tokio::spawn(async move {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
        let config = AppConfig {
            port: addr.port(),
            allow_origin: addr.to_string(),
            domain: String::from("localhost"),
        };
        let state = new_api_state(&config);
        let app = parcelec_app::build_router(state, config);
        let app = app.fallback_service(ServeDir::new(
            std::env::current_dir().unwrap().join("../client/build"),
        ));
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let assigned_addr = listener.local_addr().unwrap();
        tx.send(assigned_addr).unwrap();
        axum::serve(listener, app.into_make_service()).await
    });

    let assigned_addr = tokio::time::timeout(TESTRUN_SETUP_TIMEOUT, rx)
        .await
        .expect("test setup to not have timed out")
        .expect("socket address to have been received from the channel");
    let app_addr = format!("http://localhost:{}", assigned_addr.port());
    (app_addr, handle)
}
