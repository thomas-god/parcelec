use serde::Serialize;

use crate::{constants, game::delivery_period::DeliveryPeriodId};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "level")]
pub enum ForecastLevel {
    Low,
    Medium,
    High,
}

/// Classify a value to a [ForecastLevel].
pub fn map_value_to_forecast_level(value: isize, max: isize) -> ForecastLevel {
    if max > 0 {
        return match value {
            value if value < max / 3 => ForecastLevel::Low,
            value if value > 2 * max / 3 => ForecastLevel::High,
            _ => ForecastLevel::Medium,
        };
    }
    match value {
        value if value > max / 3 => ForecastLevel::Low,
        value if value < 2 * max / 3 => ForecastLevel::High,
        _ => ForecastLevel::Medium,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct Forecast {
    pub period: DeliveryPeriodId,
    pub value: ForecastValue,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub struct ForecastValue {
    pub value: isize,
    pub deviation: usize,
}

#[derive(Debug, Clone)]
pub struct Clip {
    min: isize,
    max: isize,
}

pub fn forecast_value(target: isize, clip: Option<Clip>) -> ForecastValue {
    let min =
        clip.as_ref()
            .map(|c| c.min)
            .unwrap_or(target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION))
            .max(target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION)) as i64;
    let max =
        clip.map(|c| c.max)
            .unwrap_or(target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION))
            .min(target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION)) as i64;
    ForecastValue {
        value: i64_to_isize_saturating(rand::random_range(min..max)),
        deviation: constants::FORECAST_BASE_DEVIATION,
    }
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

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use super::{Clip, ForecastLevel, ForecastValue, forecast_value, map_value_to_forecast_level};
    use crate::constants;

    #[test]
    fn test_map_forecast_level_default_min() {
        assert_eq!(map_value_to_forecast_level(-1, 1000), ForecastLevel::Low);
        assert_eq!(map_value_to_forecast_level(0, 1000), ForecastLevel::Low);
        assert_eq!(map_value_to_forecast_level(332, 1000), ForecastLevel::Low);
        assert_eq!(
            map_value_to_forecast_level(333, 1000),
            ForecastLevel::Medium
        );
        assert_eq!(
            map_value_to_forecast_level(666, 1000),
            ForecastLevel::Medium
        );
        assert_eq!(map_value_to_forecast_level(667, 1000), ForecastLevel::High);
        assert_eq!(map_value_to_forecast_level(1000, 1000), ForecastLevel::High);
        assert_eq!(map_value_to_forecast_level(1001, 1000), ForecastLevel::High);
    }

    #[test]
    fn test_map_forecast_level_with_negative_min() {
        assert_eq!(map_value_to_forecast_level(1, -1000), ForecastLevel::Low);
        assert_eq!(map_value_to_forecast_level(0, -1000), ForecastLevel::Low);
        assert_eq!(map_value_to_forecast_level(-332, -1000), ForecastLevel::Low);
        assert_eq!(
            map_value_to_forecast_level(-333, -1000),
            ForecastLevel::Medium
        );
        assert_eq!(
            map_value_to_forecast_level(-666, -1000),
            ForecastLevel::Medium
        );
        assert_eq!(
            map_value_to_forecast_level(-667, -1000),
            ForecastLevel::High
        );
        assert_eq!(
            map_value_to_forecast_level(-1000, -1000),
            ForecastLevel::High
        );
        assert_eq!(
            map_value_to_forecast_level(-1001, -1000),
            ForecastLevel::High
        );
    }

    #[test]
    fn test_forecast_value() {
        // Should be within [target - deviation, target + deviation]
        let target = 1000;
        for _ in 0..0x1e4 {
            let ForecastValue {
                value,
                deviation: _,
            } = forecast_value(target, None);
            assert!(value >= target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION));
            assert!(value <= target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION));
        }
    }

    #[test]
    fn test_forecast_value_within_clip() {
        let target = 50;
        let clip = Clip { min: 30, max: 60 };
        for _ in 0..0x1e4 {
            let ForecastValue {
                value,
                deviation: _,
            } = forecast_value(target, Some(clip.clone()));
            assert!(value >= clip.min);
            assert!(value <= clip.max);
        }
    }
}
