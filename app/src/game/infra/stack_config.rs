use std::{cmp::min, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    forecast::{ForecastValue, NormalizedForecastValue},
    plants::{
        PlantId, PowerPlant, StackPlants,
        technologies::{
            battery::Battery, consumers::Consumers, gas_plant::GasPlant, nuclear::NuclearPlant,
            renewable::RenewablePlant,
        },
    },
    utils::units::{Energy, EnergyCost, Power},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameStackConfig {
    Fixed(GameStackFixedConfig),
    PerPlayer(GameStackPerPlayerBaseConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameStackFixedConfig {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub consumers_revenues: EnergyCost,
    pub gas_capacity: Power,
    pub nuclear_capacity: Power,
    pub battery_capacity: Energy,
    pub consumers_forecasts: Vec<ForecastValue>,
    pub renewable_forecasts: Vec<ForecastValue>,
}

impl GameStackFixedConfig {
    pub fn generate_plants(&self) -> StackPlants {
        let mut stack: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();

        stack.insert(
            PlantId::default(),
            Box::new(Battery::new(self.battery_capacity, Energy::from(0))),
        );
        stack.insert(
            PlantId::default(),
            Box::new(GasPlant::new(self.gas_cost, self.gas_capacity)),
        );
        stack.insert(
            PlantId::default(),
            Box::new(NuclearPlant::new(self.nuclear_capacity, self.nuclear_cost)),
        );
        stack.insert(
            PlantId::default(),
            Box::new(RenewablePlant::new(self.renewable_forecasts.clone())),
        );
        stack.insert(
            PlantId::default(),
            Box::new(Consumers::new(
                self.consumers_revenues,
                self.consumers_forecasts.clone(),
            )),
        );

        StackPlants::new(stack)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameStackPerPlayerBaseConfig {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub consumers_revenues: EnergyCost,
    pub gas_max_capacity: Power,
    pub nuclear_max_capacity: Power,
    pub battery_max_capacity: Energy,
    pub consumers_max_abs_capacity: Power,
    pub consumers_forecasts: Vec<NormalizedForecastValue>,
    pub renewable_max_capacity: Power,
    pub renewable_forecasts: Vec<NormalizedForecastValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameStackPerPlayerPlayerConfig {
    pub gas_capacity: Power,
    pub nuclear_capacity: Power,
    pub battery_capacity: Energy,
    pub consumers_capacity: Power,
    pub renewable_capacity: Power,
}

impl GameStackPerPlayerBaseConfig {
    pub fn generate_plants(&self, player_config: GameStackPerPlayerPlayerConfig) -> StackPlants {
        let mut stack: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();

        stack.insert(
            PlantId::default(),
            Box::new(Battery::new(
                min(self.battery_max_capacity, player_config.battery_capacity),
                Energy::from(0),
            )),
        );
        stack.insert(
            PlantId::default(),
            Box::new(GasPlant::new(
                self.gas_cost,
                min(self.gas_max_capacity, player_config.gas_capacity),
            )),
        );
        stack.insert(
            PlantId::default(),
            Box::new(NuclearPlant::new(
                min(self.nuclear_max_capacity, player_config.nuclear_capacity),
                self.nuclear_cost,
            )),
        );
        let capacity = min(
            self.renewable_max_capacity,
            player_config.renewable_capacity,
        );
        stack.insert(
            PlantId::default(),
            Box::new(RenewablePlant::new(
                self.renewable_forecasts
                    .iter()
                    .map(|f| f.as_forecast(capacity.into()))
                    .collect(),
            )),
        );

        let capacity = -min(
            self.consumers_max_abs_capacity.abs(),
            player_config.consumers_capacity.abs(),
        );
        stack.insert(
            PlantId::default(),
            Box::new(Consumers::new(
                self.consumers_revenues,
                self.consumers_forecasts
                    .iter()
                    .map(|f| f.as_forecast(capacity.into()))
                    .collect(),
            )),
        );

        StackPlants::new(stack)
    }
}

#[cfg(test)]
mod test_fixed_config_generate_stack {

    use crate::plants::PowerPlantPublicRepr;

    use super::*;

    #[test]
    fn test_generate_stack() {
        let config = GameStackFixedConfig {
            gas_cost: EnergyCost::from(70),
            gas_capacity: Power::from(300),
            nuclear_cost: EnergyCost::from(35),
            nuclear_capacity: Power::from(1000),
            battery_capacity: Energy::from(200),
            consumers_revenues: EnergyCost::from(60),
            consumers_forecasts: vec![],
            renewable_forecasts: vec![],
        };

        let stack = config.generate_plants();
        let snapshot = stack.snapshot();

        assert_eq!(snapshot.len(), 5);

        for (_id, plant) in snapshot.iter() {
            match plant {
                PowerPlantPublicRepr::Battery(battery) => {
                    assert_eq!(battery.max_charge, Energy::from(200))
                }
                PowerPlantPublicRepr::Consumers(consumers) => {
                    assert_eq!(consumers.revenue, EnergyCost::from(60))
                }
                PowerPlantPublicRepr::GasPlant(plant) => {
                    assert_eq!(plant.settings.energy_cost(), EnergyCost::from(70));
                    assert_eq!(plant.settings.max_setpoint(), Power::from(300));
                }
                PowerPlantPublicRepr::Nuclear(plant) => {
                    assert_eq!(plant.energy_cost, EnergyCost::from(35));
                    assert_eq!(plant.max_setpoint, Power::from(1000));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test_per_player_config_generate_stack {
    use crate::plants::PowerPlantPublicRepr;

    use super::*;

    fn base_config() -> GameStackPerPlayerBaseConfig {
        GameStackPerPlayerBaseConfig {
            gas_cost: EnergyCost::from(70),
            nuclear_cost: EnergyCost::from(35),
            consumers_revenues: EnergyCost::from(60),
            gas_max_capacity: Power::from(500),
            nuclear_max_capacity: Power::from(1000),
            battery_max_capacity: Energy::from(400),
            consumers_max_abs_capacity: Power::from(-800),
            consumers_forecasts: vec![NormalizedForecastValue::try_new(1., 0.).unwrap()],
            renewable_max_capacity: Power::from(600),
            renewable_forecasts: vec![NormalizedForecastValue::try_new(1., 0.).unwrap()],
        }
    }

    #[test]
    fn test_generate_stack_player_below_max() {
        let base = base_config();
        let player_config = GameStackPerPlayerPlayerConfig {
            gas_capacity: Power::from(300),
            nuclear_capacity: Power::from(800),
            battery_capacity: Energy::from(200),
            consumers_capacity: Power::from(-500),
            renewable_capacity: Power::from(400),
        };

        let stack = base.generate_plants(player_config);
        let snapshot = stack.snapshot();

        assert_eq!(snapshot.len(), 5);

        for (_id, plant) in snapshot.iter() {
            match plant {
                PowerPlantPublicRepr::Battery(battery) => {
                    assert_eq!(battery.max_charge, Energy::from(200))
                }
                PowerPlantPublicRepr::Consumers(consumers) => {
                    assert_eq!(consumers.revenue, EnergyCost::from(60))
                }
                PowerPlantPublicRepr::GasPlant(plant) => {
                    assert_eq!(plant.settings.energy_cost(), EnergyCost::from(70));
                    assert_eq!(plant.settings.max_setpoint(), Power::from(300));
                }
                PowerPlantPublicRepr::Nuclear(plant) => {
                    assert_eq!(plant.energy_cost, EnergyCost::from(35));
                    assert_eq!(plant.max_setpoint, Power::from(800));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_generate_stack_player_exceeds_max() {
        let base = base_config();
        let player_config = GameStackPerPlayerPlayerConfig {
            gas_capacity: Power::from(900),
            nuclear_capacity: Power::from(2000),
            battery_capacity: Energy::from(1000),
            consumers_capacity: Power::from(-1500),
            renewable_capacity: Power::from(1200),
        };

        let stack = base.generate_plants(player_config);
        let snapshot = stack.snapshot();

        assert_eq!(snapshot.len(), 5);

        for (_id, plant) in snapshot.iter() {
            match plant {
                PowerPlantPublicRepr::Battery(battery) => {
                    assert_eq!(battery.max_charge, Energy::from(400))
                }
                PowerPlantPublicRepr::Consumers(consumers) => {
                    assert_eq!(consumers.revenue, EnergyCost::from(60));
                    dbg!(consumers.output.setpoint);
                    assert!(consumers.output.setpoint == base.consumers_max_abs_capacity)
                }
                PowerPlantPublicRepr::GasPlant(plant) => {
                    assert_eq!(plant.settings.energy_cost(), EnergyCost::from(70));
                    assert_eq!(plant.settings.max_setpoint(), Power::from(500));
                }
                PowerPlantPublicRepr::Nuclear(plant) => {
                    assert_eq!(plant.energy_cost, EnergyCost::from(35));
                    assert_eq!(plant.max_setpoint, Power::from(1000));
                }
                PowerPlantPublicRepr::RenewablePlant(plant) => {
                    assert!(plant.output.setpoint == base.renewable_max_capacity)
                }
            }
        }
    }
}
