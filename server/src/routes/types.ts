export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
}

export interface ClientMessage {
  username: string;
  date: Date;
  reason: "message" | "handshake";
  credentials: {
    auction_id: string;
    user_id: string;
  };
  data: any;
}

export interface Auction {
  name: string;
  id: string;
  status: "Open" | "Running" | "Close";
}

export interface User {
  username: string;
  user_id: string;
}

export interface Bid {
  auction_id: string;
  user_id: string;
  auction_step_no: string;
  bid_value: number;
}
