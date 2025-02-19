use std::collections::HashMap;
use std::fmt;
use std::future::Future;

use serde::{Deserialize, Serialize};
use technologies::battery::BatteryPublicRepr;
use technologies::consumers::ConsumersPublicRepr;
use technologies::gas_plant::GasPlantPublicRepr;
use technologies::renewable::RenewablePlantPublicRepr;
use uuid::Uuid;

pub mod actor;
pub mod service;
pub mod technologies;

pub use service::StackService;

use crate::game::delivery_period::DeliveryPeriodId;

#[derive(Debug)]
#[allow(dead_code)]
pub struct CloseStackError(DeliveryPeriodId);
#[derive(Debug)]
pub struct GetSnapshotError;

/// [Stack] is the public API of Parcelec power plants/consumption domain. A stack refers to the
/// set of power plants and consumers belonging to a player. A player can program power setpoints
/// on its plants to try to match energy consumption and production.
pub trait Stack: Clone + Send + Sync + 'static {
    /// Open the stack so that its plants can be programmed.
    fn open_stack(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

    /// Close the stack and disptach its plants based on their last setpoints. Return a map of each
    /// stack's plant output (power and cost) for the delivery period. When trying to close an already
    /// closed stack, there will be no side effects and the maps of plants outptus for that delivery
    /// period will be returned.
    fn close_stack(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> impl Future<Output = Result<HashMap<PlantId, PlantOutput>, CloseStackError>> + Send;

    /// Program a setpoint on a power plant of the stack. Each plant can be programmed any number of
    /// times a player wants. The last setpoint will be used when closing the stack for the delivery
    /// period.
    fn program_setpoint(&self, plant: PlantId, setpoint: isize) -> impl Future<Output = ()> + Send;

    /// Get a snapshot of the stack's power plants current setpoint and cost.
    fn get_snapshot(
        &self,
    ) -> impl Future<Output = Result<HashMap<PlantId, PowerPlantPublicRepr>, GetSnapshotError>> + Send;
}

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
