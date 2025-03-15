use serde::Serialize;

use crate::constants;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "type")]
pub enum Forecast {
    Level(ForecastLevel),
    Value(ForecastValue),
}

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
pub struct ForecastValue {
    value: usize,
    deviation: usize,
}

pub fn forecast_value(target: usize, max: Option<usize>) -> ForecastValue {
    let max = max
        .map(|m| m.min(target + constants::FORECAST_BASE_DEVIATION))
        .unwrap_or(target + constants::FORECAST_BASE_DEVIATION);
    let range = (target.saturating_sub(constants::FORECAST_BASE_DEVIATION))..(max);
    ForecastValue {
        value: rand::random_range(range),
        deviation: constants::FORECAST_BASE_DEVIATION,
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use crate::{
        constants,
        forecast::{ForecastLevel, ForecastValue, forecast_value, map_value_to_forecast_level},
    };

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
            assert!(value >= target - constants::FORECAST_BASE_DEVIATION);
            assert!(value <= target + constants::FORECAST_BASE_DEVIATION);
        }
    }

    #[test]
    fn test_forecast_value_positive() {
        let target = 50;
        for _ in 0..0x1e4 {
            let ForecastValue {
                value,
                deviation: _,
            } = forecast_value(target, None);
            assert!(value >= 0);
        }
    }

    #[test]
    fn test_forecast_value_saturate_with_max() {
        let target = 500;
        for _ in 0..0x1e4 {
            let ForecastValue {
                value,
                deviation: _,
            } = forecast_value(target, Some(550));
            assert!(value <= 550);
        }
    }
}
