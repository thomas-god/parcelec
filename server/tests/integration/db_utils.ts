/**
 * Utility functions and data structures to make integration tests
 * with the DB.
 */

import db from "../../src/db/index";

export const sessions = [
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

export const users = [
  {
    session_id: sessions[1].id,
    name: "User 1",
    id: "623884df-4548-4080-9fb2-96fa6f81a691",
    ready: true,
  },
  {
    session_id: sessions[1].id,
    name: "User 2",
    id: "d8226301-82a2-4dcc-b191-91af33378c29",
    ready: true,
  },
  {
    session_id: sessions[2].id,
    name: "User 1",
    id: "8ada2c5f-1def-4959-8cef-2cafb6487b69",
    ready: true,
  },
  {
    session_id: sessions[2].id,
    name: "User 2",
    id: "e227d4d5-bf2e-45c6-b064-ffe6fd432e78",
    ready: true,
  },
];

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
    })
  );

  // Insert Users
  await Promise.all(
    users.map(async (user) => {
      await db.query(
        "INSERT INTO users (id, name, session_id, game_ready) VALUES ($1, $2, $3, $4)",
        [user.id, user.name, user.session_id, user.ready]
      );
    })
  );
}

export async function prepareDB(): Promise<void> {
  await clearDB();
  await populateDB();
}
