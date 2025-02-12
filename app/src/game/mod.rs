use std::collections::HashMap;

use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot::Sender as OneShotSender,
};
use uuid::Uuid;

use crate::{
    market::{Market, MarketMessage},
    plants::stack::{StackActor, StackMessage},
};

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
        tx_back: OneShotSender<RegisterPlayerResponse>,
    },
    ConnectPlayer {
        id: String,
        tx_back: OneShotSender<ConnectPlayerResponse>,
    },
    // MarketOpen,
    // MarketClosed,
    // DispatchOpen,
    // DispatchClosed,
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success { id: String },
    PlayerAlreadyExist,
}
#[derive(Debug)]
pub enum ConnectPlayerResponse {
    OK { player_stack: Sender<StackMessage> },
    NoStackFound,
    DoesNotExist,
}

#[derive(Clone)]
pub struct GameContext {
    pub game: Sender<GameMessage>,
    pub market: Sender<MarketMessage>,
    pub stacks: HashMap<String, Sender<StackMessage>>,
}

pub struct Game {
    players: Vec<Player>,
    market: Sender<MarketMessage>,
    stacks: HashMap<String, Sender<StackMessage>>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
}

impl Game {
    pub async fn new() -> Game {
        let mut market = Market::new();
        let market_tx = market.get_tx();

        tokio::spawn(async move {
            market.process().await;
        });

        let (tx, rx) = channel::<GameMessage>(32);

        Game {
            market: market_tx,
            players: Vec::new(),
            stacks: HashMap::new(),
            rx,
            tx,
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                GameMessage::RegisterPlayer { name, tx_back } => {
                    self.register_player(name, tx_back);
                }
                GameMessage::ConnectPlayer { id, tx_back } => {
                    self.connect_player(id, tx_back);
                }
            }
        }
    }

    fn connect_player(&mut self, id: String, tx_back: OneShotSender<ConnectPlayerResponse>) {
        if !self.players.iter().any(|player| player.id == id) {
            let _ = tx_back.send(ConnectPlayerResponse::DoesNotExist);
            return;
        }

        let Some(player_stack) = self.stacks.get(&id) else {
            let _ = tx_back.send(ConnectPlayerResponse::NoStackFound);
            return;
        };

        let _ = tx_back.send(ConnectPlayerResponse::OK {
            player_stack: player_stack.clone(),
        });
    }

    fn register_player(&mut self, name: String, tx_back: OneShotSender<RegisterPlayerResponse>) {
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
            market: self.get_market(),
            stacks: self.stacks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot::channel;

    use crate::game::{ConnectPlayerResponse, Game, RegisterPlayerResponse};

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
            .send(crate::game::GameMessage::RegisterPlayer {
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
            .send(crate::game::GameMessage::RegisterPlayer {
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
    async fn test_register_player_same_name() {
        let context = start_game().await;

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        context
            .game
            .clone()
            .send(crate::game::GameMessage::RegisterPlayer {
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
            .send(crate::game::GameMessage::RegisterPlayer {
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
            .send(crate::game::GameMessage::RegisterPlayer {
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
    async fn test_connect_unregistered_player() {
        let context = start_game().await;

        // Try to connect a player that is not registered
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        context
            .game
            .clone()
            .send(crate::game::GameMessage::ConnectPlayer {
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
        game.send(crate::game::GameMessage::RegisterPlayer {
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
        game.send(crate::game::GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await
        .unwrap();
        match rx.await {
            Ok(ConnectPlayerResponse::OK { player_stack: _ }) => {}
            _ => unreachable!("Should have connected the player"),
        };

        // Connection should be idempotent
        let (tx, rx) = channel::<ConnectPlayerResponse>();
        game.send(crate::game::GameMessage::ConnectPlayer {
            id: id.clone(),
            tx_back: tx,
        })
        .await
        .unwrap();
        match rx.await {
            Ok(ConnectPlayerResponse::OK { player_stack: _ }) => {}
            _ => unreachable!("Should have connected the player"),
        };
    }
}
