use rand::random_range;
use serde::{Deserialize, Serialize};

use crate::{
    constants::SETPOINT_BASE_VALUE,
    forecast::{Forecast, ForecastValue, round_to_nearest},
    game::delivery_period::DeliveryPeriodId,
    utils::units::{NO_POWER, Power},
};

pub mod battery;
pub mod consumers;
pub mod gas_plant;
pub mod nuclear;
pub mod renewable;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PowerShape(Vec<f32>);

impl From<Vec<f32>> for PowerShape {
    fn from(value: Vec<f32>) -> Self {
        Self(value)
    }
}

impl PowerShape {
    pub fn random(len: usize) -> PowerShape {
        PowerShape((0..len).map(|_| random_range((0.)..=1.)).collect())
    }
}

pub struct ShapeBasedPlant {
    period: usize,
    shape: PowerShape,
    capacity: Power,
    forecasts_range: usize,

    setpoint: Power,
    forecasts: Vec<Forecast>,
}

impl ShapeBasedPlant {
    pub fn new(shape: PowerShape, capacity: Power, forecasts_range: usize) -> Self {
        let initial_forecasts = initial_forecasts(forecasts_range, &shape, capacity);
        let mut res = Self {
            period: 0,
            forecasts_range,
            shape,
            capacity,
            setpoint: Power::from(0),
            forecasts: initial_forecasts,
        };
        res.dispatch();
        res
    }

    pub fn setpoint(&self) -> Power {
        self.setpoint
    }

    pub fn forecasts(&self) -> &[Forecast] {
        &self.forecasts
    }

    pub fn dispatch(&mut self) {
        self.period += 1;

        self.setpoint = self.compute_setpoint(&self.forecasts);

        self.forecasts = self.compute_forecasts();
    }

    fn compute_setpoint(&self, previous_forecasts: &[Forecast]) -> Power {
        // First try to generate a setpoint compatible with the previous forecast for this period
        if let Some(previous_forecast) = previous_forecasts.first() {
            return previous_forecast.value.forecast_to_nearest();
        }

        // Else take a setpoint from the configured shape if any
        if self.shape.0.is_empty() {
            return NO_POWER;
        }
        let Some(value) = self.shape.0.get((self.period - 1) % self.shape.0.len()) else {
            return NO_POWER;
        };
        (self.capacity * value).round_to_nearest()
    }

    fn compute_forecasts(&self) -> Vec<Forecast> {
        let mut forecasts = vec![];
        let mut forecasted_period = DeliveryPeriodId::from(self.period).next();
        for idx in 1..=self.forecasts_range {
            // First try to generate a forecast from the previous one for this delivery period, else
            // take one from the configured shape if any
            let forecast = if let Some(previous_forecast) = self
                .forecasts
                .iter()
                .find(|f| f.period == forecasted_period)
            {
                previous_forecast.forecast_to_nearest()
            } else if self.shape.0.is_empty() {
                NO_POWER
            } else {
                (self.capacity
                    * self
                        .shape
                        .0
                        .get((self.period - 1 + idx) % self.shape.0.len())
                        .unwrap())
                .round_to_nearest()
            };
            forecasts.push(Forecast {
                period: DeliveryPeriodId::from(self.period + idx),
                value: ForecastValue {
                    value: forecast.as_i32(),
                    deviation: forecast_deviation(&forecast, idx),
                },
            });
            forecasted_period = forecasted_period.next();
        }
        forecasts
    }
}

fn initial_forecasts(forecast_range: usize, shape: &PowerShape, capacity: Power) -> Vec<Forecast> {
    let mut forecasts = vec![];
    for idx in 1..=forecast_range {
        let forecast = if shape.0.is_empty() {
            NO_POWER
        } else {
            (capacity * shape.0.get((idx - 1) % shape.0.len()).unwrap()).round_to_nearest()
        };
        forecasts.push(Forecast {
            period: DeliveryPeriodId::from(idx),
            value: ForecastValue {
                value: forecast.as_i32(),
                deviation: forecast_deviation(&forecast, idx),
            },
        });
    }
    forecasts
}

fn forecast_deviation(forecast: &Power, period: usize) -> u32 {
    let base_deviation = match period {
        1 => 0.08,
        2 => 0.12,
        3 => 0.15,
        _ => 0.20,
    };
    let deviation = (forecast.as_f32().abs() * base_deviation) as i32;
    round_to_nearest(deviation, SETPOINT_BASE_VALUE) as u32
}
#[cfg(test)]
mod test_shape_based_plant {

    use super::*;

    fn random_shape(len: usize) -> PowerShape {
        PowerShape((0..len).map(|_| random_range(0.0..=1.)).collect::<Vec<_>>())
    }

    // --------- Plant init, before any external dispatch

    #[test]
    fn test_init_plant_setpoint_within_range_of_shape_first_element_with_deviation() {
        let capacity = Power::from(1000);
        for _ in 0..0x1e4 {
            let shape = random_shape(3);
            let plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);
            let first = (capacity * shape.0.first().unwrap()).round_to_nearest();
            assert!(plant.setpoint >= (first * 0.92).round_to_nearest()); // - 8% deviation
            assert!(plant.setpoint <= (first * 1.08).round_to_nearest()); // + 8% deviation
        }
    }

    #[test]
    fn test_init_plant_setpoint_multiple_of_base_setpoint() {
        let capacity = Power::from(1000);
        for _ in 0..0x1e4 {
            let plant = ShapeBasedPlant::new(random_shape(3), capacity, 3);
            assert_eq!(plant.setpoint.as_i32().rem_euclid(SETPOINT_BASE_VALUE), 0);
        }
    }

    #[test]
    fn test_init_plant_setpoint_shape_is_empty() {
        let shape = PowerShape(vec![]);
        let capacity = Power::from(1000);

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        assert_eq!(plant.setpoint, Power::from(0))
    }

    #[test]
    fn test_init_plant_forecasts_length_equal_to_configured_range() {
        let capacity = Power::from(1000);
        let forecast_range = 3;
        for i in 0..=forecast_range + 1 {
            let plant = ShapeBasedPlant::new(random_shape(i), capacity, forecast_range);
            assert_eq!(plant.forecasts.len(), forecast_range);
        }
    }

    // --------- External dispatch of the plant

    #[test]
    fn test_plant_dispatch_setpoint_in_previous_forecast_range() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        for _ in 0..0x1e4 {
            let previous_forecast = *plant.forecasts.first().unwrap();
            plant.dispatch();
            let current_setpoint = plant.setpoint;

            assert_eq!(previous_forecast.period, plant.period.into());
            assert!(previous_forecast.value.lower_range() <= current_setpoint.as_i32());
            assert!(previous_forecast.value.upper_range() >= current_setpoint.as_i32());
        }
    }

    #[test]
    fn test_plant_dispatch_current_forecast_overlaps_with_previous_forecast() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(10000);

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        for _ in 0..0x1e4 {
            let previous_forecast = *plant.forecasts.get(1).unwrap();
            plant.dispatch();
            let current_forecast = plant.forecasts.first().unwrap();

            assert_eq!(previous_forecast.period, current_forecast.period);
            assert!(current_forecast.value.lower_range() <= previous_forecast.value.upper_range());
            assert!(previous_forecast.value.lower_range() <= current_forecast.value.upper_range());
        }
    }

    #[test]
    fn test_plant_forecast_empty_shape_has_proper_length() {
        let shape = PowerShape(vec![]);
        let capacity = Power::from(1000);
        let forecast_range = 3;

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert_eq!(plant.forecasts.len(), forecast_range);
        assert!(
            plant
                .forecasts
                .iter()
                .all(|f| f.value.value == NO_POWER.as_i32())
        );
    }

    #[test]
    fn test_forecast_deviation_depends_on_distance_in_the_future() {
        let shape = PowerShape(vec![0.5]);
        let capacity = Power::from(1000);
        let forecast_range = 5;

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert!(
            plant.forecasts.first().unwrap().value.deviation
                <= plant.forecasts.last().unwrap().value.deviation
        );
    }

    #[test]
    fn test_forecast_deviation_with_negative_power_values() {
        let shape = PowerShape(vec![0.5]);
        let capacity = Power::from(-1000);
        let forecast_range = 5;

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert!(
            plant.forecasts.first().unwrap().value.deviation
                <= plant.forecasts.last().unwrap().value.deviation
        );
    }

    #[test]
    fn test_forecast_periods() {
        let shape = PowerShape(vec![0.5]);
        let capacity = Power::from(1000);
        let forecast_range = 3;

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert_eq!(
            plant.forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![
                DeliveryPeriodId::from(2),
                DeliveryPeriodId::from(3),
                DeliveryPeriodId::from(4),
            ]
        );

        plant.dispatch();
        assert_eq!(
            plant.forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![
                DeliveryPeriodId::from(3),
                DeliveryPeriodId::from(4),
                DeliveryPeriodId::from(5),
            ]
        );
    }
}
