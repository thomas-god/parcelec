import { describe, it, expect } from "vitest";
import { marketPnl, plantsPnl } from "./pnl";
import type { Trade } from "./message";

describe("Market Pnl", () => {
  it("should return 0 for an empty array of trades", () => {
    const trades: Trade[] = [];
    expect(marketPnl(trades)).toBe(0);
  });

  it("should calculate PnL for a series of trades", () => {
    const trades: Trade[] = [
      {
        direction: "Buy",
        volume: 100,
        price: 20_00,
        execution_time: "",
        owner: "",
      },
      {
        direction: "Sell",
        volume: 50,
        price: 30_00,
        execution_time: "",
        owner: "",
      },
      {
        direction: "Buy",
        volume: 20,
        price: 15_00,
        execution_time: "",
        owner: "",
      },
    ];
    expect(marketPnl(trades)).toBe(-100 * 20 + 50 * 30 - 20 * 15);
  });
});

describe("Plants PnL", () => {
  it("Should return zero if no plants", () => {
    expect(plantsPnl(new Map())).toEqual(0);
  });

  it("Should sum each plant -cost (if cost is 0, then we lose money)", () => {
    expect(
      plantsPnl(
        new Map([
          [
            "2",
            {
              type: "GasPlant",
              output: {
                cost: 1000,
                setpoint: 100,
              },
              settings: {
                energy_cost: 100,
                max_setpoint: 1000,
              },
            },
          ],
          [
            "3",
            {
              type: "Consumers",
              output: {
                cost: -150,
                setpoint: -170,
              },
              max_power: 1000,
            },
          ],
        ]),
      ),
    ).toEqual(-850);
  });
});
