use derive_more::Display;
use rand::random_range;
use serde::{Deserialize, Serialize};

use crate::{constants, game::delivery_period::DeliveryPeriodId, utils::units::Power};

pub type Forecasts = Vec<Forecast>;
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Forecast {
    pub period: DeliveryPeriodId,
    pub value: ForecastValue,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct ForecastValue {
    pub value: i32,
    pub deviation: u32,
}

impl ForecastValue {
    pub fn forecast(&self) -> Power {
        Power::from(forecast_in_range(self.lower_range(), self.upper_range()))
    }

    pub fn lower_range(&self) -> i32 {
        self.value.saturating_sub_unsigned(self.deviation)
    }

    pub fn upper_range(&self) -> i32 {
        self.value.saturating_add_unsigned(self.deviation)
    }

    pub fn included_in(&self, other: &Self) -> bool {
        self.lower_range() >= other.lower_range() && self.upper_range() <= other.upper_range()
    }
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub min: i32,
    pub max: i32,
}

pub fn forecast_in_range(min: i32, max: i32) -> i32 {
    if min == max {
        return round_to_nearest(min, constants::SETPOINT_BASE_VALUE);
    }

    round_to_nearest(
        i64_to_i32_saturating(rand::random_range((min as i64)..(max as i64))),
        constants::SETPOINT_BASE_VALUE,
    )
}

fn i64_to_i32_saturating(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < i32::MIN as i64 {
        i32::MIN
    } else {
        value as i32
    }
}

fn round_to_nearest(value: i32, constant: i32) -> i32 {
    let rem = value % constant;
    let half = constant / 2;

    if rem.abs() <= half {
        value - rem // Round down
    } else if rem > 0 {
        value + (constant - rem) // Round up for positive remainder
    } else {
        value - (constant + rem) // Round up for negative remainder
    }
}

/// Forecast value within [0, 1], usefull to generate forecast shapes without knowing the actual
/// capacity
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NormalizedForecast {
    pub period: DeliveryPeriodId,
    pub value: NormalizedForecastValue,
}

impl NormalizedForecast {
    pub fn as_forecast(&self, capacity: i32) -> Forecast {
        Forecast {
            period: self.period,
            value: self.value.as_forecast(capacity),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct NormalizedForecastValue {
    value: f64,
    deviation: f64,
}

#[derive(Debug, Display, Clone, thiserror::Error, PartialEq)]
pub enum CreateNormalizedForecastError {
    ValueOutOfRange,
    DeviationOutOfRange,
}

impl NormalizedForecastValue {
    pub fn try_new(value: f64, deviation: f64) -> Result<Self, CreateNormalizedForecastError> {
        if !(0. ..=1.).contains(&value) {
            return Err(CreateNormalizedForecastError::ValueOutOfRange);
        }
        if !(0. ..=1.).contains(&deviation) {
            return Err(CreateNormalizedForecastError::DeviationOutOfRange);
        }

        Ok(Self { value, deviation })
    }

    pub fn as_forecast(&self, capacity: i32) -> ForecastValue {
        let value = self.value * f64::from(capacity);
        ForecastValue {
            value: round_to_nearest(value as i32, constants::SETPOINT_BASE_VALUE),
            deviation: round_to_nearest(
                (value * self.deviation) as i32,
                constants::SETPOINT_BASE_VALUE,
            ) as u32,
        }
    }
}

pub fn generate_random_forecasts_shape(len: usize) -> Vec<NormalizedForecastValue> {
    (0..len)
        .map(|_| {
            NormalizedForecastValue::try_new(random_range((0.)..=1.), random_range((0.)..=0.1))
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use std::ops::Rem;

    use super::forecast_in_range;
    use crate::{
        constants,
        forecast::{CreateNormalizedForecastError, NormalizedForecastValue, round_to_nearest},
    };

    #[test]
    fn test_round_to_nearest() {
        assert_eq!(round_to_nearest(100, 25), 100);
        assert_eq!(round_to_nearest(112, 25), 100);
        assert_eq!(round_to_nearest(113, 25), 125);
        assert_eq!(round_to_nearest(124, 25), 125);
    }

    #[test]
    fn test_forecast_within_range_actually_in_range() {
        let min = -100;
        let max = 1000;

        for _ in 0..0x1e4 {
            let value = forecast_in_range(min, max);
            assert!(value >= min);
            assert!(value <= max);
        }
    }

    #[test]
    fn test_forecast_within_range_multiple_of_base_setpoint() {
        let min = -100;
        let max = 1000;

        for _ in 0..0x1e4 {
            let value = forecast_in_range(min, max);
            assert_eq!(value.rem(constants::SETPOINT_BASE_VALUE), 0);
        }
    }

    #[test]
    fn test_forecast_within_empty_range() {
        let min = 100;
        let max = 100;

        assert_eq!(forecast_in_range(min, max), 100);
    }

    #[test]
    fn test_forecast_within_empty_range_not_centered_on_forecast_step() {
        let min = 105;
        let max = 105;

        assert_eq!(forecast_in_range(min, max), 100);
    }

    #[test]
    fn test_create_normalized_forecast() {
        assert_eq!(
            NormalizedForecastValue::try_new(-f64::EPSILON, 0.),
            Err(CreateNormalizedForecastError::ValueOutOfRange)
        );
        assert_eq!(
            NormalizedForecastValue::try_new(1. + f64::EPSILON, 0.),
            Err(CreateNormalizedForecastError::ValueOutOfRange)
        );

        assert_eq!(
            NormalizedForecastValue::try_new(0., -f64::EPSILON),
            Err(CreateNormalizedForecastError::DeviationOutOfRange)
        );
        assert_eq!(
            NormalizedForecastValue::try_new(0., 1. + f64::EPSILON),
            Err(CreateNormalizedForecastError::DeviationOutOfRange)
        );
    }

    #[test]
    fn test_normalized_forecast_to_forecast() {
        let normalized_forecast = NormalizedForecastValue::try_new(0.5, 0.1).unwrap();
        let capacity = 1000;

        let forecast = normalized_forecast.as_forecast(capacity);

        assert_eq!(forecast.value, 500);
        assert_eq!(forecast.deviation, 50);
    }

    #[test]
    fn test_normalized_forecast_to_forecast_multiple_of_step() {
        let normalized_forecast = NormalizedForecastValue::try_new(0.333, 0.1).unwrap();
        let capacity = 1000;

        let forecast = normalized_forecast.as_forecast(capacity);

        assert_eq!(forecast.value, 325);
        assert_eq!(forecast.deviation, 25);
    }

    #[test]
    fn test_forecast_value_forecast_within_range() {
        let fv = super::ForecastValue {
            value: 500,
            deviation: 100,
        };

        for _ in 0..0x1e4 {
            let power: i32 = fv.forecast().into();
            assert!(power >= fv.lower_range());
            assert!(power <= fv.upper_range());
            assert_eq!(power.rem(constants::SETPOINT_BASE_VALUE), 0);
        }
    }

    #[test]
    fn test_forecast_value_forecast_zero_deviation() {
        let fv = super::ForecastValue {
            value: 300,
            deviation: 0,
        };

        let power: i32 = fv.forecast().into();
        assert_eq!(power, 300);
    }
}
