use serde::Serialize;

use crate::{
    forecast::{Forecast, Forecasts},
    game::delivery_period::DeliveryPeriodId,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

use super::variable::VariablePlant;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    pub max_power: isize,
    pub output: PlantOutput,
}
pub struct RenewablePlant {
    max_power: isize,
    state: VariablePlant,
    period: DeliveryPeriodId,
    current_setpoint: isize,
    current_forecasts: Forecasts,
}

impl RenewablePlant {
    pub fn new(max_power: isize, forecasts: Vec<Forecast>) -> RenewablePlant {
        let period = DeliveryPeriodId::from(1);
        let plant = VariablePlant::new(forecasts);
        let current_setpoint = plant.get_setpoint(period).unwrap_or(0);
        let current_forecasts = plant.get_forecasts(period);

        RenewablePlant {
            current_setpoint,
            state: plant,
            max_power,
            current_forecasts,
            period,
        }
    }
}

impl PowerPlant for RenewablePlant {
    fn program_setpoint(&mut self, _setpoint: isize) -> PlantOutput {
        PlantOutput {
            setpoint: self.current_setpoint,
            cost: 0,
        }
    }
    fn dispatch(&mut self) -> PlantOutput {
        let previous_setpoint = self.current_setpoint;
        self.period = self.period.next();
        self.current_setpoint = self.state.get_setpoint(self.period).unwrap_or(0);
        self.current_forecasts = self.state.get_forecasts(self.period);

        PlantOutput {
            setpoint: previous_setpoint,
            cost: 0,
        }
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::RenewablePlant(RenewablePlantPublicRepr {
            max_power: self.max_power,
            output: PlantOutput {
                setpoint: self.current_setpoint,
                cost: 0,
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
        plants::{PlantOutput, PowerPlant},
    };

    use super::RenewablePlant;

    fn get_forecasts() -> Vec<Forecast> {
        vec![
            Forecast {
                period: DeliveryPeriodId::from(1),
                value: ForecastValue {
                    value: 100,
                    deviation: 50,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(2),
                value: ForecastValue {
                    value: 500,
                    deviation: 100,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(3),
                value: ForecastValue {
                    value: 900,
                    deviation: 100,
                },
            },
        ]
    }

    #[test]
    fn test_renewable_plant() {
        let mut plant = RenewablePlant::new(1000, get_forecasts());

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
    fn test_renewable_forecasts_periods() {
        let mut plant = RenewablePlant::new(1000, get_forecasts());

        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(2), DeliveryPeriodId::from(3)]
        );

        plant.dispatch();
        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(3)]
        );

        plant.dispatch();
        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![]
        );

        plant.dispatch();
        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![]
        );
    }
}
