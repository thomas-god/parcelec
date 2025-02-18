use tokio::sync::{mpsc, oneshot};

use crate::player::{
    connection::PlayerConnectionContext, repository::ConnectionRepositoryMessage, PlayerId,
};

use super::{
    game_repository::{GameId, GameRepositoryMessage, GetGameResponse},
    ConnectPlayerResponse, GameMessage,
};
use std::future::Future;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthPlayerToGameError {
    NoGameFound,
    NoPlayerFound,
    NoStackFound,
    Unknown,
}

pub trait AuthPlayerToGame: Clone + Send + Sync + 'static {
    fn auth_player(
        &self,
        game_id: &GameId,
        player_id: &PlayerId,
    ) -> impl Future<Output = Result<PlayerConnectionContext, AuthPlayerToGameError>> + Send;
}

#[derive(Debug, Clone)]
pub struct GameService {
    connections_repo: mpsc::Sender<ConnectionRepositoryMessage>,
    games_repo: mpsc::Sender<GameRepositoryMessage>,
}

impl GameService {
    pub fn new(
        connections: &mpsc::Sender<ConnectionRepositoryMessage>,
        games: &mpsc::Sender<GameRepositoryMessage>,
    ) -> GameService {
        GameService {
            connections_repo: connections.clone(),
            games_repo: games.clone(),
        }
    }
}

impl AuthPlayerToGame for GameService {
    async fn auth_player(
        &self,
        game_id: &GameId,
        player_id: &PlayerId,
    ) -> Result<PlayerConnectionContext, AuthPlayerToGameError> {
        let (tx_back, rx) = oneshot::channel();
        let _ = self
            .games_repo
            .send(GameRepositoryMessage::GetGame {
                game_id: game_id.clone(),
                tx_back,
            })
            .await;
        let Ok(GetGameResponse::Found(game)) = rx.await else {
            println!("No game found for ID: {game_id:?}");
            return Err(AuthPlayerToGameError::NoGameFound);
        };

        let (tx, rx) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::ConnectPlayer {
                id: player_id.clone(),
                tx_back: tx,
            })
            .await;

        match rx.await {
            Ok(ConnectPlayerResponse::OK {
                game,
                market,
                player_stack,
            }) => {
                println!("Player is connected, continuing with processing WS");
                Ok(PlayerConnectionContext {
                    game_id: game_id.clone(),
                    player_id: player_id.clone(),
                    connections_repository: self.connections_repo.clone(),
                    game,
                    market,
                    stack: player_stack,
                })
            }
            Ok(ConnectPlayerResponse::DoesNotExist) => {
                println!("Player does not exist, invalidating its cookies");
                Err(AuthPlayerToGameError::NoPlayerFound)
            }
            Ok(ConnectPlayerResponse::NoStackFound) => {
                println!("Player exists but has no matching stack");
                Err(AuthPlayerToGameError::NoStackFound)
            }
            Err(err) => {
                println!("Something went wrong: {err:?}");
                Err(AuthPlayerToGameError::Unknown)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{mpsc, watch};

    use crate::{
        game::{
            game_repository::{GameId, GameRepositoryMessage, GetGameResponse},
            game_service::{AuthPlayerToGame, AuthPlayerToGameError, GameService},
            ConnectPlayerResponse, GameContext, GameMessage, GameState,
        },
        market::{MarketContext, MarketState},
        plants::stack::{StackContext, StackState},
        player::{connection::PlayerConnectionContext, PlayerId},
    };

    #[tokio::test]
    async fn test_err_when_no_game_found() {
        let (conn_tx, _) = mpsc::channel(1024);
        let (games_tx, mut games_rx) = mpsc::channel(1024);
        let service = GameService::new(&conn_tx, &games_tx);

        // Respond with game not found
        tokio::spawn(async move {
            let Some(GameRepositoryMessage::GetGame {
                game_id: _game_id,
                tx_back,
            }) = games_rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(GetGameResponse::NotFound);
        });

        let res = service
            .auth_player(&GameId::default(), &PlayerId::default())
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AuthPlayerToGameError::NoGameFound);
    }

    #[tokio::test]
    async fn test_err_player_does_not_exist() {
        let (conn_tx, _) = mpsc::channel(1024);
        let (games_tx, mut games_rx) = mpsc::channel(1024);
        let (game_tx, mut game_rx) = mpsc::channel(1024);
        let (_, game_state_rx) = watch::channel(GameState::Open);
        let service = GameService::new(&conn_tx, &games_tx);

        // Respond with game context
        tokio::spawn(async move {
            let Some(GameRepositoryMessage::GetGame {
                game_id: _game_id,
                tx_back,
            }) = games_rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(GetGameResponse::Found(GameContext {
                tx: game_tx,
                state_rx: game_state_rx,
            }));
        });

        // Respond with player not found
        tokio::spawn(async move {
            let Some(GameMessage::ConnectPlayer { id: _, tx_back }) = game_rx.recv().await else {
                unreachable!()
            };
            let _ = tx_back.send(ConnectPlayerResponse::DoesNotExist);
        });

        // Should return err
        let res = service
            .auth_player(&GameId::default(), &PlayerId::default())
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AuthPlayerToGameError::NoPlayerFound);
    }

    #[tokio::test]
    async fn test_err_player_stack_does_not_exist() {
        let (conn_tx, _) = mpsc::channel(1024);
        let (games_tx, mut games_rx) = mpsc::channel(1024);
        let (game_tx, mut game_rx) = mpsc::channel(1024);
        let (_, game_state_rx) = watch::channel(GameState::Open);
        let service = GameService::new(&conn_tx, &games_tx);

        // Respond with game context
        tokio::spawn(async move {
            let Some(GameRepositoryMessage::GetGame {
                game_id: _game_id,
                tx_back,
            }) = games_rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(GetGameResponse::Found(GameContext {
                tx: game_tx,
                state_rx: game_state_rx,
            }));
        });

        // Respond with player's stack not found
        tokio::spawn(async move {
            let Some(GameMessage::ConnectPlayer { id: _, tx_back }) = game_rx.recv().await else {
                unreachable!()
            };
            let _ = tx_back.send(ConnectPlayerResponse::NoStackFound);
        });

        // Should return err
        let res = service
            .auth_player(&GameId::default(), &PlayerId::default())
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AuthPlayerToGameError::NoStackFound);
    }

    #[tokio::test]
    async fn test_unknown_error() {
        let (conn_tx, _) = mpsc::channel(1024);
        let (games_tx, mut games_rx) = mpsc::channel(1024);
        let (game_tx, mut game_rx) = mpsc::channel(1024);
        let (_, game_state_rx) = watch::channel(GameState::Open);
        let service = GameService::new(&conn_tx, &games_tx);

        // Respond with game context that will error
        tokio::spawn(async move {
            let Some(GameRepositoryMessage::GetGame {
                game_id: _game_id,
                tx_back,
            }) = games_rx.recv().await
            else {
                unreachable!()
            };
            game_rx.close();
            let _ = tx_back.send(GetGameResponse::Found(GameContext {
                tx: game_tx,
                state_rx: game_state_rx,
            }));
        });

        // Should return err
        let res = service
            .auth_player(&GameId::default(), &PlayerId::default())
            .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), AuthPlayerToGameError::Unknown);
    }

    #[tokio::test]
    async fn test_auth_player_to_game_happy_path() {
        let (conn_tx, _) = mpsc::channel(1024);
        let (games_tx, mut games_rx) = mpsc::channel(1024);
        let (game_tx, mut game_rx) = mpsc::channel(1024);
        let (_, game_state_rx) = watch::channel(GameState::Open);
        let service = GameService::new(&conn_tx, &games_tx);

        // Respond with game context
        tokio::spawn(async move {
            let Some(GameRepositoryMessage::GetGame {
                game_id: _game_id,
                tx_back,
            }) = games_rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(GetGameResponse::Found(GameContext {
                tx: game_tx,
                state_rx: game_state_rx,
            }));
        });

        // Respond with player's context
        tokio::spawn(async move {
            let Some(GameMessage::ConnectPlayer { id: _, tx_back }) = game_rx.recv().await else {
                unreachable!()
            };
            let (game_tx, _) = mpsc::channel(1024);
            let (_, game_state_rx) = watch::channel(GameState::Open);
            let (market_tx, _) = mpsc::channel(1024);
            let (_, market_state_rx) = watch::channel(MarketState::Open);
            let (stack_tx, _) = mpsc::channel(1024);
            let (_, stack_state_rx) = watch::channel(StackState::Open);
            let _ = tx_back.send(ConnectPlayerResponse::OK {
                game: GameContext {
                    tx: game_tx,
                    state_rx: game_state_rx,
                },
                market: MarketContext {
                    tx: market_tx,
                    state_rx: market_state_rx,
                },
                player_stack: StackContext {
                    tx: stack_tx,
                    state_rx: stack_state_rx,
                },
            });
        });

        // Should return ok with player's context
        let Ok(PlayerConnectionContext { .. }) = service
            .auth_player(&GameId::default(), &PlayerId::default())
            .await
        else {
            unreachable!()
        };
    }
}
