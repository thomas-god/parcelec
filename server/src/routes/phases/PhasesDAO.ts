import { v4 as uuid } from "uuid";
import { QueryResultRow } from "pg";
import { Database } from "../../db";
import { Dependencies } from "../../di.context";
import { Phase } from "./types";

export class PhasesDAO {
  private db: Dependencies["db"];

  constructor({ db }: { db: Database }) {
    this.db = db;
  }

  async getLastPhaseInfos(sessionID: string): Promise<Phase> {
    return (
      await this.db.execute(
        `SELECT
            session_id,
            phase_no,
            clearing_time,
            planning_time,
            bids_allowed,
            clearing_available,
            plannings_allowed,
            results_available,
            status
          FROM t_phases
          WHERE session_id=$1
          ORDER BY phase_no DESC
          LIMIT 1;`,
        [sessionID]
      )
    ).rows.map((row: QueryResultRow) => {
      return <Phase>{
        sessionId: row["session_id"],
        phaseNo: row["phase_no"],
        clearingTime: row["clearing_time"],
        planningTime: row["planning_time"],
        bidsAllowed: row["bids_allowed"],
        clearingAvailable: row["clearing_available"],
        planningsAllowed: row["plannings_allowed"],
        resultsAvailable: row["results_available"],
        status: row["status"],
      };
    })[0];
  }
}
