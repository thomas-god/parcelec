import db from "../db/index";
import { v4 as uuid } from "uuid";
import {
  Session,
  User,
  Bid,
  PowerPlant,
  PowerPlantTemplate,
  ConsoForecast,
  ProductionPlanning,
  GamePhase,
} from "./types";

export const uuid_regex =
  "[A-F0-9]{8}-[A-F0-9]{4}-4[A-F0-9]{3}-[89AB][A-F0-9]{3}-[A-F0-9]{12}";

/**
 * Get a session from the DB by its UUID. Returns `null` if no
 * session is found.
 * @param session_id Session UUID
 */
export async function getSession(session_id: string): Promise<Session> {
  const session: Session[] = (
    await db.query("SELECT * FROM sessions WHERE id=$1", [session_id])
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
    await db.query("SELECT * FROM users WHERE session_id=$1", [session_id])
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
    "INSERT INTO users (id, session_id, name) VALUES ($1, $2, $3)",
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
    await db.query("SELECT * FROM users WHERE id=$1 AND session_id=$2", [
      user_id,
      session_id,
    ])
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
    await db.query("SELECT * FROM users WHERE name=$1 AND session_id=$2", [
      username,
      session_id,
    ])
  ).rows;
  return users.length === 0;
}

/**
 * Return the number of the active phase (with status 'open').
 * @param session_id ID of the session
 */
export async function getCurrentPhaseNo(session_id: string): Promise<number> {
  const res = (
    await db.query(
      "SELECT phase_no FROM phases WHERE session_id=$1 AND status='open'",
      [session_id]
    )
  ).rows;
  return res.length === 1 ? (res[0].phase_no as number) : null;
}

const power_plants_base: PowerPlantTemplate[] = [
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

function givePowerPlantsToUser(
  power_plants: PowerPlantTemplate[],
  session_id: string,
  user_id: string
): PowerPlant[] {
  return power_plants.map((pp) => {
    return {
      ...pp,
      session_id: session_id,
      user_id: user_id,
      id: uuid(),
    };
  });
}

/**
 * Generate and insert into the DB a default portfolio for the user.
 * @param user_id User ID
 */
export async function setDefaultPortfolio(
  session_id: string,
  user_id: string
): Promise<void> {
  const pps = givePowerPlantsToUser(power_plants_base, session_id, user_id);
  await Promise.all(
    pps.map(async (pp) => {
      await db.query(
        `INSERT INTO power_plants 
          (id, session_id, user_id, type, p_min_mw, p_max_mw, stock_max_mwh, price_eur_per_mwh)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
        [
          pp.id,
          pp.session_id,
          pp.user_id,
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
    await db.query("SELECT * FROM power_plants WHERE user_id=$1", [user_id])
  ).rows;
}

/**
 * Get the current conso forecast for a given user.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getConsoForecast(
  session_id: string,
  user_id: string
): Promise<ConsoForecast> {
  const phase_no = await getCurrentPhaseNo(session_id);
  const rows: ConsoForecast[] = (
    await db.query("SELECT * FROM conso WHERE phase_no=$1 AND user_id=$2", [
      phase_no,
      user_id,
    ])
  ).rows;
  return rows.length === 1 ? rows[0] : null;
}
/**
 * Post a user user bit to the current open phase.
 * @param bid Bid object (without the phase_no)
 */
export async function postBid(bid: Omit<Bid, "phase_no">): Promise<void> {
  const phase_no = await getCurrentPhaseNo(bid.session_id);
  await db.query(
    `INSERT INTO bids 
      (id, user_id, session_id, phase_no, type, volume_mwh, price_eur_per_mwh) 
      VALUES ($1, $2, $3, $4, $5, $6, $7)`,
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
      "SELECT * FROM bids WHERE session_id=$1 AND id=$2 AND phase_no=$3",
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
    "DELETE FROM bids WHERE session_id=$1 AND id=$2 AND phase_no=$3",
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
  user_id: string
): Promise<Bid[]> {
  const phase_no = await getCurrentPhaseNo(session_id);
  return (
    await db.query(
      "SELECT * FROM bids WHERE session_id=$1 AND user_id=$2 AND phase_no=$3",
      [session_id, user_id, phase_no]
    )
  ).rows;
}

/**
 * Return all the bids for the active step of an sessions
 * @param sessions_id ID of the sessions
 */
export async function getAllBids(sessions_id: string): Promise<Bid[]> {
  const phase_no = await getCurrentPhaseNo(sessions_id);
  const res = (
    await db.query("SELECT * FROM bids WHERE sessions_id=$1 AND phase_no=$2", [
      sessions_id,
      phase_no,
    ])
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
  const phase_no = await getCurrentPhaseNo(session_id);
  return (
    await db.query(
      "SELECT * FROM production_plannings WHERE session_id=$1 AND user_id=$2 AND phase_no=$3",
      [session_id, user_id, phase_no]
    )
  ).rows as ProductionPlanning;
}

/**
 * Returns the a GamePhase object for the current phase. Returns `null` if
 * there is no active phase.
 * @param session_id Session ID
 */
export async function getPhaseInfos(session_id: string): Promise<GamePhase> {
  const phase_no = await getCurrentPhaseNo(session_id);
  return phase_no === null
    ? null
    : ((
        await db.query(
          "SELECT * FROM phases WHERE session_id=$1 AND phase_no=$2",
          [session_id, phase_no]
        )
      ).rows[0] as GamePhase);
}
/**
 * Check if users can submit bids to the current phase.
 * @param session_id Session ID
 */
export async function userCanBid(session_id: string): Promise<boolean> {
  const phase_no = await getCurrentPhaseNo(session_id);
  const rows = (
    await db.query(
      "SELECT bids_allowed FROM phases WHERE session_id=$1 AND phase_no=$2",
      [session_id, phase_no]
    )
  ).rows;
  return rows.length === 1 ? (rows[0].bids_allowed as boolean) : false;
}

/**
 * Check if users can submit plannings for the current phase.
 * @param session_id Session ID
 */
export async function userCanSubmitPlanning(
  session_id: string
): Promise<boolean> {
  const phase_no = await getCurrentPhaseNo(session_id);
  const rows = (
    await db.query(
      "SELECT plannings_allowed FROM phases WHERE session_id=$1 AND phase_no=$2",
      [session_id, phase_no]
    )
  ).rows;
  return rows.length === 1 ? (rows[0].plannings_allowed as boolean) : false;
}
