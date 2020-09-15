/**
 * Utility functions and data structures to make integration tests
 * with the DB.
 */

import db from "../../src/db/index";
import { v4 as uuid } from "uuid";

export const sessions = [
  { id: uuid(), name: "Open session", status: "open" },
  { id: uuid(), name: "Running session", status: "running" },
  { id: uuid(), name: "Close session", status: "closed" },
];

export const users = [
  { session_id: sessions[1].id, name: "User 1", id: uuid(), ready: true },
  { session_id: sessions[1].id, name: "User 2", id: uuid(), ready: true },
  { session_id: sessions[2].id, name: "User 1", id: uuid(), ready: true },
  { session_id: sessions[2].id, name: "User 2", id: uuid(), ready: true },
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
