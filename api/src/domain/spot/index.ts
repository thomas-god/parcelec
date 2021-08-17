export class SPOT {
  register(bid: Bid) {
    
  }
}

export class Bid {
  userId: String;
  side: Side;
  quantity: Number;
  price: Number;

  constructor(userId: String, side: Side, quantity: Number, price: Number) {
    this.userId = userId
    this.side = side
    this.price = price;
    this.quantity = quantity
  }
}

export enum Side {
  BUY,
  SELL
}