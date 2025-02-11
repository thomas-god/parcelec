use rand::Rng;
use serde::Serialize;

use super::{PowerPlant, PowerPlantPublicRepr};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    max_power: i64,
    setpoint: i64,
}
pub struct RenewablePlant {
    max_power: i64,
    setpoint: i64,
}

impl RenewablePlant {
    pub fn new(max_power: i64) -> RenewablePlant {
        RenewablePlant {
            setpoint: rand::rng().random_range(0..max_power),
            max_power,
        }
    }
}

impl PowerPlant for RenewablePlant {
    fn program_setpoint(&mut self, _setpoint: isize) -> isize {
        0
    }
    fn dispatch(&mut self) -> isize {
        self.setpoint = rand::rng().random_range(0..self.max_power);
        0
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::RenewablePlant(RenewablePlantPublicRepr {
            max_power: self.max_power,
            setpoint: self.setpoint,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::PowerPlant;

    use super::RenewablePlant;

    #[test]
    fn test_renewable_plant() {
        let mut plant = RenewablePlant::new(1000);

        // Plant has no associated cost
        assert_eq!(plant.program_setpoint(100), 0);

        // The plant cannot be programed
        let setpoint = plant.setpoint as isize;
        plant.program_setpoint(setpoint);
        assert_eq!(plant.setpoint as isize, setpoint);

        // Setpoint changes when dispatched
        plant.dispatch();
        assert_ne!(plant.setpoint as isize, setpoint);
    }
}
