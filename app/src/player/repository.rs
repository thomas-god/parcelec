use std::collections::HashMap;

use futures_util::future::join_all;
use tokio::sync::mpsc;

use crate::game::GameId;

use super::{
    connection::{PlayerConnection, PlayerMessage},
    PlayerId,
};

#[derive(Debug)]
pub enum ConnectionRepositoryMessage {
    RegisterConnection(GameId, PlayerConnection),
    SendToPlayer(GameId, PlayerId, PlayerMessage),
    SendToAllPlayers(GameId, PlayerMessage),
}
pub struct PlayerConnectionRepository {
    players_connections: HashMap<GameId, Vec<PlayerConnection>>,
    rx: mpsc::Receiver<ConnectionRepositoryMessage>,
}

impl PlayerConnectionRepository {
    pub fn start() -> mpsc::Sender<ConnectionRepositoryMessage> {
        let (tx, rx) = mpsc::channel(128);

        let mut repo = PlayerConnectionRepository {
            players_connections: HashMap::new(),
            rx,
        };
        tokio::spawn(async move { repo.run().await });
        tx
    }

    async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                ConnectionRepositoryMessage::RegisterConnection(game_id, new_conn) => {
                    println!("registering player {:?}", new_conn.player_id);
                    match self.players_connections.get_mut(&game_id) {
                        Some(connections) => {
                            connections.push(new_conn);
                        }
                        None => {
                            self.players_connections.insert(game_id, vec![new_conn]);
                        }
                    }
                }
                ConnectionRepositoryMessage::SendToAllPlayers(game_id, message) => {
                    join_all(self.players_connections.get(&game_id).into_iter().flat_map(
                        |connections| connections.iter().map(|conn| conn.tx.send(message.clone())),
                    ))
                    .await;
                }
                ConnectionRepositoryMessage::SendToPlayer(game_id, player_id, message) => {
                    println!("{player_id:?}: proxying msg {message:?}");
                    join_all(self.players_connections.get(&game_id).into_iter().flat_map(
                        |connections| {
                            connections
                                .iter()
                                .filter(|conn| conn.player_id == player_id)
                                .map(|conn| conn.tx.send(message.clone()))
                        },
                    ))
                    .await;
                }
            }
            self.clean_dropped_connections();
        }
    }

    fn clean_dropped_connections(&mut self) {
        let _ = self
            .players_connections
            .values_mut()
            .map(|connections| connections.retain(|conn| !conn.tx.is_closed()));
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::{
        game::GameId,
        player::{
            connection::{PlayerConnection, PlayerMessage},
            PlayerId,
        },
    };

    use super::{ConnectionRepositoryMessage, PlayerConnectionRepository};

    async fn register_connection(
        game_id: &GameId,
        repository: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> (PlayerId, mpsc::Receiver<PlayerMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let player_id = PlayerId::default();
        let player_connection = PlayerConnection {
            id: Uuid::new_v4().to_string(),
            player_id: player_id.clone(),
            tx,
        };

        let _ = repository
            .send(ConnectionRepositoryMessage::RegisterConnection(
                game_id.clone(),
                player_connection,
            ))
            .await;
        (player_id, rx)
    }

    async fn register_connection_same_player(
        game_id: &GameId,
        player_id: &PlayerId,
        repository: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> mpsc::Receiver<PlayerMessage> {
        let (tx, rx) = mpsc::channel(16);
        let player_connection = PlayerConnection {
            id: Uuid::new_v4().to_string(),
            player_id: player_id.clone(),
            tx,
        };

        let _ = repository
            .send(ConnectionRepositoryMessage::RegisterConnection(
                game_id.clone(),
                player_connection,
            ))
            .await;
        rx
    }

    #[tokio::test]
    async fn test_register_player_connection() {
        let repository = PlayerConnectionRepository::start();

        let (tx, _) = mpsc::channel(16);
        let player_connection = PlayerConnection {
            id: "connection_id".to_string(),
            player_id: PlayerId::from("player_id"),
            tx,
        };

        let _ = repository
            .send(ConnectionRepositoryMessage::RegisterConnection(
                GameId::from("game_id"),
                player_connection,
            ))
            .await;
    }

    #[tokio::test]
    async fn test_send_message_to_registered_players() {
        let repository = PlayerConnectionRepository::start();
        let game_id = GameId::from("game_id");

        let (_, mut player1_rx) = register_connection(&game_id, repository.clone()).await;
        let (_, mut player2_rx) = register_connection(&game_id, repository.clone()).await;

        let _ = repository
            .send(ConnectionRepositoryMessage::SendToAllPlayers(
                game_id,
                PlayerMessage::OrderBookSnapshot {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ))
            .await;

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = player1_rx.recv().await
        else {
            unreachable!("Should have received a message");
        };
        assert_eq!(bids, Vec::new());
        assert_eq!(offers, Vec::new());

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = player2_rx.recv().await
        else {
            unreachable!("Should have received a message");
        };
        assert_eq!(bids, Vec::new());
        assert_eq!(offers, Vec::new());
    }

    #[tokio::test]
    async fn test_send_message_to_single_player() {
        let repository = PlayerConnectionRepository::start();
        let game_id = GameId::from("game_id");

        let (player1_id, mut player1_rx) = register_connection(&game_id, repository.clone()).await;
        let (_, mut player2_rx) = register_connection(&game_id, repository.clone()).await;

        let _ = repository
            .send(ConnectionRepositoryMessage::SendToPlayer(
                game_id,
                player1_id,
                PlayerMessage::OrderBookSnapshot {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ))
            .await;

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = player1_rx.recv().await
        else {
            unreachable!("Should have received a message");
        };
        assert_eq!(bids, Vec::new());
        assert_eq!(offers, Vec::new());

        tokio::select! {
        _ = player2_rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }

    #[tokio::test]
    async fn test_send_message_all_conn_same_player() {
        let repository = PlayerConnectionRepository::start();
        let game_id = GameId::from("game_id");

        let (player_id, mut player_rx_1) = register_connection(&game_id, repository.clone()).await;
        let mut player_rx_2 =
            register_connection_same_player(&game_id, &player_id, repository.clone()).await;

        let _ = repository
            .send(ConnectionRepositoryMessage::SendToPlayer(
                game_id,
                player_id,
                PlayerMessage::OrderBookSnapshot {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ))
            .await;

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = player_rx_1.recv().await
        else {
            unreachable!("Should have received a message");
        };
        assert_eq!(bids, Vec::new());
        assert_eq!(offers, Vec::new());

        let Some(PlayerMessage::OrderBookSnapshot { bids, offers }) = player_rx_2.recv().await
        else {
            unreachable!("Should have received a message");
        };
        assert_eq!(bids, Vec::new());
        assert_eq!(offers, Vec::new());
    }

    #[tokio::test]
    async fn test_should_handle_dropped_connections() {
        let repository = PlayerConnectionRepository::start();
        let game_id = GameId::from("game_id");

        let (_, mut player1_rx) = register_connection(&game_id, repository.clone()).await;
        let (_, mut player2_rx) = register_connection(&game_id, repository.clone()).await;

        let _ = repository
            .send(ConnectionRepositoryMessage::SendToAllPlayers(
                game_id.clone(),
                PlayerMessage::OrderBookSnapshot {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ))
            .await;

        let Some(PlayerMessage::OrderBookSnapshot { .. }) = player1_rx.recv().await else {
            unreachable!("Should have received a message");
        };

        player2_rx.close();

        let _ = repository
            .send(ConnectionRepositoryMessage::SendToAllPlayers(
                game_id,
                PlayerMessage::OrderBookSnapshot {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ))
            .await;
        let Some(PlayerMessage::OrderBookSnapshot { .. }) = player1_rx.recv().await else {
            unreachable!("Should have received a message");
        };
    }
}
