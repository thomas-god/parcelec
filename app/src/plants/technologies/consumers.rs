use rand::Rng;
use serde::Serialize;

use crate::plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

use super::timeseries::{LoopingTimeseries, RngTimeseries, Timeseries};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub max_power: i64,
    pub output: PlantOutput,
}
pub struct Consumers<T: Timeseries> {
    max_power: i64,
    price_per_mwh: i64,
    setpoint: i64,
    timeseries: T,
}

impl<T: Timeseries> Consumers<T> {
    pub fn new(max_power: i64, price_per_mwh: i64, timeseries: T) -> Consumers<T> {
        Consumers {
            setpoint: rand::rng().random_range(-max_power..0),
            price_per_mwh,
            max_power,
            timeseries,
        }
    }
}

impl Consumers<RngTimeseries> {
    pub fn new_with_rng(max_power: i64, price_per_mwh: i64) -> Consumers<RngTimeseries> {
        let timeseries = RngTimeseries::new(0, max_power);
        Consumers::new(max_power, price_per_mwh, timeseries)
    }
}

impl Consumers<LoopingTimeseries> {
    pub fn new_with_looping(
        max_power: i64,
        price_per_mwh: i64,
        values: &[isize],
    ) -> Consumers<LoopingTimeseries> {
        let timeseries = LoopingTimeseries::from(values);
        Consumers::new(max_power, price_per_mwh, timeseries)
    }
}

impl<T: Timeseries> PowerPlant for Consumers<T> {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            cost: (self.setpoint * self.price_per_mwh) as isize,
            setpoint: self.setpoint as isize,
        }
    }
    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.setpoint;
        let cost = previous_setpoint * self.price_per_mwh;
        self.setpoint = self.timeseries.next_value() as i64;
        PlantOutput {
            cost: cost as isize,
            setpoint: previous_setpoint as isize,
        }
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.setpoint as isize,
                cost: (self.setpoint * self.price_per_mwh) as isize,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::{
        technologies::consumers::ConsumersPublicRepr, PlantOutput, PowerPlant, PowerPlantPublicRepr,
    };

    use super::Consumers;

    #[test]
    fn test_consumers() {
        let mut plant = Consumers::new_with_rng(1000, 65);

        // Consumers have negative setpoint, i.e. they consume energy
        assert!(plant.setpoint < 0);
        // Consumers have negative costs, i.e. they pay you
        let PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            output: PlantOutput { cost, .. },
            ..
        }) = plant.current_state()
        else {
            unreachable!()
        };
        assert!(cost < 0);

        // Consumers cannot be programed
        let initial_setpoint = plant.setpoint as isize;
        plant.program_setpoint(initial_setpoint);
        assert_eq!(plant.setpoint as isize, initial_setpoint);

        // Consumption value changes when dispatched
        plant.dispatch();
        assert_ne!(plant.setpoint as isize, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value = plant.setpoint;
        let returned_value = plant.dispatch();
        assert_eq!(previous_value as isize, returned_value.setpoint);
    }
}
