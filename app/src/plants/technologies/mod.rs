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
        let mut res = Self {
            period: 0,
            forecasts_range,
            shape,
            capacity,
            setpoint: Power::from(0),
            forecasts: vec![],
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

        self.setpoint = self.compute_setpoint();

        self.forecasts = self.compute_forecasts();
    }

    fn compute_setpoint(&self) -> Power {
        if self.shape.0.is_empty() {
            return NO_POWER;
        }
        let Some(value) = self.shape.0.get((self.period - 1) % self.shape.0.len()) else {
            return NO_POWER;
        };
        (self.capacity * value).round_to_nearest()
    }

    fn compute_forecasts(&self) -> Vec<Forecast> {
        // TODO: compute next forecast from previous forecast for this period ?
        let mut forecasts = vec![];
        for idx in 1..=self.forecasts_range {
            let forecast = if self.shape.0.is_empty() {
                NO_POWER
            } else {
                self.capacity
                    * self
                        .shape
                        .0
                        .get((self.period - 1 + idx) % self.shape.0.len())
                        .unwrap()
            };
            forecasts.push(Forecast {
                period: DeliveryPeriodId::from(self.period + idx),
                value: ForecastValue {
                    value: forecast.as_i32(),
                    deviation: self.deviation(&forecast, idx),
                },
            });
        }
        forecasts
    }

    fn deviation(&self, forecast: &Power, period: usize) -> u32 {
        let base_deviation = match period {
            1 => 0.08,
            2 => 0.12,
            3 => 0.15,
            _ => 0.20,
        };
        let deviation = (forecast.as_f32().abs() * base_deviation) as i32;
        round_to_nearest(deviation, SETPOINT_BASE_VALUE) as u32
    }
}

#[cfg(test)]
mod test_shape_based_plant {

    use super::*;

    #[test]
    fn test_init_plant_setpoint() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        assert_eq!(plant.setpoint, capacity * shape.0.first().unwrap())
    }

    #[test]
    fn test_plant_setpoint_multiple_of_base_setpoint() {
        let capacity = Power::from(1000);

        let plant = ShapeBasedPlant::new(PowerShape(vec![0.333333]), capacity, 3);
        assert_eq!(plant.setpoint, Power::from(325));

        let plant = ShapeBasedPlant::new(PowerShape(vec![0.999]), capacity, 3);
        assert_eq!(plant.setpoint, Power::from(1000))
    }

    #[test]
    fn test_init_plant_setpoint_shape_is_empty() {
        let shape = PowerShape(vec![]);
        let capacity = Power::from(1000);

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        assert_eq!(plant.setpoint, Power::from(0))
    }

    #[test]
    fn test_plant_dispatch_update_setpoint() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        plant.dispatch();
        assert_eq!(plant.setpoint, capacity * shape.0.get(1).unwrap())
    }

    #[test]
    fn test_plant_dispatch_setpoint_loop_over_shape() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, 3);

        for _ in 0..shape.0.len() {
            plant.dispatch();
        }
        assert_eq!(plant.setpoint, capacity * shape.0.first().unwrap())
    }

    #[test]
    fn test_plant_init_forecast_in_forecast_range() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);
        let forecast_range = 2;

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert_eq!(plant.forecasts.len(), forecast_range);
        assert_eq!(
            plant
                .forecasts
                .iter()
                .map(|f| f.value.value)
                .collect::<Vec<_>>(),
            shape.0[1..=forecast_range]
                .iter()
                .map(|v| (capacity * v).as_i32())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_plant_dispatch_forecast_loop() {
        let shape = PowerShape(vec![0.1, 0.25, 0.5, 1.]);
        let capacity = Power::from(1000);
        let forecast_range = 3;

        let mut plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);
        plant.dispatch();

        assert_eq!(plant.forecasts.len(), forecast_range);
        assert_eq!(
            plant
                .forecasts
                .iter()
                .map(|f| f.value.value)
                .collect::<Vec<_>>(),
            vec![500, 1000, 100] // vec[shape[2], shape[3], shape[0]]
        );
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

        assert_eq!(
            plant
                .forecasts
                .iter()
                .map(|f| f.value.deviation as i32)
                .collect::<Vec<_>>(),
            vec![50, 50, 75, 100, 100]
        );
    }

    #[test]
    fn test_forecast_deviation_with_negative_power_values() {
        let shape = PowerShape(vec![0.5]);
        let capacity = Power::from(-1000);
        let forecast_range = 5;

        let plant = ShapeBasedPlant::new(shape.clone(), capacity, forecast_range);

        assert_eq!(
            plant
                .forecasts
                .iter()
                .map(|f| f.value.deviation as i32)
                .collect::<Vec<_>>(),
            vec![50, 50, 75, 100, 100]
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
