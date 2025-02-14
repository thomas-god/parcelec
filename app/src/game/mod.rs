use std::collections::HashMap;

use delivery_period::DeliveryPeriod;
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot, watch,
};
use uuid::Uuid;

use crate::{
    market::{Market, MarketMessage, MarketState},
    plants::stack::{StackActor, StackMessage, StackState},
};

pub mod delivery_period;
pub mod game_repository;

#[derive(Debug)]
pub struct Player {
    pub id: String,
    pub name: String,
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
    StartGame,
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
        market: mpsc::Sender<MarketMessage>,
        market_state: watch::Receiver<MarketState>,
        player_stack: Sender<StackMessage>,
        stack_state: watch::Receiver<StackState>,
    },
    NoStackFound,
    DoesNotExist,
}

#[derive(Clone)]
pub struct GameContext {
    pub game: Sender<GameMessage>,
    pub game_state: watch::Receiver<GameState>,
    pub market: Sender<MarketMessage>,
    pub market_state: watch::Receiver<MarketState>,
    pub stacks: HashMap<String, Sender<StackMessage>>,
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    Open,
    Running,
}

/// Main entrypoint for a given game of parcelec. Responsible for:
/// - new player registration,
/// - passing game context to new player connection (market and player's stack tx),
/// - delivery period lifecycle
pub struct Game {
    state: GameState,
    state_watch: watch::Sender<GameState>,
    players: Vec<Player>,
    market: Sender<MarketMessage>,
    market_state: watch::Receiver<MarketState>,
    stacks: HashMap<String, Sender<StackMessage>>,
    stack_states: HashMap<String, watch::Receiver<StackState>>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
}

impl Game {
    pub async fn new() -> Game {
        let mut market = Market::new();
        let market_tx = market.get_tx();
        let market_state = market.get_state_rx();

        tokio::spawn(async move {
            market.process().await;
        });

        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(GameState::Open);

        Game {
            state: GameState::Open,
            state_watch: state_tx,
            market: market_tx,
            market_state,
            players: Vec::new(),
            stacks: HashMap::new(),
            stack_states: HashMap::new(),
            rx,
            tx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match (&self.state, msg) {
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
                    let _ = tx_back.send(self.market.clone());
                }
                (GameState::Open, GameMessage::StartGame) => {
                    self.state = GameState::Running;
                    let mut delivery_period =
                        DeliveryPeriod::new(self.market.clone(), self.stacks.clone());
                    tokio::spawn(async move {
                        delivery_period.start().await;
                    });
                    let _ = self.state_watch.send(GameState::Running);
                }
                (GameState::Running, GameMessage::StartGame) => {
                    println!("Game already running")
                }
            }
        }
    }

    fn connect_player(&mut self, id: String, tx_back: oneshot::Sender<ConnectPlayerResponse>) {
        if !self.players.iter().any(|player| player.id == id) {
            let _ = tx_back.send(ConnectPlayerResponse::DoesNotExist);
            return;
        }

        let Some(player_stack) = self.stacks.get(&id) else {
            let _ = tx_back.send(ConnectPlayerResponse::NoStackFound);
            return;
        };
        let Some(player_stack_state) = self.stack_states.get(&id) else {
            let _ = tx_back.send(ConnectPlayerResponse::NoStackFound);
            return;
        };

        let _ = tx_back.send(ConnectPlayerResponse::OK {
            market: self.market.clone(),
            market_state: self.market_state.clone(),
            player_stack: player_stack.clone(),
            stack_state: player_stack_state.clone(),
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
        };
        self.players.push(player);

        // Create a new stack for the player
        let mut player_stack = StackActor::new(player_id.clone());
        self.stacks.insert(player_id.clone(), player_stack.get_tx());
        self.stack_states
            .insert(player_id.clone(), player_stack.get_state_rx());
        tokio::spawn(async move {
            player_stack.start().await;
        });
        println!("Stack created for player {player_id}");

        let _ = tx_back.send(RegisterPlayerResponse::Success { id: player_id });
    }

    pub fn get_tx(&self) -> Sender<GameMessage> {
        self.tx.clone()
    }

    pub fn get_market(&self) -> Sender<MarketMessage> {
        self.market.clone()
    }

    pub fn get_context(&self) -> GameContext {
        GameContext {
            game: self.get_tx(),
            game_state: self.state_watch.subscribe(),
            market: self.get_market(),
            market_state: self.market_state.clone(),
            stacks: self.stacks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot::channel;

    use crate::{
        game::{ConnectPlayerResponse, Game, GameMessage, GameState, RegisterPlayerResponse},
        market::MarketState,
    };

    use super::GameContext;

    async fn start_game() -> GameContext {
        let mut game = Game::new().await;
        let context = game.get_context();
        tokio::spawn(async move {
            game.run().await;
        });
        context
    }

    #[tokio::test]
    async fn test_register_players() {
        let context = start_game().await;

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        context
            .game
            .clone()
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
        context
            .game
            .clone()
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
        let context = start_game().await;

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        context
            .game
            .clone()
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
        context
            .game
            .clone()
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
        context
            .game
            .clone()
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
        let GameContext { game, .. } = start_game().await;

        // Start the game
        let _ = game.send(GameMessage::StartGame).await;

        // Try to register a new player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
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
        let context = start_game().await;

        // Try to connect a player that is not registered
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        context
            .game
            .clone()
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
        let context = start_game().await;
        let game = context.game.clone();
        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.send(GameMessage::RegisterPlayer {
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
        game.send(GameMessage::ConnectPlayer {
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
        game.send(GameMessage::ConnectPlayer {
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

    #[tokio::test]
    async fn test_starting_the_game_should_open_market_and_stacks() {
        let GameContext {
            game,
            mut market_state,
            ..
        } = start_game().await;

        // Start the game
        let _ = game.send(GameMessage::StartGame).await;

        // Market should be open
        if *market_state.borrow_and_update() == MarketState::Closed {
            assert!(market_state.changed().await.is_ok());
            assert_eq!(*market_state.borrow_and_update(), MarketState::Open);
        }
    }

    #[tokio::test]
    async fn test_starting_the_game_should_publish_game_state_running() {
        let GameContext {
            game,
            mut game_state,
            ..
        } = start_game().await;

        assert_eq!(*game_state.borrow_and_update(), GameState::Open);
        // Start the game
        let _ = game.send(GameMessage::StartGame).await;

        // Market should be open
        assert!(game_state.changed().await.is_ok());
        assert_eq!(*game_state.borrow_and_update(), GameState::Running);
    }
}
