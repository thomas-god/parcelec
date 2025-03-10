use tokio::sync::mpsc;

use crate::{
    game::GameId,
    player::{PlayerConnections, PlayerId, PlayerMessage},
};

use super::ConnectionRepositoryMessage;

#[derive(Debug, Clone)]
pub struct PlayerConnectionsService {
    tx: mpsc::Sender<ConnectionRepositoryMessage>,
}

impl PlayerConnectionsService {
    pub fn new(tx: mpsc::Sender<ConnectionRepositoryMessage>) -> PlayerConnectionsService {
        PlayerConnectionsService { tx }
    }
}

impl PlayerConnections for PlayerConnectionsService {
    async fn send_to_all_players(&self, game: &GameId, message: PlayerMessage) {
        let _ = self
            .tx
            .send(ConnectionRepositoryMessage::SendToAllPlayers(
                game.clone(),
                message,
            ))
            .await;
    }

    async fn send_to_player(&self, game: &GameId, player: &PlayerId, message: PlayerMessage) {
        let _ = self
            .tx
            .send(ConnectionRepositoryMessage::SendToPlayer(
                game.clone(),
                player.clone(),
                message,
            ))
            .await;
    }
}
