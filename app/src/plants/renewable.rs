use rand::Rng;
use serde::Serialize;

use super::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    pub max_power: i64,
    pub output: PlantOutput,
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
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            setpoint: self.setpoint as isize,
            cost: 0,
        }
    }
    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.setpoint;
        self.setpoint = rand::rng().random_range(0..self.max_power);
        PlantOutput {
            setpoint: previous_setpoint as isize,
            cost: 0,
        }
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::RenewablePlant(RenewablePlantPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.setpoint as isize,
                cost: 0,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::{PlantOutput, PowerPlant};

    use super::RenewablePlant;

    #[test]
    fn test_renewable_plant() {
        let mut plant = RenewablePlant::new(1000);

        // Plant has no associated cost
        let PlantOutput { cost, .. } = plant.program_setpoint(100);
        assert_eq!(cost, 0);

        // The plant cannot be programed
        let initial_setpoint = plant.setpoint as isize;
        let PlantOutput { setpoint, .. } = plant.program_setpoint(initial_setpoint + 1);
        assert_eq!(setpoint, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value = plant.setpoint;
        let returned_value = plant.dispatch();
        assert_eq!(previous_value as isize, returned_value.setpoint);
    }
}
