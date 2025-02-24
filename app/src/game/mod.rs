use std::collections::HashMap;

use delivery_period::{start_delivery_period, DeliveryPeriodId, DeliveryPeriodResults};
use futures_util::future::join_all;
use scores::PlayerScore;
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot, watch,
};

use crate::{
    market::{Market, MarketContext},
    plants::{
        actor::{StackActor, StackContext, StackState},
        service::StackService,
    },
    player::{repository::ConnectionRepositoryMessage, PlayerId},
};

pub mod delivery_period;
pub mod scores;

#[derive(Debug)]
struct Player {
    id: PlayerId,
    name: String,
    ready: bool,
}

#[derive(Debug)]
pub enum GameMessage {
    RegisterPlayer {
        name: String,
        tx_back: oneshot::Sender<RegisterPlayerResponse>,
    },
    PlayerIsReady(PlayerId),
    DeliveryPeriodResults(DeliveryPeriodResults),
    GetPreviousScores {
        player_id: PlayerId,
        tx_back: oneshot::Sender<HashMap<DeliveryPeriodId, PlayerScore>>,
    },
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success {
        id: PlayerId,
        stack: StackContext<StackService>,
    },
    PlayerAlreadyExist,
    GameIsRunning,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GameState {
    Open,
    Running(DeliveryPeriodId),
    PostDelivery(DeliveryPeriodId),
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GameState", 3)?;
        state.serialize_field("type", "GameState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Running(_) => "Running",
                Self::Open => "Open",
                Self::PostDelivery(_) => "PostDelivery",
            },
        )?;
        let period = match self {
            Self::Running(period) | Self::PostDelivery(period) => *period,
            Self::Open => DeliveryPeriodId::from(0),
        };
        state.serialize_field("delivery_period", &period)?;
        state.end()
    }
}

use std::fmt;

use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct GameContext {
    pub tx: mpsc::Sender<GameMessage>,
    pub state_rx: watch::Receiver<GameState>,
}

/// Main entrypoint for a given game of parcelec. Responsible for:
/// - new player registration,
/// - passing game context to new player connection (market and player's stack tx),
/// - delivery period lifecycle
pub struct Game<MS: Market> {
    game_id: GameId,
    state: GameState,
    state_watch: watch::Sender<GameState>,
    players: Vec<Player>,
    players_scores: HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerScore>>,
    players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    market_context: MarketContext<MS>,
    stacks_contexts: HashMap<PlayerId, StackContext<StackService>>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
    delivery_period: DeliveryPeriodId,
    all_players_ready_tx: Option<oneshot::Sender<()>>,
}

impl<MS: Market> Game<MS> {
    pub fn start(
        game_id: &GameId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
        market_context: MarketContext<MS>,
    ) -> GameContext {
        let initial_state = GameState::Open;
        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(initial_state.clone());
        let mut game = Game {
            game_id: game_id.clone(),
            state: initial_state,
            state_watch: state_tx,
            market_context,
            players: Vec::new(),
            players_connections,
            players_scores: HashMap::new(),
            stacks_contexts: HashMap::new(),
            rx,
            tx,
            delivery_period: DeliveryPeriodId::default(),
            all_players_ready_tx: None,
        };
        let context = game.get_context();

        tokio::spawn(async move { game.run().await });

        context
    }

    async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match (&mut self.state, msg) {
                (GameState::Open, GameMessage::RegisterPlayer { name, tx_back }) => {
                    self.register_player(name, tx_back);
                }
                (GameState::PostDelivery(_), GameMessage::RegisterPlayer { tx_back, .. })
                | (GameState::Running(_), GameMessage::RegisterPlayer { tx_back, .. }) => {
                    let _ = tx_back.send(RegisterPlayerResponse::GameIsRunning);
                }
                (_, GameMessage::GetPreviousScores { player_id, tx_back }) => {
                    let scores = self
                        .players_scores
                        .get(&player_id)
                        .cloned()
                        .unwrap_or_else(HashMap::new);
                    let _ = tx_back.send(scores);
                }
                (GameState::PostDelivery(_), GameMessage::PlayerIsReady(player_id))
                | (GameState::Open, GameMessage::PlayerIsReady(player_id)) => {
                    self.register_player_ready(player_id);
                }
                (GameState::Running(_), GameMessage::DeliveryPeriodResults(results)) => {
                    self.store_scores(&results);
                    self.state = GameState::PostDelivery(self.delivery_period);
                    let _ = self
                        .state_watch
                        .send(GameState::PostDelivery(self.delivery_period));
                    join_all(results.players_scores.iter().map(|(player, score)| {
                        self.players_connections
                            .send(ConnectionRepositoryMessage::SendToPlayer(
                                self.game_id.clone(),
                                player.clone(),
                                crate::player::connection::PlayerMessage::DeliveryPeriodResults {
                                    score: score.clone(),
                                    delivery_period: self.delivery_period,
                                },
                            ))
                    }))
                    .await;
                }
                (_, GameMessage::DeliveryPeriodResults(results)) => {
                    println!("Warning, received results for delivery period {:?} while game is not running", results.period_id);
                }
                (GameState::Running(_), GameMessage::PlayerIsReady(player_id)) => {
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
            self.state = GameState::Running(self.delivery_period);
            self.delivery_period = self.delivery_period.next();
            for player in self.players.iter_mut() {
                player.ready = false;
            }
            let delivery_period = self.delivery_period;
            let game_tx = self.tx.clone();
            let market_service = self.market_context.service.clone();
            let stacks_tx = self
                .stacks_contexts
                .iter()
                .map(|(id, context)| (id.clone(), context.service.clone()))
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
            let _ = self
                .state_watch
                .send(GameState::Running(self.delivery_period));
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
        self.stacks_contexts
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

    fn get_context(&self) -> GameContext {
        GameContext {
            tx: self.tx.clone(),
            state_rx: self.state_watch.subscribe(),
        }
    }

    fn store_scores(&mut self, results: &DeliveryPeriodResults) {
        for (player, score) in results.players_scores.iter() {
            if let Some(scores) = self.players_scores.get_mut(player) {
                scores.insert(results.period_id, score.clone());
            } else {
                self.players_scores.insert(
                    player.clone(),
                    HashMap::from([(results.period_id, score.clone())]),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{
        mpsc,
        oneshot::{self, channel},
        watch,
    };

    use crate::{
        game::{
            delivery_period::DeliveryPeriodId, Game, GameMessage, GameState, RegisterPlayerResponse,
        },
        market::{Market, MarketContext, MarketState, OBS},
        plants::{
            actor::{StackContext, StackState},
            StackService,
        },
        player::{connection::PlayerMessage, repository::ConnectionRepositoryMessage, PlayerId},
    };

    use super::{GameContext, GameId};

    #[derive(Clone)]
    struct MockMarket {
        state_tx: watch::Sender<MarketState>,
    }
    impl Market for MockMarket {
        async fn close_market(
            &self,
            _delivery_period: super::delivery_period::DeliveryPeriodId,
        ) -> Vec<crate::market::order_book::Trade> {
            let _ = self.state_tx.send(MarketState::Closed);
            Vec::new()
        }
        async fn delete_order(&self, _order_id: String) {}
        async fn get_market_snapshot(
            &self,
            _player: PlayerId,
        ) -> (Vec<crate::market::order_book::TradeLeg>, crate::market::OBS) {
            (
                Vec::new(),
                OBS {
                    offers: Vec::new(),
                    bids: Vec::new(),
                },
            )
        }
        async fn new_order(&self, _request: crate::market::order_book::OrderRequest) {}
        async fn open_market(&self, _delivery_period: super::delivery_period::DeliveryPeriodId) {
            let _ = self.state_tx.send(MarketState::Open);
        }
    }

    fn open_game() -> (GameContext, MarketContext<MockMarket>) {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MockMarket { state_tx };

        let market_context = MarketContext {
            service: market,
            state_rx: rx,
        };
        (
            Game::start(&game_id, tx, market_context.clone()),
            market_context,
        )
    }

    async fn register_player(
        game: mpsc::Sender<GameMessage>,
    ) -> (PlayerId, StackContext<StackService>) {
        let player = PlayerId::default();
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .send(GameMessage::RegisterPlayer {
                name: player.to_string(),
                tx_back: tx,
            })
            .await;
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id, stack }) => (id, stack),
            _ => unreachable!("Should have register the player"),
        }
    }

    async fn start_game(game: mpsc::Sender<GameMessage>) -> (PlayerId, StackContext<StackService>) {
        let (player, stack) = register_player(game.clone()).await;
        let _ = game.send(GameMessage::PlayerIsReady(player.clone())).await;
        (player, stack)
    }

    #[tokio::test]
    async fn test_register_players() {
        let (game, _) = open_game();

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
        let (game, _) = open_game();

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
        let (game, _) = open_game();

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
        let (game, _) = open_game();

        // Start the game
        let (player, _) = start_game(game.tx.clone()).await;
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

    #[tokio::test(start_paused = true)]
    async fn test_starting_the_game_should_open_market_and_stacks() {
        let (game, mut market) = open_game();

        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Closed);

        let (player, mut stack) = register_player(game.tx.clone()).await;

        // Start the game
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;

        // Market should be open
        assert!(market.state_rx.changed().await.is_ok());
        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Open);

        // // Player's stack should be open
        assert!(stack.state_rx.changed().await.is_ok());
        assert_eq!(*stack.state_rx.borrow_and_update(), StackState::Open);
    }

    #[tokio::test]
    async fn test_starting_the_game_should_publish_game_state_running() {
        let (mut game, _) = open_game();
        assert_eq!(*game.state_rx.borrow_and_update(), GameState::Open);
        // Start the game
        start_game(game.tx).await;

        // Market should be open
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(1))
        );
    }

    #[tokio::test]
    async fn test_game_should_start_when_all_players_ready() {
        let (mut game, _) = open_game();

        assert_eq!(*game.state_rx.borrow_and_update(), GameState::Open);

        // Register 2 players
        let (first_player, _) = register_player(game.tx.clone()).await;
        let (second_player, _) = register_player(game.tx.clone()).await;
        assert_eq!(*game.state_rx.borrow_and_update(), GameState::Open);

        // First player is ready
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(first_player.clone()))
            .await;
        assert_eq!(*game.state_rx.borrow_and_update(), GameState::Open);

        // Second player is ready
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(second_player.clone()))
            .await;

        // Game should be running
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(1))
        );
    }

    #[tokio::test]
    async fn test_delivery_period_should_end_when_all_players_ready() {
        let (mut game, _) = open_game();
        let (player, _) = start_game(game.tx.clone()).await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(1))
        );

        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;

        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::PostDelivery(DeliveryPeriodId::from(1))
        );
    }

    #[tokio::test]
    async fn test_send_results_to_player_at_the_end_of_delivery_period() {
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let game = Game::start(&game_id, conn_tx, market);

        // Start game
        let (player, _) = start_game(game.tx.clone()).await;

        // End delivery period
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        // Flush snapshot and forecasts sent at the end of the delivery
        let _ = conn_rx.recv().await;
        let _ = conn_rx.recv().await;

        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            target_player,
            PlayerMessage::DeliveryPeriodResults {
                score: _,
                delivery_period,
            },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received player's results");
        };
        assert_eq!(delivery_period, DeliveryPeriodId::from(1));
        assert_eq!(target_player, player);
    }

    #[tokio::test]
    async fn test_get_previous_scores() {
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let game = Game::start(&game_id, conn_tx, market);

        // Start the game
        let (player, _) = start_game(game.tx.clone()).await;

        // Scores should be empty at this stage
        let (tx_back, rx) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetPreviousScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(scores) = rx.await else { unreachable!() };
        assert!(scores.is_empty());

        // End delivery period
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        let _ = conn_rx.recv().await;

        // Request player's scores
        let (tx_back, rx) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetPreviousScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(scores) = rx.await else { unreachable!() };
        assert_eq!(scores.len(), 1);
    }
}

#[cfg(test)]
mod test_game_state {
    use crate::game::GameState;

    use super::delivery_period::DeliveryPeriodId;

    #[test]
    fn test_game_state_serialize() {
        assert_eq!(
            serde_json::to_string(&GameState::Open).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Open\",\"delivery_period\":0}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&GameState::Running(DeliveryPeriodId::from(1))).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Running\",\"delivery_period\":1}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&GameState::PostDelivery(DeliveryPeriodId::from(2))).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"PostDelivery\",\"delivery_period\":2}".to_string()
        );
    }
}

#[cfg(test)]
mod test_game_id {
    use crate::game::GameId;

    #[test]
    fn test_game_id_to_string() {
        assert_eq!(GameId::from("toto").to_string(), String::from("toto"));
    }

    #[test]
    fn test_game_id_from_and_into_string() {
        assert_eq!(
            GameId::from(String::from("toto")).into_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_game_id_as_ref() {
        assert_eq!(GameId::from("toto").as_ref(), "toto");
    }
}
