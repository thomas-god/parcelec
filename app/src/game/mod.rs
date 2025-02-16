use std::collections::HashMap;

use delivery_period::{start_delivery_period, DeliveryPeriodId, DeliveryPeriodResults};
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot, watch,
};
use uuid::Uuid;

use crate::{
    market::{Market, MarketContext, MarketMessage, MarketState},
    plants::stack::{StackActor, StackContext, StackState},
};

pub mod delivery_period;
pub mod game_repository;

#[derive(Debug)]
struct Player {
    id: String,
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
        id: String,
        tx_back: oneshot::Sender<ConnectPlayerResponse>,
    },
    GetMarketTx {
        tx_back: oneshot::Sender<mpsc::Sender<MarketMessage>>,
    },
    PlayerIsReady(String),
    DeliveryPeriodResults(DeliveryPeriodResults),
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success { id: String },
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
    state: GameState,
    state_watch: watch::Sender<GameState>,
    players: Vec<Player>,
    market_context: MarketContext,
    stacks_context: HashMap<String, StackContext>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
    delivery_period: DeliveryPeriodId,
    all_players_ready_tx: Option<oneshot::Sender<()>>,
}

impl Game {
    pub async fn new() -> Game {
        let delivery_period = DeliveryPeriodId::from(0);
        let mut market = Market::new(MarketState::Closed, delivery_period);
        let market_context = market.get_context();

        tokio::spawn(async move {
            market.process().await;
        });

        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(GameState::Open);

        Game {
            state: GameState::Open,
            state_watch: state_tx,
            market_context,
            players: Vec::new(),
            stacks_context: HashMap::new(),
            rx,
            tx,
            delivery_period,
            all_players_ready_tx: None,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match (&mut self.state, msg) {
                (GameState::Open, GameMessage::RegisterPlayer { name, tx_back }) => {
                    self.register_player(name, tx_back);
                }
                (GameState::Running, GameMessage::RegisterPlayer { tx_back, .. }) => {
                    let _ = tx_back.send(RegisterPlayerResponse::GameIsRunning);
                }
                (_, GameMessage::ConnectPlayer { id, tx_back }) => {
                    self.connect_player(id, tx_back);
                }
                (_, GameMessage::GetMarketTx { tx_back }) => {
                    let _ = tx_back.send(self.market_context.tx.clone());
                }
                (GameState::PostDelivery, GameMessage::PlayerIsReady(player_id))
                | (GameState::Open, GameMessage::PlayerIsReady(player_id)) => {
                    self.register_player_ready(player_id);
                }
                (GameState::Running, GameMessage::DeliveryPeriodResults(_)) => {
                    self.state = GameState::PostDelivery;
                    let _ = self.state_watch.send(GameState::PostDelivery);
                }
                (GameState::Running, GameMessage::PlayerIsReady(player_id)) => {
                    self.register_player_ready_game_is_running(player_id);
                }
                _ => todo!(),
            }
        }
    }

    fn register_player_ready(&mut self, player_id: String) {
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
            let market_tx = self.market_context.tx.clone();
            let stacks_tx = self
                .stacks_context
                .iter()
                .map(|(id, context)| (id.clone(), context.tx.clone()))
                .collect();
            let (results_tx, results_rx) = oneshot::channel();
            let timers = None;
            tokio::spawn(async move {
                start_delivery_period(
                    delivery_period,
                    game_tx,
                    market_tx,
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

    fn register_player_ready_game_is_running(&mut self, player_id: String) {
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

    fn connect_player(&mut self, id: String, tx_back: oneshot::Sender<ConnectPlayerResponse>) {
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
        let player_id = Uuid::new_v4().to_string();
        let player = Player {
            id: player_id.clone(),
            name,
            ready: false,
        };
        self.players.push(player);

        // Create a new stack for the player
        let mut player_stack =
            StackActor::new(player_id.clone(), StackState::Closed, self.delivery_period);
        self.stacks_context
            .insert(player_id.clone(), player_stack.get_context());
        tokio::spawn(async move {
            player_stack.start().await;
        });
        println!("Stack created for player {player_id}");

        let _ = tx_back.send(RegisterPlayerResponse::Success { id: player_id });
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
    use uuid::Uuid;

    use crate::{
        game::{ConnectPlayerResponse, Game, GameMessage, GameState, RegisterPlayerResponse},
        market::MarketState,
        plants::stack::{StackContext, StackState},
    };

    use super::GameContext;

    async fn open_game() -> GameContext {
        let mut game = Game::new().await;
        let context = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });
        context
    }

    async fn register_player(game: mpsc::Sender<GameMessage>) -> String {
        let player = Uuid::new_v4().to_string();
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .send(GameMessage::RegisterPlayer {
                name: player,
                tx_back: tx,
            })
            .await;
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id }) => id,
            _ => unreachable!("Should have register the player"),
        }
    }

    async fn start_game(game: mpsc::Sender<GameMessage>) -> String {
        let player = register_player(game.clone()).await;
        let _ = game.send(GameMessage::PlayerIsReady(player.clone())).await;
        player
    }

    async fn get_player_stack(game: mpsc::Sender<GameMessage>, player_id: &str) -> StackContext {
        let (tx, rx) = oneshot::channel();
        let _ = game
            .send(GameMessage::ConnectPlayer {
                id: player_id.to_string(),
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
            Ok(RegisterPlayerResponse::Success { id }) => id,
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
            Ok(RegisterPlayerResponse::Success { id }) => id,
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
            Ok(RegisterPlayerResponse::Success { id }) => id,
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
            Ok(RegisterPlayerResponse::Success { id }) => id,
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
    async fn test_fails_connect_unregistered_player() {
        let game = open_game().await;

        // Try to connect a player that is not registered
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        game.tx
            .send(GameMessage::ConnectPlayer {
                id: "random_id".to_owned(),
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
            Ok(RegisterPlayerResponse::Success { id }) => id,
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
        let mut game = Game::new().await;
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
        let mut game = Game::new().await;
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
        let mut game = Game::new().await;
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
}
