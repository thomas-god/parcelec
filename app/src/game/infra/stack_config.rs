use std::collections::HashMap;

use crate::{
    forecast::Forecast,
    plants::{
        PlantId, PowerPlant, StackPlants,
        technologies::{
            battery::Battery, consumers::Consumers, gas_plant::GasPlant, nuclear::NuclearPlant,
            renewable::RenewablePlant,
        },
    },
    utils::units::{Energy, EnergyCost, Power},
};

pub enum GameStackConfig {
    Fixed(GameStackBaseConfig, GameStackCapacitiesConfig),
    PerPlayer(GameStackBaseConfig),
}

/// Configuration for the stack's plants that are common to all players (costs, efficiency, etc.).
#[derive(Debug, Clone)]
pub struct GameStackBaseConfig {
    pub gas_cost: EnergyCost,
    pub nuclear_cost: EnergyCost,
    pub consumers_revenues: EnergyCost,
}

/// Configuration for the stack's plants that may vary per player, typically the installed capacities.
#[derive(Debug, Clone)]
pub struct GameStackCapacitiesConfig {
    pub gas_capacity: Power,
    pub nuclear_capcity: Power,
    pub battery_capacity: Energy,
    pub consumers_forecasts: Vec<Forecast>,
    pub renewable_forecasts: Vec<Forecast>,
}

pub fn build_stack_from_configs(
    base: &GameStackBaseConfig,
    capacities: &GameStackCapacitiesConfig,
) -> StackPlants {
    let mut stack: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();

    stack.insert(
        PlantId::default(),
        Box::new(Battery::new(capacities.battery_capacity, Energy::from(0))),
    );
    stack.insert(
        PlantId::default(),
        Box::new(GasPlant::new(base.gas_cost, capacities.gas_capacity)),
    );
    stack.insert(
        PlantId::default(),
        Box::new(NuclearPlant::new(
            capacities.nuclear_capcity,
            base.nuclear_cost,
        )),
    );
    stack.insert(
        PlantId::default(),
        Box::new(RenewablePlant::new(capacities.renewable_forecasts.clone())),
    );
    stack.insert(
        PlantId::default(),
        Box::new(Consumers::new(
            base.consumers_revenues,
            capacities.consumers_forecasts.clone(),
        )),
    );

    StackPlants::new(stack)
}

#[cfg(test)]
mod test_build_stack_from_configuration {
    use crate::plants::PowerPlantPublicRepr;

    use super::*;

    #[test]
    fn test_build_stack() {
        let base_configuration = GameStackBaseConfig {
            consumers_revenues: EnergyCost::from(60),
            gas_cost: EnergyCost::from(70),
            nuclear_cost: EnergyCost::from(35),
        };
        let capacities_configuration = GameStackCapacitiesConfig {
            gas_capacity: Power::from(300),
            nuclear_capcity: Power::from(1000),
            battery_capacity: Energy::from(200),
            renewable_forecasts: vec![],
            consumers_forecasts: vec![],
        };

        let stack = build_stack_from_configs(&base_configuration, &capacities_configuration);
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
