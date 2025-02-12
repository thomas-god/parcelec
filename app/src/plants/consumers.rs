use rand::Rng;
use serde::Serialize;

use super::{PowerPlant, PowerPlantPublicRepr};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    max_power: i64,
    setpoint: i64,
    cost: isize,
}
pub struct Consumers {
    max_power: i64,
    price_per_mwh: i64,
    setpoint: i64,
}

impl Consumers {
    pub fn new(max_power: i64, price_per_mwh: i64) -> Consumers {
        Consumers {
            setpoint: rand::rng().random_range(-max_power..0),
            price_per_mwh,
            max_power,
        }
    }
}

impl PowerPlant for Consumers {
    fn program_setpoint(&mut self, _setpoint: isize) -> isize {
        (self.setpoint * self.price_per_mwh) as isize
    }
    fn dispatch(&mut self) -> isize {
        let cost = self.setpoint * self.price_per_mwh;
        self.setpoint = rand::rng().random_range(-self.max_power..0);
        cost as isize
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            max_power: self.max_power,
            setpoint: self.setpoint,
            cost: (self.setpoint * self.price_per_mwh) as isize,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::{consumers::ConsumersPublicRepr, PowerPlant, PowerPlantPublicRepr};

    use super::Consumers;

    #[test]
    fn test_consumers() {
        let mut plant = Consumers::new(1000, 65);

        // Consumers have negative setpoint, i.e. they consume energy
        assert!(plant.setpoint < 0);
        // Consumers have negative costs, i.e. they pay you
        let PowerPlantPublicRepr::Consumers(ConsumersPublicRepr { cost, .. }) =
            plant.current_state()
        else {
            unreachable!()
        };
        assert!(cost < 0);

        // Consumers cannot be programed
        let setpoint = plant.setpoint as isize;
        plant.program_setpoint(setpoint);
        assert_eq!(plant.setpoint as isize, setpoint);

        // Consumption value changes when dispatched
        plant.dispatch();
        assert_ne!(plant.setpoint as isize, setpoint);
    }
}
