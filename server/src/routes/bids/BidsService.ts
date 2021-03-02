import { v4 as uuid } from "uuid";
import { Dependencies } from "../../di.context";
import { Bid } from "../types";

import { BidTypes } from "./types";

export class BidsService {
  private BidsDAO: Dependencies["BidsDAO"];
  private SessionsDAO: Dependencies["SessionsDAO"];
  private UsersDAO: Dependencies["UsersDAO"];

  constructor({
    BidsDAO,
    SessionsDAO,
    UsersDAO,
  }: {
    BidsDAO: Dependencies["BidsDAO"];
    SessionsDAO: Dependencies["SessionsDAO"];
    UsersDAO: Dependencies["UsersDAO"];
  }) {
    this.BidsDAO = BidsDAO;
    this.SessionsDAO = SessionsDAO;
    this.UsersDAO = UsersDAO;
  }

  /**
   * Post a new bid to a running auction
   * @param sessionId Session UUID
   * @param userId User UUID
   * @param body Bid to post
   */
  async postUserBid(
    sessionId: string,
    userId: string,
    type: BidTypes,
    volume: number,
    price: number
  ): Promise<string> {
    const session = await this.SessionsDAO.getSession(sessionId);
    if (session === undefined) {
      throw new Error(`Cannot find a session with ID ${sessionId}.`);
    }
    if (session.status !== "running") {
      throw new Error(`Session is not running.`);
    }

    const user = await this.UsersDAO.getUser(sessionId, userId);
    if (user === undefined) {
      throw new Error(`Cannot find a user with ID ${userId}.`);
    }

    const bid = await this.BidsDAO.createBid(
      sessionId,
      userId,
      type,
      volume,
      price
    );
    return bid.id;
  }
}
