use parcelec_app::{AppConfig, build_router, new_api_state};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        tracing::error!("Error while setting up tracing subscriber: {err:?}");
    };

    start_server().await;
}

async fn start_server() {
    let config = AppConfig::from_env();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Unable to start TCP listener");
    tracing::info!("Listenning on {addr}");

    let state = new_api_state(&config);
    let app = build_router(state, config);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
