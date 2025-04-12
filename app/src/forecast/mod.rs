use serde::{Deserialize, Serialize};

use crate::{constants, game::delivery_period::DeliveryPeriodId};

pub type Forecasts = Vec<Forecast>;
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Forecast {
    pub period: DeliveryPeriodId,
    pub value: ForecastValue,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ForecastValue {
    pub value: isize,
    pub deviation: usize,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub min: isize,
    pub max: isize,
}

pub fn forecast_in_range(min: isize, max: isize) -> isize {
    round_to_nearest(
        i64_to_isize_saturating(rand::random_range((min as i64)..(max as i64))),
        constants::SETPOINT_BASE_VALUE,
    )
}

fn i64_to_isize_saturating(value: i64) -> isize {
    if value > isize::MAX as i64 {
        isize::MAX
    } else if value < isize::MIN as i64 {
        isize::MIN
    } else {
        value as isize
    }
}

fn round_to_nearest(value: isize, constant: isize) -> isize {
    ((value + constant - 1) / constant) * constant
}

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use std::ops::Rem;

    use super::forecast_in_range;
    use crate::constants;

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
}
