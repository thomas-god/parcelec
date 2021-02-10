import { v4 as uuid } from "uuid";
import { Dependencies } from "../../di.context";
import { Bid } from "../types";

import {
  SessionDoesNotExistError,
  SessionIsNotRunningError,
} from "../../errors/sessions.errors";
import { UserDoesNotExistError } from "../../errors/users.errors";

export interface BidInput {
  type: "sell" | "buy";
  volume_mwh: number;
  price_eur_per_mwh: number;
}

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
    body: BidInput
  ): Promise<Bid> {
    const session = await this.SessionsDAO.getSession(sessionId);
    if (session === null) {
      throw new SessionDoesNotExistError("Session does not exist");
    }
    if (session.status !== "running") {
      throw new SessionIsNotRunningError("Session is not running");
    }

    const user = await this.UsersDAO.getUser(userId, sessionId);
    if (user === null) {
      throw new UserDoesNotExistError("User does not exist");
    }

    const bid = {
      user_id: userId,
      session_id: sessionId,
      id: uuid(),
      type: body.type,
      volume_mwh: Number(body.volume_mwh),
      price_eur_per_mwh: Number(body.price_eur_per_mwh),
    };
  }
}
