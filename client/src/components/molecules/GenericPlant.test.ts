import { test, vi, expect, describe, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";

import GenericPlant from "./GenericPlant.svelte";

describe("Generic plant component", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("Changing the slider value calls updateSetpoint", async () => {
    const updateSetpoint = vi.fn();
    render(GenericPlant, {
      setpoint: 0,
      updateSetpoint,
      cost: 0,
      dispatchable: true,
      energy_cost: 0,
      max_setpoint: 1000,
      type: "gaz",
    });

    const slider = screen.getByTestId("generic-plant-input-gaz");
    await fireEvent.input(slider, { target: { value: "200" } });

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(200);
  });

  test("Non dispatachable plant cannot set setpoint", async () => {
    const updateSetpoint = vi.fn();
    render(GenericPlant, {
      setpoint: 0,
      updateSetpoint,
      cost: 0,
      dispatchable: false,
      energy_cost: 0,
      max_setpoint: 1000,
      type: "gaz",
    });

    const slider = screen.getByTestId("generic-plant-input-gaz");
    expect(slider).toBeDisabled();
  });
});
