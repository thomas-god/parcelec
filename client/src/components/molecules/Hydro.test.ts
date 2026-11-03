import { test, vi, expect, describe, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";

import Hydro from "./Hydro.svelte";

describe("Hydro component", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("Displays the reservoir stock", () => {
    render(Hydro, {
      setpoint: 0,
      stock: 500,
      max_stock: 1000,
      max_setpoint: 100,
      updateSetpoint: vi.fn(),
    });

    expect(screen.getByText(/500 \/ 1000 MWh/)).toBeInTheDocument();
  });

  test("Slider is limited by the turbine power", () => {
    render(Hydro, {
      setpoint: 0,
      stock: 1000,
      max_stock: 1000,
      max_setpoint: 100,
      updateSetpoint: vi.fn(),
    });

    const slider = screen.getByTestId("hydro-input");
    expect(slider).toHaveAttribute("max", "100");
  });

  test("Slider is limited by the remaining stock", () => {
    render(Hydro, {
      setpoint: 0,
      stock: 40,
      max_stock: 1000,
      max_setpoint: 100,
      updateSetpoint: vi.fn(),
    });

    const slider = screen.getByTestId("hydro-input");
    expect(slider).toHaveAttribute("max", "40");
  });

  test("Changing the slider value calls updateSetpoint with a positive value", async () => {
    const updateSetpoint = vi.fn();
    render(Hydro, {
      setpoint: 0,
      stock: 500,
      max_stock: 1000,
      max_setpoint: 100,
      updateSetpoint,
    });

    const slider = screen.getByTestId("hydro-input");
    await fireEvent.input(slider, { target: { value: "75" } });

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(75);
  });
});
