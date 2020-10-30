/**
 * Defines the routes regarding portfolio management.
 *  GET /session/:session_id/user/:user_id/portfolio
 *  GET /session/:session_id/user/:user_id/conso
 *  PUT /session/:session_id/user/:user_id/planning
 *  GET /session/:session_id/user/:user_id/planning
 *  GET /session/:session_id/user/:user_id/results
 *  GET /session/:session_id/user/:user_id/game_results
 */

import express from "express";
import {
  checkPlanning,
  formatUserPlanning,
  insertPlanning,
} from "./plugins/plannings";
import {
  getSession,
  getUser,
  getPortfolio,
  uuid_regex,
  getCurrentConsoValue,
  getPlanning,
  addPlanningToPortfolio,
  getUserResults,
  CustomError,
  getUserGameResults,
  getPhaseRankings,
  getConsoForecast,
} from "./utils";

// ---------------------- Routing Functions

/**
 * Get the list of auctions that are currently open.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserPortfolio(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const portfolio = await getPortfolio(user_id);
    const portfolio_with_planning = await addPlanningToPortfolio(portfolio);
    res.json(portfolio_with_planning);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get the consumption value for the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserConso(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const conso = await getCurrentConsoValue(session_id, user_id);
    res.json({ conso_mw: conso });
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get the consumption forecast.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserConsoForecast(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const conso = await getConsoForecast(session_id, user_id);
    res.json(conso);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Put into the DB a user production planning.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function putUserPlanningRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    if (session.status !== "running")
      throw new CustomError("Error, the session is not running");
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const planning = await formatUserPlanning(req.body);
    await checkPlanning(planning);
    await insertPlanning(planning);
    res.status(201).end();
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get a user production planning.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserPlanningRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const planning = await getPlanning(session_id, user_id);
    const planning_ftm = planning.map((dispatch) => {
      return {
        user_id: dispatch.user_id,
        session_id: dispatch.session_id,
        plant_id: dispatch.plant_id,
        p_mw: dispatch.p_mw,
      };
    });
    res.status(200).json(planning_ftm);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get user's results for the last phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserResultsRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    const results = await getUserResults(session_id, user_id);
    res.json(results);
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get user's game results.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getUserGameResultsRoute(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const user_id = req.params.user_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);
    const user = await getUser(session_id, user_id);
    if (user === null)
      throw new CustomError("Error, no user found with this ID", 404);

    if (session.status !== "closed") {
      res.json([]);
    } else {
      const results = await getUserGameResults(session_id, user_id);
      res.json(results);
    }
  } catch (error) {
    if (error instanceof CustomError) {
      res.status(error.code).end(error.msg);
    } else {
      res.status(400).end();
      throw error;
    }
  }
}

/**
 * Get the rankings for the current phase.
 * @param req HTTP request
 * @param res HTTP response
 */
export async function getRankings(
  req: express.Request,
  res: express.Response
): Promise<void> {
  try {
    // DB checks
    const session_id = req.params.session_id;
    const session = await getSession(session_id);
    if (session === null)
      throw new CustomError("Error, no session found with this ID", 404);

    console.log("rankings");
    const results = await getPhaseRankings(session_id);
    console.log(results);
    res.json(results);
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

router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/portfolio`,
  getUserPortfolio
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/conso`,
  getUserConso
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/conso_forecast`,
  getUserConso
);
router.put(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/planning`,
  putUserPlanningRoute
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/planning`,
  getUserPlanningRoute
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/results`,
  getUserResultsRoute
);
router.get(
  `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/game_results`,
  getUserGameResultsRoute
);
router.get(`/session/:session_id(${uuid_regex})/rankings`, getRankings);

export default router;
