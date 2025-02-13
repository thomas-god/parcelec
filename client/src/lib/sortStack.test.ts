import type { StackSnapshot } from "./message";
import { sortStack } from "./sortStack";
import { describe, expect, it } from "vitest";

describe("sortStack", () => {
  it("should sort the stack by ascending plant id (if no consumers)", () => {
    const stack: StackSnapshot = new Map([
      [
        "2",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
      [
        "3",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
      [
        "1",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
    ]);

    const sortedPlants = sortStack(stack);

    expect(Array.from(sortedPlants.keys())).toEqual(["1", "2", "3"]);
  });

  it("Should first sort the plants of type consumers, then the rest of the plants", () => {
    const stack: StackSnapshot = new Map([
      [
        "2",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
      [
        "3",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
      [
        "1",
        {
          type: "GasPlant",
          cost: 0,
          setpoint: 0,
          settings: { energy_cost: 0, max_setpoint: 0 },
        },
      ],
      ["cons_2", { type: "Consumers", cost: 0, setpoint: 0, max_power: 0 }],
      ["cons_1", { type: "Consumers", cost: 0, setpoint: 0, max_power: 0 }],
    ]);

    const sortedPlants = sortStack(stack);

    expect(Array.from(sortedPlants.keys())).toEqual([
      "cons_1",
      "cons_2",
      "1",
      "2",
      "3",
    ]);
  });
});
