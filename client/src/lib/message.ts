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
]);

type WSMessage = z.infer<typeof WSMessageSchema>;
export type OrderBook = Omit<
  Extract<WSMessage, { type: "OrderBookSnapshot" }>,
  "type"
>;
export type Trade = Omit<Extract<WSMessage, { type: "NewTrade" }>, "type">;
export type OrderBookEntry = z.infer<typeof OrderBookEntrySchema>;

export const parseMessage = (
  msg: string,
): SafeParseReturnType<
  z.infer<typeof WSMessageSchema>,
  z.infer<typeof WSMessageSchema>
> => {
  return WSMessageSchema.safeParse(JSON.parse(msg));
};
