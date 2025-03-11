import { test, vi, expect, describe, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";

import NuclearPlant from "./NuclearPlant.svelte";
import userEvent from "@testing-library/user-event";

describe("Nuclear plant component", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("Changing the slider value calls updateSetpoint", async () => {
    const updateSetpoint = vi.fn();
    render(NuclearPlant, {
      setpoint: 0,
      previous_setpoint: 0,
      updateSetpoint,
      cost: 0,
      dispatchable: true,
      energy_cost: 0,
      max_setpoint: 1000,
    });

    const slider = screen.getByTestId("nuclear-plant-input");
    await fireEvent.input(slider, { target: { value: "200" } });

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(200);
  });

  test("Resetting the plant to its previous setpoint", async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    const updateSetpoint = vi.fn();
    render(NuclearPlant, {
      setpoint: 0,
      previous_setpoint: 0,
      updateSetpoint,
      cost: 0,
      dispatchable: true,
      energy_cost: 0,
      max_setpoint: 1000,
    });

    const slider = screen.getByTestId("nuclear-plant-input");
    await fireEvent.input(slider, { target: { value: "200" } });

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(200);

    const resetButton = screen.getByText("↩️");
    await user.click(resetButton);

    expect(updateSetpoint).toHaveBeenLastCalledWith(0); // Initial setpoint
  });
});
