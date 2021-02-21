import { v4 as uuid } from "uuid";
import { QueryResultRow } from "pg";
import { Database } from "../../db";
import { Dependencies } from "../../di.context";
import { User } from "./types";

export class UsersDAO {
  private db: Dependencies["db"];

  constructor({ db }: { db: Database }) {
    this.db = db;
  }

  async createUser(sessionId: string, username: string): Promise<User> {
    return (
      await this.db.execute(
        `INSERT INTO t_users (
          id,
          session_id,
          name
      ) VALUES (
          $1::uuid,
          $2::uuid,
          $3::text
      ) RETURNING id, session_id, name, is_ready;
      `,
        [uuid(), sessionId, username]
      )
    ).rows.map(
      (row: QueryResultRow): User => {
        return <User>{
          id: row["id"],
          sessionId: row["session_id"],
          name: row["name"],
          isReady: row["is_ready"],
        };
      }
    )[0];
  }

  /**
   * Return a user object from its ID
   * @param userID User UUID
   * @param sessionID Session UUID
   */
  async getUser(sessionID: string, userID: string): Promise<User> {
    return (
      await this.db.execute(
        `SELECT
          id,
          session_id,
          name,
          is_ready
        FROM t_users
        WHERE
          id=$1::uuid
          AND session_id=$2::uuid;`,
        [userID, sessionID]
      )
    ).rows.map(
      (row: QueryResultRow): User => {
        return <User>{
          id: row["id"],
          sessionId: row["session_id"],
          name: row["name"],
          isReady: row["is_ready"],
        };
      }
    )[0];
  }

  async markUserReady(sessionId: string, userId: string): Promise<void> {
    await this.db.execute(
      `UPDATE t_users
      SET is_ready = true
      WHERE session_id = $1::uuid
        AND id = $2::uuid;`,
      [sessionId, userId]
    );
  }
}
