use std::collections::HashMap;

use delivery_period::{DeliveryPeriodId, DeliveryPeriodResults};
use derive_more::{AsRef, Display, From};
use petname::petname;
use scores::PlayerScore;
use serde::{ser::SerializeStruct, Serialize};
use tokio::sync::{
    mpsc::{self},
    oneshot, watch,
};

use crate::{
    plants::{actor::StackContext, service::StackService},
    player::{connection::PlayerResultView, PlayerId, PlayerName},
};

pub mod actor;
pub mod delivery_period;
pub mod scores;

pub use actor::Game;

#[derive(Debug)]
struct Player {
    id: PlayerId,
    name: PlayerName,
    ready: bool,
}

#[derive(Debug)]
pub enum GetPreviousScoresResult {
    PlayerScores {
        scores: HashMap<DeliveryPeriodId, PlayerScore>,
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
    PlayerIsReady(PlayerId),
    DeliveryPeriodResults(DeliveryPeriodResults),
    GetScores {
        player_id: PlayerId,
        tx_back: oneshot::Sender<GetPreviousScoresResult>,
    },
}

#[derive(Debug)]
pub enum RegisterPlayerResponse {
    Success {
        id: PlayerId,
        stack: StackContext<StackService>,
    },
    PlayerAlreadyExist,
    GameStarted,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GameState {
    Open,
    Running(DeliveryPeriodId),
    PostDelivery(DeliveryPeriodId),
    Ended(DeliveryPeriodId),
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
                Self::Ended(_) => "Ended",
            },
        )?;
        let period = match self {
            Self::Running(period) | Self::PostDelivery(period) | Self::Ended(period) => *period,
            Self::Open => DeliveryPeriodId::from(0),
        };
        state.serialize_field("delivery_period", &period)?;
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

#[derive(Debug, Clone)]
pub struct GameContext {
    pub id: GameId,
    pub name: GameName,
    pub tx: mpsc::Sender<GameMessage>,
    pub state_rx: watch::Receiver<GameState>,
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
        assert_eq!(
            serde_json::to_string(&GameState::Ended(DeliveryPeriodId::from(3))).unwrap(),
            "{\"type\":\"GameState\",\"state\":\"Ended\",\"delivery_period\":3}".to_string()
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
