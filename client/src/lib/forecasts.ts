import type { Forecast } from "../components/organisms/Forecasts.svelte";

export const FORECAST_STEP = 25; // In MW

export const generateForecast = (
  number_of_periods: number,
  min: number,
  max: number,
): Forecast => {
  const forecasts: Forecast = [];
  for (let period = 1; period <= number_of_periods; period++) {
    forecasts.push({
      period,
      value: { value: random(min, max), deviation: deviation(period) },
    });
  }
  return forecasts;
};

const random = (min: number, max: number): number => {
  return (
    Math.round((Math.random() * (max - min) + min) / FORECAST_STEP) *
    FORECAST_STEP
  );
};

const deviation = (period: number): number => {
  if (period === 1) {
    return 1 * FORECAST_STEP;
  }
  if (period === 2) {
    return 2 * FORECAST_STEP;
  }
  if (period === 3) {
    return 3 * FORECAST_STEP;
  }
  return 4 * FORECAST_STEP;
};
