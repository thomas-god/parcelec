use std::fmt;

use battery::BatteryPublicRepr;
use consumers::ConsumersPublicRepr;
use gas_plant::GasPlantPublicRepr;
use renewable::RenewablePlantPublicRepr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod battery;
pub mod consumers;
pub mod gas_plant;
pub mod models;
pub mod renewable;
pub mod stack;

pub use models::{Stack, StackService};

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum PowerPlantPublicRepr {
    Battery(BatteryPublicRepr),
    GasPlant(GasPlantPublicRepr),
    RenewablePlant(RenewablePlantPublicRepr),
    Consumers(ConsumersPublicRepr),
}

#[derive(Debug, PartialEq, Serialize, Clone, Copy)]
pub struct PlantOutput {
    pub setpoint: isize,
    pub cost: isize,
}

pub trait PowerPlant {
    /// Program the setpoint for the next delivery period.
    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput;

    /// Apply the programmed setpoint, and update the state of the plant.
    fn dispatch(&mut self) -> PlantOutput;

    /// Retrieve a string representation of the plant's state
    fn current_state(&self) -> PowerPlantPublicRepr;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlantId(String);

impl From<&str> for PlantId {
    fn from(value: &str) -> Self {
        PlantId(value.to_string())
    }
}

impl From<String> for PlantId {
    fn from(value: String) -> Self {
        PlantId(value)
    }
}

impl Default for PlantId {
    fn default() -> Self {
        PlantId(Uuid::new_v4().to_string())
    }
}
impl fmt::Display for PlantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl PlantId {
    pub fn into_string(self) -> String {
        self.0
    }
}
impl AsRef<str> for PlantId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::plants::PlantId;

    #[test]
    fn test_plant_id_from_into_string() {
        assert_eq!(
            PlantId::from(String::from("toto")).into_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_plant_id_as_ref() {
        assert_eq!(PlantId::from("toto").as_ref(), "toto");
    }
}
