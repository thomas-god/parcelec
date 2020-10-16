/**
 * Utility functions and data structures to make integration tests
 * with the DB.
 */

import db from "../../src/db/index";
import { v4 as uuid } from "uuid";
import superagent from "superagent";
import {
  PowerPlantWithPlanning,
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
 * Initialize the default scenarios by calling the GET /scenarios route,
 * and return the default scenario ID.
 * 
 * @param multi_game Boolean, default true, return the multi default scenario if 
 * true, else the solo default scenario
 */
export async function getDefaultScenarioID(multi_game = true): Promise<string> {
  const res = await superagent.get(`${url}/scenarios`);
  return res.body.find(s => s.multi_game === multi_game).id;
}

/**
 * Insert a new session and return its ID.
 */
export async function insertNewSession(
  session_name: string,
  scenario_id?: string,
  multi_game=true
): Promise<string> {
  if(scenario_id === undefined)
    scenario_id = await getDefaultScenarioID(multi_game)
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
 * as ready. Names of the users are `User 1` and `User 2`.
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

  // Wait for session to be in the 'running' state.
  await new Promise((resolve, reject) => {
    let i = 0;
    const interval = setInterval(async () => {
      const res = await superagent.get(`${url}/session/${session_id}`);
      if (res.body.status === "running") {
        resolve();
        clearInterval(interval);
      }
      if (i > 5) {
        reject();
        clearInterval(interval);
      }
      i++;
    }, 15);
  });
  return { session_id, user_id_1, user_id_2 };
}

/**
 * Insert a bid on behalf of a user.
 * @param session_id Session ID
 * @param user_id User ID
 * @param type Type of the bid (`sell` | `buy`). Default `sell`.
 * @param volume_mwh Volume of the bid in MWh. Default 10 MWh.
 * @param price_eur_per_mwh Price of the bid in €/MWh. Default 10 €/MWh.
 */
export async function insertBid(
  session_id: string,
  user_id: string,
  type = "sell",
  volume_mwh = 10,
  price_eur_per_mwh = 10
): Promise<string> {
  return (
    await superagent
      .post(`${url}/session/${session_id}/user/${user_id}/bid`)
      .send({
        bid: {
          type: type,
          volume_mwh: volume_mwh,
          price_eur_per_mwh: price_eur_per_mwh,
        },
      })
  ).body.bid_id as string;
}
