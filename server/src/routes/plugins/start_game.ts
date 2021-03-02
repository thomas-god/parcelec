/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */

import { v4 as uuid } from 'uuid';
import {
  getLastPhaseInfos,
  getLastPhaseNo,
  getSession,
  getSessionOptions,
  getSessionUsers,
} from '../utils';
import db from '../../db';
import logger from '../../utils/log';
import { sendUpdateToUsers } from '../websocket';
import { Bid, Session } from '../types';
import clearing from './clearing';
import { endGame } from './end_game';
import { generateEmptyPlanning, insertPlanning } from './plannings';

/**
 * Hold the callbacks and timers IDs for the current phases.
 */
const callbacks = {};

/**
 * When all users are ready, check if an action can be performed:
 * - Start a new game if the session is open,
 * - Start a new game phase if the previous phase is closed,
 * - Do the clearing in advance if bids are still allowed,
 * - Do the results computation in advance if plannings are still allowed.
 */
export async function checkUserReadyAction(session_id: string): Promise<void> {
  const session = await getSession(session_id);
  const options = await getSessionOptions(session_id);
  const users = await getSessionUsers(session_id);
  const phase = await getLastPhaseInfos(session_id);

  const users_ready =
    (!options.multi_game || users.length >= 2) &&
    users.filter((u) => u.game_ready).length === users.length;

  if ((session.status === 'open' || phase.status === 'closed') && users_ready) {
    if (session.status === 'open') {
      // Update session status to prevent new users registration
      logger.info('session started', { session_id });
      await setSessionStatus(session, 'running');
    }
    startNewGamePhase(session_id);
    resetUsersReady(session_id);
  } else if (phase !== null && phase.bids_allowed === true && users_ready) {
    const t0_sec = Date.now() / 1000;
    // Clear the timer and directly call the clearing callback
    logger.info('skipping to clearing', {
      session_id,
      phase_no: phase.phase_no,
    });
    clearTimeout(callbacks[session_id].id_timer_clearing);
    callbacks[session_id].cb_clearing();

    // Update the timeout for the end_game callback and store the new timer
    clearTimeout(callbacks[session_id].id_timer_results);
    const dt_sec = options.plannings_duration_sec - options.bids_duration_sec;
    callbacks[session_id].id_timer_results = setTimeout(
      callbacks[session_id].cb_results,
      dt_sec * 1000
    );

    // Update the DB record
    await db.query(
      `UPDATE phases 
      SET 
        clearing_time=to_timestamp($1),
        planning_time=to_timestamp($2)
      WHERE session_id=$3 AND phase_no=$4`,
      [t0_sec, t0_sec + dt_sec, session_id, phase.phase_no]
    );
  } else if (
    phase !== null &&
    phase.clearing_available === true &&
    phase.plannings_allowed === true &&
    users_ready
  ) {
    logger.info('skipping to results', {
      session_id,
      phase_no: phase.phase_no,
    });
    const t0_sec = Date.now() / 1000;
    // Clear the timer and directly call the end_game callback
    clearTimeout(callbacks[session_id].id_timer_results);
    callbacks[session_id].cb_results();

    // Update the DB record
    await db.query(
      `UPDATE phases 
      SET 
        planning_time=to_timestamp($1)
      WHERE session_id=$2 AND phase_no=$3`,
      [t0_sec, session_id, phase.phase_no]
    );
  }
}

/**
 * Start a new game phase. Create a new phase record, set the timeouts and
 * callbacks for clearing and results.
 * @param session_id Session UUID
 */
export async function startNewGamePhase(session_id: string): Promise<void> {
  const session = await getSession(session_id);
  const options = await getSessionOptions(session_id);
  const users = await getSessionUsers(session_id);
  const current_phase_no = await getLastPhaseNo(session_id);
  const next_phase_no = current_phase_no === null ? 0 : current_phase_no + 1;

  // Insert a new phase item
  await db.query(
    'INSERT INTO phases (session_id, phase_no, status) VALUES ($1, $2, $3)',
    [session_id, next_phase_no, 'open']
  );

  // Generate conso forecast for each user
  const conso_value = options.conso_forecast_mwh[next_phase_no];
  await Promise.all(
    users.map(async (user) => {
      await db.query(
        'INSERT INTO conso (user_id, session_id, phase_no, value_mw) VALUES ($1, $2, $3, $4)',
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

  // Insert external bids from scenario (with no user attached)
  const bids: Bid[] = (
    await db.query(
      `SELECT
        type,
        volume_mwh,
        price_eur_per_mwh
      FROM scenarios_bids
      WHERE
        scenario_id=$1
        AND phase_no=$2`,
      [session.scenario_id, next_phase_no]
    )
  ).rows;
  await Promise.all(
    bids.map(async (bid) => {
      await db.query(
        `INSERT INTO bids
          (
            id,
            session_id,
            phase_no,
            type,
            volume_mwh,
            price_eur_per_mwh
          )
        VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          uuid(),
          session_id,
          next_phase_no,
          bid.type,
          bid.volume_mwh,
          bid.price_eur_per_mwh,
        ]
      );
    })
  );

  // Update phase with the correct timings and set the callbacks
  const t_start = Date.now() / 1000; // ms -> s for PSQL
  const t_clearing = t_start + options.bids_duration_sec;
  const t_end = t_start + options.plannings_duration_sec;
  const cb_clearing = () => {
    resetUsersReady(session_id);
    clearing(session_id, next_phase_no);
  };
  const id_timer_clearing = setTimeout(
    cb_clearing,
    options.bids_duration_sec * 1000
  );
  const cb_results = () => {
    resetUsersReady(session_id);
    endGame(session_id, next_phase_no);
  };
  const id_timer_results = setTimeout(
    cb_results,
    options.plannings_duration_sec * 1000
  );
  // Store callbacks and timer IDs for later use
  callbacks[session_id] = {
    cb_clearing,
    id_timer_clearing,
    cb_results,
    id_timer_results,
  };
  // Update the DB record
  await db.query(
    `UPDATE phases 
    SET 
      start_time=to_timestamp($1),
      clearing_time=to_timestamp($2),
      planning_time=to_timestamp($3)
    WHERE session_id=$4 AND phase_no=$5`,
    [t_start, t_clearing, t_end, session_id, next_phase_no]
  );

  logger.info('phase started', { session_id, phase_no: next_phase_no });

  // Notify users that a new phase has started
  sendUpdateToUsers(session_id, 'new-game-phase', {});
}

/**
 * Update the session game status (`open`, `running` or `closed`).
 * Update in place session object status.
 * @param session Session object
 * @param status New status
 */
async function setSessionStatus(
  session: Session,
  status: Session['status']
): Promise<void> {
  await db.query('UPDATE sessions SET status=$1 WHERE id=$2', [
    status,
    session.id,
  ]);
  session.status = status;
}

/**
 * Set all session's users `game_ready` status to false.
 * @param session_id Session UUID
 */
async function resetUsersReady(session_id: string): Promise<void> {
  await db.query(
    `UPDATE users
    SET game_ready=false
    WHERE session_id=$1;`,
    [session_id]
  );
  sendUpdateToUsers(session_id, 'reset-game-ready', {});
}
