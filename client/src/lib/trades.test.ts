import { describe, it, expect } from "vitest";
import { extract_bbo } from "./trades";
import type { OrderBook, OrderBookEntry } from "./message";
import { isSome } from "./Options";

const entry = (price: number, direction: "Buy" | "Sell"): OrderBookEntry => ({
  order_id: "id",
  direction,
  volume: 10,
  price,
  owned: false,
  created_at: "2024-01-01T00:00:00.000Z",
});

describe("extracting the best bid and the best offer from an order book", () => {
  it("returns none for both sides when the order book is empty", () => {
    const book: OrderBook = { bids: [], offers: [] };
    const { bestBid, bestOffer } = extract_bbo(book);
    expect(isSome(bestBid)).toBe(false);
    expect(isSome(bestOffer)).toBe(false);
  });

  it("returns the only bid when there is one", () => {
    const book: OrderBook = { bids: [entry(50, "Buy")], offers: [] };
    const { bestBid, bestOffer } = extract_bbo(book);
    expect(isSome(bestBid)).toBe(true);
    if (isSome(bestBid)) expect(bestBid.value.price).toBe(50);
    expect(isSome(bestOffer)).toBe(false);
  });

  it("returns the only offer when there is one", () => {
    const book: OrderBook = { bids: [], offers: [entry(60, "Sell")] };
    const { bestBid, bestOffer } = extract_bbo(book);
    expect(isSome(bestBid)).toBe(false);
    expect(isSome(bestOffer)).toBe(true);
    if (isSome(bestOffer)) expect(bestOffer.value.price).toBe(60);
  });

  it("selects the bid with the highest price among multiple bids", () => {
    const book: OrderBook = {
      bids: [entry(30, "Buy"), entry(50, "Buy"), entry(40, "Buy")],
      offers: [],
    };
    const { bestBid } = extract_bbo(book);
    expect(isSome(bestBid)).toBe(true);
    if (isSome(bestBid)) expect(bestBid.value.price).toBe(50);
  });

  it("selects the offer with the lowest price among multiple offers", () => {
    const book: OrderBook = {
      bids: [],
      offers: [entry(70, "Sell"), entry(55, "Sell"), entry(80, "Sell")],
    };
    const { bestOffer } = extract_bbo(book);
    expect(isSome(bestOffer)).toBe(true);
    if (isSome(bestOffer)) expect(bestOffer.value.price).toBe(55);
  });

  it("returns both best bid and best offer together", () => {
    const book: OrderBook = {
      bids: [entry(40, "Buy"), entry(45, "Buy")],
      offers: [entry(60, "Sell"), entry(55, "Sell")],
    };
    const { bestBid, bestOffer } = extract_bbo(book);
    if (isSome(bestBid)) expect(bestBid.value.price).toBe(45);
    if (isSome(bestOffer)) expect(bestOffer.value.price).toBe(55);
  });
});
