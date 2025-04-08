use serde::Serialize;

use crate::{
    constants, game::delivery_period::DeliveryPeriodId,
    plants::technologies::timeseries::Timeseries,
};

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
    use std::ops::Rem;

    use super::{Clip, ForecastValue, forecast_in_range, forecast_value};
    use crate::{
        constants, forecast::forecast_from_timeseries, game::delivery_period::DeliveryPeriodId,
        plants::technologies::timeseries::Timeseries,
    };

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
