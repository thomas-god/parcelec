use std::collections::HashMap;
use std::fmt::{self, Display};
use std::future::Future;
use std::ops::Add;

use derive_more::{AsRef, Display, From};
use serde::{Deserialize, Serialize};
use technologies::battery::BatteryPublicRepr;
use technologies::consumers::ConsumersPublicRepr;
use technologies::gas_plant::GasPlantPublicRepr;
use technologies::nuclear::NuclearPublicRepr;
use technologies::renewable::RenewablePlantPublicRepr;
use uuid::Uuid;

pub mod infra;
pub mod technologies;

pub use infra::StackService;

use crate::forecast::Forecast;
use crate::game::delivery_period::DeliveryPeriodId;
use crate::utils::units::{Energy, Money, Power, TIMESTEP};

#[derive(Debug)]
#[allow(dead_code)]
pub struct CloseStackError(DeliveryPeriodId);
#[derive(thiserror::Error, Debug)]
pub struct GetSnapshotError;

impl Display for GetSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GetSnapshotError")
    }
}

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
    ) -> impl Future<Output = Result<StackDispatchResults, CloseStackError>> + Send;

    /// Program a setpoint on a power plant of the stack. Each plant can be programmed any number of
    /// times a player wants. The last setpoint will be used when closing the stack for the delivery
    /// period.
    fn program_setpoint(&self, plant: PlantId, setpoint: Power) -> impl Future<Output = ()> + Send;

    /// Get a snapshot of the stack's power plants current setpoint and cost.
    fn get_snapshot(
        &self,
    ) -> impl Future<Output = Result<HashMap<PlantId, PowerPlantPublicRepr>, GetSnapshotError>> + Send;

    /// Get a forecast, if available, for each plant of the stack, for the next delivery period.
    fn get_forecasts(&self)
    -> impl Future<Output = HashMap<PlantId, Option<Vec<Forecast>>>> + Send;

    /// Get an output history for each plant of the stack.
    fn get_history(&self) -> impl Future<Output = HashMap<PlantId, Vec<PlantOutput>>> + Send;
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum PowerPlantPublicRepr {
    Battery(BatteryPublicRepr),
    GasPlant(GasPlantPublicRepr),
    RenewablePlant(RenewablePlantPublicRepr),
    Consumers(ConsumersPublicRepr),
    Nuclear(NuclearPublicRepr),
}

#[derive(Debug, PartialEq, Serialize, Clone, Copy)]
pub struct PlantOutput {
    pub setpoint: Power,
    pub cost: Money,
}

pub trait PowerPlant {
    /// Program the setpoint for the next delivery period.
    fn program_setpoint(&mut self, setpoint: Power) -> PlantOutput;

    /// Apply the programmed setpoint, and update the state of the plant.
    fn dispatch(&mut self) -> PlantOutput;

    /// Retrieve a string representation of the plant's state
    fn current_state(&self) -> PowerPlantPublicRepr;

    /// Get a forecast of the plant's output for the next delivery period
    fn get_forecast(&self) -> Option<Vec<Forecast>>;

    /// Get history of plant's setpoint
    fn get_history(&self) -> Vec<PlantOutput>;

    fn category(&self) -> PlantCategory;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Display, AsRef)]
#[as_ref(str)]
#[from(&str, String)]
pub struct PlantId(String);

impl Default for PlantId {
    fn default() -> Self {
        PlantId(Uuid::new_v4().to_string())
    }
}

pub enum PlantCategory {
    Battery,
    GasPlant,
    RenewablePlant,
    Consumers,
    Nuclear,
}

pub struct StackPlants(HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>>);

impl StackPlants {
    pub fn new(plants: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>>) -> StackPlants {
        Self(plants)
    }

    pub fn snapshot(&self) -> HashMap<PlantId, PowerPlantPublicRepr> {
        self.0
            .iter()
            .map(|(plant_id, plant)| (plant_id.to_owned(), plant.current_state()))
            .collect()
    }

    pub fn forecasts(&self) -> HashMap<PlantId, Option<Vec<Forecast>>> {
        self.0
            .iter()
            .map(|(plant_id, plant)| (plant_id.to_owned(), plant.get_forecast()))
            .collect()
    }

    pub fn history(&self) -> HashMap<PlantId, Vec<PlantOutput>> {
        self.0
            .iter()
            .map(|(plant_id, plant)| (plant_id.to_owned(), plant.get_history()))
            .collect()
    }

    pub fn program_setpoint(&mut self, plant_id: &PlantId, setpoint: Power) -> Option<PlantOutput> {
        if let Some(plant) = self.0.get_mut(plant_id) {
            return Some(plant.program_setpoint(setpoint));
        };
        None
    }

    pub fn dispatch_plants(&mut self) -> StackDispatchResults {
        let mut outputs = HashMap::new();
        let mut state = StackAggregatedState::empty();

        for (id, plant) in self.0.iter_mut() {
            let output = plant.dispatch();
            outputs.insert(id.clone(), output);

            match plant.category() {
                PlantCategory::Consumers => {
                    state.consumers = state.consumers + output;
                }
                PlantCategory::GasPlant => {
                    state.gas = state.gas + output;
                }
                PlantCategory::Nuclear => {
                    state.nuclear = state.nuclear + output;
                }
                PlantCategory::RenewablePlant => {
                    state.renewables = state.renewables + output;
                }
                PlantCategory::Battery if output.setpoint > Power::from(0) => {
                    state.battery_discharge = state.battery_discharge + output;
                }
                PlantCategory::Battery if output.setpoint < Power::from(0) => {
                    state.battery_charge = state.battery_charge + output;
                }
                _ => {}
            }
        }

        StackDispatchResults::new(outputs, state)
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    volume: Energy,
    money: Money,
}

impl Output {
    pub fn new(volume: Energy, money: Money) -> Self {
        Self { volume, money }
    }
    pub fn empty() -> Self {
        Self {
            volume: Energy::from(0),
            money: Money::from(0),
        }
    }
    pub fn volume(&self) -> &Energy {
        &self.volume
    }
    pub fn money(&self) -> &Money {
        &self.money
    }
}

impl Add for Output {
    type Output = Output;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            volume: self.volume + rhs.volume,
            money: self.money + rhs.money,
        }
    }
}

impl Add<PlantOutput> for Output {
    type Output = Output;

    fn add(self, rhs: PlantOutput) -> Self::Output {
        Self::Output {
            volume: self.volume + rhs.setpoint * TIMESTEP,
            money: self.money + rhs.cost,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackAggregatedState {
    consumers: Output,
    renewables: Output,
    gas: Output,
    nuclear: Output,
    battery_discharge: Output,
    battery_charge: Output,
}

impl StackAggregatedState {
    pub fn new(
        consumers: Output,
        renewables: Output,
        gas: Output,
        nuclear: Output,
        battery_discharge: Output,
        battery_charge: Output,
    ) -> Self {
        Self {
            consumers,
            renewables,
            gas,
            nuclear,
            battery_discharge,
            battery_charge,
        }
    }

    pub fn empty() -> Self {
        Self {
            consumers: Output::empty(),
            renewables: Output::empty(),
            gas: Output::empty(),
            nuclear: Output::empty(),
            battery_discharge: Output::empty(),
            battery_charge: Output::empty(),
        }
    }

    pub fn consumers(&self) -> &Output {
        &self.consumers
    }
    pub fn renewables(&self) -> &Output {
        &self.renewables
    }
    pub fn gas(&self) -> &Output {
        &self.gas
    }
    pub fn nuclear(&self) -> &Output {
        &self.nuclear
    }
    pub fn battery_discharge(&self) -> &Output {
        &self.battery_discharge
    }
    pub fn battery_charge(&self) -> &Output {
        &self.battery_charge
    }
    pub fn position(&self) -> Energy {
        self.consumers.volume
            + self.renewables.volume
            + self.gas.volume
            + self.nuclear.volume
            + self.battery_discharge.volume
            + self.battery_charge.volume
    }
    pub fn pnl(&self) -> Money {
        self.consumers.money
            + self.renewables.money
            + self.gas.money
            + self.nuclear.money
            + self.battery_discharge.money
            + self.battery_charge.money
    }
}

#[derive(Debug, Clone)]
pub struct StackDispatchResults {
    plants_outputs: HashMap<PlantId, PlantOutput>,
    aggregated_state: StackAggregatedState,
}

impl StackDispatchResults {
    pub fn new(plants_outputs: HashMap<PlantId, PlantOutput>, state: StackAggregatedState) -> Self {
        Self {
            plants_outputs,
            aggregated_state: state,
        }
    }
    pub fn plants_outputs(&self) -> &HashMap<PlantId, PlantOutput> {
        &self.plants_outputs
    }
    pub fn aggregated_state(&self) -> &StackAggregatedState {
        &self.aggregated_state
    }
    pub fn position(&self) -> Energy {
        self.aggregated_state.position()
    }
    pub fn pnl(&self) -> Money {
        self.aggregated_state.pnl()
    }
}

impl Default for StackDispatchResults {
    fn default() -> Self {
        Self {
            plants_outputs: HashMap::new(),
            aggregated_state: StackAggregatedState::empty(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::forecast::ForecastValue;
    use crate::plants::technologies::battery::Battery;
    use crate::plants::technologies::consumers::Consumers;
    use crate::plants::technologies::gas_plant::GasPlant;
    use crate::plants::technologies::nuclear::NuclearPlant;
    use crate::plants::technologies::renewable::RenewablePlant;
    use crate::plants::{PlantId, StackDispatchResults, StackPlants};
    use crate::utils::units::{Energy, EnergyCost, Money, Power};

    fn make_stack() -> (StackPlants, PlantId) {
        let id = PlantId::from("plant-a");
        let plant = GasPlant::new(EnergyCost::from(10), Power::from(100));
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(id.clone(), Box::new(plant));
        (StackPlants::new(plants), id)
    }

    #[test]
    fn test_plant_id_from_into_string() {
        assert_eq!(
            PlantId::from(String::from("toto")).to_string(),
            String::from("toto")
        );
    }

    #[test]
    fn test_plant_id_as_ref() {
        assert_eq!(PlantId::from("toto").as_ref(), "toto");
    }

    #[test]
    fn test_snapshot_contains_all_plants() {
        let (stack, id) = make_stack();
        let snapshot = stack.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(snapshot.contains_key(&id));
    }

    #[test]
    fn test_forecasts_returns_none_for_gas_plant() {
        let (stack, id) = make_stack();
        let forecasts = stack.forecasts();
        assert_eq!(forecasts.len(), 1);
        assert_eq!(forecasts[&id], None);
    }

    #[test]
    fn test_history_is_empty_before_dispatch() {
        let (stack, id) = make_stack();
        let history = stack.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[&id], vec![]);
    }

    #[test]
    fn test_program_setpoint_returns_output_for_known_plant() {
        let (mut stack, id) = make_stack();
        let result = stack.program_setpoint(&id, Power::from(50));
        assert!(result.is_some());
        let output = result.unwrap();
        assert_eq!(output.setpoint, Power::from(50));
    }

    #[test]
    fn test_program_setpoint_returns_none_for_unknown_plant() {
        let (mut stack, _) = make_stack();
        let unknown = PlantId::from("unknown");
        let result = stack.program_setpoint(&unknown, Power::from(50));
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_plants_returns_outputs_for_all_plants() {
        let (mut stack, id) = make_stack();
        stack.program_setpoint(&id, Power::from(60));
        let StackDispatchResults { plants_outputs, .. } = stack.dispatch_plants();
        assert_eq!(plants_outputs.len(), 1);
        assert_eq!(plants_outputs[&id].setpoint, Power::from(60));
    }

    #[test]
    fn test_history_tracks_dispatches() {
        let (mut stack, id) = make_stack();
        stack.program_setpoint(&id, Power::from(40));
        stack.dispatch_plants();
        stack.program_setpoint(&id, Power::from(80));
        stack.dispatch_plants();
        let history = stack.history();
        assert_eq!(history[&id].len(), 2);
        assert_eq!(history[&id][0].setpoint, Power::from(40));
        assert_eq!(history[&id][1].setpoint, Power::from(80));
    }

    fn make_single_plant_stack(
        plant: impl crate::plants::PowerPlant + Send + Sync + 'static,
        id: &str,
    ) -> (StackPlants, PlantId) {
        let plant_id = PlantId::from(id);
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(plant_id.clone(), Box::new(plant));
        (StackPlants::new(plants), plant_id)
    }

    #[test]
    fn test_dispatch_aggregates_gas_plant_output() {
        let (mut stack, id) = make_stack();
        stack.program_setpoint(&id, Power::from(60));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.gas().volume(), &Energy::from(60));
        assert_eq!(state.gas().money(), &Money::from(-600));
    }

    #[test]
    fn test_dispatch_aggregates_nuclear_plant_output() {
        let plant = NuclearPlant::new(Power::from(100), EnergyCost::from(5));
        let (mut stack, id) = make_single_plant_stack(plant, "nuclear-a");
        stack.program_setpoint(&id, Power::from(80));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.nuclear().volume(), &Energy::from(80));
        assert_eq!(state.nuclear().money(), &Money::from(-400));
    }

    #[test]
    fn test_dispatch_aggregates_consumers_output() {
        let plant = Consumers::new(
            EnergyCost::from(50),
            vec![ForecastValue {
                value: -100,
                deviation: 0,
            }],
        );
        let (mut stack, _) = make_single_plant_stack(plant, "consumers-a");
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.consumers().volume(), &Energy::from(-100));
        assert_eq!(state.consumers().money(), &Money::from(5000));
    }

    #[test]
    fn test_dispatch_aggregates_renewable_plant_output() {
        let plant = RenewablePlant::new(vec![ForecastValue {
            value: 75,
            deviation: 0,
        }]);
        let (mut stack, _) = make_single_plant_stack(plant, "renewable-a");
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.renewables().volume(), &Energy::from(75));
        assert_eq!(state.renewables().money(), &Money::from(0));
    }

    #[test]
    fn test_dispatch_battery_positive_setpoint_goes_to_discharge() {
        let plant = Battery::new(Energy::from(200), Energy::from(100));
        let (mut stack, id) = make_single_plant_stack(plant, "battery-a");
        stack.program_setpoint(&id, Power::from(50));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_discharge().volume(), &Energy::from(50));
        assert_eq!(state.battery_charge().volume(), &Energy::from(0));
    }

    #[test]
    fn test_dispatch_battery_negative_setpoint_goes_to_charge() {
        let plant = Battery::new(Energy::from(200), Energy::from(100));
        let (mut stack, id) = make_single_plant_stack(plant, "battery-a");
        stack.program_setpoint(&id, Power::from(-30));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_charge().volume(), &Energy::from(-30));
        assert_eq!(state.battery_discharge().volume(), &Energy::from(0));
    }

    #[test]
    fn test_dispatch_battery_zero_setpoint_updates_neither() {
        let plant = Battery::new(Energy::from(200), Energy::from(100));
        let (mut stack, _) = make_single_plant_stack(plant, "battery-a");
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_discharge().volume(), &Energy::from(0));
        assert_eq!(state.battery_charge().volume(), &Energy::from(0));
    }

    #[test]
    fn test_dispatch_multiple_gas_plants_outputs_are_summed() {
        let id_a = PlantId::from("gas-a");
        let id_b = PlantId::from("gas-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(GasPlant::new(EnergyCost::from(10), Power::from(100))),
        );
        plants.insert(
            id_b.clone(),
            Box::new(GasPlant::new(EnergyCost::from(20), Power::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_a, Power::from(40));
        stack.program_setpoint(&id_b, Power::from(60));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.gas().volume(), &Energy::from(100));
        assert_eq!(state.gas().money(), &Money::from(-(40 * 10 + 60 * 20)));
    }

    #[test]
    fn test_dispatch_multiple_nuclear_plants_outputs_are_summed() {
        let id_a = PlantId::from("nuclear-a");
        let id_b = PlantId::from("nuclear-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(NuclearPlant::new(Power::from(100), EnergyCost::from(5))),
        );
        plants.insert(
            id_b.clone(),
            Box::new(NuclearPlant::new(Power::from(100), EnergyCost::from(5))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_a, Power::from(50));
        stack.program_setpoint(&id_b, Power::from(70));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.nuclear().volume(), &Energy::from(120));
        assert_eq!(state.nuclear().money(), &Money::from(-120 * 5));
    }

    #[test]
    fn test_dispatch_multiple_consumers_outputs_are_summed() {
        let id_a = PlantId::from("consumers-a");
        let id_b = PlantId::from("consumers-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(Consumers::new(
                EnergyCost::from(50),
                vec![ForecastValue {
                    value: -100,
                    deviation: 0,
                }],
            )),
        );
        plants.insert(
            id_b.clone(),
            Box::new(Consumers::new(
                EnergyCost::from(50),
                vec![ForecastValue {
                    value: -75,
                    deviation: 0,
                }],
            )),
        );
        let mut stack = StackPlants::new(plants);
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.consumers().volume(), &Energy::from(-175));
        assert_eq!(state.consumers().money(), &Money::from(175 * 50));
    }

    #[test]
    fn test_dispatch_multiple_renewable_plants_outputs_are_summed() {
        let id_a = PlantId::from("renewable-a");
        let id_b = PlantId::from("renewable-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(RenewablePlant::new(vec![ForecastValue {
                value: 50,
                deviation: 0,
            }])),
        );
        plants.insert(
            id_b.clone(),
            Box::new(RenewablePlant::new(vec![ForecastValue {
                value: 25,
                deviation: 0,
            }])),
        );
        let mut stack = StackPlants::new(plants);
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.renewables().volume(), &Energy::from(75));
        assert_eq!(state.renewables().money(), &Money::from(0));
    }

    #[test]
    fn test_dispatch_multiple_batteries_discharging_are_summed() {
        let id_a = PlantId::from("battery-a");
        let id_b = PlantId::from("battery-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        plants.insert(
            id_b.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_a, Power::from(30));
        stack.program_setpoint(&id_b, Power::from(40));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_discharge().volume(), &Energy::from(70));
        assert_eq!(state.battery_charge().volume(), &Energy::from(0));
    }

    #[test]
    fn test_dispatch_multiple_batteries_charging_are_summed() {
        let id_a = PlantId::from("battery-a");
        let id_b = PlantId::from("battery-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        plants.insert(
            id_b.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_a, Power::from(-20));
        stack.program_setpoint(&id_b, Power::from(-50));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_charge().volume(), &Energy::from(-70));
        assert_eq!(state.battery_discharge().volume(), &Energy::from(0));
    }

    #[test]
    fn test_dispatch_batteries_with_opposite_signs_do_not_cancel_each_other() {
        let id_a = PlantId::from("battery-a");
        let id_b = PlantId::from("battery-b");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_a.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        plants.insert(
            id_b.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_a, Power::from(40));
        stack.program_setpoint(&id_b, Power::from(-30));
        let result = stack.dispatch_plants();
        let state = result.aggregated_state();
        assert_eq!(state.battery_discharge().volume(), &Energy::from(40));
        assert_eq!(state.battery_charge().volume(), &Energy::from(-30));
    }

    #[test]
    fn test_position_sums_volumes_across_all_categories() {
        let id_gas = PlantId::from("gas-a");
        let id_nuclear = PlantId::from("nuclear-a");
        let id_battery = PlantId::from("battery-a");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_gas.clone(),
            Box::new(GasPlant::new(EnergyCost::from(10), Power::from(100))),
        );
        plants.insert(
            id_nuclear.clone(),
            Box::new(NuclearPlant::new(Power::from(100), EnergyCost::from(5))),
        );
        plants.insert(
            id_battery.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_gas, Power::from(60));
        stack.program_setpoint(&id_nuclear, Power::from(40));
        stack.program_setpoint(&id_battery, Power::from(-20));
        let result = stack.dispatch_plants();
        assert_eq!(result.position(), Energy::from(60 + 40 - 20));
    }

    #[test]
    fn test_position_is_zero_when_generation_and_charge_balance() {
        let id_gas = PlantId::from("gas-a");
        let id_battery = PlantId::from("battery-a");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_gas.clone(),
            Box::new(GasPlant::new(EnergyCost::from(10), Power::from(100))),
        );
        plants.insert(
            id_battery.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_gas, Power::from(50));
        stack.program_setpoint(&id_battery, Power::from(-50));
        let result = stack.dispatch_plants();
        assert_eq!(result.position(), Energy::from(0));
    }

    #[test]
    fn test_pnl_sums_costs_across_all_categories() {
        let id_gas = PlantId::from("gas-a");
        let id_nuclear = PlantId::from("nuclear-a");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_gas.clone(),
            Box::new(GasPlant::new(EnergyCost::from(10), Power::from(100))),
        );
        plants.insert(
            id_nuclear.clone(),
            Box::new(NuclearPlant::new(Power::from(100), EnergyCost::from(5))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_gas, Power::from(60));
        stack.program_setpoint(&id_nuclear, Power::from(40));
        let result = stack.dispatch_plants();
        assert_eq!(result.pnl(), Money::from(-(60 * 10 + 40 * 5)));
    }

    #[test]
    fn test_pnl_excludes_zero_cost_plants() {
        let id_gas = PlantId::from("gas-a");
        let id_renewable = PlantId::from("renewable-a");
        let id_battery = PlantId::from("battery-a");
        let mut plants: HashMap<PlantId, Box<dyn crate::plants::PowerPlant + Send + Sync>> =
            HashMap::new();
        plants.insert(
            id_gas.clone(),
            Box::new(GasPlant::new(EnergyCost::from(10), Power::from(100))),
        );
        plants.insert(
            id_renewable.clone(),
            Box::new(RenewablePlant::new(vec![ForecastValue {
                value: 50,
                deviation: 0,
            }])),
        );
        plants.insert(
            id_battery.clone(),
            Box::new(Battery::new(Energy::from(200), Energy::from(100))),
        );
        let mut stack = StackPlants::new(plants);
        stack.program_setpoint(&id_gas, Power::from(60));
        stack.program_setpoint(&id_battery, Power::from(30));
        let result = stack.dispatch_plants();
        assert_eq!(result.pnl(), Money::from(-60 * 10));
    }
}
