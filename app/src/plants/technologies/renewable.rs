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
    current_setpoint: i64,
    setpoints: T,
    current_forecast: Option<i64>,
    forecasts: Option<T>,
}

impl<T: Timeseries> RenewablePlant<T> {
    pub fn new(max_power: i64, mut timeseries: T, mut forecasts: Option<T>) -> RenewablePlant<T> {
        RenewablePlant {
            current_setpoint: timeseries.next_value() as i64,
            setpoints: timeseries,
            max_power,
            current_forecast: forecasts.as_mut().map(|f| f.next_value() as i64),
            forecasts,
        }
    }
}

impl RenewablePlant<RngTimeseries> {
    pub fn new_with_rng(max_power: i64) -> RenewablePlant<RngTimeseries> {
        let timeseries = RngTimeseries::new(0, max_power);
        RenewablePlant::new(max_power, timeseries, None)
    }
}

impl RenewablePlant<LoopingTimeseries> {
    pub fn new_with_looping(
        max_power: i64,
        mut values: Vec<isize>,
    ) -> RenewablePlant<LoopingTimeseries> {
        let setpoints = LoopingTimeseries::from(&values[..]);
        values.rotate_left(1);
        let forecasts = LoopingTimeseries::from(&values[..]);
        RenewablePlant::new(max_power, setpoints, Some(forecasts))
    }
}

impl<T: Timeseries> PowerPlant for RenewablePlant<T> {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            setpoint: self.current_setpoint as isize,
            cost: 0,
        }
    }
    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.current_setpoint;
        self.current_forecast = self.forecasts.as_mut().map(|f| f.next_value() as i64);
        self.current_setpoint = self.setpoints.next_value() as i64;
        PlantOutput {
            setpoint: previous_setpoint as isize,
            cost: 0,
        }
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::RenewablePlant(RenewablePlantPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.current_setpoint as isize,
                cost: 0,
            },
        })
    }
    fn get_forecast(&self) -> Option<isize> {
        self.current_forecast.map(|f| f as isize)
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
        let initial_setpoint = plant.current_setpoint as isize;
        let PlantOutput { setpoint, .. } = plant.program_setpoint(initial_setpoint + 1);
        assert_eq!(setpoint, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value = plant.current_setpoint;
        let returned_value = plant.dispatch();
        assert_eq!(previous_value as isize, returned_value.setpoint);
    }

    #[test]
    fn test_rng_renewable_plant_has_no_forecast() {
        let plant = RenewablePlant::new_with_rng(1000);
        assert!(plant.get_forecast().is_none());
    }

    #[test]
    fn test_looping_consumers_forecasts() {
        let mut plant = RenewablePlant::new_with_looping(1000, vec![1, 2, 3]);

        assert_eq!(plant.get_forecast(), Some(2));

        plant.dispatch();

        assert_eq!(plant.get_forecast(), Some(3));

        plant.dispatch();

        assert_eq!(plant.get_forecast(), Some(1));
    }
}
