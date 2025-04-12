use std::collections::HashMap;

use crate::{
    constants,
    forecast::{Forecast, ForecastValue, forecast_in_range},
    game::delivery_period::DeliveryPeriodId,
};

pub type TargetPeriod = DeliveryPeriodId;
pub type CurrentPeriod = DeliveryPeriodId;

#[derive(Debug, Clone)]
pub enum State {
    Forecast(ForecastValue),
    Setpoint(isize),
}

type PlantState = HashMap<TargetPeriod, HashMap<CurrentPeriod, State>>;

/// [VariablePlant] handles the forecasts and setpoints generation for variable sources like
/// renewable plants and consumers. Target period refers to the [DeliveryPeriodId] the forecast or
/// the setpoint apply to, in contrast to the current period.
///
/// Forecasts and setpoints are generated during the [VariablePlant] instanciation.
///
/// When requesting the plant state for a given target [DeliveryPeriodId], it will return a
/// [State::Forecast] if the current period is before the target period, as [State::Setpoint] if the
/// current period equals the target period, and [None] if the current period is past the target
/// period.
///
/// A forecast range will always be included in the range of the previous forecast. A setpoint will
/// always be included in the range of the last forecast for this target period.
pub struct VariablePlant {
    state: PlantState,
}

impl VariablePlant {
    pub fn new(forecasts: Vec<Forecast>) -> VariablePlant {
        let mut state = HashMap::new();
        for forecast in forecasts.iter() {
            let target_period = forecast.period;
            let mut target_period_state = HashMap::new();
            // For first period, we take the initial forecast
            target_period_state.insert(DeliveryPeriodId::from(1), State::Forecast(forecast.value));

            // For subsequent periods, generate forecast up to the target period (excluded)
            let mut previous_forecast = forecast.value;
            for idx in 2..target_period.into() {
                let current_forecast = generate_forecast(
                    DeliveryPeriodId::from(idx),
                    target_period,
                    previous_forecast,
                );
                target_period_state.insert(
                    DeliveryPeriodId::from(idx),
                    State::Forecast(current_forecast),
                );
                previous_forecast = current_forecast;
            }

            // Generate a setpoint for target period, based on last forecast
            let last_forecast = match target_period_state.get(&target_period.previous()) {
                Some(State::Forecast(f)) => *f,
                _ => forecast.value,
            };
            target_period_state.insert(
                target_period,
                State::Setpoint(forecast_in_range(
                    last_forecast
                        .value
                        .saturating_sub_unsigned(last_forecast.deviation),
                    last_forecast
                        .value
                        .saturating_add_unsigned(last_forecast.deviation),
                )),
            );
            state.insert(target_period, target_period_state);
        }
        VariablePlant { state }
    }

    pub fn get_state(
        &self,
        target_period: DeliveryPeriodId,
        current_period: DeliveryPeriodId,
    ) -> Option<State> {
        self.state
            .get(&target_period)
            .and_then(|state| state.get(&current_period).cloned())
    }

    pub fn get_setpoint(&self, target_period: DeliveryPeriodId) -> Option<isize> {
        match self.get_state(target_period, target_period) {
            Some(State::Setpoint(setpoint)) => Some(setpoint),
            _ => None,
        }
    }

    pub fn get_forecasts(&self, current_period: DeliveryPeriodId) -> Vec<Forecast> {
        let mut forecasts = vec![];
        for (period, state) in self.state.iter() {
            if let Some(State::Forecast(forecast)) = state.get(&current_period) {
                forecasts.push(Forecast {
                    period: *period,
                    value: *forecast,
                });
            }
        }
        forecasts.sort_by(|a, b| a.period.cmp(&b.period));
        forecasts
    }
}

fn generate_forecast(
    current_period: DeliveryPeriodId,
    target_period: DeliveryPeriodId,
    previous_forecast: ForecastValue,
) -> ForecastValue {
    let distance_to_target: usize =
        usize::from(target_period).saturating_sub(usize::from(current_period));

    let deviation = if distance_to_target <= 4 {
        distance_to_target * constants::SETPOINT_BASE_VALUE as usize
    } else {
        4 * constants::SETPOINT_BASE_VALUE as usize
    };

    let deviation_variation_in_steps = previous_forecast.deviation.saturating_sub(deviation)
        / constants::SETPOINT_BASE_VALUE as usize;

    if deviation_variation_in_steps == 0 {
        return previous_forecast;
    }

    let new_value_offset = rand::random_range(
        (-(deviation_variation_in_steps as i64))..=(deviation_variation_in_steps as i64),
    ) as isize;

    ForecastValue {
        value: previous_forecast.value + new_value_offset * constants::SETPOINT_BASE_VALUE,
        deviation,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        forecast::{Forecast, ForecastValue},
        game::delivery_period::DeliveryPeriodId,
    };

    use super::{State, VariablePlant};

    fn get_forecasts() -> Vec<Forecast> {
        vec![
            Forecast {
                period: DeliveryPeriodId::from(1),
                value: ForecastValue {
                    value: 500,
                    deviation: 100,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(2),
                value: ForecastValue {
                    value: 300,
                    deviation: 50,
                },
            },
            Forecast {
                period: DeliveryPeriodId::from(3),
                value: ForecastValue {
                    value: 800,
                    deviation: 150,
                },
            },
        ]
    }

    #[test]
    fn test_first_period_no_forecast() {
        let forecasts = get_forecasts();
        let plant = VariablePlant::new(forecasts);

        let Some(State::Setpoint(_)) =
            plant.get_state(DeliveryPeriodId::from(1), DeliveryPeriodId::from(1))
        else {
            unreachable!("Should be a Some(State::Setpoint)")
        };
    }

    #[test]
    fn test_when_current_period_less_than_target_period_state_is_a_forecast() {
        let forecasts = get_forecasts();
        let plant = VariablePlant::new(forecasts);

        let target = DeliveryPeriodId::from(2);
        let current = DeliveryPeriodId::from(1);
        let Some(State::Forecast(_)) = plant.get_state(target, current) else {
            unreachable!("Should be a Some(State::Forecast)")
        };

        let target = DeliveryPeriodId::from(3);
        for i in 1..=2 {
            let current = DeliveryPeriodId::from(i);
            let Some(State::Forecast(_)) = plant.get_state(target, current) else {
                unreachable!("Should be a Some(State::Forecast)")
            };
        }
    }

    #[test]
    fn test_when_current_period_equals_target_period_state_is_a_setpoint() {
        let forecasts = get_forecasts();
        let forecasts_len = forecasts.len();
        let plant = VariablePlant::new(forecasts);

        for idx in 1..=forecasts_len {
            let target = DeliveryPeriodId::from(idx);
            let current = DeliveryPeriodId::from(idx);
            let Some(State::Setpoint(_)) = plant.get_state(target, current) else {
                unreachable!("Should be a Some(State::Setpoint)")
            };
        }
    }

    #[test]
    fn test_when_current_period_greater_than_target_period_state_is_none() {
        let forecasts = get_forecasts();
        let forecasts_len = forecasts.len();
        let plant = VariablePlant::new(forecasts);

        for idx in 1..=forecasts_len {
            let target = DeliveryPeriodId::from(idx);
            let current = DeliveryPeriodId::from(idx + 1);
            let None = plant.get_state(target, current) else {
                unreachable!("Should be a None")
            };
        }
    }

    #[test]
    fn test_for_a_given_target_period_forecast_is_included_in_previous_period_forecast() {
        for _ in 0..0x1e4 {
            let forecasts = get_forecasts();
            let plant = VariablePlant::new(forecasts);

            let Some(State::Forecast(previous_forecast)) =
                plant.get_state(DeliveryPeriodId::from(3), DeliveryPeriodId::from(1))
            else {
                unreachable!("Should be a Some(State::Forecast)")
            };
            let Some(State::Forecast(next_forecast)) =
                plant.get_state(DeliveryPeriodId::from(3), DeliveryPeriodId::from(2))
            else {
                unreachable!("Should be a Some(State::Forecast)")
            };

            assert!(
                next_forecast
                    .value
                    .saturating_sub_unsigned(next_forecast.deviation)
                    >= previous_forecast
                        .value
                        .saturating_sub_unsigned(previous_forecast.deviation)
            );
            assert!(
                next_forecast
                    .value
                    .saturating_add_unsigned(next_forecast.deviation)
                    <= previous_forecast
                        .value
                        .saturating_add_unsigned(previous_forecast.deviation)
            );
        }
    }

    #[test]
    fn test_setpoint_included_in_last_forecast() {
        for _ in 0..0x1e4 {
            let forecasts = get_forecasts();
            let plant = VariablePlant::new(forecasts);

            let Some(State::Forecast(last_forecast)) =
                plant.get_state(DeliveryPeriodId::from(3), DeliveryPeriodId::from(2))
            else {
                unreachable!("Should be a Some(State::Forecast)")
            };
            let Some(State::Setpoint(setpoint)) =
                plant.get_state(DeliveryPeriodId::from(3), DeliveryPeriodId::from(3))
            else {
                unreachable!("Should be a Some(State::Setpoint)")
            };

            assert!(
                setpoint
                    <= last_forecast
                        .value
                        .saturating_add_unsigned(last_forecast.deviation)
            );
            assert!(
                setpoint
                    >= last_forecast
                        .value
                        .saturating_sub_unsigned(last_forecast.deviation)
            );
        }
    }

    #[test]
    fn test_get_setpoint() {
        let forecasts = get_forecasts();
        let forecasts_len = forecasts.len();
        let plant = VariablePlant::new(forecasts);

        for idx in 1..=forecasts_len {
            let Some(_) = plant.get_setpoint(DeliveryPeriodId::from(idx)) else {
                unreachable!("Should not be None")
            };
        }
        let None = plant.get_setpoint(DeliveryPeriodId::from(forecasts_len + 1)) else {
            unreachable!("Should be a None")
        };
    }

    #[test]
    fn test_get_forecasts() {
        let forecasts = get_forecasts();
        let plant = VariablePlant::new(forecasts);

        let forecasts = plant.get_forecasts(DeliveryPeriodId::from(1));
        assert_eq!(forecasts.len(), 2);

        let forecasts = plant.get_forecasts(DeliveryPeriodId::from(2));
        assert_eq!(forecasts.len(), 1);

        let forecasts = plant.get_forecasts(DeliveryPeriodId::from(3));
        assert_eq!(forecasts.len(), 0);

        let forecasts = plant.get_forecasts(DeliveryPeriodId::from(4));
        assert_eq!(forecasts.len(), 0);
    }

    #[test]
    fn test_get_forecasts_in_order() {
        for _ in 0..100 {
            let forecasts = get_forecasts();
            let plant = VariablePlant::new(forecasts);

            let forecasts = plant.get_forecasts(DeliveryPeriodId::from(1));
            assert_eq!(forecasts.len(), 2);
            assert_eq!(forecasts.get(0).unwrap().period, DeliveryPeriodId::from(2));
            assert_eq!(forecasts.get(1).unwrap().period, DeliveryPeriodId::from(3));
        }
    }
}

#[cfg(test)]
mod tests_generate_forecast {
    use crate::{forecast::ForecastValue, game::delivery_period::DeliveryPeriodId};

    use super::generate_forecast;

    #[test]
    fn test_generated_forecast_within_previous_forecast_range() {
        let current_period = DeliveryPeriodId::from(1);
        let target_period = DeliveryPeriodId::from(2);
        let previous_forecast = ForecastValue {
            value: 500,
            deviation: 50,
        };

        let forecast = generate_forecast(current_period, target_period, previous_forecast);

        assert!(forecast.included_in(&previous_forecast));
    }

    #[test]
    fn test_generated_forecast_depends_on_current_to_target_period_distance() {
        let target_period = DeliveryPeriodId::from(6);
        let forecast = ForecastValue {
            value: 500,
            deviation: 200,
        };
        let test_cases = vec![
            // (current_period, expected_deviation)
            (DeliveryPeriodId::from(1), 100),
            (DeliveryPeriodId::from(2), 100),
            (DeliveryPeriodId::from(3), 75),
            (DeliveryPeriodId::from(4), 50),
            (DeliveryPeriodId::from(5), 25),
            (DeliveryPeriodId::from(6), 0),
            (DeliveryPeriodId::from(7), 0),
        ];

        for (current_period, expected_deviation) in test_cases.iter() {
            let forecast = generate_forecast(*current_period, target_period, forecast);
            assert_eq!(forecast.deviation, *expected_deviation);
        }
    }

    #[test]
    fn test_generated_forecast_when_new_variation_greater_than_previous_forecast() {
        let target_period = DeliveryPeriodId::from(4);
        let current_period = DeliveryPeriodId::from(1);
        let forecast = ForecastValue {
            value: 500,
            deviation: 50,
        };

        // expected deviation is 75, but greater than 50 so we dont update the forecast
        assert_eq!(
            generate_forecast(current_period, target_period, forecast),
            ForecastValue {
                value: 500,
                deviation: 50
            }
        );
    }

    #[test]
    fn test_generated_forecast_update_value() {
        let target_period = DeliveryPeriodId::from(4);
        let current_period = DeliveryPeriodId::from(1);
        let forecast = ForecastValue {
            value: 500,
            deviation: 100,
        };

        // new deviation = 75
        // possibile forecast values that are included in 500 +/- 100 are
        // 475 +/- 75
        // 500 +/- 75
        // 525 +/- 75

        let values: Vec<isize> = (0..1000)
            .into_iter()
            .map(|_| generate_forecast(current_period, target_period, forecast).value)
            .collect();

        assert!(values.contains(&475));
        assert!(values.contains(&500));
        assert!(values.contains(&525));
    }
}
