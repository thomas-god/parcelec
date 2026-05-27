use crate::{
    forecast::{Forecast, ForecastValue},
    game::delivery_period::DeliveryPeriodId,
    utils::units::Power,
};

pub mod battery;
pub mod consumers;
pub mod gas_plant;
pub mod nuclear;
pub mod renewable;
mod variable;

pub struct ForecastsBasedPlant {
    period: usize,
    base_forecasts: Vec<ForecastValue>,
    forecasts_range: usize,

    setpoint: Power,
    forecasts: Vec<Forecast>,
}

impl ForecastsBasedPlant {
    pub fn new(base_forecasts: Vec<ForecastValue>, forecasts_range: usize) -> Self {
        let mut res = Self {
            period: 0,
            forecasts_range,
            base_forecasts,
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
        self.base_forecasts
            .get(self.period - 1)
            .unwrap_or(&ForecastValue::default())
            .forecast()
    }

    fn compute_forecasts(&self) -> Vec<Forecast> {
        let mut forecasts = vec![];
        for idx in 1..=self.forecasts_range {
            let forecast = if self.base_forecasts.is_empty() {
                ForecastValue::default()
            } else {
                *self
                    .base_forecasts
                    .get((self.period - 1 + idx) % self.base_forecasts.len())
                    .unwrap()
            };
            forecasts.push(Forecast {
                period: DeliveryPeriodId::from(self.period + idx),
                value: forecast,
            });
        }
        forecasts
    }
}

#[cfg(test)]
mod test_forecasts_based_plant {
    use super::*;

    fn test_forecasts() -> Vec<ForecastValue> {
        vec![
            ForecastValue {
                value: 100,
                deviation: 25,
            },
            ForecastValue {
                value: 500,
                deviation: 50,
            },
            ForecastValue {
                value: 1000,
                deviation: 50,
            },
            ForecastValue {
                value: 2000,
                deviation: 50,
            },
        ]
    }

    #[test]
    fn test_new_plant() {
        let forecasts_range = 2;
        let plant = ForecastsBasedPlant::new(test_forecasts(), forecasts_range);

        assert!((75..=125).contains(&plant.setpoint.into()));
        assert_eq!(plant.forecasts().len(), forecasts_range);
        assert_eq!(
            plant.forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(2), DeliveryPeriodId::from(3),]
        );
        assert!((450..=550).contains(&plant.forecasts.first().unwrap().value.value));
        assert!((950..=1050).contains(&plant.forecasts.get(1).unwrap().value.value));
    }

    #[test]
    fn test_plant_dispatch() {
        let forecasts_range = 2;
        let mut plant = ForecastsBasedPlant::new(test_forecasts(), forecasts_range);

        plant.dispatch();

        assert!((450..=550).contains(&plant.setpoint.into()));
        assert_eq!(plant.forecasts().len(), forecasts_range);
        assert_eq!(
            plant.forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(3), DeliveryPeriodId::from(4),]
        );
        assert!((950..=1050).contains(&plant.forecasts.first().unwrap().value.value));
        assert!((1950..=2050).contains(&plant.forecasts.get(1).unwrap().value.value));
    }

    #[test]
    fn test_plant_dispatch_overflow_base_forecasts_length() {
        let forecasts_range = 2;
        let mut plant = ForecastsBasedPlant::new(test_forecasts(), forecasts_range);

        plant.dispatch();
        plant.dispatch();

        assert!((950..=1050).contains(&plant.setpoint.into()));
        assert_eq!(plant.forecasts().len(), forecasts_range);
        assert_eq!(
            plant.forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(4), DeliveryPeriodId::from(5),]
        );
        assert!((1950..=2050).contains(&plant.forecasts.first().unwrap().value.value));
        assert!((75..=125).contains(&plant.forecasts.get(1).unwrap().value.value));
    }

    #[test]
    fn test_forecast_range_greater_than_base_forecasts_length() {
        let forecasts_range = test_forecasts().len() + 1;
        let mut plant = ForecastsBasedPlant::new(test_forecasts(), forecasts_range);

        assert_eq!(plant.forecasts().len(), forecasts_range);

        plant.dispatch();
        assert_eq!(plant.forecasts().len(), forecasts_range);
    }

    #[test]
    fn test_base_forecasts_empty() {
        let forecasts_range = 2;
        let mut plant = ForecastsBasedPlant::new(vec![], forecasts_range);

        assert_eq!(plant.setpoint(), Power::from(0));
        assert_eq!(plant.forecasts().len(), forecasts_range);
        assert_eq!(
            plant.forecasts().first().unwrap().value,
            ForecastValue::default()
        );

        plant.dispatch();
        assert_eq!(plant.forecasts().len(), forecasts_range);
        assert_eq!(plant.setpoint(), Power::from(0));
        assert_eq!(
            plant.forecasts().first().unwrap().value,
            ForecastValue::default()
        );
    }
}
