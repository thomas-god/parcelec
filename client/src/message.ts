import { type SafeParseReturnType, z } from "zod";

const Direction = z.enum(["Buy", "Sell"]);
const Volume = z.number().int().positive();
const Price = z.number().int();

const OrderBookEntry = z.object({
	direction: Direction,
	volume: Volume,
	price: Price,
	created_at: z.string().datetime(),
});

const WSMessageSchema = z.discriminatedUnion("type", [
	z.object({
		type: z.literal("OrderBookSnapshot"),
		bids: z.array(OrderBookEntry),
		offers: z.array(OrderBookEntry),
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
export type OrderBook = Omit<Extract<WSMessage, { type: "OrderBookSnapshot" }>, "type">
export type Trade = Omit<Extract<WSMessage, { type: "NewTrade" }>, "type">

export const parseMessage = (
	msg: string,
): SafeParseReturnType<
	z.infer<typeof WSMessageSchema>,
	z.infer<typeof WSMessageSchema>
> => {
	return WSMessageSchema.safeParse(JSON.parse(msg));
};
