import { describe, expect, it } from "vitest";
import { FORECAST_STEP, generateForecast } from "./forecasts";

describe("Generating forecasts", () => {
  it("Should generate the correct number of forecast values", () => {
    const number_of_periods = 4;
    const forecasts = generateForecast(number_of_periods, 0, 1000);

    expect(forecasts.length).toEqual(number_of_periods);
  });

  it("Should generate forecasts, starting at period 1", () => {
    const number_of_periods = 4;
    const forecasts = generateForecast(number_of_periods, 0, 1000);

    expect(forecasts.at(0)?.period).toEqual(1);
    expect(forecasts.at(-1)?.period).toEqual(number_of_periods);
  });

  it("Should generate forecast values between [min, max]", () => {
    const forecasts = generateForecast(1000, -100, 1000);

    expect(
      forecasts.every(
        (forecast) =>
          forecast.value.value >= -100 && forecast.value.value <= 1000,
      ),
    ).toEqual(true);
  });

  it("Should generate forecast values multiple of FORECAST_STEP", () => {
    const forecasts = generateForecast(1000, -100, 1000);

    expect(
      forecasts.every((forecast) => forecast.value.value % FORECAST_STEP === 0),
    ).toEqual(true);
  });

  it("Should generate forecast deviation depending on the period", () => {
    const forecasts = generateForecast(10, 0, 500);

    expect(forecasts.at(0)?.value.deviation).toEqual(1 * FORECAST_STEP);
    expect(forecasts.at(1)?.value.deviation).toEqual(2 * FORECAST_STEP);
    expect(forecasts.at(2)?.value.deviation).toEqual(3 * FORECAST_STEP);
    expect(
      forecasts
        .slice(3)
        .every((forecast) => forecast.value.deviation === 4 * FORECAST_STEP),
    );
  });
});
