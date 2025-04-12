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

impl ForecastValue {
    pub fn lower_range(&self) -> isize {
        self.value.saturating_sub_unsigned(self.deviation)
    }

    pub fn upper_range(&self) -> isize {
        self.value.saturating_add_unsigned(self.deviation)
    }

    pub fn included_in(&self, other: &Self) -> bool {
        self.lower_range() >= other.lower_range() && self.upper_range() <= other.upper_range()
    }
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub min: isize,
    pub max: isize,
}

pub fn forecast_in_range(min: isize, max: isize) -> isize {
    if min == max {
        return round_to_nearest(min, constants::SETPOINT_BASE_VALUE);
    }

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
}
