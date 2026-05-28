import type { StackSnapshot, Trade } from "./message";

export type PortfolioType =
  | "consumers"
  | "renewables"
  | "nuclear"
  | "gas"
  | "storage"
  | "market";

export type PortfolioVolumes = {
  consumers: number;
  renewable: number;
  nuclear: number;
  gas: number;
  storage: number;
  marketSold: number;
  marketBought: number;
};

export const marketPosition = (trades: Trade[]): number =>
  trades.reduce(
    (acc, trade) =>
      acc + (trade.direction === "Buy" ? trade.volume : -trade.volume),
    0,
  );

export const plantsPosition = (plants: StackSnapshot): number => {
  if (plants === null) {
    return 0;
  }
  /// Cannot use plants.entries().reduce(/.../) on WebKit...
  let total = 0;
  for (const [_, plant] of plants.entries()) {
    total += plant.output.setpoint;
  }
  return total;
};

export const computePortfolio = (
  plants: StackSnapshot,
  trades: Trade[],
): PortfolioVolumes => {
  const portfolio = {
    consumers: 0,
    renewable: 0,
    nuclear: 0,
    gas: 0,
    storage: 0,
    marketSold: 0,
    marketBought: 0,
  };

  if (plants !== null) {
    for (const [_, plant] of plants.entries()) {
      switch (plant.type) {
        case "Consumers":
          portfolio.consumers += plant.output.setpoint;
          break;
        case "RenewablePlant":
          portfolio.renewable += plant.output.setpoint;
          break;
        case "Nuclear":
          portfolio.nuclear += plant.output.setpoint;
          break;
        case "GasPlant":
          portfolio.gas += plant.output.setpoint;
          break;
        case "Battery":
          portfolio.storage += plant.output.setpoint;
          break;
      }
    }
  }

  for (const trade of trades) {
    if (trade.direction === "Buy") {
      portfolio.marketBought += trade.volume;
    } else {
      portfolio.marketSold += trade.volume;
    }
  }

  return portfolio;
};
