export enum BidTypes {
  SELL = "sell",
  BUY = "BUY",
}

export interface Bid {
  id: string;
  userId: string;
  sessionId: string;
  type: BidTypes;
  volume: number;
  price: number;
}
