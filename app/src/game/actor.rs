use std::collections::HashMap;

use super::delivery_period::{start_delivery_period, DeliveryPeriodId, DeliveryPeriodResults};
use super::scores::{GameRankings, PlayerResult, PlayerScore};
use super::{GameContext, GameId, GameMessage, GetPreviousScoresResult};
use futures_util::future::join_all;
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot, watch,
};

use crate::game::{GameState, Player, RegisterPlayerResponse};
use crate::player::connection::{PlayerMessage, PlayerResultView};
use crate::player::PlayerName;
use crate::{
    market::{Market, MarketContext},
    plants::{
        actor::{StackActor, StackContext, StackState},
        service::StackService,
    },
    player::{repository::ConnectionRepositoryMessage, PlayerId},
};

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
    current_delivery_period: DeliveryPeriodId,
    last_delivery_period: Option<DeliveryPeriodId>,
    all_players_ready_tx: Option<oneshot::Sender<()>>,
    ranking_calculator: GameRankings,
}

impl<MS: Market> Game<MS> {
    pub fn start(
        game_id: &GameId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
        market_context: MarketContext<MS>,
        last_delivery_period: Option<DeliveryPeriodId>,
        ranking_calculator: GameRankings,
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
            current_delivery_period: DeliveryPeriodId::default(),
            last_delivery_period,
            all_players_ready_tx: None,
            ranking_calculator,
        };
        let context = game.get_context();

        tokio::spawn(async move { game.run().await });

        context
    }

    async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            self.process_message(msg).await;
        }
    }
    #[tracing::instrument(name = "GameActor::process_message", skip(self))]
    async fn process_message(&mut self, message: GameMessage) {
        match (&mut self.state, message) {
            (GameState::Open, GameMessage::RegisterPlayer { name, tx_back }) => {
                self.register_player(name, tx_back);
            }
            (GameState::PostDelivery(_), GameMessage::RegisterPlayer { tx_back, .. })
            | (GameState::Running(_), GameMessage::RegisterPlayer { tx_back, .. })
            | (GameState::Ended(_), GameMessage::RegisterPlayer { tx_back, .. }) => {
                let _ = tx_back.send(RegisterPlayerResponse::GameStarted);
            }
            (_, GameMessage::GetScores { player_id, tx_back }) => {
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
                            .get(&player_id)
                            .cloned()
                            .unwrap_or_else(HashMap::new),
                    },
                };
                let _ = tx_back.send(scores);
            }
            (GameState::PostDelivery(_), GameMessage::PlayerIsReady(player_id))
            | (GameState::Open, GameMessage::PlayerIsReady(player_id)) => {
                let _ = self.register_player_ready(player_id).await;
            }
            (GameState::Running(_), GameMessage::DeliveryPeriodResults(results)) => {
                self.store_scores(&results);
                self.state = GameState::PostDelivery(self.current_delivery_period);
                let _ = self
                    .state_watch
                    .send(GameState::PostDelivery(self.current_delivery_period));
                join_all(results.players_scores.iter().map(|(player, score)| {
                    self.players_connections
                        .send(ConnectionRepositoryMessage::SendToPlayer(
                            self.game_id.clone(),
                            player.clone(),
                            crate::player::connection::PlayerMessage::DeliveryPeriodResults {
                                score: score.clone(),
                                delivery_period: self.current_delivery_period,
                            },
                        ))
                }))
                .await;
            }
            (_, GameMessage::DeliveryPeriodResults(results)) => {
                tracing::warn!(
                    "Received results for delivery period {:?} while game is not running",
                    results.period_id
                );
            }
            (GameState::Running(_), GameMessage::PlayerIsReady(player_id)) => {
                self.register_player_ready_game_is_running(player_id);
            }
            (GameState::Ended(_), GameMessage::PlayerIsReady(_)) => {
                tracing::info!("Player ready in ended game - ignoring");
            }
        }
    }

    fn check_last_delivery_period_reached(&mut self) -> bool {
        if let Some(last_period) = self.last_delivery_period {
            if self.current_delivery_period >= last_period {
                tracing::info!(
                    "Maximum delivery periods reached ({}), ending game",
                    last_period
                );
                self.state = GameState::Ended(self.current_delivery_period);
                let _ = self
                    .state_watch
                    .send(GameState::Ended(self.current_delivery_period));
                return true;
            }
        }
        false
    }

    async fn register_player_ready(&mut self, player_id: PlayerId) {
        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.id == player_id)
        {
            player.ready = true;
        }

        // If all players are ready, check if max periods reached or start the next delivery period
        if self.players.iter().all(|player| player.ready) {
            // Check if we've reached the last delivery period
            if self.check_last_delivery_period_reached() {
                let _ = self
                    .players_connections
                    .send(ConnectionRepositoryMessage::SendToAllPlayers(
                        self.game_id.clone(),
                        PlayerMessage::GameResults {
                            rankings: map_rankings_to_player_name(
                                self.ranking_calculator.compute_scores(&self.players_scores),
                                &self.players,
                            ),
                        },
                    ))
                    .await;
                return; // Game has ended
            }

            tracing::info!(
                "All players ready, starting delivery period {}",
                self.current_delivery_period.next()
            );
            self.state = GameState::Running(self.current_delivery_period);
            self.current_delivery_period = self.current_delivery_period.next();
            for player in self.players.iter_mut() {
                player.ready = false;
            }
            let delivery_period = self.current_delivery_period;
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
                .send(GameState::Running(self.current_delivery_period));
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
            tracing::info!("All players ready, ending delivery period");
            if let Some(tx) = self.all_players_ready_tx.take() {
                let _ = tx.send(());
                for player in self.players.iter_mut() {
                    player.ready = false;
                }
            }
        }
    }

    fn register_player(
        &mut self,
        name: PlayerName,
        tx_back: oneshot::Sender<RegisterPlayerResponse>,
    ) {
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
            self.current_delivery_period,
            self.players_connections.clone(),
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
mod tests {
    use std::time::Duration;

    use tokio::{
        sync::{
            mpsc,
            oneshot::{self, channel},
            watch,
        },
        time::timeout,
    };

    use crate::{
        game::{
            delivery_period::DeliveryPeriodId, scores::GameRankings, GameContext, GameId,
            GameMessage, GameState, GetPreviousScoresResult, RegisterPlayerResponse,
        },
        market::{order_book::TradeLeg, Market, MarketContext, MarketForecast, MarketState, OBS},
        plants::{
            actor::{StackContext, StackState},
            StackService,
        },
        player::{
            connection::PlayerMessage, repository::ConnectionRepositoryMessage, PlayerId,
            PlayerName,
        },
    };

    use super::Game;

    #[derive(Clone)]
    struct MockMarket {
        state_tx: watch::Sender<MarketState>,
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
        async fn get_market_snapshot(
            &self,
            _player: PlayerId,
        ) -> (Vec<TradeLeg>, OBS, Vec<MarketForecast>) {
            (
                Vec::new(),
                OBS {
                    offers: Vec::new(),
                    bids: Vec::new(),
                },
                Vec::new(),
            )
        }
        async fn new_order(&self, _request: crate::market::order_book::OrderRequest) {}
        async fn open_market(&self, _delivery_period: DeliveryPeriodId) {
            let _ = self.state_tx.send(MarketState::Open);
        }
        async fn register_forecast(&self, _forecast: crate::market::MarketForecast) {}
    }

    fn open_game() -> (GameContext, MarketContext<MockMarket>) {
        let (tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MockMarket { state_tx };
        let ranking_calculator = GameRankings { tier_limits: None };

        let market_context = MarketContext {
            service: market,
            state_rx: rx,
        };
        (
            Game::start(
                &game_id,
                tx,
                market_context.clone(),
                None,
                ranking_calculator,
            ),
            market_context,
        )
    }

    async fn register_player(
        game: mpsc::Sender<GameMessage>,
    ) -> (PlayerId, StackContext<StackService>) {
        let player = PlayerName::random();
        let (tx, rx) = channel::<RegisterPlayerResponse>();
        let _ = game
            .send(GameMessage::RegisterPlayer {
                name: player,
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
        let ranking_calculator = GameRankings { tier_limits: None };
        let game = Game::start(&game_id, conn_tx, market, None, ranking_calculator);

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
        let ranking_calculator = GameRankings { tier_limits: None };
        let game = Game::start(&game_id, conn_tx, market, None, ranking_calculator);

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
        let _ = conn_rx.recv().await;

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
        let (conn_tx, _conn_rx) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };

        let ranking_calculator = GameRankings { tier_limits: None };
        // Create a game with 2 max delivery periods
        let mut game = Game::start(
            &game_id,
            conn_tx,
            market,
            Some(DeliveryPeriodId::from(2)),
            ranking_calculator,
        );

        // Register and start the game with one player
        let (player, _) = register_player(game.tx.clone()).await;

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
        let (conn_tx, mut conn_rx) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let ranking_calculator = GameRankings { tier_limits: None };
        let mut game = Game::start(
            &game_id,
            conn_tx,
            market,
            Some(DeliveryPeriodId::from(1)),
            ranking_calculator,
        );

        // Register two players
        let (player1, _) = register_player(game.tx.clone()).await;
        let (player2, _) = register_player(game.tx.clone()).await;

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
        while let Ok(Some(msg)) = timeout(Duration::from_micros(10), conn_rx.recv()).await {
            if let ConnectionRepositoryMessage::SendToAllPlayers(
                _,
                PlayerMessage::GameResults { rankings: _ },
            ) = msg
            {
                message_found = true;
                break;
            }
        }
        assert!(message_found)
    }

    #[tokio::test]
    async fn test_get_final_scores_as_previous_scores_when_game_ended() {
        // Create a game with 1 delivery period and register 1 player
        let (conn_tx, _) = mpsc::channel(16);
        let game_id = GameId::default();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let ranking_calculator = GameRankings { tier_limits: None };
        let mut game = Game::start(
            &game_id,
            conn_tx,
            market,
            Some(DeliveryPeriodId::from(1)),
            ranking_calculator,
        );
        let (player, _) = register_player(game.tx.clone()).await;

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
}

#[cfg(test)]
mod test_rankings_mapping {
    use crate::{
        game::{actor::map_rankings_to_player_name, scores::PlayerResult, Player},
        player::{connection::PlayerResultView, PlayerId, PlayerName},
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
