import db from "../db/index";
import { v4 as uuid, validate } from "uuid";
import {
  Session,
  User,
  Bid,
  PowerPlant,
  PowerPlantTemplate,
  ConsoForecast,
  ProductionPlanning,
  GamePhase,
  PowerPlantWithPlanning,
  PhaseResults,
  SessionOptions,
  ScenarioOptions,
  OTCEnergyExchangeNoIDs,
  OTCEnergyExchange,
} from "./types";

export class CustomError extends Error {
  msg: string;
  code: number;

  constructor(msg: string, code?: number, ...params) {
    super(...params);
    this.msg = msg;
    this.code = code || 400;
  }
}

export const uuid_regex =
  "[A-F0-9]{8}-[A-F0-9]{4}-4[A-F0-9]{3}-[89AB][A-F0-9]{3}-[A-F0-9]{12}";

/**
 * Get the list of registered users to a session. Returns an empty list if
 * no users are found.
 * @param session_id Session UUID
 */
export async function getSessionUsers(session_id: string): Promise<User[]> {
  return (
    await db.query(
      `SELECT 
        id, session_id, name, game_ready
      FROM users
      WHERE session_id=$1;`,
      [session_id]
    )
  ).rows;
}

/**
 * Insert a new user to the DB and return the inserted user_id.
 * Does not check if the user can be inserted or not.
 * @param session_id Session ID
 * @param username Username
 */
export async function insertNewUser(
  session_id: string,
  username: string
): Promise<string> {
  const user_id = uuid();
  await db.query(
    `INSERT INTO users 
      (id, session_id, name) 
    VALUES ($1, $2, $3);`,
    [user_id, session_id, username]
  );
  return user_id;
}

/**
 * Get a user Object. Returns `null` if it's not found.
 * @param session_id Session UUID
 * @param user_id User UUID
 */
export async function getUser(
  session_id: string,
  user_id: string
): Promise<User> {
  const user = (
    await db.query(
      `SELECT 
        id, 
        session_id, 
        name, 
        game_ready
      FROM users 
      WHERE 
        id=$1 
        AND session_id=$2;`,
      [user_id, session_id]
    )
  ).rows;
  return user.length === 1 ? user[0] : null;
}

/**
 * Check if a given username can be registered to a session : either the username
 * is not taken, or there is no user in game of a solo session.
 *
 * Return an integer :
 * - 1 if the user can be inserted
 * - -1 is the session cannot accept new users
 * - -2 if the username already exists
 * @param session_id Session UUID
 * @param username Username to be registered
 */
export async function checkUsername(
  session_id: string,
  username: string
): Promise<number> {
  // Check if solo game with already one user
  const options = await getSessionOptions(session_id);
  if (!options.multi_game) {
    const n_users = (
      await db.query(
        `SELECT
          COUNT(*) AS n_users
        FROM users
        WHERE session_id=$1;`,
        [session_id]
      )
    ).rows[0].n_users;
    return n_users > 0 ? -1 : 1;
  } else {
    // Check username
    const users_same_name = (
      await db.query(
        `SELECT 1
      FROM users 
      WHERE 
        name=$1 
        AND session_id=$2;`,
        [username, session_id]
      )
    ).rows;
    return users_same_name.length === 0 ? 1 : -2;
  }
}

/**
 * Return the number of the active phase (with status 'open'), and `null`
 * if no phase is found.
 * @param session_id ID of the session
 */
export async function getCurrentPhaseNo(session_id: string): Promise<number> {
  const res = (
    await db.query(
      `SELECT phase_no 
      FROM phases 
      WHERE 
        session_id=$1 
        AND status='open';`,
      [session_id]
    )
  ).rows;
  return res.length === 1 ? (res[0].phase_no as number) : null;
}

/**
 * Return the number of the last phase, regardless of its active state
 * or not.
 * @param session_id Session ID
 */
export async function getLastPhaseNo(session_id: string): Promise<number> {
  const res = (
    await db.query(
      `SELECT phase_no 
      FROM phases
      WHERE session_id=$1
      ORDER BY phase_no DESC
      LIMIT 1;`,
      [session_id]
    )
  ).rows;
  return res.length === 1 ? (res[0].phase_no as number) : null;
}

/**
 * Return a scenario's default portfolio.
 * @param scenario_id Scenario ID
 */
async function getScenarioDefaultPortfolio(
  scenario_id
): Promise<PowerPlantTemplate[]> {
  return (
    await db.query(
      `SELECT 
        type,
        p_min_mw,
        p_max_mw,
        stock_max_mwh,
        stock_start_mwh,
        price_eur_per_mwh
      FROM scenarios_power_plants
      WHERE scenario_id=$1`,
      [scenario_id]
    )
  ).rows;
}

/**
 * Generate and insert into the DB a default portfolio for the user.
 * @param user_id User ID
 */
export async function setDefaultPortfolio(
  session_id: string,
  user_id: string
): Promise<void> {
  const scenario_id = await getScenarioID(session_id);
  const pps = await getScenarioDefaultPortfolio(scenario_id);
  await Promise.all(
    pps.map(async (pp) => {
      await db.query(
        `INSERT INTO power_plants 
          (
            id, 
            session_id, 
            user_id, 
            type, 
            p_min_mw, 
            p_max_mw, 
            stock_max_mwh,
            stock_start_mwh,
            price_eur_per_mwh
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);`,
        [
          uuid(),
          session_id,
          user_id,
          pp.type,
          pp.p_min_mw,
          pp.p_max_mw,
          pp.stock_max_mwh,
          pp.stock_start_mwh,
          pp.price_eur_per_mwh,
        ]
      );
    })
  );
}

/**
 * Get the user's portfolio, i.e. the list of power plants owned by the user.
 * @param user_id User ID
 */
export async function getPortfolio(user_id: string): Promise<PowerPlant[]> {
  return (
    await db.query(
      `SELECT 
        id,
        session_id, 
        user_id, type, 
        p_min_mw, 
        p_max_mw, 
        stock_max_mwh,
        stock_start_mwh,
        price_eur_per_mwh
      FROM power_plants 
      WHERE user_id=$1;`,
      [user_id]
    )
  ).rows;
}
/**
 * Add power plants dispatch information to a user's portfolio from its
 * production planning.
 * @param portfolio User portfolio
 */
export async function addPlanningToPortfolio(
  portfolio: PowerPlant[]
): Promise<PowerPlantWithPlanning[]> {
  if (portfolio.length > 0) {
    const session_id = portfolio[0].session_id;
    const user_id = portfolio[0].user_id;
    const planning = await getUserLastPhasePlanning(session_id, user_id);
    return portfolio.map((pp) => {
      const plan = planning.find((p) => p.plant_id === pp.id);
      return {
        ...pp,
        planning: plan === undefined ? 0 : plan.p_mw,
        stock_mwh:
          plan === undefined ? pp.stock_start_mwh : plan.stock_start_mwh,
      };
    });
  }
}

/**
 * Get the current consumption value for a given user.
 * @param session_id Session ID (UUID)
 * @param user_id User ID (UUID)
 * @param phase_no Optional, ID (int) of the current phase
 */
export async function getCurrentConsoValue(
  session_id: string,
  user_id: string,
  phase_no?: number
): Promise<number> {
  if (phase_no === undefined) {
    phase_no = await getLastPhaseNo(session_id);
  }
  if (phase_no !== null) {
    const rows: ConsoForecast[] = (
      await db.query(
        `SELECT
          value_mw
        FROM conso 
        WHERE 
          phase_no=$1 
          AND user_id=$2;`,
        [phase_no, user_id]
      )
    ).rows;
    return rows.length === 1 ? rows[0].value_mw : 0;
  } else {
    return 0;
  }
}

/**
 * Get the consumption forecast for a given user. Data before the current phase
 * (included) are actual data while data after the current phase are actual forecasts
 * depending on the scenario forecast type.
 * @param session_id Session ID
 * @param user_id User ID
 * @param phase_no Optional, ID (int) of the current phase
 */
export async function getConsoForecast(
  session_id: string,
  user_id: string,
  phase_no?: number
): Promise<number[]> {
  if (phase_no === undefined) {
    phase_no = await getLastPhaseNo(session_id);
  }
  const session_options = await getSessionOptions(session_id);
  let forecast = [];
  switch (session_options.conso_forecast_type) {
    case "none":
      forecast = [];
      break;
    case "perfect":
      forecast = session_options.conso_forecast_mwh;
      break;
    default:
      forecast = [];
      break;
  }
  return forecast;
}

/**
 * Get user's results for the last phase.
 * @param session_id Session ID
 * @param user_id User ID
 * @param phase_no Optional, phase ID (int)
 */
export async function getUserPhaseResults(
  session_id: string,
  user_id: string,
  phase_no?: number
): Promise<PhaseResults> {
  if (phase_no === undefined) {
    phase_no = await getLastPhaseNo(session_id);
  }
  let results = {} as PhaseResults;
  if (phase_no !== null) {
    const rows: PhaseResults[] = (
      await db.query(
        `SELECT
          conso_mwh,
          conso_eur,
          prod_mwh,
          prod_eur,
          sell_mwh,
          sell_eur,
          buy_mwh,
          buy_eur,
          imbalance_mwh,
          imbalance_costs_eur,
          balance_eur,
          ranking_current,
          ranking_overall
        FROM results 
        WHERE 
          phase_no=$1 
          AND user_id=$2;`,
        [phase_no, user_id]
      )
    ).rows;
    if (rows.length === 1) results = rows[0];
  }
  return results;
}

/**
 * Get user's results for the whole game.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserAllPhasesResults(
  session_id: string,
  user_id: string
): Promise<PhaseResults[]> {
  return (
    await db.query(
      `SELECT
        phase_no,
        conso_mwh,
        conso_eur,
        prod_mwh,
        prod_eur,
        sell_mwh,
        sell_eur,
        buy_mwh,
        buy_eur,
        imbalance_mwh,
        imbalance_costs_eur,
        balance_eur,
        ranking_current,
        ranking_overall
      FROM results 
      WHERE 
        session_id=$1
        AND user_id=$2
      ORDER BY phase_no;`,
      [session_id, user_id]
    )
  ).rows as PhaseResults[];
}

/**
 * Get the phase and overall rankings for the current phase.
 * @param session_id Session ID
 * @param phase_no Optional, phase ID (int)
 */
export async function getPhaseRankings(
  session_id: string,
  phase_no?: number
): Promise<{
  phase: { username: string; rank: number; balance: number }[];
  overall: { username: string; rank: number; balance: number }[];
}> {
  if (phase_no === undefined) {
    phase_no = await getLastPhaseNo(session_id);
  }
  const rankings = {
    phase: [],
    overall: [],
  };
  const rows = (
    await db.query(
      `SELECT
      r.ranking_current AS rank_phase,
      r.ranking_overall AS rank_overall,
      r.balance_eur AS balance_phase,
      r.balance_overall_eur AS balance_overall,
      u.name AS username
    FROM results AS r
    INNER JOIN users AS u
      ON r.user_id=u.id
    WHERE 
      r.session_id=$1
      AND r.phase_no=$2;`,
      [session_id, phase_no]
    )
  ).rows as {
    rank_phase: number;
    rank_overall: number;
    username: string;
    balance_phase: number;
    balance_overall: number;
  }[];
  console.log(rows);
  rankings.phase = rows.map((u) => {
    return {
      username: u.username,
      rank: u.rank_phase,
      balance: u.balance_phase,
    };
  });
  rankings.overall = rows.map((u) => {
    return {
      username: u.username,
      rank: u.rank_overall,
      balance: u.balance_overall,
    };
  });
  return rankings;
}

/**
 * Post a user user bit to the current open phase.
 * @param bid Bid object (without the phase_no)
 */
export async function postBid(bid: Omit<Bid, "phase_no">): Promise<void> {
  const phase_no = await getCurrentPhaseNo(bid.session_id);
  await db.query(
    `INSERT INTO bids 
      (
        id, 
        user_id, 
        session_id, 
        phase_no, 
        type, 
        volume_mwh, 
        price_eur_per_mwh
      ) 
      VALUES ($1, $2, $3, $4, $5, $6, $7);`,
    [
      bid.id,
      bid.user_id,
      bid.session_id,
      phase_no,
      bid.type,
      bid.volume_mwh,
      bid.price_eur_per_mwh,
    ]
  );
}

/**
 * Get a user's bid for the active step of an session. Return null if the
 * user has not bid yet.
 * @param session_id ID of the session
 * @param bid_id ID of the bid
 */
export async function getUserBid(
  session_id: string,
  bid_id: string
): Promise<Bid> {
  const phase_no = await getCurrentPhaseNo(session_id);
  const res = (
    await db.query(
      `SELECT
        id, 
        user_id, 
        session_id, 
        phase_no, 
        type, 
        volume_mwh, 
        price_eur_per_mwh
      FROM bids 
      WHERE 
        session_id=$1 
        AND id=$2 
        AND phase_no=$3;`,
      [session_id, bid_id, phase_no]
    )
  ).rows;
  return res.length === 1 ? (res[0] as Bid) : null;
}

/**
 * Get a user's bid for the active step of an session. Return null if the
 * user has not bid yet.
 * @param session_id ID of the session
 * @param bid_id ID of the bid
 */
export async function deleteUserBid(
  session_id: string,
  bid_id: string
): Promise<void> {
  const phase_no = await getCurrentPhaseNo(session_id);
  await db.query(
    `DELETE FROM bids 
    WHERE 
      session_id=$1 
      AND id=$2 
      AND phase_no=$3;`,
    [session_id, bid_id, phase_no]
  );
}

/**
 * Returns a list of of all user's bids for the current phase.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserBids(
  session_id: string,
  user_id: string,
  phase_no?: number
): Promise<Bid[]> {
  if (phase_no === undefined) {
    phase_no = await getCurrentPhaseNo(session_id);
  }
  let bids = [];
  if (phase_no !== null) {
    bids = (
      await db.query(
        `SELECT 
          id, 
          user_id, 
          session_id, 
          phase_no, 
          type, 
          volume_mwh, 
          price_eur_per_mwh
        FROM bids 
        WHERE 
          session_id=$1 
          AND user_id=$2 
          AND phase_no=$3;`,
        [session_id, user_id, phase_no]
      )
    ).rows;
  }
  return bids;
}

/**
 * Return all the bids for the active step of an sessions
 * @param sessions_id ID of the sessions
 */
export async function getAllBids(sessions_id: string): Promise<Bid[]> {
  const phase_no = await getCurrentPhaseNo(sessions_id);
  const res = (
    await db.query(
      `SELECT 
        id, 
        user_id, 
        session_id, 
        phase_no, 
        type, 
        volume_mwh, 
        price_eur_per_mwh
      FROM bids 
      WHERE 
        session_id=$1 
        AND phase_no=$2;`,
      [sessions_id, phase_no]
    )
  ).rows as Bid[];
  return res.length > 0 ? res : null;
}

/**
 * Returns the production planning (list of power plants dispatch) of a user.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserLastPhasePlanning(
  session_id: string,
  user_id: string
): Promise<ProductionPlanning> {
  const phase_no = await getLastPhaseNo(session_id);
  if (phase_no !== null) {
    return (
      await db.query(
        `SELECT
          user_id, 
          session_id, 
          phase_no, 
          plant_id, 
          p_mw, 
          stock_start_mwh, 
          stock_end_mwh
        FROM production_plannings 
        WHERE  
          session_id=$1 
          AND user_id=$2 
          AND phase_no=$3;`,
        [session_id, user_id, phase_no]
      )
    ).rows as ProductionPlanning;
  } else {
    return [];
  }
}

/**
 * Returns the production plannings (list of power plants dispatch) of a user
 * for all previous phases.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserAllPhasesPlanning(
  session_id: string,
  user_id: string
): Promise<
  {
    phase_no: number;
    plant_id: string;
    p_dispatch_mw: number;
    stock_start_mwh: number;
    stock_end_mwh: number;
    type: PowerPlant["type"];
  }[]
> {
  return (
    await db.query(
      `SELECT
        plannings.phase_no AS phase_no, 
        plannings.plant_id AS plant_id, 
        plannings.p_mw AS p_dispatch_mw, 
        plannings.stock_start_mwh AS stock_start_mwh, 
        plannings.stock_end_mwh AS stock_end_mwh,
        pp.type AS type
      FROM production_plannings AS plannings
      INNER JOIN power_plants AS pp
        ON pp.id = plannings.plant_id
      WHERE  
        plannings.session_id=$1 
        AND plannings.user_id=$2
      ORDER BY plannings.phase_no;`,
      [session_id, user_id]
    )
  ).rows as {
    phase_no: number;
    plant_id: string;
    p_dispatch_mw: number;
    stock_start_mwh: number;
    stock_end_mwh: number;
    type: PowerPlant["type"];
  }[];
}

/**
 * Returns a GamePhase object for corresponding phase. Return `null` if not
 * phase is found.
 * @param session_id Session ID
 * @param phase_no Phase number (int)
 */
export async function getPhaseInfos(
  session_id: string,
  phase_no: number
): Promise<GamePhase> {
  const rows = (
    await db.query(
      `SELECT
        session_id,
        phase_no,
        start_time,
        clearing_time,
        planning_time,
        bids_allowed,
        clearing_available,
        plannings_allowed,
        results_available,
        status
      FROM phases 
      WHERE 
        session_id=$1 
        AND phase_no=$2;`,
      [session_id, phase_no]
    )
  ).rows as GamePhase[];
  if (rows.length > 0) return rows[0];
  else return null;
}

/**
 * Returns a GamePhase object for the last phase. Returns `null` if
 * there is no active phase.
 * @param session_id Session ID
 */
export async function getLastPhaseInfos(
  session_id: string
): Promise<GamePhase> {
  const rows = (
    await db.query(
      `SELECT
        session_id,
        phase_no,
        start_time,
        clearing_time,
        planning_time,
        bids_allowed,
        clearing_available,
        plannings_allowed,
        results_available,
        status
      FROM phases 
      WHERE session_id=$1 
      ORDER BY phase_no DESC;`,
      [session_id]
    )
  ).rows as GamePhase[];
  if (rows.length > 0) return rows[0];
  else return null;
}

/**
 * Return session's total number of phases form its scenario. Return `null`
 * if cannot find scenario.
 * @param session_id Session ID
 */
export async function getSessionNbPhases(session_id: string): Promise<number> {
  const row = (
    await db.query(
      `SELECT 
        phases_number
      FROM scenarios_options AS so
      INNER JOIN sessions AS s
        ON so.id = s.scenario_id
      WHERE 
        s.id=$1`,
      [session_id]
    )
  ).rows;

  return row.length === 1 ? row[0].phases_number : null;
}

/**
 * Check if users can submit bids to the current phase.
 * @param session_id Session ID
 */
export async function userCanBid(session_id: string): Promise<boolean> {
  const phase_no = await getCurrentPhaseNo(session_id);
  let bids_allowed = false;
  if (phase_no !== null) {
    const rows = (
      await db.query(
        `SELECT bids_allowed 
        FROM phases
        WHERE 
          session_id=$1 
          AND phase_no=$2;`,
        [session_id, phase_no]
      )
    ).rows;
    bids_allowed =
      rows.length === 1 ? (rows[0].bids_allowed as boolean) : false;
  }
  return bids_allowed;
}

/**
 * Check if users can submit plannings for the current phase.
 * @param session_id Session ID
 */
export async function userCanSubmitPlanning(
  session_id: string
): Promise<boolean> {
  const phase_no = await getCurrentPhaseNo(session_id);
  let plannings_allowed = false;
  if (phase_no !== null) {
    const rows = (
      await db.query(
        `SELECT plannings_allowed 
        FROM phases 
        WHERE 
          session_id=$1 
          AND phase_no=$2;`,
        [session_id, phase_no]
      )
    ).rows;
    plannings_allowed =
      rows.length === 1 ? (rows[0].plannings_allowed as boolean) : false;
  }
  return plannings_allowed;
}

interface PhaseBooleans {
  bids_allowed: boolean;
  clearing_available: boolean;
  plannings_allowed: boolean;
  results_available: boolean;
}

export async function getSessionBooleans(
  session_id: string
): Promise<PhaseBooleans> {
  let bools = {
    bids_allowed: false,
    clearing_available: false,
    plannings_allowed: false,
    results_available: false,
  };
  const rows = (
    await db.query(
      `SELECT 
        bids_allowed, 
        clearing_available, 
        plannings_allowed, 
        results_available 
      FROM phases 
      WHERE session_id=$1 
      ORDER BY phase_no DESC;`,
      [session_id]
    )
  ).rows;
  if (rows.length > 0) {
    bools = rows[0] as PhaseBooleans;
  }
  return bools;
}

/**
 * Return the clearing information.
 * @param session_id Session ID
 */
export async function getClearing(
  session_id: string
): Promise<{
  phase_id: number;
  volume_mwh: number;
  price_eur_per_mwh: number;
}> {
  const clearing = (
    await db.query(
      `SELECT 
        phase_no, 
        volume_mwh, 
        price_eur_per_mwh 
      FROM clearings 
      WHERE session_id=$1 
      ORDER BY phase_no DESC;`,
      [session_id]
    )
  ).rows;
  return clearing.length > 0
    ? (clearing[0] as {
        phase_id: number;
        volume_mwh: number;
        price_eur_per_mwh: number;
      })
    : ({} as {
        phase_id: number;
        volume_mwh: number;
        price_eur_per_mwh: number;
      });
}

/**
 * Return anonymously all the bids for a given phase after it has cleared.
 * User's bids get a `own_bid` flag set to `true`.
 * @param session_id Session ID
 * @param user_id Optional, user ID
 * @param phase_no Optional, id (int) of the phase
 */
export async function getClearedPhaseBids(
  session_id: string,
  user_id?: string,
  phase_no?: number
): Promise<
  {
    type: "buy" | "sell";
    volume_mwh: number;
    price_eur_per_mwh: number;
  }[]
> {
  if (phase_no === undefined) {
    const req_phase = (
      await db.query(
        `SELECT phase_no
        FROM phases
        WHERE session_id=$1
        ORDER BY phase_no DESC;`,
        [session_id]
      )
    ).rows;
    phase_no = req_phase.length > 0 ? req_phase[0].phase_no : null;
  }
  if (phase_no !== null) {
    const bids = (
      await db.query(
        `SELECT
          b.type,
          b.volume_mwh,
          b.price_eur_per_mwh,
          CASE 
            WHEN b.user_id=$1 THEN true
            ELSE false
          END AS own_bid
        FROM bids AS b
        INNER JOIN phases AS p
          ON (
            b.phase_no=p.phase_no
            AND b.session_id=p.session_id
          )
        WHERE
          b.session_id=$2
          AND b.phase_no=$3
          AND p.clearing_available=true;`,
        [user_id, session_id, phase_no]
      )
    ).rows;
    return bids as {
      type: "buy" | "sell";
      volume_mwh: number;
      price_eur_per_mwh: number;
    }[];
  } else {
    return [];
  }
}

/**
 * Return the user's energy exchanges following bids clearing.
 * @param session_id Session ID
 */
export async function getUserEnergyExchanges(
  session_id: string,
  user_id: string
): Promise<
  {
    type: "buy" | "sell";
    volume_mwh: number;
    price_eur_per_mwh: number;
  }[]
> {
  const req_phase = (
    await db.query(
      `SELECT phase_no 
      FROM phases 
      WHERE session_id=$1 
      ORDER BY phase_no DESC;`,
      [session_id]
    )
  ).rows;
  const phase_no = req_phase.length > 0 ? req_phase[0].phase_no : null;
  if (phase_no !== null) {
    const exchanges = (
      await db.query(
        `SELECT 
          type, 
          volume_mwh, 
          price_eur_per_mwh 
        FROM exchanges 
        WHERE 
          session_id=$1 
          AND user_id=$2 
          AND phase_no=$3;`,
        [session_id, user_id, phase_no]
      )
    ).rows;
    return exchanges.length > 0
      ? (exchanges as {
          type: "buy" | "sell";
          volume_mwh: number;
          price_eur_per_mwh: number;
        }[])
      : [];
  } else {
    return [];
  }
}

/**
 * Return the session games options.
 * @param session_id Session ID
 */
export async function getSessionOptions(
  session_id: string
): Promise<SessionOptions> {
  let options = {
    scenario_id: "",
    multi_game: false,
    bids_duration_sec: 0,
    plannings_duration_sec: 0,
    phases_number: 0,
    conso_forecast_mwh: [],
    conso_forecast_type: "none" as SessionOptions["conso_forecast_type"],
    conso_price_eur: [],
    imbalance_costs_factor: [],
  };
  const query = (
    await db.query(
      `
      SELECT 
        scenario_id,
        multi_game,
        bids_duration_sec,
        plannings_duration_sec,
        phases_number,
        conso_forecast_mwh,
        conso_forecast_type,
        conso_price_eur,
        imbalance_costs_factor
      FROM options
      WHERE session_id=$1;`,
      [session_id]
    )
  ).rows;
  if (query.length === 1) options = query[0];
  return options;
}

/**
 * Return the scenario ID of a session.
 * @param session_id Session ID
 */
export async function getScenarioID(session_id: string): Promise<string> {
  const rows = (
    await db.query(
      `SELECT scenario_id
    FROM options
    WHERE session_id=$1`,
      [session_id]
    )
  ).rows;
  return rows.length === 1 ? rows[0].scenario_id : null;
}

/**
 * Insert a new session record and its corresponding options.
 * @param session Session object
 */
export async function createNewSession(session: Session): Promise<void> {
  // Create new session
  await db.query(
    `INSERT INTO sessions 
      (
        name, 
        id, 
        status,
        scenario_id
      ) 
    VALUES($1, $2, $3, $4);`,
    [session.name, session.id, session.status, session.scenario_id]
  );

  const scenario_options = await getScenarioOptions(session.scenario_id);
  await db.query(
    `INSERT INTO options
      (
        session_id, 
        scenario_id,
        multi_game,
        bids_duration_sec,
        plannings_duration_sec,
        phases_number,
        conso_forecast_mwh,
        conso_forecast_type,
        conso_price_eur,
        imbalance_costs_factor
      )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`,
    [
      session.id,
      session.scenario_id,
      scenario_options.multi_game,
      scenario_options.bids_duration_sec,
      scenario_options.plannings_duration_sec,
      scenario_options.phases_number,
      scenario_options.conso_forecast_mwh,
      scenario_options.conso_forecast_type,
      scenario_options.conso_price_eur,
      scenario_options.imbalance_costs_factor,
    ]
  );
}

type ScenarioInfos = Pick<
  ScenarioOptions,
  "id" | "name" | "description" | "difficulty" | "multi_game"
>;
/**
 * Return a list of base informations on available scenarios.
 */
export async function getScenariosList(): Promise<ScenarioInfos[]> {
  return (
    await db.query(
      `SELECT 
        id,
        name,
        description,
        difficulty,
        multi_game
      FROM scenarios_options`,
      []
    )
  ).rows;
}

/**
 * Return the options of a scenario by its ID.
 * @param scenario_id Scenario ID
 */
export async function getScenarioOptions(
  scenario_id: string
): Promise<ScenarioOptions> {
  let scenario_options = {} as ScenarioOptions;
  const res = (
    await db.query(
      `SELECT
        id,
        name,
        description,
        difficulty,
        multi_game,
        bids_duration_sec,
        plannings_duration_sec,
        phases_number,
        conso_forecast_mwh,
        conso_forecast_type,
        conso_price_eur,
        imbalance_costs_factor
      FROM scenarios_options
      WHERE id=$1`,
      [scenario_id]
    )
  ).rows;
  if (res.length === 1) scenario_options = res[0];
  return scenario_options;
}

/**
 * Return the portfolio of a scenario by its ID.
 * @param scenario_id Scenario ID
 */
export async function getScenarioPortfolio(
  scenario_id: string
): Promise<PowerPlantTemplate[]> {
  const portfolio = (
    await db.query(
      `SELECT
        type,
        p_min_mw,
        p_max_mw,
        stock_max_mwh,
        price_eur_per_mwh
        
      FROM scenarios_power_plants
      WHERE scenario_id=$1`,
      [scenario_id]
    )
  ).rows as PowerPlantTemplate[];
  return portfolio;
}

/**
 * Check if a session name already exists, and return `true` if not.
 * @param session_name Session name
 */
export async function checkSessionName(session_name: string): Promise<boolean> {
  return (
    (
      await db.query(
        `SELECT id 
        FROM sessions 
        WHERE name=$1`,
        [session_name]
      )
    ).rows.length === 0
  );
}

/**
 * Check if a scenario with a given ID exists, return true if yes.
 * @param scenario_id Scenario ID
 */
export async function checkScenarioID(scenario_id: string): Promise<boolean> {
  return (
    (
      await db.query(
        `SELECT id 
        FROM scenarios_options 
        WHERE id=$1`,
        [scenario_id]
      )
    ).rows.length === 1
  );
}

/**
 * Check by its name if a user belongs to a session. Return its UUID (string) if true,
 * else return `null`.
 * @param session_id Session UID
 * @param username Username, string
 */
export async function checkUserInSessionByName(
  session_id: string,
  username: string
): Promise<string> {
  const rows = (
    await db.query(
      `SELECT
      id
    FROM users
    WHERE 
      session_id=$1
      AND name=$2;`,
      [session_id, username]
    )
  ).rows;
  return rows.length === 1 ? rows[0].id : null;
}

/**
 * Return the list a user's OTCs.
 * @param session_id Session UUID
 * @param user_id User UUID
 */
export async function getUserOTCs(
  session_id: string,
  user_id: string
): Promise<OTCEnergyExchangeNoIDs[]> {
  const phase_no = await getLastPhaseNo(session_id);
  console.log(phase_no);
  return (
    await db.query(
      `SELECT
        otc.id AS id,
        otc.session_id AS session_id,
        otc.phase_no AS phase_no,
        otc.type AS type,
        otc.volume_mwh AS volume_mwh,
        otc.price_eur_per_mwh AS price_eur_per_mwh,
        otc.status AS status, 
        u_from.name AS user_from,
        u_to.name AS user_to
      FROM otc_exchanges AS otc
      INNER JOIN users AS u_from
        ON otc.user_from_id=u_from.id
      INNER JOIN users AS u_to
        ON otc.user_to_id=u_to.id
      WHERE 
        (otc.user_from_id=$1
        OR otc.user_to_id=$1)
        AND otc.session_id=$2
        AND otc.phase_no=$3;`,
      [user_id, session_id, phase_no]
    )
  ).rows as OTCEnergyExchangeNoIDs[];
}

/**
 * Find and return an OTC by its ID.
 * @param otc_id OTC UUID
 */
export async function getOTCByID(otc_id: string): Promise<OTCEnergyExchange> {
  const rows = (
    await db.query(
      `SELECT 
        id,
        user_from_id,
        user_to_id,
        session_id,
        phase_no,
        type,
        volume_mwh,
        price_eur_per_mwh,
        status
      FROM otc_exchanges
      WHERE id=$1;`,
      [otc_id]
    )
  ).rows as OTCEnergyExchange[];
  return rows.length === 1 ? rows[0] : null;
}
