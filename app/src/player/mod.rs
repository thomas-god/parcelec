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
        infra::stack_config::{
            GameStackConfig, GameStackFixedConfig, GameStackPerPlayerBaseConfig,
        },
        scores::{PlayerDetailedScore, PlayerScore},
    },
    market::{OrderRepr, order_book::TradeLeg},
    plants::{PlantId, PlantOutput, PowerPlantPublicRepr},
    utils::units::{Energy, EnergyCost, Money, Power},
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
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum GameStackConfigView {
    Fixed(GameStackFixedConfigView),
    PerPlayer(GameStackPerPlayerBaseConfigView),
}

impl From<&GameStackConfig> for GameStackConfigView {
    fn from(value: &GameStackConfig) -> Self {
        match value {
            GameStackConfig::Fixed(config) => Self::Fixed(config.into()),
            GameStackConfig::PerPlayer(config) => Self::PerPlayer(config.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct GameStackFixedConfigView {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub consumers_revenues: EnergyCost,
    pub gas_capacity: Power,
    pub nuclear_capacity: Power,
    pub battery_capacity: Energy,
    pub consumers_forecasts_range: usize,
    pub renewable_forecasts_range: usize,
}

impl From<&GameStackFixedConfig> for GameStackFixedConfigView {
    fn from(value: &GameStackFixedConfig) -> Self {
        Self {
            gas_cost: value.gas_cost,
            gas_capacity: value.gas_capacity,
            nuclear_cost: value.nuclear_cost,
            nuclear_capacity: value.nuclear_capacity,
            battery_capacity: value.battery_capacity,
            consumers_forecasts_range: value.consumers_forecasts_range,
            consumers_revenues: value.consumers_revenues,
            renewable_forecasts_range: value.renewable_forecasts_range,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct GameStackPerPlayerBaseConfigView {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub gas_max_capacity: Power,
    pub nuclear_max_capacity: Power,
    pub battery_max_capacity: Energy,
    pub consumers_max_abs_capacity: Power,
    pub consumers_revenues: EnergyCost,
    pub consumers_forecasts_range: usize,
    pub renewable_max_capacity: Power,
    pub renewable_forecasts_range: usize,
}

impl From<&GameStackPerPlayerBaseConfig> for GameStackPerPlayerBaseConfigView {
    fn from(value: &GameStackPerPlayerBaseConfig) -> Self {
        Self {
            gas_cost: value.gas_cost,
            gas_max_capacity: value.gas_max_capacity,
            nuclear_cost: value.nuclear_cost,
            nuclear_max_capacity: value.nuclear_max_capacity,
            battery_max_capacity: value.battery_max_capacity,
            consumers_forecasts_range: value.consumers_forecasts_range,
            consumers_revenues: value.consumers_revenues,
            consumers_max_abs_capacity: value.consumers_max_abs_capacity,
            renewable_forecasts_range: value.renewable_forecasts_range,
            renewable_max_capacity: value.renewable_max_capacity,
        }
    }
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
    StackConfig {
        config: GameStackConfigView,
    },
    StackSnapshot {
        plants: Option<HashMap<PlantId, PowerPlantPublicRepr>>,
    },
    StackForecasts {
        forecasts: HashMap<PlantId, Option<Vec<Forecast>>>,
    },
    StackHistory {
        history: HashMap<PlantId, Vec<PlantOutput>>,
    },
    DeliveryPeriodResults {
        delivery_period: DeliveryPeriodId,
        score: PlayerScore,
        detailed_score: Option<PlayerDetailedScore>,
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
