use serde::Serialize;

use crate::{
    forecast::{Forecast, Forecasts},
    game::delivery_period::DeliveryPeriodId,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
    utils::units::{EnergyCost, Power, TIMESTEP},
};

use super::variable::VariablePlant;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub output: PlantOutput,
}
pub struct Consumers {
    price_per_mwh: EnergyCost,
    state: VariablePlant,
    period: DeliveryPeriodId,
    current_setpoint: Power,
    current_forecasts: Forecasts,
    history: Vec<PlantOutput>,
}

impl Consumers {
    pub fn new(price_per_mwh: EnergyCost, forecasts: Vec<Forecast>) -> Consumers {
        let period = DeliveryPeriodId::from(1);
        let state = VariablePlant::new(forecasts);
        let current_setpoint = Power::from(state.get_setpoint(period).unwrap_or(0));
        let current_forecasts = state.get_forecasts(period);

        Consumers {
            current_setpoint,
            price_per_mwh,
            current_forecasts,
            period,
            state,
            history: Vec::new(),
        }
    }
}

impl PowerPlant for Consumers {
    fn program_setpoint(&mut self, _setpoint: Power) -> PlantOutput {
        PlantOutput {
            cost: self.current_setpoint * TIMESTEP * self.price_per_mwh,
            setpoint: self.current_setpoint,
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.current_setpoint;
        let cost = previous_setpoint * TIMESTEP * self.price_per_mwh;
        self.period = self.period.next();
        self.current_forecasts = self.state.get_forecasts(self.period);
        self.current_setpoint = Power::from(self.state.get_setpoint(self.period).unwrap_or(0));
        let output = PlantOutput {
            cost,
            setpoint: previous_setpoint,
        };
        self.history.push(output.clone());
        output
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            output: PlantOutput {
                setpoint: self.current_setpoint,
                cost: (self.current_setpoint * TIMESTEP * self.price_per_mwh),
            },
        })
    }

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        Some(self.current_forecasts.clone())
    }

    fn get_history(&self) -> Vec<PlantOutput> {
        self.history.clone()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        forecast::{Forecast, ForecastValue},
        game::delivery_period::DeliveryPeriodId,
        plants::PowerPlant,
        utils::units::{EnergyCost, Power},
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
        let energy_cost = EnergyCost::from(75);
        let forecasts = get_forecasts();
        let mut consumers = Consumers::new(energy_cost, forecasts);

        // Consumers cannot be programed
        let initial_setpoint = consumers.current_setpoint;
        consumers.program_setpoint(initial_setpoint.into());
        assert_eq!(consumers.current_setpoint, initial_setpoint);

        // Initial history is empty
        assert!(consumers.get_history().is_empty());

        // Consumption value changes when dispatched
        let mut history = Vec::new();
        let output = consumers.dispatch();
        history.push(output);

        assert_ne!(consumers.current_setpoint, initial_setpoint);
        assert_eq!(consumers.get_history(), history);

        // Dispatching should return the previous setpoint
        let previous_value: Power = consumers.current_setpoint.into();
        let returned_value = consumers.dispatch();
        assert_eq!(previous_value, returned_value.setpoint);
        history.push(returned_value);
        assert_eq!(consumers.get_history(), history);
    }

    #[test]
    fn test_consumers_forecasts_periods() {
        let energy_cost = EnergyCost::from(75);
        let forecsts = get_forecasts();
        let mut consumers = Consumers::new(energy_cost, forecsts);

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
