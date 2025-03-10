use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    game::{GameContext, GameId},
    market::{MarketContext, MarketService},
    plants::{StackService, actor::StackContext},
    player::{PlayerId, infra::ConnectionRepositoryMessage},
};

pub type ApiState = Arc<RwLock<AppState>>;
pub struct AppState {
    pub market_services: HashMap<GameId, MarketContext<MarketService>>,
    pub game_services: HashMap<GameId, GameContext>,
    pub stack_services: HashMap<GameId, HashMap<PlayerId, StackContext<StackService>>>,
    pub player_connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    pub cleanup_tx: mpsc::Sender<GameId>,
}

impl AppState {
    pub fn remove_game(&mut self, game_id: &GameId) {
        self.market_services.remove(game_id);
        self.game_services.remove(game_id);
        self.stack_services.remove(game_id);
    }
}

pub fn new_api_state(connections: mpsc::Sender<ConnectionRepositoryMessage>) -> ApiState {
    let (cleanup_tx, mut cleanup_rx) = mpsc::channel(128);

    let state = Arc::new(RwLock::new(AppState {
        game_services: HashMap::new(),
        market_services: HashMap::new(),
        stack_services: HashMap::new(),
        player_connections_repository: connections,
        cleanup_tx,
    }));

    let cloned_state = state.clone();
    tokio::spawn(async move {
        while let Some(game_id) = cleanup_rx.recv().await {
            let mut state = cloned_state.write().await;
            state.remove_game(&game_id);
            tracing::info!("AppState cleaned for game {game_id:?}");
        }
    });

    state
}

pub fn cleanup_state(game: GameId, token: CancellationToken, cleanup_tx: mpsc::Sender<GameId>) {
    tokio::spawn(async move {
        token.cancelled().await;
        if let Err(err) = cleanup_tx.send(game).await {
            tracing::warn!("Unable to send cleanup message to ApiState: {err:?}");
        }
    });
}
