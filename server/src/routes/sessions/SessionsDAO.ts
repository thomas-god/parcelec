import { v4 as uuid } from "uuid";
import { QueryResultRow } from "pg";
import { Database } from "../../db";
import { Dependencies } from "../../di.context";
import { Session } from "./types";

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

  async createSession(sessionName: string): Promise<Session> {
    return (
      await this.db.execute(
        `INSERT INTO t_sessions (
          id,
          name
        ) VALUES (
          $1::uuid,
          $2::text
        ) RETURNING id, name, status;
        `,
        [uuid(), sessionName]
      )
    ).rows.map((row: QueryResultRow) => {
      return <Session>{
        id: row["id"],
        name: row["name"],
        status: row["status"],
      };
    })[0];
  }
}
