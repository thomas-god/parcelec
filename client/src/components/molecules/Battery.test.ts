import { test, vi, expect, describe, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";

import Battery from "./Battery.svelte";

describe("Battery component", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("Display 'Charge' when battery is empty", async () => {
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: 0,
      charge: 0,
      max_charge: 1000,
      updateSetpoint,
    });

    expect(screen.getByText("Charge 🔄")).toBeInTheDocument();
  });
  test("Display 'Décharge' when battery is full", async () => {
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: 0,
      charge: 1000,
      max_charge: 1000,
      updateSetpoint,
    });

    expect(screen.getByText("Décharge 🔄")).toBeInTheDocument();
  });

  test("Display 'Charge' when setpoint is negative", async () => {
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: -100,
      charge: 500,
      max_charge: 1000,
      updateSetpoint,
    });

    expect(screen.getByText("Charge 🔄")).toBeInTheDocument();
  });

  test("Display 'Décharge' when setpoint is positive", async () => {
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: 100,
      charge: 500,
      max_charge: 1000,
      updateSetpoint,
    });

    expect(screen.getByText("Décharge 🔄")).toBeInTheDocument();
  });

  test("Toggling charge state calls updateSetpoint", async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: -100,
      charge: 500,
      max_charge: 1000,
      updateSetpoint,
    });

    let toggle = screen.getByText("Charge 🔄");
    expect(toggle).toBeVisible();

    await user.click(toggle);

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(expect.any(Number));
  });

  test("Changing the slider value calls updateSetpoint", async () => {
    const updateSetpoint = vi.fn();
    render(Battery, {
      setpoint: 0,
      charge: 500,
      max_charge: 1000,
      updateSetpoint,
    });

    const slider = screen.getByTestId("battery-input");
    await fireEvent.input(slider, { target: { value: "200" } });

    vi.runAllTimers(); // Run the debounced function

    expect(updateSetpoint).toHaveBeenCalledWith(-200); // Negative because default is charge mode
  });
});
