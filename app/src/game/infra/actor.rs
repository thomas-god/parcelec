use std::{collections::HashMap, time::Duration};

use futures_util::future::join_all;
use tokio::{
    sync::{
        mpsc::{Receiver, Sender, channel},
        oneshot, watch,
    },
    time::sleep,
};
use tokio_util::sync::CancellationToken;

use crate::{
    game::{
        Game, GameContext, GameEvent, GameId, GameMessage, GameName, GameState,
        GetPreviousScoresResult, RegisterPlayerResponse,
        delivery_period::{DeliveryPeriodId, DeliveryPeriodResults, start_delivery_period},
        scores::{PlayerDetailedScore, PlayerResult, PlayerScore, compute_game_rankings},
    },
    plants::{
        StackPlants,
        infra::{StackActor, StackState},
    },
};
use crate::{
    market::{Market, MarketContext},
    plants::infra::{StackContext, StackService},
    player::{PlayerConnections, PlayerId, PlayerMessage, PlayerName, PlayerResultView},
};

/// Main entrypoint for a given game of parcelec. Responsible for:
/// - new player registration,
/// - passing game context to new player connection (market and player's stack tx),
/// - delivery period lifecycle
pub struct GameActor<
    MS: Market,
    PC: PlayerConnections,
    F: Fn() -> StackPlants + Clone + Send + Sync + 'static,
> {
    game: Game,
    cache: GameCache,
    config: GameActorConfig<F>,
    state_watch: watch::Sender<GameState>,
    players_connections: PC,
    market_context: MarketContext<MS>,
    stacks_contexts: HashMap<PlayerId, StackContext<StackService>>,
    rx: Receiver<GameMessage>,
    tx: Sender<GameMessage>,
    delivery_period_all_players_ready_tx: Option<oneshot::Sender<()>>,
    cancellation_token: CancellationToken,
}

struct GameCache {
    state: GameState,
    players_readiness: HashMap<PlayerName, bool>,
    players_scores: HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerScore>>,
    players_detailed_scores: HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerDetailedScore>>,
    players_id_to_name: HashMap<PlayerId, PlayerName>,
}

impl Default for GameCache {
    fn default() -> Self {
        Self {
            state: GameState::Open,
            players_readiness: HashMap::new(),
            players_scores: HashMap::new(),
            players_detailed_scores: HashMap::new(),
            players_id_to_name: HashMap::new(),
        }
    }
}

pub struct GameActorConfig<F>
where
    F: Fn() -> StackPlants + Clone + Send + Sync + 'static,
{
    pub id: GameId,
    pub name: GameName,
    pub number_of_delivery_periods: usize,
    pub delivery_period_duration: Option<Duration>,
    pub build_stack: F,
}

impl<MS: Market, PC: PlayerConnections, F: Fn() -> StackPlants + Clone + Send + Sync + 'static>
    GameActor<MS, PC, F>
{
    pub fn start(
        config: GameActorConfig<F>,
        players_connections: PC,
        market_context: MarketContext<MS>,
        cancelation_token: CancellationToken,
    ) -> GameContext {
        let game = Game::init(
            DeliveryPeriodId::from(config.number_of_delivery_periods),
            config.delivery_period_duration,
        );
        let (tx, rx) = channel::<GameMessage>(32);
        let (state_tx, _) = watch::channel(game.state.clone());
        let mut game = GameActor {
            cache: GameCache::default(),
            game,
            config,
            state_watch: state_tx,
            market_context,
            players_connections,
            stacks_contexts: HashMap::new(),
            rx,
            tx,
            delivery_period_all_players_ready_tx: None,
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
                    tracing::info!("Game actor {:?} terminated", self.config.id);
                    break;
                }
            }
        }
    }

    #[tracing::instrument(name = "GameActor::process_message", skip(self))]
    async fn process_message(&mut self, message: GameMessage) {
        let events = match message {
            GameMessage::RegisterPlayer { name, tx_back } => {
                match self.game.try_register_player(name) {
                    Ok(events) => {
                        if let Some(id) = events.iter().find_map(|e| match e {
                            GameEvent::PlayerJoined { id, .. } => Some(id.clone()),
                            _ => None,
                        }) {
                            self.create_player_stack(&id, tx_back).await;
                        }
                        events
                    }
                    Err(err) => {
                        let _ = tx_back.send(match err {
                            crate::game::RegisterPlayerError::GameStarted => {
                                RegisterPlayerResponse::GameStarted
                            }
                            crate::game::RegisterPlayerError::NameAlreadyExists => {
                                RegisterPlayerResponse::PlayerAlreadyExist
                            }
                        });
                        vec![]
                    }
                }
            }
            GameMessage::GetScores { player_id, tx_back } => {
                self.send_scores(player_id, tx_back);
                vec![]
            }
            GameMessage::GetReadiness { tx_back } => {
                let _ = tx_back.send(self.cache.players_readiness.clone());
                vec![]
            }
            GameMessage::PlayerIsReady(player_id) => self.game.register_player_ready(&player_id),
            GameMessage::DeliveryPeriodResults(results) => {
                self.update_cached_scores(&results);
                let events = self.game.process_delivery_period_results(&results);
                self.send_scores_to_all_players(&results.period_id).await;
                events
            }
            GameMessage::PostDeliveryPeriodEnded(period) => {
                self.game.process_post_delivery_period_ends(&period)
            }
        };

        self.process_game_events(events).await;
    }

    async fn process_game_events(&mut self, events: Vec<GameEvent>) {
        for event in events {
            match event {
                GameEvent::PlayerJoined { id, name } => {
                    self.cache.players_id_to_name.insert(id, name);
                }
                GameEvent::StateUpdated(state) => {
                    self.cache.state = state;
                    let _ = self.state_watch.send(self.cache.state.clone());

                    if let GameState::Ended(_) = self.cache.state {
                        self.send_final_scores().await;
                    }
                }
                GameEvent::PlayerReadinessChanged { readiness } => {
                    self.cache.players_readiness = readiness;
                    let _ = self
                        .players_connections
                        .send_to_all_players(
                            &self.config.id,
                            PlayerMessage::ReadinessStatus {
                                readiness: self.cache.players_readiness.clone(),
                            },
                        )
                        .await;
                }
                GameEvent::DeliveryPeriodStarted { id } => {
                    self.start_delivery_period_tasks(id);
                }
                GameEvent::DeliveryPeriodEnded { id } => {
                    if let Some(tx) = self.delivery_period_all_players_ready_tx.take() {
                        let _ = tx.send(());
                    }
                    let timer = self.config.delivery_period_duration;
                    let game_tx = self.tx.clone();
                    tokio::spawn(async move {
                        wait_for_post_delivery_period_end(id, timer, game_tx).await;
                    });
                }
            }
        }
    }

    fn start_delivery_period_tasks(&mut self, id: DeliveryPeriodId) {
        let game_tx = self.tx.clone();
        let market_service = self.market_context.service.clone();
        let stacks_tx = self
            .stacks_contexts
            .iter()
            .map(|(id, context)| (id.clone(), context.service.clone()))
            .collect();
        let (all_players_ready_tx, all_players_ready_rx) = oneshot::channel();
        let timers = self.config.delivery_period_duration;
        let token = self.cancellation_token.clone();
        tokio::spawn(async move {
            start_delivery_period(
                id,
                game_tx,
                market_service,
                stacks_tx,
                all_players_ready_rx,
                timers,
                token,
            )
            .await;
        });
        self.delivery_period_all_players_ready_tx = Some(all_players_ready_tx);
    }

    fn update_cached_scores(&mut self, results: &DeliveryPeriodResults) {
        for (player, scores) in results.players_scores.iter() {
            match self.cache.players_scores.get_mut(player) {
                Some(cached_scores) => {
                    cached_scores.insert(results.period_id, scores.clone());
                }
                None => {
                    self.cache.players_scores.insert(
                        player.clone(),
                        HashMap::from_iter([(results.period_id, scores.clone())]),
                    );
                }
            }
        }

        for (player, scores) in results.players_detailed_scores.iter() {
            match self.cache.players_detailed_scores.get_mut(player) {
                Some(cached_scores) => {
                    cached_scores.insert(results.period_id, scores.clone());
                }
                None => {
                    self.cache.players_detailed_scores.insert(
                        player.clone(),
                        HashMap::from_iter([(results.period_id, scores.clone())]),
                    );
                }
            }
        }
    }

    fn send_scores(&self, player: PlayerId, tx_back: oneshot::Sender<GetPreviousScoresResult>) {
        use GetPreviousScoresResult::*;
        let scores = match self.cache.state {
            GameState::Ended(_) => PlayersRanking {
                scores: map_rankings_to_player_name(
                    compute_game_rankings(&self.cache.players_scores.clone()),
                    &self.cache.players_id_to_name,
                ),
            },
            _ => PlayerScores {
                scores: self
                    .cache
                    .players_scores
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(HashMap::new),
                detailed_scores: self
                    .cache
                    .players_detailed_scores
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(HashMap::new),
            },
        };
        let _ = tx_back.send(scores);
    }

    async fn send_scores_to_all_players(&self, period: &DeliveryPeriodId) {
        let mut tasks = vec![];

        for (player, scores) in self.cache.players_scores.iter() {
            let Some(score) = scores.get(period) else {
                continue;
            };
            let detailed_score = self
                .cache
                .players_detailed_scores
                .get(player)
                .and_then(|scores| scores.get(period));

            tasks.push(self.players_connections.send_to_player(
                &self.config.id,
                player,
                PlayerMessage::DeliveryPeriodResults {
                    delivery_period: *period,
                    score: score.clone(),
                    detailed_score: detailed_score.cloned(),
                },
            ))
        }

        join_all(tasks).await;
    }

    async fn send_final_scores(&self) {
        let _ = self
            .players_connections
            .send_to_all_players(
                &self.config.id,
                PlayerMessage::GameResults {
                    rankings: map_rankings_to_player_name(
                        compute_game_rankings(&self.cache.players_scores),
                        &self.cache.players_id_to_name,
                    ),
                },
            )
            .await;
    }

    fn get_context(&self) -> GameContext {
        GameContext {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            last_delivery_period: DeliveryPeriodId::from(self.config.number_of_delivery_periods),
            tx: self.tx.clone(),
            state_rx: self.state_watch.subscribe(),
        }
    }

    async fn create_player_stack(
        &mut self,
        player_id: &PlayerId,
        tx_back: tokio::sync::oneshot::Sender<RegisterPlayerResponse>,
    ) {
        // Create a new stack for the player
        let mut player_stack = StackActor::new(
            self.config.id.clone(),
            player_id.clone(),
            (self.config.build_stack)(),
            StackState::Closed,
            self.game.current_delivery_period(),
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
            id: player_id.clone(),
            stack: stack_context,
        });
    }
}

async fn wait_for_post_delivery_period_end(
    period: DeliveryPeriodId,
    timer: Option<Duration>,
    game_tx: Sender<GameMessage>,
) {
    if let Some(duration) = timer {
        sleep(duration).await;
        let _ = game_tx
            .send(GameMessage::PostDeliveryPeriodEnded(period))
            .await;
    }
}

fn map_rankings_to_player_name(
    rankings: Vec<PlayerResult>,
    players_mapping: &HashMap<PlayerId, PlayerName>,
) -> Vec<PlayerResultView> {
    rankings
        .iter()
        .filter_map(|rank| {
            let name = players_mapping.get(&rank.player)?;

            Some(PlayerResultView {
                player: name.clone(),
                rank: rank.rank,
                score: rank.score,
            })
        })
        .collect()
}

#[cfg(test)]
mod test_utils {
    use tokio::sync::mpsc;

    use crate::{
        market::{MarketState, OBS, order_book::TradeLeg},
        plants::infra::actor::default_stack_plants_builder,
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

    pub fn default_game_config() -> GameActorConfig<impl Fn() -> StackPlants + Clone + Send + Sync>
    {
        GameActorConfig {
            delivery_period_duration: None,
            id: GameId::default(),
            name: GameName::default(),
            number_of_delivery_periods: 4,
            build_stack: default_stack_plants_builder(),
        }
    }

    pub struct TestComms {
        pub state_watch_rx: tokio::sync::watch::Receiver<GameState>,
        pub rx_player: Receiver<(PlayerId, PlayerMessage)>,
        #[allow(dead_code)]
        pub rx_all_players: Receiver<PlayerMessage>,
    }

    pub fn build_game_actor() -> (
        GameActor<
            MockMarket,
            test_utils::MockPlayerConnections,
            impl (Fn() -> StackPlants) + Clone + std::marker::Send + Sync,
        >,
        TestComms,
    ) {
        let config = default_game_config();
        let game = Game::init(
            DeliveryPeriodId::from(config.number_of_delivery_periods),
            config.delivery_period_duration,
        );
        let (tx, rx) = channel::<GameMessage>(32);
        let (state_watch, state_watch_rx) = watch::channel(game.state.clone());
        let cancellation_token = CancellationToken::new();
        let (players_connections, rx_player, rx_all_players) = MockPlayerConnections::new();
        let (state_tx, state_rx) = watch::channel(MarketState::Closed);
        let market_context = MarketContext {
            service: MockMarket { state_tx },
            state_rx,
        };

        let actor = GameActor {
            config,
            state_watch,
            market_context,
            players_connections,
            stacks_contexts: HashMap::new(),
            rx,
            tx,
            delivery_period_all_players_ready_tx: None,
            cancellation_token,
            cache: GameCache::default(),
            game,
        };
        let comms = TestComms {
            state_watch_rx,
            rx_player,
            rx_all_players,
        };

        (actor, comms)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use tokio::sync::{mpsc, watch};
    use tokio_util::sync::CancellationToken;

    use crate::{
        game::{
            Game, GameId, GameName,
            delivery_period::DeliveryPeriodId,
            infra::{
                GameActorConfig,
                actor::{
                    GameCache,
                    test_utils::{MockMarket, MockPlayerConnections},
                },
            },
        },
        market::{MarketContext, MarketState},
        plants::infra::actor::default_stack_plants_builder,
    };

    use super::GameActor;

    #[tokio::test]
    async fn test_terminate_actor() {
        let (connections, ..) = MockPlayerConnections::new();
        let (state_tx, rx) = watch::channel(MarketState::Closed);
        let market_context = MarketContext {
            service: MockMarket { state_tx },
            state_rx: rx,
        };
        let game = Game::init(DeliveryPeriodId::from(3), None);
        let (state_tx, _) = watch::channel(game.state.clone());
        let cancellation_token = CancellationToken::new();
        let (tx, rx) = mpsc::channel(128);
        let config = GameActorConfig {
            id: GameId::default(),
            name: GameName::default(),
            build_stack: default_stack_plants_builder(),
            number_of_delivery_periods: 3,
            delivery_period_duration: None,
        };
        let mut game = GameActor {
            config,
            state_watch: state_tx,
            players_connections: connections,
            market_context,
            stacks_contexts: HashMap::new(),
            tx,
            rx,
            delivery_period_all_players_ready_tx: None,
            cancellation_token: cancellation_token.clone(),
            cache: GameCache::default(),
            game,
        };
        let handle = tokio::spawn(async move {
            game.run().await;
        });

        cancellation_token.cancel();
        if tokio::time::timeout(Duration::from_millis(10), handle)
            .await
            .is_err()
        {
            unreachable!("Should have ended game actor")
        }
    }
}

#[cfg(test)]
mod test_game_actor_process_game_messages {
    use crate::game::infra::actor::test_utils::build_game_actor;

    use super::*;

    #[tokio::test]
    async fn test_register_player_success_create_stack() {
        let (mut game, _) = build_game_actor();
        let (tx_back, rx) = oneshot::channel();
        let msg = GameMessage::RegisterPlayer {
            name: PlayerName::from("p1"),
            tx_back,
        };

        assert!(game.stacks_contexts.is_empty());
        game.process_message(msg).await;

        let Ok(_) = rx.await else {
            unreachable!("Should have return success message")
        };
        assert!(!game.stacks_contexts.is_empty());
    }

    #[tokio::test]
    async fn test_get_scores() {
        let (mut game, _) = build_game_actor();
        let (tx_back, rx) = oneshot::channel();
        game.cache
            .players_scores
            .insert(PlayerId::from("p1"), HashMap::new());
        let msg = GameMessage::GetScores {
            player_id: PlayerId::from("p1"),
            tx_back,
        };

        game.process_message(msg).await;

        let Ok(_) = rx.await else {
            unreachable!("Should have return success message")
        };
    }

    #[tokio::test]
    async fn test_get_readiness() {
        let (mut game, _) = build_game_actor();
        let (tx_back, rx) = oneshot::channel();
        let msg = GameMessage::GetReadiness { tx_back };

        game.process_message(msg).await;

        let Ok(_) = rx.await else {
            unreachable!("Should have return success message")
        };
    }

    #[tokio::test]
    async fn test_delivery_period_results() {
        let (mut game, mut comms) = build_game_actor();
        game.cache.players_scores.insert(
            PlayerId::from("p1"),
            HashMap::from_iter([(DeliveryPeriodId::from(1), PlayerScore::default())]),
        );

        let results = DeliveryPeriodResults {
            period_id: DeliveryPeriodId::from(1),
            players_scores: HashMap::new(),
            players_detailed_scores: HashMap::new(),
        };
        let msg = GameMessage::DeliveryPeriodResults(results);

        game.process_message(msg).await;

        let Some((
            _id,
            PlayerMessage::DeliveryPeriodResults {
                delivery_period,
                score,
                detailed_score,
            },
        )) = comms.rx_player.recv().await
        else {
            unreachable!("Should have received a PlayerMessage::DeliveryPeriodResults")
        };
        assert_eq!(delivery_period, DeliveryPeriodId::from(1));
        assert_eq!(score, PlayerScore::default());
        assert!(detailed_score.is_none());
    }
}

#[cfg(test)]
mod test_game_actor_process_game_events {
    use crate::game::infra::actor::test_utils::build_game_actor;

    use super::*;

    #[tokio::test]
    async fn test_process_player_joined_mapping_updated() {
        let (mut game, _) = build_game_actor();

        assert!(game.cache.players_id_to_name.is_empty());

        let events = vec![GameEvent::PlayerJoined {
            id: PlayerId::from("p1"),
            name: PlayerName::from("p1"),
        }];

        game.process_game_events(events).await;
        assert_eq!(
            game.cache.players_id_to_name.get(&PlayerId::from("p1")),
            Some(&PlayerName::from("p1"))
        );
    }

    #[tokio::test]
    async fn test_process_game_state_updated() {
        let (mut game, comms) = build_game_actor();

        assert_eq!(game.cache.state, GameState::Open);

        let state = GameState::Running {
            period: DeliveryPeriodId::from(1),
            end_at: None,
        };
        let events = vec![GameEvent::StateUpdated(state.clone())];

        game.process_game_events(events).await;
        assert_eq!(game.cache.state, state);

        assert_eq!(*comms.state_watch_rx.borrow(), state);
    }

    #[tokio::test]
    async fn test_process_delivery_period_start() {
        let (mut game, _) = build_game_actor();

        let events = vec![GameEvent::DeliveryPeriodStarted {
            id: DeliveryPeriodId::from(1),
        }];
        assert!(game.delivery_period_all_players_ready_tx.is_none());

        game.process_game_events(events).await;

        // It's the best proxy we have to check the delivery period tasks are spwaned
        assert!(game.delivery_period_all_players_ready_tx.is_some());
    }

    #[tokio::test]
    async fn test_process_delivery_period_end() {
        let (mut game, _) = build_game_actor();
        let (all_players_ready_tx, all_players_ready_rx) = oneshot::channel();
        game.delivery_period_all_players_ready_tx = Some(all_players_ready_tx);

        let events = vec![GameEvent::DeliveryPeriodEnded {
            id: DeliveryPeriodId::from(1),
        }];

        game.process_game_events(events).await;

        let Ok(_) = all_players_ready_rx.await else {
            unreachable!("Should have trigger the all players ready channel")
        };
        // TODO: can we test the creation/end of the post delivery timer ?
    }
}

#[cfg(test)]
mod test_rankings_mapping {
    use std::collections::HashMap;

    use crate::{
        game::{infra::actor::map_rankings_to_player_name, scores::PlayerResult},
        player::{PlayerId, PlayerName, PlayerResultView},
        utils::units::Money,
    };

    #[test]
    fn test_map_to_players_name() {
        let player_id = PlayerId::default();
        let player_name = PlayerName::random();
        let players_mapping = HashMap::from_iter([(player_id.clone(), player_name.clone())]);
        let rankings = vec![PlayerResult {
            player: player_id.clone(),
            rank: 1,
            score: Money::from(0),
        }];

        assert_eq!(
            map_rankings_to_player_name(rankings, &players_mapping),
            vec![PlayerResultView {
                player: player_name.clone(),
                rank: 1,
                score: Money::from(0),
            }]
        );
    }

    #[test]
    fn test_mapping_no_player_name_is_dropped() {
        let player_id = PlayerId::default();
        let players_mapping = HashMap::new();
        let rankings = vec![PlayerResult {
            player: player_id.clone(),
            rank: 1,
            score: Money::from(0),
        }];

        assert!(map_rankings_to_player_name(rankings, &players_mapping).is_empty());
    }
}
