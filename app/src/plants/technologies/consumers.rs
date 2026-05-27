use serde::Serialize;

use crate::{
    forecast::{Forecast, ForecastValue},
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr, technologies::ForecastsBasedPlant},
    utils::units::{EnergyCost, GENERATOR_CONVENTION_TO_MONEY, Money, Power, TIMESTEP},
};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ConsumersPublicRepr {
    pub output: PlantOutput,
    pub revenue: EnergyCost,
}
pub struct Consumers {
    price_per_mwh: EnergyCost,
    state: ForecastsBasedPlant,
    history: Vec<PlantOutput>,
}

impl Consumers {
    pub fn new(price_per_mwh: EnergyCost, forecasts: Vec<ForecastValue>) -> Consumers {
        // TODO expose forecast range in constructor
        let state = ForecastsBasedPlant::new(forecasts, 2);

        Consumers {
            price_per_mwh,
            state,
            history: Vec::new(),
        }
    }

    fn cost(&self) -> Money {
        self.state.setpoint() * TIMESTEP * self.price_per_mwh * GENERATOR_CONVENTION_TO_MONEY
    }

    fn setpoint(&self) -> Power {
        self.state.setpoint()
    }
}

impl PowerPlant for Consumers {
    fn program_setpoint(&mut self, _setpoint: Power) -> PlantOutput {
        PlantOutput {
            cost: self.cost(),
            setpoint: self.setpoint(),
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        let output = PlantOutput {
            cost: self.cost(),
            setpoint: self.setpoint(),
        };
        self.history.push(output);
        self.state.dispatch();
        output
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Consumers(ConsumersPublicRepr {
            revenue: self.price_per_mwh,
            output: PlantOutput {
                setpoint: self.state.setpoint(),
                cost: self.cost(),
            },
        })
    }

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        Some(self.state.forecasts.clone())
    }

    fn get_history(&self) -> Vec<PlantOutput> {
        self.history.clone()
    }

    fn category(&self) -> crate::plants::PlantCategory {
        crate::plants::PlantCategory::Consumers
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        forecast::ForecastValue,
        game::delivery_period::DeliveryPeriodId,
        plants::PowerPlant,
        utils::units::{EnergyCost, Power},
    };

    use super::Consumers;

    fn get_forecasts() -> Vec<ForecastValue> {
        vec![
            ForecastValue {
                value: -100,
                deviation: 50,
            },
            ForecastValue {
                value: -600,
                deviation: 100,
            },
            ForecastValue {
                value: -1000,
                deviation: 150,
            },
        ]
    }

    #[test]
    fn test_consumers() {
        let energy_cost = EnergyCost::from(75);
        let forecasts = get_forecasts();
        let mut consumers = Consumers::new(energy_cost, forecasts);

        // Consumers cannot be programed
        let initial_setpoint = consumers.state.setpoint();
        consumers.program_setpoint(initial_setpoint);
        assert_eq!(consumers.state.setpoint(), initial_setpoint);

        // Initial history is empty
        assert!(consumers.get_history().is_empty());

        // Consumption value changes when dispatched
        let mut history = Vec::new();
        let output = consumers.dispatch();
        history.push(output);

        dbg!(consumers.state.setpoint());
        dbg!(initial_setpoint);
        assert_ne!(consumers.state.setpoint(), initial_setpoint);
        assert_eq!(consumers.get_history(), history);

        // Dispatching should return the previous setpoint
        let previous_value: Power = consumers.state.setpoint();
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
            vec![DeliveryPeriodId::from(3), DeliveryPeriodId::from(4)]
        );
    }
}
