use serde::Serialize;

use crate::{
    forecast::{Forecast, Forecasts},
    game::delivery_period::DeliveryPeriodId,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

use super::variable::VariablePlant;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub max_power: isize,
    pub output: PlantOutput,
}
pub struct Consumers {
    max_power: isize,
    price_per_mwh: isize,
    state: VariablePlant,
    period: DeliveryPeriodId,
    current_setpoint: isize,
    current_forecasts: Forecasts,
}

impl Consumers {
    pub fn new(max_power: isize, price_per_mwh: isize, forecasts: Vec<Forecast>) -> Consumers {
        let period = DeliveryPeriodId::from(1);
        let state = VariablePlant::new(forecasts);
        let current_setpoint = state.get_setpoint(period).unwrap_or(0);
        let current_forecasts = state.get_forecasts(period);

        Consumers {
            current_setpoint,
            price_per_mwh,
            max_power,
            current_forecasts,
            period,
            state,
        }
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
        self.current_forecasts = self.state.get_forecasts(self.period);
        self.current_setpoint = self.state.get_setpoint(self.period).unwrap_or(0);
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

    use crate::{
        forecast::{Forecast, ForecastValue},
        game::delivery_period::DeliveryPeriodId,
        plants::PowerPlant,
    };

    use super::Consumers;

    fn get_forecasts() -> Vec<Forecast> {
        vec![
            Forecast {
                period: DeliveryPeriodId::from(1),
                value: ForecastValue {
                    value: -100,
                    deviation: 50,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(2),
                value: ForecastValue {
                    value: -600,
                    deviation: 100,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(3),
                value: ForecastValue {
                    value: -1000,
                    deviation: 150,
                },
            },
        ]
    }

    #[test]
    fn test_consumers() {
        let max_power = 1000;
        let energy_cost = 75;
        let forecasts = get_forecasts();
        let mut consumers = Consumers::new(max_power, energy_cost, forecasts);

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
        let forecsts = get_forecasts();
        let mut consumers = Consumers::new(max_power, energy_cost, forecsts);

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
