import type { OrderBook, OrderBookEntry } from "./message";
import { none, some, type Option, isNone } from "./Options";

export interface BBO {
  bestBid: Option<OrderBookEntry>;
  bestOffer: Option<OrderBookEntry>;
}

export const extract_bbo = (orderbook: OrderBook): BBO => {
  let bestBid: Option<OrderBookEntry> = none();
  for (const bid of orderbook.bids) {
    if (isNone(bestBid)) {
      bestBid = some(bid);
      continue;
    }
    if (bid.price > bestBid.value.price) {
      bestBid = some(bid);
    }
  }

  let bestOffer: Option<OrderBookEntry> = none();
  for (const offer of orderbook.offers) {
    if (isNone(bestOffer)) {
      bestOffer = some(offer);
      continue;
    }
    if (offer.price < bestOffer.value.price) {
      bestOffer = some(offer);
    }
  }

  return { bestBid, bestOffer };
};
