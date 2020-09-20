/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */

import {
  getLastPhaseNo,
  getSession,
  getSessionOptions,
  getSessionUsers,
} from "../utils";
import db from "../../db";
import { sendUpdateToUsers } from "../websocket";
import clearing from "./clearing";
import { endGame } from "./end_game";
import { generateEmptyPlanning, insertPlanning } from "./plannings";
import { Session } from "../types";

export async function startGamePhase(session_id: string): Promise<void> {
  const session = await getSession(session_id);
  const options = await getSessionOptions(session_id);
  const users = await getSessionUsers(session_id);
  const current_phase_no = await getLastPhaseNo(session_id);
  const next_phase_no = current_phase_no === null ? 0 : current_phase_no + 1;

  // Check if all users are ready
  // TODO: check if there is not an open phase already
  if (
    users.length >= 2 &&
    users.filter((u) => u.game_ready).length === users.length &&
    next_phase_no < options.phases_number
  ) {
    if (session.status === "open") {
      // Update session status to prevent new users registration
      await setSessionStatus(session, "running");
    }
    // Insert a new phase item
    await db.query(
      "INSERT INTO phases (session_id, phase_no, status) VALUES ($1, $2, $3)",
      [session_id, next_phase_no, "open"]
    );

    // Generate conso forecast for each user
    const conso_value = options.conso_forecast_mwh[next_phase_no];
    await Promise.all(
      users.map(async (user) => {
        await db.query(
          "INSERT INTO conso (user_id, session_id, phase_no, value_mw) VALUES ($1, $2, $3, $4)",
          [user.id, user.session_id, next_phase_no, conso_value]
        );
      })
    );

    // Generate empty planning for each user
    await Promise.all(
      users.map(async (user) => {
        await insertPlanning(await generateEmptyPlanning(user, next_phase_no));
      })
    );

    // Update phase with the correct timings and set the callbacks
    const t_start = Date.now() / 1000; // ms -> s for PSQL
    const t_clearing = t_start + options.bids_duration_sec;
    const t_end = t_start + options.plannings_duration_sec;
    setTimeout(
      () => clearing(session_id, next_phase_no),
      options.bids_duration_sec * 1000
    );
    setTimeout(
      () => endGame(session_id, next_phase_no),
      options.plannings_duration_sec * 1000
    );
    await db.query(
      `UPDATE phases 
      SET 
        start_time=to_timestamp($1),
        clearing_time=to_timestamp($2),
        planning_time=to_timestamp($3)
      WHERE session_id=$4 AND phase_no=$5`,
      [t_start, t_clearing, t_end, session_id, next_phase_no]
    );
    // Notify users that a new phase has started
    sendUpdateToUsers(session_id, "new-game-phase", {});
  }
}

/**
 * Update the session game status (`open`, `running` or `closed`).
 * Update in place session object status.
 * @param session Session object
 * @param status New status
 */
async function setSessionStatus(
  session: Session,
  status: Session["status"]
): Promise<void> {
  await db.query("UPDATE sessions SET status=$1 WHERE id=$2", [
    status,
    session.id,
  ]);
  session.status = status;
}
