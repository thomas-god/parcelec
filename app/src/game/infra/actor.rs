use std::collections::HashMap;

use futures_util::future::join_all;
use tokio::sync::{
    mpsc::{Receiver, Sender, channel},
    oneshot, watch,
};
use tokio_util::sync::CancellationToken;

use crate::{
    game::{
        GameContext, GameId, GameMessage, GameName, GameState, GetPreviousScoresResult, Player,
        RegisterPlayerResponse,
        delivery_period::{
            DeliveryPeriodId, DeliveryPeriodResults, DeliveryPeriodTimers, start_delivery_period,
        },
        scores::{GameRankings, PlayerResult, PlayerScore},
    },
    plants::infra::actor::StackPlants,
};
use crate::{
    market::{Market, MarketContext},
    plants::infra::{StackActor, StackContext, StackService, StackState},
    player::{PlayerConnections, PlayerId, PlayerMessage, PlayerName, PlayerResultView},
};

/// Main entrypoint for a given game of parcelec. Responsible for:
/// - new player registration,
/// - passing game context to new player connection (market and player's stack tx),
/// - delivery period lifecycle
pub struct GameActor<MS: Market, PC: PlayerConnections> {
    game_id: GameId,
    name: GameName,
    state: GameState,
    state_watch: watch::Sender<GameState>,
    players: Vec<Player>,
    players_scores: HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerScore>>,
    players_connections: PC,
    market_context: MarketContext<MS>,
    stacks_contexts: HashMap<PlayerId, StackContext<StackService>>,
    build_stack: fn() -> StackPlants,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
    current_delivery_period: DeliveryPeriodId,
    last_delivery_period: DeliveryPeriodId,
    delivery_period_timers: Option<DeliveryPeriodTimers>,
    all_players_ready_tx: Option<oneshot::Sender<()>>,
    ranking_calculator: GameRankings,
    cancellation_token: CancellationToken,
}

pub struct GameActorConfig {
    pub id: GameId,
    pub name: Option<GameName>,
    pub number_of_delivery_periods: usize,
    pub ranking_calculator: GameRankings,
    pub delivery_period_timers: Option<DeliveryPeriodTimers>,
    pub build_stack: fn() -> StackPlants,
}

impl<MS: Market, PC: PlayerConnections> GameActor<MS, PC> {
    pub fn start(
        config: GameActorConfig,
        players_connections: PC,
        market_context: MarketContext<MS>,
        cancelation_token: CancellationToken,
    ) -> GameContext {
        let initial_state = GameState::Open;
        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(initial_state.clone());
        let mut game = GameActor {
            game_id: config.id.clone(),
            name: config.name.unwrap_or_default(),
            state: initial_state,
            state_watch: state_tx,
            market_context,
            players: Vec::new(),
            players_connections,
            players_scores: HashMap::new(),
            stacks_contexts: HashMap::new(),
            build_stack: config.build_stack,
            rx,
            tx,
            current_delivery_period: DeliveryPeriodId::default(),
            delivery_period_timers: config.delivery_period_timers,
            last_delivery_period: DeliveryPeriodId::from(config.number_of_delivery_periods),
            all_players_ready_tx: None,
            ranking_calculator: config.ranking_calculator,
            cancellation_token: cancelation_token,
        };
        let context = game.get_context();

        tokio::spawn(async move { game.run().await });

        context
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    self.process_message(msg).await;
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("Game actor {:?} terminated", self.game_id);
                    break;
                }
            }
        }
    }
    #[tracing::instrument(name = "GameActor::process_message", skip(self))]
    async fn process_message(&mut self, message: GameMessage) {
        match message {
            GameMessage::RegisterPlayer { name, tx_back } => {
                let _ = self.register_player(name, tx_back).await;
            }
            GameMessage::GetScores { player_id, tx_back } => {
                self.send_scores(player_id, tx_back);
            }
            GameMessage::GetReadiness { tx_back } => {
                let _ = tx_back.send(
                    self.players
                        .iter()
                        .map(|player| (player.name.clone(), player.ready))
                        .collect(),
                );
            }
            GameMessage::PlayerIsReady(player_id) => {
                let _ = self.register_player_ready(player_id).await;
            }
            GameMessage::DeliveryPeriodResults(results) => {
                let _ = self.process_delivery_period_results(results).await;
            }
        }
    }

    async fn register_player_ready(&mut self, player_id: PlayerId) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.ready = true;
        }

        let all_players_ready = self.players.iter().all(|player| player.ready);

        use GameState::*;
        match (all_players_ready, &self.state) {
            (false, _) => { /* Do nothing */ }
            (true, Ended(_)) => { /* Game already ended */ }
            (true, Open) => {
                self.start_next_delivery_period();
            }
            (true, Running(_)) => {
                tracing::info!("All players ready, ending delivery period");
                if let Some(tx) = self.all_players_ready_tx.take() {
                    let _ = tx.send(());
                    self.reset_players_ready();
                }
            }
            (true, PostDelivery(_)) => {
                if self.game_should_end() {
                    let _ = self.end_game().await;
                } else {
                    self.start_next_delivery_period();
                }
            }
        };
        let _ = self.send_readiness_status().await;
    }

    fn start_next_delivery_period(&mut self) {
        tracing::info!(
            "All players ready, starting delivery period {}",
            self.current_delivery_period.next()
        );
        self.current_delivery_period = self.current_delivery_period.next();
        self.state = GameState::Running(self.current_delivery_period);
        self.reset_players_ready();

        let delivery_period = self.current_delivery_period;
        let game_tx = self.tx.clone();
        let market_service = self.market_context.service.clone();
        let stacks_tx = self
            .stacks_contexts
            .iter()
            .map(|(id, context)| (id.clone(), context.service.clone()))
            .collect();
        let (results_tx, results_rx) = oneshot::channel();
        let timers = self.delivery_period_timers.clone();
        let token = self.cancellation_token.clone();
        tokio::spawn(async move {
            start_delivery_period(
                delivery_period,
                game_tx,
                market_service,
                stacks_tx,
                results_rx,
                timers,
                token,
            )
            .await;
        });
        self.all_players_ready_tx = Some(results_tx);
        let _ = self
            .state_watch
            .send(GameState::Running(self.current_delivery_period));
    }

    fn reset_players_ready(&mut self) {
        for player in self.players.iter_mut() {
            player.ready = false;
        }
    }

    fn game_should_end(&self) -> bool {
        self.current_delivery_period >= self.last_delivery_period
    }

    async fn end_game(&mut self) {
        tracing::info!(
            "Maximum delivery periods reached ({}), ending game",
            self.current_delivery_period
        );
        // Update state to Ended
        self.state = GameState::Ended(self.current_delivery_period);
        let _ = self
            .state_watch
            .send(GameState::Ended(self.current_delivery_period));

        // Send final scores and rankings
        let _ = self
            .players_connections
            .send_to_all_players(
                &self.game_id,
                PlayerMessage::GameResults {
                    rankings: map_rankings_to_player_name(
                        self.ranking_calculator.compute_scores(&self.players_scores),
                        &self.players,
                    ),
                },
            )
            .await;
    }

    async fn register_player(
        &mut self,
        name: PlayerName,
        tx_back: oneshot::Sender<RegisterPlayerResponse>,
    ) {
        if self.state != GameState::Open {
            let _ = tx_back.send(RegisterPlayerResponse::GameStarted);
            return;
        }
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
            (self.build_stack)(),
            StackState::Closed,
            self.current_delivery_period,
            self.players_connections.clone(),
            self.cancellation_token.clone(),
        );
        let stack_context = player_stack.get_context();
        self.stacks_contexts
            .insert(player_id.clone(), stack_context.clone());
        tokio::spawn(async move {
            player_stack.run().await;
        });
        tracing::info!("Stack created for player {player_id}");

        let _ = tx_back.send(RegisterPlayerResponse::Success {
            id: player_id,
            stack: stack_context,
        });

        let _ = self.send_readiness_status().await;
    }

    async fn send_readiness_status(&self) {
        let _ = self
            .players_connections
            .send_to_all_players(
                &self.game_id,
                PlayerMessage::ReadinessStatus {
                    readiness: self
                        .players
                        .iter()
                        .map(|player| (player.name.clone(), player.ready))
                        .collect(),
                },
            )
            .await;
    }

    fn send_scores(&self, player: PlayerId, tx_back: oneshot::Sender<GetPreviousScoresResult>) {
        use GetPreviousScoresResult::*;
        let scores = match self.state {
            GameState::Ended(_) => PlayersRanking {
                scores: map_rankings_to_player_name(
                    self.ranking_calculator
                        .compute_scores(&self.players_scores.clone()),
                    &self.players,
                ),
            },
            _ => PlayerScores {
                scores: self
                    .players_scores
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(HashMap::new),
            },
        };
        let _ = tx_back.send(scores);
    }

    fn get_context(&self) -> GameContext {
        GameContext {
            id: self.game_id.clone(),
            name: self.name.clone(),
            last_delivery_period: self.last_delivery_period,
            tx: self.tx.clone(),
            state_rx: self.state_watch.subscribe(),
        }
    }

    async fn process_delivery_period_results(&mut self, results: DeliveryPeriodResults) {
        if self.state != GameState::Running(results.period_id) {
            tracing::warn!(
                "Received results for delivery period {:?} while game is not running",
                results.period_id
            );
            return;
        }

        self.store_scores(&results);
        self.state = GameState::PostDelivery(self.current_delivery_period);
        let _ = self
            .state_watch
            .send(GameState::PostDelivery(self.current_delivery_period));
        join_all(results.players_scores.iter().map(|(player, score)| {
            self.players_connections.send_to_player(
                &self.game_id,
                player,
                PlayerMessage::DeliveryPeriodResults {
                    score: score.clone(),
                    delivery_period: self.current_delivery_period,
                },
            )
        }))
        .await;
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

fn map_rankings_to_player_name(
    rankings: Vec<PlayerResult>,
    players: &[Player],
) -> Vec<PlayerResultView> {
    rankings
        .iter()
        .map(|rank| PlayerResultView {
            player: players
                .iter()
                .find(|player| player.id == rank.player)
                .map(|player| player.name.clone())
                .unwrap_or_else(|| PlayerName::from(rank.player.to_string())),
            rank: rank.rank,
            score: rank.score,
            tier: rank.tier.clone(),
        })
        .collect()
}

#[cfg(test)]
mod test_utils {
    use tokio::sync::mpsc;

    use crate::{
        market::{MarketState, OBS, order_book::TradeLeg},
        plants::infra::actor::default_stack_plants,
    };

    use super::*;

    #[derive(Clone)]
    pub struct MockMarket {
        pub state_tx: watch::Sender<MarketState>,
    }
    impl Market for MockMarket {
        async fn close_market(
            &self,
            _delivery_period: DeliveryPeriodId,
        ) -> Vec<crate::market::order_book::Trade> {
            let _ = self.state_tx.send(MarketState::Closed);
            Vec::new()
        }
        async fn delete_order(&self, _order_id: String) {}
        async fn get_market_snapshot(&self, _player: PlayerId) -> (Vec<TradeLeg>, OBS) {
            (
                Vec::new(),
                OBS {
                    offers: Vec::new(),
                    bids: Vec::new(),
                },
            )
        }
        async fn new_order(&self, _request: crate::market::order_book::OrderRequest) {}
        async fn open_market(&self, _delivery_period: DeliveryPeriodId) {
            let _ = self.state_tx.send(MarketState::Open);
        }
    }

    #[derive(Debug, Clone)]
    pub struct MockPlayerConnections {
        pub tx_send_to_player: mpsc::Sender<(PlayerId, PlayerMessage)>,
        pub tx_send_to_all_players: mpsc::Sender<PlayerMessage>,
    }
    impl MockPlayerConnections {
        pub fn new() -> (
            MockPlayerConnections,
            mpsc::Receiver<(PlayerId, PlayerMessage)>,
            mpsc::Receiver<PlayerMessage>,
        ) {
            let (tx_send_to_player, rx_send_to_player) = mpsc::channel(16);
            let (tx_send_to_all_players, rx_send_to_all_players) = mpsc::channel(16);
            (
                MockPlayerConnections {
                    tx_send_to_player,
                    tx_send_to_all_players,
                },
                rx_send_to_player,
                rx_send_to_all_players,
            )
        }
    }
    impl PlayerConnections for MockPlayerConnections {
        async fn send_to_all_players(&self, _game: &GameId, message: PlayerMessage) -> () {
            let _ = self.tx_send_to_all_players.send(message).await;
        }
        async fn send_to_player(
            &self,
            _game: &GameId,
            player: &PlayerId,
            message: PlayerMessage,
        ) -> () {
            let _ = self.tx_send_to_player.send((player.clone(), message)).await;
        }
    }

    pub fn default_game_config() -> GameActorConfig {
        GameActorConfig {
            delivery_period_timers: None,
            id: GameId::default(),
            name: Some(GameName::default()),
            number_of_delivery_periods: 4,
            ranking_calculator: GameRankings { tier_limits: None },
            build_stack: default_stack_plants,
        }
    }

    pub fn open_game() -> (GameContext, MarketContext<MockMarket>) {
        let (tx_1, _) = mpsc::channel(16);
        let (tx_2, _) = mpsc::channel(16);
        let connections = MockPlayerConnections {
            tx_send_to_all_players: tx_1,
            tx_send_to_player: tx_2,
        };
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MockMarket { state_tx };
        let token = CancellationToken::new();

        let market_context = MarketContext {
            service: market,
            state_rx: rx,
        };
        let game_config = default_game_config();
        (
            GameActor::start(game_config, connections, market_context.clone(), token),
            market_context,
        )
    }

    pub async fn register_player(
        game: mpsc::Sender<GameMessage>,
    ) -> (PlayerId, PlayerName, StackContext<StackService>) {
        let player = PlayerName::random();
        let (tx, rx) = oneshot::channel::<RegisterPlayerResponse>();
        let _ = game
            .send(GameMessage::RegisterPlayer {
                name: player.clone(),
                tx_back: tx,
            })
            .await;
        match rx.await {
            Ok(RegisterPlayerResponse::Success { id, stack }) => (id, player, stack),
            _ => unreachable!("Should have register the player"),
        }
    }

    pub async fn start_game(
        game: mpsc::Sender<GameMessage>,
    ) -> (PlayerId, StackContext<StackService>) {
        let (player, _, stack) = register_player(game.clone()).await;
        let _ = game.send(GameMessage::PlayerIsReady(player.clone())).await;
        (player, stack)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use tokio::{
        sync::{
            mpsc,
            oneshot::{self, channel},
            watch,
        },
        time::timeout,
    };
    use tokio_util::sync::CancellationToken;

    use crate::{
        game::{
            GameId, GameMessage, GameName, GameState, GetPreviousScoresResult,
            RegisterPlayerResponse,
            delivery_period::DeliveryPeriodId,
            infra::actor::{
                GameActorConfig,
                test_utils::{
                    MockMarket, MockPlayerConnections, default_game_config, open_game,
                    register_player, start_game,
                },
            },
            scores::GameRankings,
        },
        market::{MarketContext, MarketState},
        plants::infra::{StackState, actor::default_stack_plants},
        player::{PlayerId, PlayerMessage, PlayerName},
    };

    use super::GameActor;

    #[tokio::test]
    async fn test_register_players() {
        let (game, _) = open_game();

        // Register a player
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        game.tx
            .send(GameMessage::RegisterPlayer {
                name: PlayerName::from("toto"),
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
                name: PlayerName::from("tutu"),
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
                name: PlayerName::from("toto"),
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
                name: PlayerName::from("toto"),
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
                name: PlayerName::from("tata"),
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
                name: PlayerName::from("toto"),
                tx_back: tx,
            })
            .await;

        match rx.await {
            Ok(RegisterPlayerResponse::GameStarted) => {}
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
                name: PlayerName::from("toto"),
                tx_back: tx,
            })
            .await;

        match rx.await {
            Ok(RegisterPlayerResponse::GameStarted) => {}
            _ => unreachable!("Should have rejected the request"),
        };
    }

    #[tokio::test(start_paused = true)]
    async fn test_starting_the_game_should_open_market_and_stacks() {
        let (game, mut market) = open_game();

        assert_eq!(*market.state_rx.borrow_and_update(), MarketState::Closed);

        let (player, _, mut stack) = register_player(game.tx.clone()).await;

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
        let (first_player, _, _) = register_player(game.tx.clone()).await;
        let (second_player, _, _) = register_player(game.tx.clone()).await;
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
        let (connections, mut rx_send_to_player, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let token = CancellationToken::new();
        let game = GameActor::start(default_game_config(), connections, market, token);

        // Start game
        let (player, _) = start_game(game.tx.clone()).await;

        // End delivery period
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        // Flush stack snapshot sent at the end of the delivery
        let _ = rx_send_to_player.recv().await;
        let _ = rx_send_to_player.recv().await;

        let Some((
            target_player,
            PlayerMessage::DeliveryPeriodResults {
                score: _,
                delivery_period,
            },
        )) = rx_send_to_player.recv().await
        else {
            unreachable!("Should have received player's results");
        };
        assert_eq!(delivery_period, DeliveryPeriodId::from(1));
        assert_eq!(target_player, player);
    }

    #[tokio::test]
    async fn test_get_previous_scores() {
        // FIXME
        let (connections, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let token = CancellationToken::new();
        let mut game = GameActor::start(default_game_config(), connections, market, token);
        game.state_rx.borrow_and_update();

        // Start the game
        let (player, _) = start_game(game.tx.clone()).await;

        // Scores should be empty at this stage
        let (tx_back, rx) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(GetPreviousScoresResult::PlayerScores { scores }) = rx.await else {
            unreachable!()
        };
        assert!(scores.is_empty());

        // End delivery period
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        // Wait for game state to update
        while *game.state_rx.borrow_and_update()
            != GameState::PostDelivery(DeliveryPeriodId::from(1))
        {
            let _ = game.state_rx.changed().await;
        }

        // Request player's scores
        let (tx_back, rx) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(GetPreviousScoresResult::PlayerScores { scores }) = rx.await else {
            unreachable!()
        };
        assert_eq!(scores.len(), 1);
    }

    #[tokio::test]
    async fn test_game_ends_after_max_delivery_periods() {
        let (connections, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };

        let token = CancellationToken::new();
        // Create a game with 2 max delivery periods
        let mut game = GameActor::start(
            GameActorConfig {
                number_of_delivery_periods: 2,
                ..default_game_config()
            },
            connections,
            market,
            token,
        );

        // Register and start the game with one player
        let (player, _, _) = register_player(game.tx.clone()).await;

        // Player is ready for the first time - starts period 1
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(1))
        );

        // End delivery period 1
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::PostDelivery(DeliveryPeriodId::from(1))
        );

        // Player is ready for the second time - starts period 2
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(2))
        );

        // End delivery period 2
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::PostDelivery(DeliveryPeriodId::from(2))
        );

        // Player is ready again - this should end the game instead of starting period 3
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Ended(DeliveryPeriodId::from(2))
        );
    }

    #[tokio::test]
    async fn test_send_all_scores_when_game_ends() {
        async fn mark_players_ready(
            game_tx: &mpsc::Sender<GameMessage>,
            player1: &PlayerId,
            player2: &PlayerId,
        ) {
            let _ = game_tx
                .send(GameMessage::PlayerIsReady(player1.clone()))
                .await;
            let _ = game_tx
                .send(GameMessage::PlayerIsReady(player2.clone()))
                .await;
        }

        // Create a game with 1 delivery period
        let (connections, _, mut rx_send_to_all_players) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let token = CancellationToken::new();
        let mut game = GameActor::start(
            GameActorConfig {
                number_of_delivery_periods: 1,
                ..default_game_config()
            },
            connections,
            market,
            token,
        );

        // Register two players
        let (player1, _, _) = register_player(game.tx.clone()).await;
        let (player2, _, _) = register_player(game.tx.clone()).await;

        // All players ready, starts period 1
        mark_players_ready(&game.tx, &player1, &player2).await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Running(DeliveryPeriodId::from(1))
        );

        // End delivery period 1
        mark_players_ready(&game.tx, &player1, &player2).await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::PostDelivery(DeliveryPeriodId::from(1))
        );

        // End game - both players ready
        mark_players_ready(&game.tx, &player1, &player2).await;
        assert!(game.state_rx.changed().await.is_ok());
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Ended(DeliveryPeriodId::from(1))
        );

        // Game should send all players scores to every player
        let mut message_found = false;
        while let Ok(Some(msg)) =
            timeout(Duration::from_micros(10), rx_send_to_all_players.recv()).await
        {
            if let PlayerMessage::GameResults { rankings: _ } = msg {
                message_found = true;
                break;
            }
        }
        assert!(message_found)
    }

    #[tokio::test]
    async fn test_get_final_scores_as_previous_scores_when_game_ended() {
        // Create a game with 1 delivery period and register 1 player
        let (connections, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let token = CancellationToken::new();
        let mut game = GameActor::start(
            GameActorConfig {
                number_of_delivery_periods: 1,
                ..default_game_config()
            },
            connections,
            market,
            token,
        );
        let (player, _, _) = register_player(game.tx.clone()).await;

        // Start the game
        let _ = game
            .tx
            .send(GameMessage::PlayerIsReady(player.clone()))
            .await;
        assert!(game.state_rx.changed().await.is_ok());

        let (tx_back, rx_back) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(GetPreviousScoresResult::PlayerScores { scores: _ }) = rx_back.await else {
            unreachable!("Should have received scores for the player");
        };

        // End the game, previous scores should include scores for all players
        for _ in 0..2 {
            let _ = game
                .tx
                .send(GameMessage::PlayerIsReady(player.clone()))
                .await;
            assert!(game.state_rx.changed().await.is_ok());
        }
        assert_eq!(
            *game.state_rx.borrow_and_update(),
            GameState::Ended(DeliveryPeriodId::from(1))
        );

        let (tx_back, rx_back) = oneshot::channel();
        let _ = game
            .tx
            .send(GameMessage::GetScores {
                player_id: player.clone(),
                tx_back,
            })
            .await;
        let Ok(GetPreviousScoresResult::PlayersRanking { scores: _ }) = rx_back.await else {
            unreachable!("Should have received scores for all players");
        };
    }

    #[tokio::test]
    async fn test_terminate_actor() {
        let (connections, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market_context = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let (state_tx, _) = watch::channel(GameState::Open);
        let cancellation_token = CancellationToken::new();
        let (tx, rx) = mpsc::channel(128);
        let mut game = GameActor {
            game_id: GameId::default(),
            name: GameName::default(),
            state: GameState::Open,
            state_watch: state_tx,
            players: Vec::new(),
            players_scores: HashMap::new(),
            players_connections: connections,
            market_context,
            stacks_contexts: HashMap::new(),
            build_stack: default_stack_plants,
            tx,
            rx,
            current_delivery_period: DeliveryPeriodId::default(),
            last_delivery_period: DeliveryPeriodId::from(3),
            delivery_period_timers: None,
            all_players_ready_tx: None,
            ranking_calculator: GameRankings { tier_limits: None },
            cancellation_token: cancellation_token.clone(),
        };
        let handle = tokio::spawn(async move {
            game.run().await;
        });

        cancellation_token.cancel();
        match tokio::time::timeout(Duration::from_millis(10), handle).await {
            Err(_) => unreachable!("Should have ended game actor"),
            Ok(_) => {}
        }
    }
}

#[cfg(test)]
mod test_rankings_mapping {
    use crate::{
        game::{Player, infra::actor::map_rankings_to_player_name, scores::PlayerResult},
        player::{PlayerId, PlayerName, PlayerResultView},
    };

    #[test]
    fn test_map_to_players_name() {
        let player_id = PlayerId::default();
        let player_name = PlayerName::random();
        let players = vec![Player {
            id: player_id.clone(),
            name: player_name.clone(),
            ready: false,
        }];
        let rankings = vec![PlayerResult {
            player: player_id.clone(),
            rank: 1,
            score: 0,
            tier: None,
        }];

        assert_eq!(
            map_rankings_to_player_name(rankings, &players),
            vec![PlayerResultView {
                player: player_name.clone(),
                rank: 1,
                score: 0,
                tier: None,
            }]
        );
    }

    #[test]
    fn test_mapping_no_player_name_found_keeps_its_id() {
        let player_id = PlayerId::default();
        let players = vec![];
        let rankings = vec![PlayerResult {
            player: player_id.clone(),
            rank: 1,
            score: 0,
            tier: None,
        }];

        assert_eq!(
            map_rankings_to_player_name(rankings, &players),
            vec![PlayerResultView {
                player: PlayerName::from(player_id.to_string()),
                rank: 1,
                score: 0,
                tier: None,
            }]
        );
    }
}

#[cfg(test)]
mod test_readiness_status {

    use crate::{game::infra::actor::test_utils::register_player, market::MarketState};

    use super::{
        test_utils::{MockMarket, MockPlayerConnections, default_game_config},
        *,
    };
    use tokio::sync::mpsc;

    fn start_game() -> (mpsc::Receiver<PlayerMessage>, GameContext) {
        let (connections, _, rx_send_to_all_players) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let token = CancellationToken::new();
        let game = GameActor::start(
            GameActorConfig {
                number_of_delivery_periods: 1,
                ..default_game_config()
            },
            connections,
            market,
            token,
        );
        (rx_send_to_all_players, game)
    }

    #[tokio::test]
    async fn test_send_readiness_status_when_player_registers() {
        let (mut conn_rx, game) = start_game();
        let (_, name, _) = register_player(game.tx.clone()).await;

        let Some(PlayerMessage::ReadinessStatus { readiness }) = conn_rx.recv().await else {
            unreachable!("Should have received a readiness status message")
        };
        assert!(readiness.contains_key(&name));
        assert_eq!(*readiness.get(&name).unwrap(), false);
    }

    #[tokio::test]
    async fn test_send_readiness_status_when_player_is_ready() {
        let (mut conn_rx, game) = start_game();
        let (player_id, name, _) = register_player(game.tx.clone()).await;
        // Consume first readiness status
        let _ = conn_rx.recv().await;

        let (_, _, _) = register_player(game.tx.clone()).await;
        // Consume second readiness status
        let _ = conn_rx.recv().await;

        // Player is ready
        let _ = game.tx.send(GameMessage::PlayerIsReady(player_id)).await;

        // Receive updated readiness status
        let Some(PlayerMessage::ReadinessStatus { readiness }) = conn_rx.recv().await else {
            unreachable!("Should have received a readiness status message")
        };
        assert!(readiness.contains_key(&name));
        assert_eq!(*readiness.get(&name).unwrap(), true);
    }

    #[tokio::test]
    async fn test_readiness_status_is_reset_to_false_when_all_players_ready() {
        let (mut conn_rx, game) = start_game();
        let (player_id, name, _) = register_player(game.tx.clone()).await;
        // Consume first readiness status
        let _ = conn_rx.recv().await;

        let (player2, _, _) = register_player(game.tx.clone()).await;
        // Consume second readiness status
        let _ = conn_rx.recv().await;

        // Player is ready, consume readiness message
        let _ = game.tx.send(GameMessage::PlayerIsReady(player_id)).await;
        let _ = conn_rx.recv().await;

        // All players are ready
        let _ = game.tx.send(GameMessage::PlayerIsReady(player2)).await;

        // Receive updated readiness status
        let Some(PlayerMessage::ReadinessStatus { readiness }) = conn_rx.recv().await else {
            unreachable!("Should have received a readiness status message")
        };
        assert!(readiness.contains_key(&name));
        assert_eq!(*readiness.get(&name).unwrap(), false);
    }
}
