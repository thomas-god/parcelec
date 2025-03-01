use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
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

#[cfg(test)]
mod tests {
    use crate::forecast::{ForecastLevel, map_value_to_forecast_level};

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
}
