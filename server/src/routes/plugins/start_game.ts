/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */

import { getCurrentPhaseNo, getSession, getSessionUsers } from "../utils";
import db from "../../db";
import { sendUpdateToUsers } from "../websocket";

export async function startGamePhase(session_id: string): Promise<void> {
  const session = await getSession(session_id);
  const users = await getSessionUsers(session_id);
  const current_phase_no = await getCurrentPhaseNo(session_id);
  const next_phase_no = current_phase_no === null ? 0 : current_phase_no + 1;

  if (session.status === "open") {
    // The game has not started yet, check if all users are ready
    if (
      users.length >= 2 &&
      users.filter((u) => u.game_ready).length === users.length
    ) {
      // Update session status to prevent new users registration
      await db.query("UPDATE sessions SET status='running' WHERE id=$1", [
        session_id,
      ]);
      session.status = "running";
    }
  }

  if (session.status === "running") {
    // Close previous phase and insert a new phase item
    await db.query(
      "UPDATE phases SET status='closed' WHERE session_id=$1 AND phase_no=$2",
      [session_id, current_phase_no]
    );
    await db.query(
      "INSERT INTO phases (session_id, phase_no, status) VALUES ($1, $2, $3)",
      [session_id, next_phase_no, "open"]
    );

    // Generate conso forecast for each user
    const conso_value = 3450;
    await Promise.all(
      users.map(async (user) => {
        await db.query(
          "INSERT INTO conso (user_id, session_id, phase_no, value_mw) VALUES ($1, $2, $3, $4)",
          [user.id, user.session_id, next_phase_no, conso_value]
        );
      })
    );

    // Update phase with the correct timings
    const t_start = Date.now() / 1000; // ms -> s for PSQL
    const t_clearing = t_start + 60 * 3;
    const t_end = t_clearing + 60 * 3;
    try {
      await db.query(
        `UPDATE phases 
      SET 
        start_time=to_timestamp($1),
        clearing_time=to_timestamp($2),
        planning_time=to_timestamp($3)
      WHERE session_id=$4 AND phase_no=$5`,
        [t_start, t_clearing, t_end, session_id, next_phase_no]
      );
    } catch (err) {
      console.log(err);
    }
  }

  // Notify users that a new phase has started
  sendUpdateToUsers(session_id, "new-game-phase", {});
}
