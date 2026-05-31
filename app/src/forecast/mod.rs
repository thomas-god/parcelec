use serde::{Deserialize, Serialize};

use crate::{
    constants::{self, SETPOINT_BASE_VALUE},
    game::delivery_period::DeliveryPeriodId,
    utils::units::Power,
};

pub type Forecasts = Vec<Forecast>;
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Forecast {
    pub period: DeliveryPeriodId,
    pub value: ForecastValue,
}

impl Forecast {
    pub fn forecast_to_nearest(&self) -> Power {
        self.value.forecast_to_nearest()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct ForecastValue {
    pub value: i32,
    pub deviation: u32,
}

impl ForecastValue {
    pub fn forecast_to_nearest(&self) -> Power {
        Power::from(forecast_in_range(self.lower_range(), self.upper_range())).round_to_nearest()
    }

    pub fn lower_range(&self) -> i32 {
        round_to_nearest(
            self.value.saturating_sub_unsigned(self.deviation),
            SETPOINT_BASE_VALUE,
        )
    }

    pub fn upper_range(&self) -> i32 {
        round_to_nearest(
            self.value.saturating_add_unsigned(self.deviation),
            SETPOINT_BASE_VALUE,
        )
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

pub fn round_to_nearest(value: i32, constant: i32) -> i32 {
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

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use std::ops::Rem;

    use super::forecast_in_range;
    use crate::{constants, forecast::round_to_nearest};

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
    fn test_forecast_value_forecast_within_range() {
        let fv = super::ForecastValue {
            value: 500,
            deviation: 100,
        };

        for _ in 0..0x1e4 {
            let power: i32 = fv.forecast_to_nearest().into();
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

        let power: i32 = fv.forecast_to_nearest().into();
        assert_eq!(power, 300);
    }
}
