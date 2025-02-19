use std::collections::HashMap;

use delivery_period::{start_delivery_period, DeliveryPeriodId, DeliveryPeriodResults};
use futures_util::future::join_all;
use game_repository::GameId;
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot, watch,
};

use crate::{
    market::{MarketActor, MarketContext, MarketService, MarketState},
    plants::{
        models::StackService,
        stack::{StackActor, StackContext, StackState},
    },
    player::{repository::ConnectionRepositoryMessage, PlayerId},
};

pub mod delivery_period;
pub mod game_repository;
pub mod scores;

#[derive(Debug)]
struct Player {
    id: PlayerId,
    name: String,
    ready: bool,
}

pub struct Stack {}
#[derive(Debug)]
pub enum GameMessage {
    RegisterPlayer {
        name: String,
        tx_back: oneshot::Sender<RegisterPlayerResponse>,
    },
    ConnectPlayer {
        id: PlayerId,
        tx_back: oneshot::Sender<ConnectPlayerResponse>,
    },
    PlayerIsReady(PlayerId),
    DeliveryPeriodResults(DeliveryPeriodResults),
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success { id: PlayerId, stack: StackContext },
    PlayerAlreadyExist,
    GameIsRunning,
}

#[derive(Debug)]
pub enum ConnectPlayerResponse {
    OK {
        game: GameContext,
        market: MarketContext,
        player_stack: StackContext,
    },
    NoStackFound,
    DoesNotExist,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GameState {
    Open,
    Running,
    PostDelivery,
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GameState", 2)?;
        state.serialize_field("type", "GameState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Running => "Running",
                Self::Open => "Open",
                Self::PostDelivery => "PostDelivery",
            },
        )?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct GameContext {
    pub tx: mpsc::Sender<GameMessage>,
    pub state_rx: watch::Receiver<GameState>,
}

/// Main entrypoint for a given game of parcelec. Responsible for:
/// - new player registration,
/// - passing game context to new player connection (market and player's stack tx),
/// - delivery period lifecycle
pub struct Game {
    game_id: GameId,
    state: GameState,
    state_watch: watch::Sender<GameState>,
    players: Vec<Player>,
    players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    market_context: MarketContext,
    stacks_context: HashMap<PlayerId, StackContext>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
    delivery_period: DeliveryPeriodId,
    all_players_ready_tx: Option<oneshot::Sender<()>>,
}

impl Game {
    pub async fn new(
        game_id: GameId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    ) -> Game {
        let delivery_period = DeliveryPeriodId::from(0);
        let mut market = MarketActor::new(
            game_id.clone(),
            MarketState::Closed,
            delivery_period,
            players_connections.clone(),
        );
        let market_context = market.get_context();

        tokio::spawn(async move {
            market.process().await;
        });

        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(GameState::Open);

        Game {
            game_id,
            state: GameState::Open,
            state_watch: state_tx,
            market_context,
            players: Vec::new(),
            players_connections,
            stacks_context: HashMap::new(),
            rx,
            tx,
            delivery_period,
            all_players_ready_tx: None,
        }
    }

    pub fn start(
        game_id: &GameId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
        market_context: MarketContext,
    ) -> GameContext {
        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(GameState::Open);
        let mut game = Game {
            game_id: game_id.clone(),
            state: GameState::Open,
            state_watch: state_tx,
            market_context,
            players: Vec::new(),
            players_connections,
            stacks_context: HashMap::new(),
            rx,
            tx,
            delivery_period: DeliveryPeriodId::default(),
            all_players_ready_tx: None,
        };
        let context = game.get_context();

        tokio::spawn(async move { game.run().await });

        context
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match (&mut self.state, msg) {
                (GameState::Open, GameMessage::RegisterPlayer { name, tx_back }) => {
                    self.register_player(name, tx_back);
                }
                (GameState::PostDelivery, GameMessage::RegisterPlayer { tx_back, .. })
                | (GameState::Running, GameMessage::RegisterPlayer { tx_back, .. }) => {
                    let _ = tx_back.send(RegisterPlayerResponse::GameIsRunning);
                }
                (_, GameMessage::ConnectPlayer { id, tx_back }) => {
                    self.connect_player(id, tx_back);
                }
                (GameState::PostDelivery, GameMessage::PlayerIsReady(player_id))
                | (GameState::Open, GameMessage::PlayerIsReady(player_id)) => {
                    self.register_player_ready(player_id);
                }
                (GameState::Running, GameMessage::DeliveryPeriodResults(results)) => {
                    self.state = GameState::PostDelivery;
                    let _ = self.state_watch.send(GameState::PostDelivery);
                    join_all(results.players_scores.iter().map(|(player, score)| {
                        self.players_connections
                            .send(ConnectionRepositoryMessage::SendToPlayer(
                                self.game_id.clone(),
                                player.clone(),
                                crate::player::connection::PlayerMessage::DeliveryPeriodResults(
                                    score.clone(),
                                ),
                            ))
                    }))
                    .await;
                }
                (_, GameMessage::DeliveryPeriodResults(results)) => {
                    println!("Warning, received results for delivery period {:?} while game is not running", results.period_id);
                }
                (GameState::Running, GameMessage::PlayerIsReady(player_id)) => {
                    self.register_player_ready_game_is_running(player_id);
                }
            }
        }
    }

    fn register_player_ready(&mut self, player_id: PlayerId) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.ready = true;
        }

        // If all players are ready, start the next delivery period
        if self.players.iter().all(|player| player.ready) {
            println!(
                "All players ready, starting delivery period {}",
                self.delivery_period.next()
            );
            self.state = GameState::Running;
            self.delivery_period = self.delivery_period.next();
            for player in self.players.iter_mut() {
                player.ready = false;
            }
            let delivery_period = self.delivery_period;
            let game_tx = self.tx.clone();
            let market_service = MarketService::new(self.market_context.tx.clone());
            let stacks_tx = self
                .stacks_context
                .iter()
                .map(|(id, context)| (id.clone(), StackService::new(context.tx.clone())))
                .collect();
            let (results_tx, results_rx) = oneshot::channel();
            let timers = None;
            tokio::spawn(async move {
                start_delivery_period(
                    delivery_period,
                    game_tx,
                    market_service,
                    stacks_tx,
                    results_rx,
                    timers,
                )
                .await;
            });
            self.all_players_ready_tx = Some(results_tx);
            let _ = self.state_watch.send(GameState::Running);
        }
    }

    fn register_player_ready_game_is_running(&mut self, player_id: PlayerId) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.ready = true;
        }

        if self.players.iter().all(|player| player.ready) {
            println!("All players ready, ending delivery period");
            if let Some(tx) = self.all_players_ready_tx.take() {
                let _ = tx.send(());
                for player in self.players.iter_mut() {
                    player.ready = false;
                }
            }
        }
    }

    fn connect_player(&mut self, id: PlayerId, tx_back: oneshot::Sender<ConnectPlayerResponse>) {
        if !self.players.iter().any(|player| player.id == id) {
            let _ = tx_back.send(ConnectPlayerResponse::DoesNotExist);
            return;
        }

        let Some(stack) = self.stacks_context.get(&id) else {
            let _ = tx_back.send(ConnectPlayerResponse::NoStackFound);
            return;
        };

        let _ = tx_back.send(ConnectPlayerResponse::OK {
            game: self.get_context(),
            market: self.market_context.clone(),
            player_stack: stack.clone(),
        });
    }

    fn register_player(&mut self, name: String, tx_back: oneshot::Sender<RegisterPlayerResponse>) {
        if self.players.iter().any(|player| player.name == name) {
            let _ = tx_back.send(RegisterPlayerResponse::PlayerAlreadyExist);
            return;
        }
        // Generate player ID
        let player_id = PlayerId::default();
        let player = Player {
            id: player_id.clone(),
            name,
            ready: false,
        };
        self.players.push(player);

        // Create a new stack for the player
        let mut player_stack = StackActor::new(
            self.game_id.clone(),
            player_id.clone(),
            StackState::Closed,
            self.delivery_period,
            self.players_connections.clone(),
        );
        let stack_context = player_stack.get_context();
        self.stacks_context
            .insert(player_id.clone(), stack_context.clone());
        tokio::spawn(async move {
            player_stack.run().await;
        });
        println!("Stack created for player {player_id}");

        let _ = tx_back.send(RegisterPlayerResponse::Success {
            id: player_id,
            stack: stack_context,
        });
    }

    pub fn get_context(&self) -> GameContext {
        GameContext {
            tx: self.tx.clone(),
            state_rx: self.state_watch.subscribe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{
        mpsc,
        oneshot::{self, channel},
    };

    use crate::{
        game::{ConnectPlayerResponse, Game, GameMessage, GameState, RegisterPlayerResponse},
        market::MarketState,
        plants::stack::{StackContext, StackState},
        player::{connection::PlayerMessage, repository::ConnectionRepositoryMessage, PlayerId},
    };

    use super::{game_repository::GameId, GameContext};

    async fn open_game() -> GameContext {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let mut game = Game::new(game_id, tx).await;
        let context = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });
        context
    }

    async fn register_player(game: mpsc::Sender<GameMessage>) -> PlayerId {
        let player = PlayerId::default();
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .send(GameMessage::RegisterPlayer {
                name: player.to_string(),
                tx_back: tx,
            })
            .await;
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        }
    }

    async fn start_game(game: mpsc::Sender<GameMessage>) -> PlayerId {
        let player = register_player(game.clone()).await;
        let _ = game.send(GameMessage::PlayerIsReady(player.clone())).await;
        player
    }

    async fn get_player_stack(
        game: mpsc::Sender<GameMessage>,
        player_id: &PlayerId,
    ) -> StackContext {
        let (tx, rx) = oneshot::channel();
        let _ = game
            .send(GameMessage::ConnectPlayer {
                id: player_id.clone(),
                tx_back: tx,
            })
            .await;
        let ConnectPlayerResponse::OK { player_stack, .. } =
            rx.await.expect("Should have received a msg")
        else {
            unreachable!("Should have received player connection info");
        };
        player_stack
    }

    #[tokio::test]
    async fn test_register_players() {
        let game = open_game().await;

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        let first_id = match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        };

        // Register another player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "tutu".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        let second_id = match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        };

        // Should have different IDs
        assert_ne!(first_id, second_id);
    }

    #[tokio::test]
    async fn test_fails_register_player_with_existing_name() {
        let game = open_game().await;

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        };

        // Register a player with the same name
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(RegisterPlayerResponse::PlayerAlreadyExist) => {}
            _ => unreachable!("Should have refused the registration"),
        };
        // Register another player with a different name
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "tata".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        };
    }

    #[tokio::test]
    async fn test_fails_register_player_game_is_running() {
        let game = open_game().await;

        // Start the game
        start_game(game.tx.clone()).await;

        // Try to register a new player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await;

        match rx.await {
            Ok(RegisterPlayerResponse::GameIsRunning) => {}
            _ => unreachable!("Should have rejected the request"),
        };
    }

    #[tokio::test]
    async fn test_fails_register_player_game_is_in_post_delivery() {
        let game = open_game().await;

        // Start the game
        let player = start_game(game.tx.clone()).await;
        // End delivery period
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;

        // Try to register a new player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await;

        match rx.await {
            Ok(RegisterPlayerResponse::GameIsRunning) => {}
            _ => unreachable!("Should have rejected the request"),
        };
    }

    #[tokio::test]
    async fn test_fails_connect_unregistered_player() {
        let game = open_game().await;

        // Try to connect a player that is not registered
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        game.tx
            .send(GameMessage::ConnectPlayer {
                id: PlayerId::from("random_id"),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(ConnectPlayerResponse::DoesNotExist) => {}
            _ => unreachable!("Should have refused the connection"),
        };
    }

    #[tokio::test]
    async fn test_connect_player_ok() {
        let game = open_game().await;
        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: "toto".to_owned(),
                tx_back: tx,
            })
            .await
            .unwrap();
        let id = match rx.await {
            Ok(RegisterPlayerResponse::Success { id, .. }) => id,
            _ => unreachable!("Should have register the player"),
        };

        // Connect the player
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        game.tx
            .send(GameMessage::ConnectPlayer {
                id: id.clone(),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(ConnectPlayerResponse::OK { .. }) => {}
            _ => unreachable!("Should have connected the player"),
        };

        // Connection should be idempotent
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        game.tx
            .send(GameMessage::ConnectPlayer {
                id: id.clone(),
                tx_back: tx,
            })
            .await
            .unwrap();
        match rx.await {
            Ok(ConnectPlayerResponse::OK { .. }) => {}
            _ => unreachable!("Should have connected the player"),
        };
    }

    #[tokio::test(start_paused = true)]
    async fn test_starting_the_game_should_open_market_and_stacks() {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let mut game = Game::new(game_id, tx).await;
        let context = game.get_context();
        let mut market_state = game.market_context.state_rx.clone();
        tokio::spawn(async move {
            game.run().await;
        });
        assert_eq!(*market_state.borrow_and_update(), MarketState::Closed);

        let player = register_player(context.tx.clone()).await;
        let mut player_stack = get_player_stack(context.tx.clone(), &player).await;
        assert_eq!(
            *player_stack.state_rx.borrow_and_update(),
            StackState::Closed
        );

        // Start the game
        let _ = context
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;

        // Market should be open
        assert!(market_state.changed().await.is_ok());
        assert_eq!(*market_state.borrow_and_update(), MarketState::Open);

        // // Player's stack should be open
        assert!(player_stack.state_rx.changed().await.is_ok());
        assert_eq!(*player_stack.state_rx.borrow_and_update(), StackState::Open);
    }

    #[tokio::test]
    async fn test_starting_the_game_should_publish_game_state_running() {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let mut game = Game::new(game_id, tx).await;
        let GameContext {
            tx: game_tx,
            state_rx: mut game_state,
        } = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });

        assert_eq!(*game_state.borrow_and_update(), GameState::Open);
        // Start the game
        start_game(game_tx).await;

        // Market should be open
        assert!(game_state.changed().await.is_ok());
        assert_eq!(*game_state.borrow_and_update(), GameState::Running);
    }

    #[tokio::test]
    async fn test_game_should_start_when_all_players_ready() {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let mut game = Game::new(game_id, tx).await;
        let GameContext {
            tx: game_tx,
            state_rx: mut game_state,
        } = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });

        assert_eq!(*game_state.borrow_and_update(), GameState::Open);

        // Register 2 players
        let first_player = register_player(game_tx.clone()).await;
        let second_player = register_player(game_tx.clone()).await;
        assert_eq!(*game_state.borrow_and_update(), GameState::Open);

        // First player is ready
        let _ = game_tx
            .send(GameMessage::PlayerIsReady(first_player.clone()))
            .await;
        assert_eq!(*game_state.borrow_and_update(), GameState::Open);

        // Second player is ready
        let _ = game_tx
            .send(GameMessage::PlayerIsReady(second_player.clone()))
            .await;

        // Game should be running
        assert!(game_state.changed().await.is_ok());
        assert_eq!(*game_state.borrow_and_update(), GameState::Running);
    }

    #[tokio::test]
    async fn test_delivery_period_should_end_when_all_players_ready() {
        let mut game = open_game().await;
        let player = start_game(game.tx.clone()).await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(*game.state_rx.borrow_and_update(), GameState::Running);

        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;

        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(*game.state_rx.borrow_and_update(), GameState::PostDelivery);
    }

    #[tokio::test]
    async fn test_send_results_to_player_at_the_end_of_delivery_period() {
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let game_id = GameId::default();
        let mut game = Game::new(game_id, conn_tx).await;
        let context = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });

        // Start game
        let player = start_game(context.tx.clone()).await;

        // End delivery period
        let _ = context
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        // Flush snapshot sent at the end of the delivery
        let _ = conn_rx.recv().await;

        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            target_player,
            PlayerMessage::DeliveryPeriodResults(_),
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received player's results");
        };
        assert_eq!(target_player, player);
    }
}

#[cfg(test)]
mod test_game_state {
    use crate::game::GameState;

    #[test]
    fn test_game_state_serialize() {
        assert_eq!(
            serde_json::to_string(&GameState::Open).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Open\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&GameState::Running).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Running\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&GameState::PostDelivery).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"PostDelivery\"}".to_string()
        );
    }
}
