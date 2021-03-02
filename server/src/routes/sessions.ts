/**
 * GET /scenarios
 * GET /scenario/:scenario_id
 * GET /sessions/open
 * PUT /session/
 * GET /session/:session_id
 */

import { v4 as uuidv4 } from 'uuid';
import express from 'express';
import db from '../db/index';
import { Session } from './types';
import {
  checkScenarioID,
  checkSessionName,
  createNewSession,
  CustomError,
  getLastPhaseInfos,
  getScenarioOptions,
  getScenarioPortfolio,
  getScenariosList,
  getSession,
  getSessionBooleans,
  getSessionNbPhases,
  getSessionOptions,
  getSessionUsers,
  uuid_regex,
} from './utils';
import generateDefaultScenarios from './plugins/default_scenarios';

// ---------------------- Routing Functions

/**
 * Get the list of sessions that are currently open.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getOpenSessions(
  req: express.Request,
  res: express.Response
): Promise<void> {
  const sessions: Session[] = (
    await db.query(
      `
      WITH os AS 
      (
        SELECT 
          id AS session_id, 
          name 
        FROM sessions 
        WHERE status='open'
      ),
      n_users AS 
      (
        SELECT
          COUNT(*) AS n_users,
          session_id
        FROM users
        GROUP BY session_id
      ),
      solo AS 
      (
        SELECT
          session_id,
          multi_game
        FROM options
      )
      SELECT 
        os.session_id AS id,
        os.name
      FROM os
      LEFT JOIN solo
        ON os.session_id=solo.session_id
      LEFT JOIN n_users
        ON os.session_id=n_users.session_id
      WHERE
        solo.multi_game
        OR COALESCE(n_users.n_users, 0)=0;`,
      []
    )
  ).rows;
  res.json(sessions);
}

/**
 * Get the list of available scenarios.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getScenarios(
  req: express.Request,
  res: express.Response
): Promise<void> {
  let scenarios = await getScenariosList();
  if (scenarios.length === 0) {
    await generateDefaultScenarios();
    scenarios = await getScenariosList();
  }
  res.json(scenarios);
}

/**
 * Get the options (portfolio, session options) of a given scenario.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getScenarioOptionsRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  const scenario_id: string = req.params.scenario_id;
  if (!(await checkScenarioID(scenario_id))) {
    throw new CustomError('Error, no scenario found with this ID.');
  }

  const scenario_options = await getScenarioOptions(scenario_id);
  const scenario_portfolio = await getScenarioPortfolio(scenario_id);
  res.json({
    options: scenario_options,
    portfolio: scenario_portfolio,
  });
}

/**
 * Open a new session with a user provided name. Name must be unique.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function openNewSession(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    const session_name: string = req.body.session_name;
    const scenario_id: string = req.body.scenario_id;
    // Checks
    if (session_name === undefined) {
      throw new CustomError('Error, please provide a valid game session name');
    }

    if (!(await checkSessionName(session_name))) {
      throw new CustomError(
        'Error, a session already exists with this name',
        409
      );
    }

    if (scenario_id === undefined || !(await checkScenarioID(scenario_id))) {
      throw new CustomError('Error, please provide a valid scenario ID');
    }

    // Insertion
    const session: Session = {
      name: session_name,
      id: uuidv4(),
      status: 'open',
      scenario_id: scenario_id,
    };
    await createNewSession(session);
    res.status(201).json(session);
  } catch (error) {
    res.status(error.code).end(error.msg);
    return;
  }
}

/**
 * Get informations for a specific session (status, step_no)
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getSessionInfos(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null) {
      throw new CustomError(
        'Error, the session_id does not correspond to an existing session',
        404
      );
    }

    // Base infos
    let body: any = {
      id: session.id,
      name: session.name,
      status: session.status,
    };

    // Session options
    const options = await getSessionOptions(session_id);
    body.multi_game = options.multi_game;

    // Session booleans
    const bools = await getSessionBooleans(session_id);
    body = { ...body, ...bools };

    // List of users
    body.users = (await getSessionUsers(session_id))
      .map((user) => {
        return { name: user.name, ready: user.game_ready };
      })
      .sort((a, b) => (a.name > b.name ? 1 : -1));

    // Timing infos
    const phase_infos = await getLastPhaseInfos(session_id);
    if (phase_infos !== null) {
      const nb_phases = await getSessionNbPhases(session_id);
      body.phase_infos = {
        start_time: phase_infos.start_time,
        clearing_time: phase_infos.clearing_time,
        planning_time: phase_infos.planning_time,
        phase_no: phase_infos.phase_no,
        nb_phases: nb_phases,
      };
    }

    res.json(body);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

const router = express.Router();

router.get('/scenarios', getScenarios);
router.get(`/scenario/:scenario_id(${uuid_regex})`, getScenarioOptionsRoute);
router.get('/sessions/open', getOpenSessions);
router.put('/session/', openNewSession);
router.get(`/session/:session_id(${uuid_regex})`, getSessionInfos);

export default router;
