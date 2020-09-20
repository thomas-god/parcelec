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
  Session,
  User,
} from "../../src/routes/types";

export const sessions: Session[] = [
  {
    id: "188ad7b8-c994-4313-a77f-70a055591bf5",
    name: "Open session",
    status: "open",
  },
  {
    id: "e2ff5d6e-095c-41ca-ae18-c19b6ab1b874",
    name: "Running session",
    status: "running",
  },
  {
    id: "ab574505-aad4-4686-947b-07439c28404c",
    name: "Close session",
    status: "closed",
  },
];

export const session_options = {
  bids_duration_sec: 180,
  plannings_duration_sec: 300,
  phases_number: 3,
  conso_forecast_mwh: [1000, 1800, 2400],
  conso_price_eur: 35,
  imbalance_costs_eur: 45,
};

export const users: User[] = [
  {
    session_id: sessions[1].id,
    name: "User 1",
    id: "623884df-4548-4080-9fb2-96fa6f81a691",
    game_ready: true,
  },
  {
    session_id: sessions[1].id,
    name: "User 2",
    id: "d8226301-82a2-4dcc-b191-91af33378c29",
    game_ready: true,
  },
  {
    session_id: sessions[2].id,
    name: "User 1",
    id: "8ada2c5f-1def-4959-8cef-2cafb6487b69",
    game_ready: true,
  },
  {
    session_id: sessions[2].id,
    name: "User 2",
    id: "e227d4d5-bf2e-45c6-b064-ffe6fd432e78",
    game_ready: true,
  },
];

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
  user: User
): PowerPlant[] {
  return power_plants.map((pp) => {
    return {
      ...pp,
      session_id: user.session_id,
      user_id: user.id,
      id: uuid(),
    };
  });
}

export const power_plants: PowerPlant[] = [].concat(
  ...users.map((u) => givePowerPlantsToUser(power_plants_base, u))
);

export async function clearDB(): Promise<void> {
  await db.query("DELETE FROM results CASCADE", []);
  await db.query("DELETE FROM production_plannings CASCADE", []);
  await db.query("DELETE FROM exchanges CASCADE", []);
  await db.query("DELETE FROM clearings CASCADE", []);
  await db.query("DELETE FROM bids CASCADE", []);
  await db.query("DELETE FROM conso CASCADE", []);
  await db.query("DELETE FROM phases CASCADE", []);
  await db.query("DELETE FROM power_plants CASCADE", []);
  await db.query("DELETE FROM users CASCADE", []);
  await db.query("DELETE FROM sessions CASCADE", []);
}
async function populateDB() {
  // Insert game sessions
  await Promise.all(
    sessions.map(async (session) => {
      await db.query(
        "INSERT INTO sessions (id, name, status) VALUES ($1, $2, $3)",
        [session.id, session.name, session.status]
      );
      await db.query(
        `INSERT INTO options
          (
            session_id, 
            bids_duration_sec,
            plannings_duration_sec,
            phases_number,
            conso_forecast_mwh,
            conso_price_eur,
            imbalance_costs_eur
          )
        VALUES ($1, $2, $3, $4, $5, $6, $7)`,
        [
          session.id,
          session_options.bids_duration_sec,
          session_options.plannings_duration_sec,
          session_options.phases_number,
          session_options.conso_forecast_mwh,
          session_options.conso_price_eur,
          session_options.imbalance_costs_eur,
        ]
      );
    })
  );

  // Insert Users
  await Promise.all(
    users.map(async (user) => {
      await db.query(
        "INSERT INTO users (id, name, session_id, game_ready) VALUES ($1, $2, $3, $4)",
        [user.id, user.name, user.session_id, user.game_ready]
      );
    })
  );

  // Insert power plants
  await Promise.all(
    power_plants.map(async (pp) => {
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
 * Util function to start a session and trigger the server-side start logic.
 * @param url base route URL
 * @param session_id Session ID
 */
export async function startSession(
  url: string,
  session_id: string
): Promise<string[]> {
  const user1_id = (
    await superagent
      .put(`${url}/session/${session_id}/register_user`)
      .send({ username: "User 1" })
  ).body.user_id;
  await superagent.put(`${url}/session/${session_id}/user/${user1_id}/ready`);
  const user2_id = (
    await superagent
      .put(`${url}/session/${session_id}/register_user`)
      .send({ username: "User 2" })
  ).body.user_id;
  await superagent.put(`${url}/session/${session_id}/user/${user2_id}/ready`);
  await new Promise((r) => setTimeout(r, 50));
  return [user1_id, user2_id];
}

export async function prepareDB(): Promise<void> {
  await clearDB();
  await populateDB();
}
