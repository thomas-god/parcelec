import type { StackSnapshot, Trade } from "./message";

export const marketPosition = (trades: Trade[]): number =>
  trades.reduce(
    (acc, trade) =>
      acc + (trade.direction === "Buy" ? trade.volume : -trade.volume),
    0,
  );

export const plantsPosition = (plants: StackSnapshot): number => {
  /// Cannot use plants.entries().reduce(/.../) on WebKit...
  let total = 0;
  for (const [_, plant] of plants.entries()) {
    total += plant.output.setpoint;
  }
  return total;
};
