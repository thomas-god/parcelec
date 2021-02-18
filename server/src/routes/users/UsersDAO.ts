import { Database } from "../../db";
import { Dependencies } from "../../di.context";
import { User } from "../types";

export class UsersDAO {
  private db: Dependencies["db"];

  constructor({ db }: { db: Database }) {
    this.db = db;
  }

  /**
   * Return a user object from its ID
   * @param userID User UUID
   * @param sessionID Session UUID
   */
  async getUser(userID: string, sessionID: string): Promise<User> {
    const rows = (
      await this.db.execute(
        `SELECT
          id,
          session_id,
          name,
          game_ready
        FROM users
        WHERE
          id=$1
          AND session_id=$2;`,
        [userID, sessionID]
      )
    ).rows;
    return rows.length === 1 ? (rows[0] as User) : null;
  }
}
