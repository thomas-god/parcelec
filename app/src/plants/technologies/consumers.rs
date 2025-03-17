use serde::Serialize;

use crate::{
    forecast::{Clip, Forecast, Forecasts, forecast_from_timeseries},
    game::delivery_period::DeliveryPeriodId,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

use super::timeseries::Timeseries;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub max_power: isize,
    pub output: PlantOutput,
}
pub struct Consumers {
    max_power: isize,
    price_per_mwh: isize,
    setpoints: Timeseries,
    period: DeliveryPeriodId,
    current_setpoint: isize,
    current_forecasts: Forecasts,
}

impl Consumers {
    pub fn new(max_power: isize, price_per_mwh: isize, setpoints: Timeseries) -> Consumers {
        let period = DeliveryPeriodId::from(1);
        let current_setpoint = setpoints.value_at(period);
        let current_forecasts = forecast_from_timeseries(
            &setpoints,
            period.next(),
            &Some(Clip {
                min: -max_power,
                max: 0,
            }),
        );

        Consumers {
            current_setpoint,
            setpoints,
            price_per_mwh,
            max_power,
            current_forecasts,
            period,
        }
    }
}

impl Consumers {
    pub fn from_values(max_power: isize, price_per_mwh: isize, values: Vec<isize>) -> Consumers {
        let timeseries = Timeseries::from(&values[..]);

        Consumers::new(max_power, price_per_mwh, timeseries)
    }
}

impl PowerPlant for Consumers {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            cost: self.current_setpoint * self.price_per_mwh,
            setpoint: self.current_setpoint,
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.current_setpoint;
        let cost = previous_setpoint * self.price_per_mwh;
        self.period = self.period.next();
        self.current_forecasts = forecast_from_timeseries(
            &self.setpoints,
            self.period.next(),
            &Some(Clip {
                min: -self.max_power,
                max: 0,
            }),
        );
        self.current_setpoint = self.setpoints.value_at(self.period);
        PlantOutput {
            cost,
            setpoint: previous_setpoint,
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.current_setpoint,
                cost: (self.current_setpoint * self.price_per_mwh),
            },
        })
    }

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        Some(self.current_forecasts.clone())
    }
}

#[cfg(test)]
mod tests {

    use crate::{game::delivery_period::DeliveryPeriodId, plants::PowerPlant};

    use super::Consumers;

    #[test]
    fn test_consumers() {
        let max_power = 1000;
        let energy_cost = 75;
        let mut consumers = Consumers::from_values(max_power, energy_cost, vec![-100, -500, -900]);

        // Consumers cannot be programed
        let initial_setpoint = consumers.current_setpoint;
        consumers.program_setpoint(initial_setpoint);
        assert_eq!(consumers.current_setpoint, initial_setpoint);

        // Consumption value changes when dispatched
        consumers.dispatch();
        assert_ne!(consumers.current_setpoint, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value = consumers.current_setpoint;
        let returned_value = consumers.dispatch();
        assert_eq!(previous_value, returned_value.setpoint);
    }

    #[test]
    fn test_consumers_forecasts_periods() {
        let max_power = 1000;
        let energy_cost = 75;
        let mut consumers = Consumers::from_values(max_power, energy_cost, vec![-100, -500, -900]);

        let forecasts = consumers.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(2), DeliveryPeriodId::from(3)]
        );

        consumers.dispatch();

        let forecasts = consumers.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(3)]
        );

        consumers.dispatch();

        let forecasts = consumers.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![]
        );

        consumers.dispatch();

        let forecasts = consumers.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![]
        );
    }
}
