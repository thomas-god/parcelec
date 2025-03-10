use infra::api::{start_server, state::new_api_state};
use player::infra::PlayerConnectionRepository;

pub mod forecast;
pub mod game;
pub mod infra;
pub mod market;
pub mod plants;
pub mod player;
pub mod utils;

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
    let connections_repo = PlayerConnectionRepository::start();
    let app_state = new_api_state(connections_repo);

    start_server(app_state).await;
}
