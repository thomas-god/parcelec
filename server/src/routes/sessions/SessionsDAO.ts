import { Session } from "../types";
import { Database } from "../../db";
import { Dependencies } from "../../di.context";

export class SessionsDAO {
  private db: Dependencies["db"];

  constructor({ db }: { db: Database }) {
    this.db = db;
  }

  /**
   * Return a session object from its ID
   * @param sessionID Session UUID
   */
  async getSession(sessionID: string): Promise<Session> {
    const rows = (
      await this.db.execute(
        `SELECT
          id, name, status, scenario_id
        FROM sessions
        WHERE id=$1;`,
        [sessionID]
      )
    ).rows;
    return rows.length === 1 ? (rows[0] as Session) : null;
  }
}
