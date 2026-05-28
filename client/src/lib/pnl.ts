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
  return pnl;
};

export const plantsCosts = (plants: StackSnapshot): number => {
  if (plants === null) {
    return 0;
  }
  /// Cannot use plants.entries().reduce(/.../) on WebKit...
  let total = 0;
  for (const [_, plant] of plants.entries()) {
    if (["GasPlant", "Nuclear"].includes(plant.type)) {
      total += plant.output.cost;
    }
  }
  return total;
};
