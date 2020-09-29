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
 * Get a session from the DB by its UUID. Returns `null` if no
 * session is found.
 * @param session_id Session UUID
 */
export async function getSession(session_id: string): Promise<Session> {
  const session: Session[] = (
    await db.query(
      `SELECT 
        id, name, status
      FROM sessions 
      WHERE id=$1;`,
      [session_id]
    )
  ).rows;
  return session.length === 1 ? session[0] : null;
}

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
 * Check if a given username can be registered to an session (i.e. is
 * not already registered). Return `true` if the user can be inserted
 * with this username.
 * @param session_id Session UUID
 * @param username Username to be registered
 */
export async function checkUsername(
  session_id: string,
  username: string
): Promise<boolean> {
  const users = (
    await db.query(
      `SELECT 1
      FROM users 
      WHERE 
        name=$1 
        AND session_id=$2;`,
      [username, session_id]
    )
  ).rows;
  return users.length === 0;
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
            price_eur_per_mwh
          )
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8);`,
        [
          uuid(),
          session_id,
          user_id,
          pp.type,
          pp.p_min_mw,
          pp.p_max_mw,
          pp.stock_max_mwh,
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
    const planning = await getPlanning(session_id, user_id);
    return portfolio.map((pp) => {
      const plan = planning.find((p) => p.plant_id === pp.id);
      return {
        ...pp,
        planning: plan === undefined ? 0 : plan.p_mw,
        stock_mwh: plan === undefined ? pp.stock_max_mwh : plan.stock_start_mwh,
      };
    });
  }
}

/**
 * Get the current conso forecast for a given user.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getConsoForecast(
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
 * Get the current conso forecast for a given user.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserResults(
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
          balance_eur
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
export async function getPlanning(
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
 * Returns the a GamePhase object for the current phase. Returns `null` if
 * there is no active phase.
 * @param session_id Session ID
 */
export async function getPhaseInfos(session_id: string): Promise<GamePhase> {
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
        results_available
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
  const phase_no = req_phase.length === 1 ? req_phase[0].phase_no : null;
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
    conso_price_eur: [],
    imbalance_costs_eur: [],
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
        conso_price_eur,
        imbalance_costs_eur
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
  // Insert default scenario options if no scenario provided
  if (session.scenario_id === undefined || !validate(session.scenario_id)) {
    session.scenario_id = await getDefaultScenarioID();
  }
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
        conso_price_eur,
        imbalance_costs_eur
      )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
    [
      session.id,
      session.scenario_id,
      scenario_options.multi_game,
      scenario_options.bids_duration_sec,
      scenario_options.plannings_duration_sec,
      scenario_options.phases_number,
      scenario_options.conso_forecast_mwh,
      scenario_options.conso_price_eur,
      scenario_options.imbalance_costs_eur,
    ]
  );
}

/**
 * Return the ID of the default game scenario.
 * If the default scenario does not exists, it is created along
 * with its default portfolio.
 */
export async function getDefaultScenarioID(): Promise<string> {
  let id = "";
  const res = (
    await db.query(
      `SELECT id
    FROM scenarios_options
    WHERE name='default';`,
      []
    )
  ).rows;
  if (res.length === 1) {
    id = res[0].id;
  } else {
    id = await generateDefaultScenario();
  }
  return id;
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
 * Insert into the database the default scenario options and power plants
 * list.
 */
export async function generateDefaultScenario(): Promise<string> {
  const id = uuid();
  const default_options = {
    id: id,
    name: "default",
    difficulty: "easy",
    description:
      "Le scénario par défaut vous permet de prendre en main les fonctionnalités de parcelec.",
    multi_game: false,
    bids_duration_sec: 180,
    plannings_duration_sec: 300,
    phases_number: 3,
    conso_forecast_mwh: [1000, 1800, 2400],
    conso_price_eur: [35, 35, 35],
    imbalance_costs_eur: [20, 30, 40],
  };

  await db.query(
    `INSERT INTO scenarios_options
    (
      id,
      name,
      difficulty,
      description,
      multi_game,
      bids_duration_sec,
      plannings_duration_sec,
      phases_number,
      conso_forecast_mwh,
      conso_price_eur,
      imbalance_costs_eur
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);`,
    [
      default_options.id,
      default_options.name,
      default_options.difficulty,
      default_options.description,
      default_options.multi_game,
      default_options.bids_duration_sec,
      default_options.plannings_duration_sec,
      default_options.phases_number,
      default_options.conso_forecast_mwh,
      default_options.conso_price_eur,
      default_options.imbalance_costs_eur,
    ]
  );

  const default_power_plants = [
    {
      type: "nuc",
      p_min_mw: 400,
      p_max_mw: 1300,
      stock_max_mwh: -1,
      price_eur_per_mwh: 17,
    },
    {
      type: "therm",
      p_min_mw: 150,
      p_max_mw: 600,
      stock_max_mwh: -1,
      price_eur_per_mwh: 65,
    },
    {
      type: "hydro",
      p_min_mw: 50,
      p_max_mw: 500,
      stock_max_mwh: 5000,
      price_eur_per_mwh: 0,
    },
  ];

  await Promise.all(
    default_power_plants.map(async (pp) => {
      await db.query(
        `INSERT INTO scenarios_power_plants
        (
          scenario_id,
          type,
          p_min_mw,
          p_max_mw,
          stock_max_mwh,
          price_eur_per_mwh
        )
        VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          id,
          pp.type,
          pp.p_min_mw,
          pp.p_max_mw,
          pp.stock_max_mwh,
          pp.price_eur_per_mwh,
        ]
      );
    })
  );
  return id;
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
        conso_price_eur,
        imbalance_costs_eur
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
