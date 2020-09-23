/**
 * Utility functions and data structures to make integration tests
 * with the DB.
 */

import db from "../../src/db/index";
import { v4 as uuid } from "uuid";
import superagent from "superagent";
import {
  PowerPlant,
  PowerPlantTemplate,
  PowerPlantWithPlanning,
  ScenarioOptions,
  Session,
  User,
} from "../../src/routes/types";

const url = process.env.API_URL;

/**
 * Remove all records from the DB using the property that `scenarios_options`
 * table is referenced with ON DELETE CASCADE clauses by all other tables.
 */
export async function clearDB(): Promise<void> {
  await db.query("DELETE FROM scenarios_options;", []);
}

/**
 * Initialize the default scenario by calling the GET /scenarios route,
 * and return the default scenario ID.
 */
export async function getDefaultScenarioID(): Promise<string> {
  const res = await superagent.get(`${url}/scenarios`);
  return res.body.id;
}

/**
 * Insert a new session and return its ID.
 */
export async function insertNewSession(
  session_name: string,
  scenario_id?: string
): Promise<string> {
  const res = await superagent
    .put(`${url}/session`)
    .send({ session_name: session_name, scenario_id: scenario_id });
  return res.body.id;
}

/**
 * Register a new user to a game session and return its ID.
 * @param session_id Session ID
 * @param username Username
 */
export async function insertNewUser(
  session_id: string,
  username: string
): Promise<string> {
  const res = await superagent
    .put(`${url}/session/${session_id}/register_user`)
    .send({ username: username });
  return res.body.user_id;
}

/**
 * Register a new user to a game session and return its ID.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function setUserReady(
  session_id: string,
  user_id: string
): Promise<void> {
  await superagent.put(`${url}/session/${session_id}/user/${user_id}/ready`);
}

/**
 * Return the portfolio of a user.
 * @param session_id Session ID
 * @param user_id User ID
 */
export async function getUserPortfolio(
  session_id: string,
  user_id: string
): Promise<PowerPlantWithPlanning[]> {
  const res = await superagent.get(
    `${url}/session/${session_id}/user/${user_id}/portfolio`
  );
  return res.body;
}

/**
 * Create a running session by inserting 2 users and marking them
 * as ready.
 * @param session_name Name of the session to create
 */
export async function insertRunningSession(
  session_name: string
): Promise<{ session_id: string; user_id_1: string; user_id_2: string }> {
  const session_id = await insertNewSession(session_name);
  const user_id_1 = await insertNewUser(session_id, "User 1");
  const user_id_2 = await insertNewUser(session_id, "User 2");
  await setUserReady(session_id, user_id_1);
  await setUserReady(session_id, user_id_2);
  return { session_id, user_id_1, user_id_2 };
}
