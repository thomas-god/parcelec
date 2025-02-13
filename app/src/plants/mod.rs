use battery::BatteryPublicRepr;
use consumers::ConsumersPublicRepr;
use gas_plant::GasPlantPublicRepr;
use renewable::RenewablePlantPublicRepr;
use serde::Serialize;

pub mod battery;
pub mod consumers;
pub mod gas_plant;
pub mod renewable;
pub mod stack;

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
    setpoint: isize,
    cost: isize,
}

pub trait PowerPlant {
    /// Program the setpoint for the next delivery period.
    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput;

    /// Apply the programmed setpoint, and update the state of the plant.
    fn dispatch(&mut self) -> PlantOutput;

    /// Retrieve a string representation of the plant's state
    fn current_state(&self) -> PowerPlantPublicRepr;
}
