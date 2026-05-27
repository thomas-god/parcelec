import { describe, expect, it } from "vitest";
import { FORECAST_STEP, generateForecastValues } from "./forecasts";

describe("Generating forecasts", () => {
  it("Should generate the correct number of forecast values", () => {
    const number_of_periods = 4;
    const forecasts = generateForecastValues(number_of_periods, 0, 1000);

    expect(forecasts.length).toEqual(number_of_periods);
  });

  it("Should generate forecast values between [min, max]", () => {
    const forecasts = generateForecastValues(1000, -100, 1000);

    expect(
      forecasts.every(
        (forecast) => forecast.value >= -100 && forecast.value <= 1000,
      ),
    ).toEqual(true);
  });

  it("Should generate forecast values multiple of FORECAST_STEP", () => {
    const forecasts = generateForecastValues(1000, -100, 1000);

    expect(
      forecasts.every((forecast) => forecast.value % FORECAST_STEP === 0),
    ).toEqual(true);
  });

  it("Should generate forecast deviation depending on the period", () => {
    const forecasts = generateForecastValues(10, 0, 500);

    expect(forecasts.at(0)?.deviation).toEqual(1 * FORECAST_STEP);
    expect(forecasts.at(1)?.deviation).toEqual(2 * FORECAST_STEP);
    expect(forecasts.at(2)?.deviation).toEqual(3 * FORECAST_STEP);
    expect(
      forecasts
        .slice(3)
        .every((forecast) => forecast.deviation === 4 * FORECAST_STEP),
    );
  });
});
