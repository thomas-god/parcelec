/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */
import db from "../../db";
import { sendUpdateToUsers } from "../websocket";

export async function endGame(
  session_id: string,
  phase_no: number
): Promise<void> {
  // Notify users that session is finished
  sendUpdateToUsers(session_id, "plannings-closed", {});

  // Mark plannings_allowed to false
  await db.query(
    "UPDATE phases SET plannings_allowed=false WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );

  // Do the metering and results computation, it may take some time
  await new Promise((r) => setTimeout(r, 10000));

  // Close the phase and mark all users as not ready for the next phase
  await db.query(
    "UPDATE phases SET status='closed' WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  await db.query("UPDATE users SET game_ready=false WHERE session_id=$1", [
    session_id,
  ]);

  // When results are computed, notify the users and mark it the phase table
  await db.query(
    "UPDATE phases SET results_available=true WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  sendUpdateToUsers(session_id, "results-available", {});
}
