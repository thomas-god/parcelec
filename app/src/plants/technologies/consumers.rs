use serde::Serialize;

use crate::{
    forecast::{ForecastLevel, map_value_to_forecast_level},
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

use super::timeseries::{LoopingTimeseries, RngTimeseries, Timeseries};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub max_power: i64,
    pub output: PlantOutput,
}
pub struct Consumers<T: Timeseries> {
    max_power: i64,
    price_per_mwh: i64,
    current_setpoint: i64,
    setpoints: T,
    current_forecast: Option<i64>,
    forecasts: Option<T>,
}

impl<T: Timeseries> Consumers<T> {
    pub fn new(
        max_power: i64,
        price_per_mwh: i64,
        mut setpoints: T,
        mut forecasts: Option<T>,
    ) -> Consumers<T> {
        Consumers {
            current_setpoint: setpoints.next_value() as i64,
            setpoints,
            price_per_mwh,
            max_power,
            current_forecast: forecasts.as_mut().map(|f| f.next_value() as i64),
            forecasts,
        }
    }
}

impl Consumers<RngTimeseries> {
    pub fn new_with_rng(max_power: i64, price_per_mwh: i64) -> Consumers<RngTimeseries> {
        let timeseries = RngTimeseries::new(-max_power, 0);
        let forecasts = None;
        Consumers::new(max_power, price_per_mwh, timeseries, forecasts)
    }
}

impl Consumers<LoopingTimeseries> {
    pub fn new_with_looping(
        max_power: i64,
        price_per_mwh: i64,
        mut values: Vec<isize>,
    ) -> Consumers<LoopingTimeseries> {
        let timeseries = LoopingTimeseries::from(&values[..]);
        values.rotate_left(1);
        let forecasts = LoopingTimeseries::from(&values[..]);
        Consumers::new(max_power, price_per_mwh, timeseries, Some(forecasts))
    }
}

impl<T: Timeseries> PowerPlant for Consumers<T> {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            cost: (self.current_setpoint * self.price_per_mwh) as isize,
            setpoint: self.current_setpoint as isize,
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.current_setpoint;
        let cost = previous_setpoint * self.price_per_mwh;
        self.current_forecast = self.forecasts.as_mut().map(|f| f.next_value() as i64);
        self.current_setpoint = self.setpoints.next_value() as i64;
        PlantOutput {
            cost: cost as isize,
            setpoint: previous_setpoint as isize,
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.current_setpoint as isize,
                cost: (self.current_setpoint * self.price_per_mwh) as isize,
            },
        })
    }

    fn get_forecast(&self) -> Option<ForecastLevel> {
        self.current_forecast
            .map(|f| map_value_to_forecast_level(f as isize, -self.max_power as isize))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        forecast::ForecastLevel,
        plants::{
            PlantOutput, PowerPlant, PowerPlantPublicRepr,
            technologies::consumers::ConsumersPublicRepr,
        },
    };

    use super::Consumers;

    #[test]
    fn test_consumers() {
        let mut plant = Consumers::new_with_rng(1000, 65);

        // Consumers have negative setpoint, i.e. they consume energy
        assert!(plant.current_setpoint < 0);
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
        let initial_setpoint = plant.current_setpoint as isize;
        plant.program_setpoint(initial_setpoint);
        assert_eq!(plant.current_setpoint as isize, initial_setpoint);

        // Consumption value changes when dispatched
        plant.dispatch();
        assert_ne!(plant.current_setpoint as isize, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value = plant.current_setpoint;
        let returned_value = plant.dispatch();
        assert_eq!(previous_value as isize, returned_value.setpoint);
    }

    #[test]
    fn test_rng_consumers_have_no_forecast() {
        let consumers = Consumers::new_with_rng(1000, 85);
        assert!(consumers.get_forecast().is_none());
    }

    #[test]
    fn test_looping_consumers_forecasts() {
        let mut consumers = Consumers::new_with_looping(1000, 85, vec![-100, -500, -900]);

        assert_eq!(consumers.get_forecast(), Some(ForecastLevel::Medium));

        consumers.dispatch();

        assert_eq!(consumers.get_forecast(), Some(ForecastLevel::High));

        consumers.dispatch();

        assert_eq!(consumers.get_forecast(), Some(ForecastLevel::Low));
    }
}
