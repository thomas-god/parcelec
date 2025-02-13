import { describe, expect, it, test } from "vitest";
import { marketPosition, plantsPosition } from "./position";

describe("Computing market position from trades", () => {
  it("should return 0 is the trade list is empty", () => {
    expect(marketPosition([])).toEqual(0);
  });

  it("Should be positive if there is more buy than sell orders", () => {
    expect(
      marketPosition([
        {
          direction: "Buy",
          volume: 100,
          price: 10,
          execution_time: "",
          owner: "toto",
        },
        {
          direction: "Sell",
          volume: 50,
          price: 10,
          execution_time: "",
          owner: "toto",
        },
      ]),
    ).toEqual(50);
  });

  it("Should be negative if there is more sell than buy orders", () => {
    expect(
      marketPosition([
        {
          direction: "Sell",
          volume: 100,
          price: 10,
          execution_time: "",
          owner: "toto",
        },
        {
          direction: "Buy",
          volume: 50,
          price: 10,
          execution_time: "",
          owner: "toto",
        },
      ]),
    ).toEqual(-50);
  });
});

describe("Computing plants position from stack snapshot", () => {
  it("Should be zero if there is no plants", () => {
    expect(plantsPosition(new Map())).toEqual(0);
  });

  it("Should sum each plant setpoint", () => {
    expect(
      plantsPosition(
        new Map([
          [
            "2",
            {
              type: "GasPlant",
              output: {
                cost: 0,
                setpoint: 50,
              },
              settings: { energy_cost: 0, max_setpoint: 0 },
            },
          ],
          [
            "3",
            {
              type: "Consumers",
              output: {
                cost: 0,
                setpoint: -100,
              },
              max_power: 1000,
            },
          ],
        ]),
      ),
    ).toEqual(-50);
  });
});
