use serde::Serialize;

use crate::{
    forecast::{Forecast, map_value_to_forecast_level},
    game::delivery_period::DeliveryPeriodId,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

use super::timeseries::Timeseries;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    pub max_power: isize,
    pub output: PlantOutput,
}
pub struct RenewablePlant {
    max_power: isize,
    setpoints: Timeseries,
    period: DeliveryPeriodId,
    current_setpoint: isize,
    current_forecast: isize,
}

impl RenewablePlant {
    pub fn new(max_power: isize, setpoints: Timeseries) -> RenewablePlant {
        let period = DeliveryPeriodId::from(1);
        let current_setpoint = setpoints.value_at(period);
        let current_forecast = setpoints.value_at(period.next());

        RenewablePlant {
            current_setpoint,
            setpoints,
            max_power,
            current_forecast,
            period,
        }
    }
}

impl RenewablePlant {
    pub fn from_values(max_power: isize, values: Vec<isize>) -> RenewablePlant {
        let setpoints = Timeseries::from(&values[..]);
        RenewablePlant::new(max_power, setpoints)
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
        self.current_setpoint = self.setpoints.value_at(self.period);
        self.current_forecast = self.setpoints.value_at(self.period.next());
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
    fn get_forecast(&self) -> Option<Forecast> {
        Some(Forecast::Level(map_value_to_forecast_level(
            self.current_forecast,
            self.max_power,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        forecast::{Forecast, ForecastLevel},
        plants::{PlantOutput, PowerPlant},
    };

    use super::RenewablePlant;

    #[test]
    fn test_renewable_plant() {
        let mut plant = RenewablePlant::from_values(1000, vec![100, 500]);

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
    fn test_renewable_plant_forecasts() {
        let mut plant = RenewablePlant::from_values(1000, vec![100, 500, 900]);

        assert_eq!(
            plant.get_forecast(),
            Some(Forecast::Level(ForecastLevel::Medium))
        );

        plant.dispatch();

        assert_eq!(
            plant.get_forecast(),
            Some(Forecast::Level(ForecastLevel::High))
        );

        plant.dispatch();

        assert_eq!(
            plant.get_forecast(),
            Some(Forecast::Level(ForecastLevel::Low))
        );
    }
}
