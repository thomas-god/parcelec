use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use delivery_period::{DeliveryPeriodId, DeliveryPeriodResults};
use derive_more::{AsRef, Display, From};
use petname::petname;
use scores::PlayerScore;
use serde::{Serialize, ser::SerializeStruct};
use tokio::sync::{
    mpsc::{self},
    oneshot, watch,
};

use crate::{
    game::{infra::stack_config::GameStackPerPlayerPlayerConfig, scores::PlayerDetailedScore},
    plants::infra::{StackContext, StackService},
    player::{PlayerId, PlayerName, PlayerResultView},
};

pub mod delivery_period;
pub mod infra;
pub mod scores;

pub use infra::actor::GameActor;

#[derive(Debug)]
pub struct Player {
    id: PlayerId,
    name: PlayerName,
    ready: bool,
}

#[derive(Debug)]
pub enum GetPreviousScoresResult {
    PlayerScores {
        scores: HashMap<DeliveryPeriodId, PlayerScore>,
        detailed_scores: HashMap<DeliveryPeriodId, PlayerDetailedScore>,
    },
    PlayersRanking {
        scores: Vec<PlayerResultView>,
    },
}

#[derive(Debug)]
pub enum GameMessage {
    RegisterPlayer {
        name: PlayerName,
        tx_back: oneshot::Sender<RegisterPlayerResponse>,
    },
    RegisterPlayerStackConfig {
        player: PlayerId,
        config: GameStackPerPlayerPlayerConfig,
        tx_back: oneshot::Sender<Result<StackContext<StackService>, RegisterPlayerStackError>>,
    },
    PlayerIsReady(PlayerId),
    DeliveryPeriodResults(DeliveryPeriodResults),
    PostDeliveryPeriodEnded(DeliveryPeriodId),
    GetScores {
        player_id: PlayerId,
        tx_back: oneshot::Sender<GetPreviousScoresResult>,
    },
    GetReadiness {
        tx_back: oneshot::Sender<HashMap<PlayerName, bool>>,
    },
}

#[derive(Debug, Display, thiserror::Error)]
pub enum RegisterPlayerStackError {
    PlayerDoesNotExist,
    GameConfigDoesNotAllowPerPlayerStack,
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success {
        id: PlayerId,
        stack: Option<StackContext<StackService>>,
    },
    PlayerAlreadyExist,
    GameStarted,
}

#[derive(Debug, PartialEq, Clone, Display)]
pub enum GameState {
    /// Game is open for players to join.
    Open,
    /// Game has started and stacks and market are open for [DeliveryPeriodId]
    #[display("Running")]
    Running {
        period: DeliveryPeriodId,
        end_at: Option<chrono::DateTime<Utc>>,
    },
    /// [DeliveryPeriodId] is closed.
    #[display("PostDelivery")]
    PostDelivery {
        period: DeliveryPeriodId,
        end_at: Option<chrono::DateTime<Utc>>,
    },
    /// Game has ended.
    Ended(DeliveryPeriodId),
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GameState", 4)?;
        state.serialize_field("type", "GameState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Running { .. } => "Running",
                Self::Open => "Open",
                Self::PostDelivery { .. } => "PostDelivery",
                Self::Ended(_) => "Ended",
            },
        )?;
        let period = match self {
            Self::Running { period, .. }
            | Self::PostDelivery { period, .. }
            | Self::Ended(period) => *period,
            Self::Open => DeliveryPeriodId::from(0),
        };
        state.serialize_field("delivery_period", &period)?;

        match self {
            Self::Running {
                end_at: Some(end_at),
                ..
            }
            | Self::PostDelivery {
                end_at: Some(end_at),
                ..
            } => {
                state.serialize_field("end_at", &end_at.to_rfc3339())?;
            }
            _ => {
                state.serialize_field("end_at", "None")?;
            }
        }

        state.end()
    }
}

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, AsRef, Serialize)]
#[serde(transparent)]
#[from(String, &str)]
#[as_ref(str)]
pub struct GameId(String);
impl GameId {
    pub fn into_string(self) -> String {
        self.0
    }

    pub fn parse(value: &str) -> Option<GameId> {
        if value.is_empty() {
            return None;
        }
        Some(GameId(value.to_string()))
    }
}

impl Default for GameId {
    fn default() -> Self {
        GameId(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Display, thiserror::Error)]
pub enum NewGameNameError {
    EmptyName,
}

#[derive(Debug, Clone, PartialEq, Hash, Display, From, Serialize)]
#[serde(transparent)]
pub struct GameName(String);
impl GameName {
    pub fn new(name: String) -> Result<GameName, NewGameNameError> {
        if name.is_empty() {
            return Err(NewGameNameError::EmptyName);
        }
        Ok(GameName(name))
    }
}

impl Default for GameName {
    fn default() -> Self {
        GameName(petname(3, "-").unwrap_or_else(|| Uuid::new_v4().to_string()))
    }
}

#[derive(Debug)]
pub enum GameEvent {
    PlayerJoined {
        id: PlayerId,
        name: PlayerName,
    },
    StateUpdated(GameState),
    DeliveryPeriodStarted {
        id: DeliveryPeriodId,
    },
    DeliveryPeriodEnded {
        id: DeliveryPeriodId,
    },
    PlayersReadinessChanged {
        readiness: HashMap<PlayerName, bool>,
    },
}

pub struct Game {
    state: GameState,
    players: Vec<Player>,
    last_delivery_period: DeliveryPeriodId,
    delivery_period_duration: Option<Duration>,
}

impl Game {
    pub fn init(
        last_delivery_period: DeliveryPeriodId,
        delivery_period_duration: Option<Duration>,
    ) -> Self {
        Self {
            state: GameState::Open,
            players: Vec::new(),
            last_delivery_period,
            delivery_period_duration,
        }
    }

    pub fn new(
        state: GameState,
        players: Vec<Player>,
        last_delivery_period: DeliveryPeriodId,
        delivery_period_duration: Option<Duration>,
    ) -> Self {
        Self {
            state,
            players,
            last_delivery_period,
            delivery_period_duration,
        }
    }

    pub fn try_register_player(
        &mut self,
        name: PlayerName,
    ) -> Result<Vec<GameEvent>, RegisterPlayerError> {
        if self.state != GameState::Open {
            return Err(RegisterPlayerError::GameStarted);
        }

        if self.players.iter().any(|player| player.name == name) {
            return Err(RegisterPlayerError::NameAlreadyExists);
        }

        let id = PlayerId::default();
        let player = Player {
            id: id.clone(),
            name: name.clone(),
            ready: false,
        };

        self.players.push(player);

        Ok(vec![
            GameEvent::PlayerJoined { id, name },
            GameEvent::PlayersReadinessChanged {
                readiness: self.players_readines(),
            },
        ])
    }

    pub fn register_player_ready(&mut self, id: &PlayerId) -> Vec<GameEvent> {
        let Some(player) = self.players.iter_mut().find(|p| p.id == *id) else {
            return vec![];
        };
        player.ready = true;

        if !self.all_players_ready() {
            return vec![GameEvent::PlayersReadinessChanged {
                readiness: self.players_readines(),
            }];
        }

        let mut events = vec![];
        match &self.state {
            GameState::Open | GameState::PostDelivery { .. } => {
                if self.current_delivery_period() >= self.last_delivery_period {
                    return self.end_game();
                }

                self.reset_players_readiness();

                self.state = GameState::Running {
                    period: self.next_delivery_period(),
                    end_at: self.period_end_at(),
                };
                events.push(GameEvent::DeliveryPeriodStarted {
                    id: self.current_delivery_period(),
                });
            }
            GameState::Running { .. } => {
                self.reset_players_readiness();
                self.state = GameState::PostDelivery {
                    period: self.current_delivery_period(),
                    end_at: self.period_end_at(),
                };
                events.push(GameEvent::DeliveryPeriodEnded {
                    id: self.current_delivery_period(),
                });
            }

            _ => todo!(),
        }

        events.push(GameEvent::StateUpdated(self.state.clone()));
        events.push(GameEvent::PlayersReadinessChanged {
            readiness: self.players_readines(),
        });

        events
    }

    pub fn current_delivery_period(&self) -> DeliveryPeriodId {
        match &self.state {
            GameState::Open => DeliveryPeriodId::from(0),
            GameState::Ended(period) => *period,
            GameState::Running { period, .. } | GameState::PostDelivery { period, .. } => *period,
        }
    }
    fn next_delivery_period(&self) -> DeliveryPeriodId {
        match &self.state {
            GameState::Open => DeliveryPeriodId::from(1),
            GameState::Ended(period) => *period,
            GameState::Running { period, .. } | GameState::PostDelivery { period, .. } => {
                period.next()
            }
        }
    }

    fn all_players_ready(&self) -> bool {
        self.players.iter().all(|player| player.ready)
    }

    fn reset_players_readiness(&mut self) {
        for player in self.players.iter_mut() {
            player.ready = false;
        }
    }

    fn players_readines(&self) -> HashMap<PlayerName, bool> {
        HashMap::from_iter(self.players.iter().map(|p| (p.name.clone(), p.ready)))
    }

    fn period_end_at(&self) -> Option<DateTime<Utc>> {
        self.delivery_period_duration
            .map(|period| Utc::now() + period)
    }

    fn end_game(&mut self) -> Vec<GameEvent> {
        self.state = GameState::Ended(self.current_delivery_period());
        vec![GameEvent::StateUpdated(self.state.clone())]
    }

    pub fn process_delivery_period_results(
        &mut self,
        results: &DeliveryPeriodResults,
    ) -> Vec<GameEvent> {
        match &self.state {
            GameState::Open | GameState::PostDelivery { .. } | GameState::Ended(..) => {
                vec![]
            }
            GameState::Running { period, .. } => {
                if period != &results.period_id {
                    return vec![];
                }
                self.reset_players_readiness();
                self.state = GameState::PostDelivery {
                    period: self.current_delivery_period(),
                    end_at: self.period_end_at(),
                };
                vec![
                    GameEvent::DeliveryPeriodEnded {
                        id: self.current_delivery_period(),
                    },
                    GameEvent::StateUpdated(self.state.clone()),
                    GameEvent::PlayersReadinessChanged {
                        readiness: self.players_readines(),
                    },
                ]
            }
        }
    }

    pub fn process_post_delivery_period_ends(
        &mut self,
        period: &DeliveryPeriodId,
    ) -> Vec<GameEvent> {
        let GameState::PostDelivery {
            period: current_period,
            ..
        } = &self.state
        else {
            return vec![];
        };
        if period != current_period {
            return vec![];
        }

        self.reset_players_readiness();
        self.state = GameState::Running {
            period: self.next_delivery_period(),
            end_at: self.period_end_at(),
        };
        vec![
            GameEvent::DeliveryPeriodStarted {
                id: self.current_delivery_period(),
            },
            GameEvent::StateUpdated(self.state.clone()),
            GameEvent::PlayersReadinessChanged {
                readiness: self.players_readines(),
            },
        ]
    }
}

#[derive(Debug, Display, thiserror::Error)]
pub enum RegisterPlayerError {
    NameAlreadyExists,
    GameStarted,
}

#[derive(Debug, Clone)]
pub struct GameContext {
    pub id: GameId,
    pub name: GameName,
    pub last_delivery_period: DeliveryPeriodId,
    pub tx: mpsc::Sender<GameMessage>,
    pub state_rx: watch::Receiver<GameState>,
}

#[cfg(test)]
mod test_game_state {
    use chrono::Utc;

    use crate::game::GameState;

    use super::delivery_period::DeliveryPeriodId;

    #[test]
    fn test_game_state_serialize() {
        assert_eq!(
            serde_json::to_string(&GameState::Open).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Open\",\"delivery_period\":0,\"end_at\":\"None\"}"
                .to_string()
        );
        let date = Utc::now();
        assert_eq!(
            serde_json::to_string(&GameState::Running {
                period: DeliveryPeriodId::from(1),
                end_at: Some(date)
            })
            .unwrap(),
            format!(
                "{{\"type\":\"GameState\",\"state\":\"Running\",\"delivery_period\":1,\"end_at\":\"{}\"}}",
                date.to_rfc3339()
            )
        );
        assert_eq!(
            serde_json::to_string(&GameState::PostDelivery {
                period: DeliveryPeriodId::from(2),
                end_at: Some(date)
            })
            .unwrap(),
            format!(
                "{{\"type\":\"GameState\",\"state\":\"PostDelivery\",\"delivery_period\":2,\"end_at\":\"{}\"}}",
                date.to_rfc3339()
            )
        );
        assert_eq!(
            serde_json::to_string(&GameState::Ended(DeliveryPeriodId::from(3))).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Ended\",\"delivery_period\":3,\"end_at\":\"None\"}".to_string()
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

#[cfg(test)]
mod test_game_name {
    use crate::game::{GameName, NewGameNameError};

    #[test]
    fn test_game_name_new_valid() {
        let name = GameName::new(String::from("test-game")).expect("Should return a game name");
        assert_eq!(name.to_string(), "test-game");
    }

    #[test]
    fn test_game_name_new_empty() {
        let result = GameName::new(String::from(""));
        assert!(matches!(result, Err(NewGameNameError::EmptyName)));
    }

    #[test]
    fn test_game_name_default() {
        let name = GameName::default();
        // Default is random, best we can do is test for the invariant, i.e. being non empty
        assert!(!name.to_string().is_empty());
    }
}

#[cfg(test)]
mod test_game {
    use crate::utils::units::{Money, Power};

    use super::*;

    fn build_empty_game() -> Game {
        Game {
            state: GameState::Open,
            players: Vec::new(),
            last_delivery_period: DeliveryPeriodId::from(2),
            delivery_period_duration: None,
        }
    }

    #[test]
    fn test_register_player_open_game() {
        let mut game = build_empty_game();
        assert_eq!(game.state, GameState::Open);

        let player = PlayerName::from("test-player");

        let Ok(events) = game.try_register_player(player.clone()) else {
            unreachable!("Should have register the player")
        };

        let Some(GameEvent::PlayerJoined { name, .. }) = events.first() else {
            unreachable!("Should be GameEvent::PlayerJoined")
        };
        assert_eq!(*name, player);
    }

    #[test]
    fn test_register_player_send_readiness() {
        let mut game = build_empty_game();
        assert_eq!(game.state, GameState::Open);

        let player = PlayerName::from("test-player");

        let Ok(events) = game.try_register_player(player.clone()) else {
            unreachable!("Should have register the player")
        };

        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should be GameEvent::PlayerReadinessChanged")
        };
        assert!(readiness.contains_key(&player));
    }

    #[test]
    fn test_register_player_name_already_exists() {
        let mut game = build_empty_game();
        assert_eq!(game.state, GameState::Open);

        let player = PlayerName::from("test-player");
        let _ = game.try_register_player(player.clone());

        let Err(RegisterPlayerError::NameAlreadyExists) = game.try_register_player(player.clone())
        else {
            unreachable!("Should have failed to register the player")
        };
    }

    #[test]
    fn test_register_player_name_game_started() {
        let mut game = build_empty_game();

        for state in [
            GameState::Running {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            GameState::PostDelivery {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            GameState::Ended(DeliveryPeriodId::from(1)),
        ] {
            game.state = state;
            let player = PlayerName::from("test-player");

            let Err(RegisterPlayerError::GameStarted) = game.try_register_player(player.clone())
            else {
                unreachable!("Should have failed to register the player")
            };
        }
    }

    fn build_game_with_players() -> Game {
        Game {
            state: GameState::Open,
            players: vec![
                Player {
                    id: PlayerId::from("p1"),
                    name: PlayerName::from("p1"),
                    ready: false,
                },
                Player {
                    id: PlayerId::from("p2"),
                    name: PlayerName::from("p2"),
                    ready: false,
                },
            ],
            last_delivery_period: DeliveryPeriodId::from(2),
            delivery_period_duration: None,
        }
    }

    fn game_states_to_test() -> Vec<GameState> {
        vec![
            GameState::Open,
            GameState::Running {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            GameState::PostDelivery {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            GameState::Ended(DeliveryPeriodId::from(1)),
        ]
    }

    fn get_readiness(events: &[GameEvent]) -> Option<HashMap<PlayerName, bool>> {
        events
            .iter()
            .flat_map(|event| match event {
                GameEvent::PlayersReadinessChanged { readiness } => Some(readiness),
                _ => None,
            })
            .next()
            .cloned()
    }

    fn get_game_state(events: &[GameEvent]) -> Option<GameState> {
        events
            .iter()
            .flat_map(|event| match event {
                GameEvent::StateUpdated(state) => Some(state),
                _ => None,
            })
            .next()
            .cloned()
    }

    fn get_period_started_id(events: &[GameEvent]) -> Option<DeliveryPeriodId> {
        events
            .iter()
            .flat_map(|event| match event {
                GameEvent::DeliveryPeriodStarted { id } => Some(id),
                _ => None,
            })
            .next()
            .cloned()
    }

    fn get_period_ended_id(events: &[GameEvent]) -> Option<DeliveryPeriodId> {
        events
            .iter()
            .flat_map(|event| match event {
                GameEvent::DeliveryPeriodEnded { id } => Some(id),
                _ => None,
            })
            .next()
            .cloned()
    }

    #[test]
    fn test_register_player_ready() {
        let mut game = build_game_with_players();

        for state in game_states_to_test() {
            game.state = state;

            let events = game.register_player_ready(&PlayerId::from("p1"));

            let Some(readiness) = get_readiness(&events) else {
                unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
            };

            assert_eq!(
                readiness,
                HashMap::from_iter([
                    (PlayerName::from("p1"), true),
                    (PlayerName::from("p2"), false)
                ])
            )
        }
    }

    #[test]
    fn test_register_player_ready_idempotent() {
        let mut game = build_game_with_players();

        for state in game_states_to_test() {
            game.state = state;

            let _ = game.register_player_ready(&PlayerId::from("p1"));
            let events = game.register_player_ready(&PlayerId::from("p1"));

            let Some(readiness) = get_readiness(&events) else {
                unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
            };

            assert_eq!(
                readiness,
                HashMap::from_iter([
                    (PlayerName::from("p1"), true),
                    (PlayerName::from("p2"), false)
                ])
            )
        }
    }

    #[test]
    fn test_register_player_ready_all_players_ready_game_open() {
        let mut game = build_game_with_players();
        let state = GameState::Open;
        game.state = state;

        let _ = game.register_player_ready(&PlayerId::from("p1"));
        let events = game.register_player_ready(&PlayerId::from("p2"));

        // State updated
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(
            state,
            GameState::Running {
                period: DeliveryPeriodId::from(1),
                end_at: None
            }
        );

        // New delivery period started
        let Some(period) = get_period_started_id(&events) else {
            unreachable!("Should have return a GameEvent::DeliveryPeriodStarted");
        };
        assert_eq!(period, DeliveryPeriodId::from(1));

        // Readiness has been reset to false for all players
        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
        };
        assert_eq!(
            readiness,
            HashMap::from_iter([
                (PlayerName::from("p1"), false),
                (PlayerName::from("p2"), false)
            ])
        )
    }

    #[test]
    fn test_register_player_ready_all_players_ready_game_running() {
        let mut game = build_game_with_players();
        let state = GameState::Running {
            period: DeliveryPeriodId::from(1),
            end_at: None,
        };
        game.state = state;

        let _ = game.register_player_ready(&PlayerId::from("p1"));
        let events = game.register_player_ready(&PlayerId::from("p2"));

        // Game state changed
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(
            state,
            GameState::PostDelivery {
                period: DeliveryPeriodId::from(1),
                end_at: None
            }
        );

        // Delivery period ended
        let Some(period) = get_period_ended_id(&events) else {
            unreachable!("Should have return a GameEvent::DeliveryPeriodEnded");
        };
        assert_eq!(period, DeliveryPeriodId::from(1));

        // Readiness has been reset to false for all players
        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
        };
        assert_eq!(
            readiness,
            HashMap::from_iter([
                (PlayerName::from("p1"), false),
                (PlayerName::from("p2"), false)
            ])
        )
    }

    #[test]
    fn test_register_player_ready_all_players_ready_game_post_delivery() {
        let mut game = build_game_with_players();
        game.state = GameState::PostDelivery {
            period: DeliveryPeriodId::from(1),
            end_at: None,
        };

        let _ = game.register_player_ready(&PlayerId::from("p1"));
        let events = game.register_player_ready(&PlayerId::from("p2"));

        // Game state updated to running, period = 2
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(
            state,
            GameState::Running {
                period: DeliveryPeriodId::from(2),
                end_at: None
            }
        );

        // New delivery period started
        let Some(period) = get_period_started_id(&events) else {
            unreachable!("Should have return a GameEvent::DeliveryPeriodStarted");
        };
        assert_eq!(period, DeliveryPeriodId::from(2));

        // Readiness has been reset to false for all players
        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
        };
        assert_eq!(
            readiness,
            HashMap::from_iter([
                (PlayerName::from("p1"), false),
                (PlayerName::from("p2"), false)
            ])
        )
    }

    #[test]
    fn test_register_player_ready_all_players_ready_game_end() {
        let mut game = build_game_with_players();
        game.state = GameState::PostDelivery {
            period: game.last_delivery_period,
            end_at: None,
        };

        let _ = game.register_player_ready(&PlayerId::from("p1"));
        let events = game.register_player_ready(&PlayerId::from("p2"));

        // Game ended, period = 2
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(state, GameState::Ended(game.last_delivery_period));
    }

    fn build_results() -> DeliveryPeriodResults {
        DeliveryPeriodResults {
            period_id: DeliveryPeriodId::from(1),
            players_scores: HashMap::from_iter([(
                PlayerId::from("p1"),
                PlayerScore {
                    balance: Power::from(0),
                    imbalance_cost: Money::from(0),
                    pnl: Money::from(0),
                },
            )]),
            players_detailed_scores: HashMap::from_iter([(
                PlayerId::from("p1"),
                PlayerDetailedScore::default(),
            )]),
        }
    }

    #[test]
    fn test_process_delivery_period_results_open_game() {
        let mut game = build_game_with_players();
        game.state = GameState::Open;
        let results = build_results();

        let events = game.process_delivery_period_results(&results);
        assert!(events.is_empty());
    }

    #[test]
    fn test_process_delivery_period_results_running_game() {
        let mut game = build_game_with_players();
        game.state = GameState::Running {
            period: DeliveryPeriodId::from(1),
            end_at: None,
        };
        if let Some(p) = game.players.get_mut(0) {
            p.ready = true;
        }

        let results = build_results();

        let events = game.process_delivery_period_results(&results);

        // Game state changed
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(
            state,
            GameState::PostDelivery {
                period: DeliveryPeriodId::from(1),
                end_at: None
            }
        );

        // Delivery period ended
        let Some(period) = get_period_ended_id(&events) else {
            unreachable!("Should have return a GameEvent::DeliveryPeriodEnded");
        };
        assert_eq!(period, DeliveryPeriodId::from(1));

        // Readiness has been reset to false for all players
        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
        };
        assert_eq!(
            readiness,
            HashMap::from_iter([
                (PlayerName::from("p1"), false),
                (PlayerName::from("p2"), false)
            ])
        )
    }

    #[test]
    fn test_process_delivery_period_results_running_game_wrong_period() {
        let mut game = build_game_with_players();
        game.state = GameState::Running {
            period: DeliveryPeriodId::from(2),
            end_at: None,
        };
        let results = build_results();

        assert_ne!(results.period_id, game.current_delivery_period());

        let events = game.process_delivery_period_results(&results);
        assert!(events.is_empty());
    }

    #[test]
    fn test_process_delivery_period_results_post_delivery() {
        let mut game = build_game_with_players();
        game.state = GameState::PostDelivery {
            period: DeliveryPeriodId::from(1),
            end_at: None,
        };
        if let Some(p) = game.players.get_mut(0) {
            p.ready = true;
        }

        let results = build_results();

        let events = game.process_delivery_period_results(&results);

        assert!(events.is_empty());
    }

    #[test]
    fn test_process_delivery_period_results_post_delivery_wrong_period() {
        let mut game = build_game_with_players();
        game.state = GameState::PostDelivery {
            period: DeliveryPeriodId::from(2),
            end_at: None,
        };
        if let Some(p) = game.players.get_mut(0) {
            p.ready = true;
        }

        let results = build_results();
        assert_ne!(results.period_id, game.current_delivery_period());

        let events = game.process_delivery_period_results(&results);

        assert!(events.is_empty());
    }

    #[test]
    fn test_process_delivery_period_results_game_ended() {
        let mut game = build_game_with_players();
        game.state = GameState::Ended(DeliveryPeriodId::from(2));
        if let Some(p) = game.players.get_mut(0) {
            p.ready = true;
        }

        let results = build_results();

        let events = game.process_delivery_period_results(&results);

        assert!(events.is_empty());
    }

    fn build_game_post_delivery() -> Game {
        Game {
            state: GameState::PostDelivery {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            players: vec![
                Player {
                    id: PlayerId::from("p1"),
                    name: PlayerName::from("p1"),
                    ready: false,
                },
                Player {
                    id: PlayerId::from("p2"),
                    name: PlayerName::from("p2"),
                    ready: true,
                },
            ],
            last_delivery_period: DeliveryPeriodId::from(2),
            delivery_period_duration: None,
        }
    }

    #[test]
    fn test_process_post_delivery_periods_ends_ok() {
        let mut game = build_game_post_delivery();

        let events = game.process_post_delivery_period_ends(&DeliveryPeriodId::from(1));

        // Game state changed
        let Some(state) = get_game_state(&events) else {
            unreachable!("Should have updated game state")
        };
        assert_eq!(
            state,
            GameState::Running {
                period: DeliveryPeriodId::from(2),
                end_at: None
            }
        );

        // New delivery period started
        let Some(period) = get_period_started_id(&events) else {
            unreachable!("Should have return a GameEvent::DeliveryPeriodEnded");
        };
        assert_eq!(period, DeliveryPeriodId::from(2));

        // Readiness has been reset to false for all players
        let Some(readiness) = get_readiness(&events) else {
            unreachable!("Should contain a GameEvent::PlayerReadinessChanged")
        };
        assert_eq!(
            readiness,
            HashMap::from_iter([
                (PlayerName::from("p1"), false),
                (PlayerName::from("p2"), false)
            ])
        )
    }

    #[test]
    fn test_process_post_delivery_periods_ends_wrong_period() {
        let mut game = build_game_post_delivery();

        let events = game.process_post_delivery_period_ends(&DeliveryPeriodId::from(0));

        assert!(events.is_empty());
    }

    #[test]
    fn test_process_post_delivery_periods_ends_wrong_state() {
        for state in [
            GameState::Open,
            GameState::Running {
                period: DeliveryPeriodId::from(1),
                end_at: None,
            },
            GameState::Ended(DeliveryPeriodId::from(2)),
        ] {
            let mut game = build_game_post_delivery();
            game.state = state;
            let events = game.process_post_delivery_period_ends(&DeliveryPeriodId::from(1));

            assert!(events.is_empty());
        }
    }
}
