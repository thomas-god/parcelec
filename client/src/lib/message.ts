import { type SafeParseReturnType, string, z } from "zod";

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
    max_power: z.number(),
    output: PlantOutput,
  }),
  z.object({
    type: z.literal("Consumers"),
    max_power: z.number(),
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
    type: z.literal("StackSnapshot"),
    plants: z
      .record(z.string(), PowerPlantRepr)
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("StackForecasts"),
    forecasts: z
      .record(z.string(), z.nullable(z.enum(["Low", "Medium", "High"])))
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("MarketState"),
    state: z.enum(["Open", "Closed"]),
  }),
  z.object({
    type: z.literal("StackState"),
    state: z.enum(["Open", "Closed"]),
  }),
  z.object({
    type: z.literal("GameState"),
    state: z.enum(["Open", "Running", "PostDelivery"]),
    delivery_period: z.number(),
  }),
  z.object({
    type: z.literal("DeliveryPeriodResults"),
    delivery_period: z.number(),
    score: z.object({
      balance: z.number(),
      pnl: z.number(),
      imbalance_cost: z.number(),
    }),
  }),
  z.object({
    type: z.literal("PlayerScores"),
    scores: z
      .record(
        z.coerce.number(),
        z.object({
          balance: z.number(),
          pnl: z.number(),
          imbalance_cost: z.number(),
        }),
      )
      .transform((rec) => new Map(Object.entries(rec))),
  }),
  z.object({
    type: z.literal("NewMarketForecast"),
    issuer: z.string(),
    period: z.number(),
    direction: Direction,
    volume: z.enum(["Low", "Medium", "High"]),
    price: z.number().nullable(),
  }),
  z.object({
    type: z.literal("MarketForecasts"),
    forecasts: z.array(
      z.object({
        issuer: z.string(),
        period: z.number(),
        direction: Direction,
        volume: z.enum(["Low", "Medium", "High"]),
        price: z.number().nullable(),
      }),
    ),
  }),
]);

type WSMessage = z.infer<typeof WSMessageSchema>;
export type OrderBook = Omit<
  Extract<WSMessage, { type: "OrderBookSnapshot" }>,
  "type"
>;
export type Trade = Omit<Extract<WSMessage, { type: "NewTrade" }>, "type">;
export type OrderBookEntry = z.infer<typeof OrderBookEntrySchema>;
export type StackSnapshot = Omit<
  Extract<WSMessage, { type: "StackSnapshot" }>,
  "type"
>["plants"];
export type StackForecasts = Omit<
  Extract<WSMessage, { type: "StackForecasts" }>,
  "type"
>["forecasts"];
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
export type PlayerScores = Pick<
  Extract<WSMessage, { type: "PlayerScores" }>,
  "scores"
>["scores"];
export type MarketForecast = Omit<
  Extract<WSMessage, { type: "NewMarketForecast" }>,
  "type"
>;

export const parseMessage = (
  msg: string,
): SafeParseReturnType<
  z.infer<typeof WSMessageSchema>,
  z.infer<typeof WSMessageSchema>
> => {
  return WSMessageSchema.safeParse(JSON.parse(msg));
};
