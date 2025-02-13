import type { StackSnapshot, Trade } from "./message";

export const marketPnl = (trades: Trade[]): number => {
  const pnl = trades.reduce(
    (acc, trade) =>
      acc +
      (trade.direction === "Sell"
        ? trade.volume * trade.price
        : -(trade.volume * trade.price)),
    0,
  );
  return pnl / 100; // Convert price from cents to euros
};

export const plantsPnl = (plants: StackSnapshot): number => {
  /// Cannot use plants.entries().reduce(/.../) on WebKit...
  let total = 0;
  for (const [_, plant] of plants.entries()) {
    total -= plant.output.cost;
  }
  return total;
};
