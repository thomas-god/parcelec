use serde::Serialize;

use crate::plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

use super::timeseries::{LoopingTimeseries, RngTimeseries, Timeseries};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    pub max_power: i64,
    pub output: PlantOutput,
}
pub struct RenewablePlant<T: Timeseries> {
    max_power: i64,
    setpoint: i64,
    timeseries: T,
}

impl<T: Timeseries> RenewablePlant<T> {
    pub fn new(max_power: i64, mut timeseries: T) -> RenewablePlant<T> {
        RenewablePlant {
            setpoint: timeseries.next_value() as i64,
            max_power,
            timeseries,
        }
    }
}

impl RenewablePlant<RngTimeseries> {
    pub fn new_with_rng(max_power: i64) -> RenewablePlant<RngTimeseries> {
        let timeseries = RngTimeseries::new(0, max_power);
        RenewablePlant::new(max_power, timeseries)
    }
}

impl RenewablePlant<LoopingTimeseries> {
    pub fn new_with_looping(max_power: i64, values: &[isize]) -> RenewablePlant<LoopingTimeseries> {
        let timeseries = LoopingTimeseries::from(values);
        RenewablePlant::new(max_power, timeseries)
    }
}

impl<T: Timeseries> PowerPlant for RenewablePlant<T> {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            setpoint: self.setpoint as isize,
            cost: 0,
        }
    }
    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.setpoint;
        self.setpoint = self.timeseries.next_value() as i64;
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
        let mut plant = RenewablePlant::new_with_rng(1000);

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
