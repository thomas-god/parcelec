import { QueryResultRow } from "pg";
import { v4 as uuid } from "uuid";
import { Dependencies } from "../../di.context";
import { BidTypes, Bid } from "./types";

export class BidsDAO {
  private db: Dependencies["db"];

  constructor({ db }: { db: Dependencies["db"] }) {
    this.db = db;
  }

  async createBid(sessionId: string, userId: string, type: BidTypes, volume: number, price: number): Promise<Bid> {
    return (
      await this.db.execute(
        `INSERT INTO t_bids (
          id,
          session_id,
          user_id,
          type,
          volume,
          price
        ) VALUES (
          $1::uuid,
          $2::uuid,
          $3::uuid,
          $4::text,
          $5::numeric,
          $6::numeric
        ) RETURNING id, session_id, user_id, type, volume, price;
        `,
        [uuid(), sessionId, userId, type, volume, price]
      )
    ).rows.map((row: QueryResultRow) => {
      return <Bid>{
        id: row["id"],
        sessionId: row["session_id"],
        userId: row["user_id"],
        type: row["type"],
        volume: row["volume"],
        price: row["price"]
      }
    })[0];
  }
}
