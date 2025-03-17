use serde::Serialize;

use crate::{
    constants, game::delivery_period::DeliveryPeriodId,
    plants::technologies::timeseries::Timeseries,
};

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

pub type Forecasts = Vec<Forecast>;
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
    pub min: isize,
    pub max: isize,
}

/// Generate a forecast from a given target value. If a [Clip] is given, the forecasted value will
/// be within this range. If target is out of the [Clip]'s range, this function will not fail and
/// will return the closest bound of the [Clip]. Forecasted value will be a multiple of
/// [constants::SETPOINT_BASE_VALUE].
pub fn forecast_value(target: isize, clip: &Option<Clip>) -> ForecastValue {
    let mut min =
        clip.as_ref()
            .map(|c| c.min)
            .unwrap_or(target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION))
            .max(target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION)) as i64;
    let mut max =
        clip.as_ref()
            .map(|c| c.max)
            .unwrap_or(target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION))
            .min(target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION)) as i64;
    let value = round_to_nearest(
        if min == max {
            min as isize
        } else {
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }
            i64_to_isize_saturating(rand::random_range(min..max))
        },
        constants::SETPOINT_BASE_VALUE,
    );
    ForecastValue {
        value,
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

fn round_to_nearest(value: isize, constant: isize) -> isize {
    ((value + constant - 1) / constant) * constant
}

pub fn forecast_from_timeseries(
    timeseries: &Timeseries,
    start_period: DeliveryPeriodId,
    clip: &Option<Clip>,
) -> Forecasts {
    let mut forecasts = Vec::new();

    let start_idx: usize = start_period.into();
    if start_idx > timeseries.len() {
        return forecasts;
    }

    let number_of_forecasts = timeseries.len() - start_idx + 1;
    let mut period = start_period;
    for _ in 0..number_of_forecasts {
        forecasts.push(Forecast {
            period,
            value: forecast_value(timeseries.value_at(period), clip),
        });
        period = period.next();
    }

    forecasts
}

#[cfg(test)]
mod tests {
    #![allow(unused_comparisons)]
    use super::{Clip, ForecastLevel, ForecastValue, forecast_value, map_value_to_forecast_level};
    use crate::{
        constants, forecast::forecast_from_timeseries, game::delivery_period::DeliveryPeriodId,
        plants::technologies::timeseries::Timeseries,
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
            } = forecast_value(target, &None);
            assert!(value >= target.saturating_sub_unsigned(constants::FORECAST_BASE_DEVIATION));
            assert!(value <= target.saturating_add_unsigned(constants::FORECAST_BASE_DEVIATION));
        }
    }

    #[test]
    fn test_forecast_value_within_clip() {
        let target = 50;
        let clip = Clip { min: 25, max: 75 };
        for _ in 0..0x1e4 {
            let ForecastValue {
                value,
                deviation: _,
            } = forecast_value(target, &Some(clip.clone()));
            assert!(value >= clip.min);
            assert!(value <= clip.max);
        }
    }

    #[test]
    fn test_forecast_value_should_not_fail_if_target_out_of_clip_range() {
        let clip = Some(Clip { min: 0, max: 500 });
        forecast_value(-100, &clip);
        forecast_value(0, &clip);
        forecast_value(500, &clip);
        forecast_value(1000, &clip);
    }

    #[test]
    fn test_forecast_from_timeseries() {
        let timeseries = Timeseries::from([100, 200, 300].as_slice());
        let clip = None;

        assert_eq!(
            forecast_from_timeseries(&timeseries, DeliveryPeriodId::from(1), &clip).len(),
            3
        );
        assert_eq!(
            forecast_from_timeseries(&timeseries, DeliveryPeriodId::from(2), &clip).len(),
            2
        );
        assert_eq!(
            forecast_from_timeseries(&timeseries, DeliveryPeriodId::from(3), &clip).len(),
            1
        );
        assert_eq!(
            forecast_from_timeseries(&timeseries, DeliveryPeriodId::from(4), &clip).len(),
            0
        );
    }

    #[test]
    fn test_forecast_from_timeseries_periods() {
        let timeseries = Timeseries::from([100, 200, 300].as_slice());
        let clip = None;

        let forecasts_periods: Vec<DeliveryPeriodId> =
            forecast_from_timeseries(&timeseries, DeliveryPeriodId::from(1), &clip)
                .iter()
                .map(|f| f.period)
                .collect();

        assert_eq!(
            forecasts_periods,
            vec![
                DeliveryPeriodId::from(1),
                DeliveryPeriodId::from(2),
                DeliveryPeriodId::from(3)
            ]
        );
    }
}
