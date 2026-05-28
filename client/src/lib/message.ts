import { type SafeParseReturnType, z } from "zod";

const Direction = z.enum(["Buy", "Sell"]);
const Volume = z.number().int().positive();
const Price = z.number().int();

const OrderBookEntrySchema = z.object({
  order_id: z.string(),
  direction: Direction,
  volume: Volume,
  price: Price,
  owned: z.boolean(),
  created_at: z.string().datetime(),
});

const PlantOutput = z.object({
  setpoint: z.number(),
  cost: z.number(),
});
const PowerPlantRepr = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("Battery"),
    max_charge: z.number().int(),
    charge: z.number().int(),
    output: PlantOutput,
  }),
  z.object({
    type: z.literal("GasPlant"),
    settings: z.object({
      energy_cost: z.number(),
      max_setpoint: z.number(),
    }),
    output: PlantOutput,
  }),
  z.object({
    type: z.literal("RenewablePlant"),
    output: PlantOutput,
  }),
  z.object({
    type: z.literal("Consumers"),
    output: PlantOutput,
  }),
  z.object({
    type: z.literal("Nuclear"),
    output: PlantOutput,
    max_setpoint: z.number(),
    previous_setpoint: z.number(),
    energy_cost: z.number(),
    locked: z.boolean(),
    touched: z.boolean(),
  }),
]);
const PlayerScore = z.object({
  balance: z.number(),
  pnl: z.number(),
  imbalance_cost: z.number(),
});
const PlayerDetailedScore = z.object({
  consumers: z.object({ volume: z.number(), pnl: z.number() }),
  renewables: z.object({ volume: z.number(), pnl: z.number() }),
  gas: z.object({ volume: z.number(), pnl: z.number() }),
  nuclear: z.object({ volume: z.number(), pnl: z.number() }),
  battery_discharge: z.object({ volume: z.number(), pnl: z.number() }),
  battery_charge: z.object({ volume: z.number(), pnl: z.number() }),
  market_bought: z.object({ volume: z.number(), pnl: z.number() }),
  market_sold: z.object({ volume: z.number(), pnl: z.number() }),
  imbalance: z.object({ volume: z.number(), pnl: z.number() }),
});

const FixedStackConfigSchema = z.object({
  Fixed: z.object({
    gas_cost: z.number(),
    nuclear_cost: z.number(),
    consumers_revenues: z.number(),
    gas_capacity: z.number(),
    nuclear_capacity: z.number(),
    battery_capacity: z.number(),
    consumers_forecasts_range: z.number(),
    renewable_forecasts_range: z.number(),
  }),
});

const PerPlayerStackConfigSchema = z.object({
  PerPlayer: z.object({
    gas_cost: z.number(),
    gas_max_capacity: z.number(),
    nuclear_cost: z.number(),
    nuclear_max_capacity: z.number(),
    battery_max_capacity: z.number().optional(),
    consumers_revenues: z.number(),
    consumers_max_abs_capacity: z.number(),
    consumers_forecasts_range: z.number(),
    renewable_max_capacity: z.number(),
    renewable_forecasts_range: z.number(),
  }),
});

export const StackConfigSchema = z.union([
  FixedStackConfigSchema,
  PerPlayerStackConfigSchema,
]);

const WSMessageSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("OrderBookSnapshot"),
    bids: z.array(OrderBookEntrySchema),
    offers: z.array(OrderBookEntrySchema),
  }),
  z.object({
    type: z.literal("NewTrade"),
    direction: Direction,
    volume: Volume,
    price: Price,
    owner: z.string(),
    execution_time: z.string().datetime(),
  }),
  z.object({
    type: z.literal("TradeList"),
    trades: z.array(
      z.object({
        direction: Direction,
        volume: Volume,
        price: Price,
        owner: z.string(),
        execution_time: z.string().datetime(),
      }),
    ),
  }),
  z.object({
    type: z.literal("StackConfig"),
    config: StackConfigSchema,
  }),
  z.object({
    type: z.literal("StackSnapshot"),
    plants: z
      .record(z.string(), PowerPlantRepr)
      .nullable()
      .transform((rec) => {
        if (rec === null) {
          return null;
        }
        return new Map(Object.entries(rec));
      }),
  }),
  z.object({
    type: z.literal("StackForecasts"),
    forecasts: z
      .record(
        z.string(),
        z.nullable(
          z.array(
            z.object({
              period: z.number(),
              value: z.object({
                value: z.number(),
                deviation: z.number(),
              }),
            }),
          ),
        ),
      )
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("StackHistory"),
    history: z
      .record(
        z.string(),
        z.array(
          z.object({
            cost: z.number(),
            setpoint: z.number(),
          }),
        ),
      )
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("GameState"),
    state: z.enum(["Open", "Running", "PostDelivery", "Ended"]),
    delivery_period: z.number(),
    end_at: z.string(),
  }),
  z.object({
    type: z.literal("DeliveryPeriodResults"),
    delivery_period: z.number(),
    score: PlayerScore,
    detailed_score: PlayerDetailedScore,
  }),
  z.object({
    type: z.literal("PlayerScores"),
    scores: z
      .record(z.coerce.number(), PlayerScore)
      .transform((rec) => new Map(Object.entries(rec))),
    detailed_scores: z
      .record(z.coerce.number(), PlayerDetailedScore)
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("GameResults"),
    rankings: z.array(
      z.object({
        player: z.string(),
        rank: z.number(),
        score: z.number(),
      }),
    ),
  }),
  z.object({
    type: z.literal("ReadinessStatus"),
    readiness: z
      .record(z.string(), z.boolean())
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("YourName"),
    name: z.string(),
  }),
  z.object({
    type: z.literal("GameDuration"),
    last_period: z.number(),
  }),
]);

type WSMessage = z.infer<typeof WSMessageSchema>;
export type OrderBook = Omit<
  Extract<WSMessage, { type: "OrderBookSnapshot" }>,
  "type"
>;
export type Trade = Omit<Extract<WSMessage, { type: "NewTrade" }>, "type">;
export type OrderBookEntry = z.infer<typeof OrderBookEntrySchema>;
export type StackConfig = Omit<
  Extract<WSMessage, { type: "StackConfig" }>,
  "type"
>["config"];
export type StackSnapshot = Omit<
  Extract<WSMessage, { type: "StackSnapshot" }>,
  "type"
>["plants"];
export type Plant = StackSnapshot extends Map<any, infer I> ? I : never;
export type StackForecasts = Omit<
  Extract<WSMessage, { type: "StackForecasts" }>,
  "type"
>["forecasts"];
export type StackHistory = Omit<
  Extract<WSMessage, { type: "StackHistory" }>,
  "type"
>["history"];
export type BatteryState = Extract<
  StackSnapshot extends Map<any, infer I> ? I : never,
  { type: "Battery" }
>;
export type GasPlantState = Extract<
  StackSnapshot extends Map<any, infer I> ? I : never,
  { type: "GasPlant" }
>;
export type RenewablePlantState = Extract<
  StackSnapshot extends Map<any, infer I> ? I : never,
  { type: "RenewablePlant" }
>;
export type ConsumersState = Extract<
  StackSnapshot extends Map<any, infer I> ? I : never,
  { type: "Consumers" }
>;
export type NuclearPlantState = Extract<
  StackSnapshot extends Map<any, infer I> ? I : never,
  { type: "Nuclear" }
>;
export type DeliveryPeriodScore = Pick<
  Extract<WSMessage, { type: "DeliveryPeriodResults" }>,
  "score"
>["score"];
export type DeliveryPeriodDetailedScore = Pick<
  Extract<WSMessage, { type: "DeliveryPeriodResults" }>,
  "detailed_score"
>["detailed_score"];
export type PlayerScores = Pick<
  Extract<WSMessage, { type: "PlayerScores" }>,
  "scores"
>["scores"];
export type GameResults = Pick<
  Extract<WSMessage, { type: "GameResults" }>,
  "rankings"
>["rankings"];

export type ReadinessStatus = Extract<
  WSMessage,
  { type: "ReadinessStatus" }
>["readiness"];
export type GameState = Extract<WSMessage, { type: "GameState" }>["state"];

export const parseMessage = (
  msg: string,
): SafeParseReturnType<
  z.infer<typeof WSMessageSchema>,
  z.infer<typeof WSMessageSchema>
> => {
  return WSMessageSchema.safeParse(JSON.parse(msg));
};

/// Outbound message types
export type OrderRequest = {
  price: number;
  volume: number;
  direction: "Sell" | "Buy";
};
