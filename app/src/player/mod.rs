use std::collections::HashMap;

use derive_more::{AsRef, Display, From, Into};
use petname::petname;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    forecast::Forecast,
    game::{
        GameId,
        delivery_period::DeliveryPeriodId,
        scores::{PlayerScore, RankTier},
    },
    market::{OrderRepr, order_book::TradeLeg},
    plants::{PlantId, PowerPlantPublicRepr},
    utils::units::Money,
};

pub mod infra;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Display, AsRef, Into)]
#[into(String)]
#[as_ref(str)]
#[from(&str, String)]
pub struct PlayerId(String);

impl Default for PlayerId {
    fn default() -> Self {
        PlayerId(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Serialize, From, Display, Into, PartialEq, Eq, Clone, Hash)]
#[from(String, &str)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn random() -> PlayerName {
        PlayerName::from(petname(3, "-").unwrap_or("default".to_string()))
    }

    pub fn parse(value: &str) -> Option<PlayerName> {
        if value.is_empty() {
            return None;
        }
        Some(PlayerName(value.to_string()))
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PlayerResultView {
    pub player: PlayerName,
    pub rank: usize,
    pub score: Money,
    pub tier: Option<RankTier>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
pub enum PlayerMessage {
    NewTrade(TradeLeg),
    OrderBookSnapshot {
        bids: Vec<OrderRepr>,
        offers: Vec<OrderRepr>,
    },
    TradeList {
        trades: Vec<TradeLeg>,
    },
    StackSnapshot {
        plants: HashMap<PlantId, PowerPlantPublicRepr>,
    },
    StackForecasts {
        forecasts: HashMap<PlantId, Option<Vec<Forecast>>>,
    },
    DeliveryPeriodResults {
        delivery_period: DeliveryPeriodId,
        score: PlayerScore,
    },
    GameResults {
        rankings: Vec<PlayerResultView>,
    },
    ReadinessStatus {
        readiness: HashMap<PlayerName, bool>,
    },
    YourName {
        name: PlayerName,
    },
    GameDuration {
        last_period: DeliveryPeriodId,
    },
}

pub trait PlayerConnections: Clone + Send + Sync + 'static {
    /// Send a message to all connections beloging to a given player.
    fn send_to_player(
        &self,
        game: &GameId,
        player: &PlayerId,
        message: PlayerMessage,
    ) -> impl Future<Output = ()> + Send;

    /// Send the same message to all players' connections from a given game.
    fn send_to_all_players(
        &self,
        game: &GameId,
        message: PlayerMessage,
    ) -> impl Future<Output = ()> + Send;
}

#[cfg(test)]
mod test {
    use crate::player::PlayerId;

    #[test]
    fn test_player_id_from_into_string() {
        assert_eq!(
            PlayerId::from(String::from("toto")).to_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_player_id_as_ref() {
        assert_eq!(PlayerId::from("toto").as_ref(), "toto");
    }
}
