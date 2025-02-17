use std::{collections::HashMap, fmt};

use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::player::repository::ConnectionRepositoryMessage;

use super::{Game, GameContext};

pub enum GameRepositoryMessage {
    CreateNewGame {
        tx_back: oneshot::Sender<CreateNewGameResponse>,
    },
    GetGame {
        game_id: GameId,
        tx_back: oneshot::Sender<GetGameResponse>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId(String);
impl GameId {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Default for GameId {
    fn default() -> Self {
        GameId(Uuid::new_v4().to_string())
    }
}
impl From<String> for GameId {
    fn from(value: String) -> Self {
        GameId(value)
    }
}
impl From<&str> for GameId {
    fn from(value: &str) -> Self {
        GameId(value.to_string())
    }
}
impl AsRef<str> for GameId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct CreateNewGameResponse {
    pub game_id: GameId,
    pub game_context: GameContext,
}

pub enum GetGameResponse {
    Found(GameContext),
    NotFound,
}

pub struct GameRepository {
    player_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    games: HashMap<GameId, GameContext>,
    rx: mpsc::Receiver<GameRepositoryMessage>,
}

impl GameRepository {
    pub fn start(
        player_connections: &mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> mpsc::Sender<GameRepositoryMessage> {
        let (tx, rx) = mpsc::channel(16);

        let mut repo = GameRepository {
            player_connections: player_connections.clone(),
            games: HashMap::new(),
            rx,
        };
        tokio::spawn(async move { repo.run().await });
        tx
    }

    async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                GameRepositoryMessage::CreateNewGame { tx_back } => {
                    let game_id = GameId::default();
                    let mut game =
                        Game::new(game_id.clone(), self.player_connections.clone()).await;
                    let game_context = game.get_context();

                    self.games.insert(game_id.clone(), game_context.clone());

                    tokio::spawn(async move { game.run().await });

                    let _ = tx_back.send(CreateNewGameResponse {
                        game_id,
                        game_context,
                    });
                }
                GameRepositoryMessage::GetGame { game_id, tx_back } => {
                    let _ = tx_back.send(match self.games.get(&game_id) {
                        Some(game) => GetGameResponse::Found(game.clone()),
                        None => GetGameResponse::NotFound,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{mpsc, oneshot};

    use crate::{
        game::{
            game_repository::{
                CreateNewGameResponse, GameId, GameRepository, GameRepositoryMessage,
                GetGameResponse,
            },
            GameMessage,
        },
        player::PlayerId,
    };

    #[tokio::test]
    async fn test_create_game() {
        let (tx, _) = mpsc::channel(16);
        let repository_tx = GameRepository::start(&tx);

        let (tx_back, rx) = oneshot::channel();
        let _ = repository_tx
            .send(GameRepositoryMessage::CreateNewGame { tx_back })
            .await;

        let Ok(CreateNewGameResponse { .. }) = rx.await else {
            unreachable!("Should have received a game ID")
        };
    }

    #[tokio::test]
    async fn test_created_game_should_be_running() {
        let (tx, _) = mpsc::channel(16);
        let repository_tx = GameRepository::start(&tx);

        let (tx_back, rx) = oneshot::channel();
        let _ = repository_tx
            .send(GameRepositoryMessage::CreateNewGame { tx_back })
            .await;
        let Ok(CreateNewGameResponse { game_context, .. }) = rx.await else {
            unreachable!("Should have received a game ID")
        };

        let (tx_back, rx) = oneshot::channel();
        let _ = game_context
            .tx
            .send(GameMessage::ConnectPlayer {
                id: PlayerId::from("toto"),
                tx_back,
            })
            .await;
        let Ok(_) = rx.await else {
            unreachable!("Should have received any message");
        };
    }

    #[tokio::test]
    async fn test_connect_to_existing_game() {
        let (tx, _) = mpsc::channel(16);
        let repository_tx = GameRepository::start(&tx);

        let (tx_back, rx) = oneshot::channel();
        let _ = repository_tx
            .send(GameRepositoryMessage::CreateNewGame { tx_back })
            .await;

        let Ok(CreateNewGameResponse { game_id, .. }) = rx.await else {
            unreachable!("Should have received a game ID")
        };

        let (tx_back, rx) = oneshot::channel();
        let _ = repository_tx
            .send(GameRepositoryMessage::GetGame { tx_back, game_id })
            .await;

        let Ok(GetGameResponse::Found(_)) = rx.await else {
            unreachable!("Should have found a game")
        };
    }

    #[tokio::test]
    async fn test_connect_to_non_existing_game() {
        let (tx, _) = mpsc::channel(16);
        let repository_tx = GameRepository::start(&tx);

        let non_existing_game_id = GameId::default();
        let (tx_back, rx) = oneshot::channel();
        let _ = repository_tx
            .send(GameRepositoryMessage::GetGame {
                tx_back,
                game_id: non_existing_game_id,
            })
            .await;

        let Ok(GetGameResponse::NotFound) = rx.await else {
            unreachable!("Should not have found a game")
        };
    }
}
