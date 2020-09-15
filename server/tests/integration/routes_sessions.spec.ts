/**
 * Integration tests for session management routes.
 *
 * The tested routes are :
 *  GET /sessions/open
 *  PUT /session
 *  GET /session/:session_id
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import { clearDB, prepareDB, sessions, users } from "./db_utils";

const url = process.env.API_URL;

/**
 * Get /sessions/open
 */
describe("Listing open game sessions", () => {
  const endpoint = "/sessions/open";
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should list the available game sessions that are currently open", async () => {
    const res = await superagent.get(`${url}${endpoint}`);
    expect(Array.isArray(res.body)).toBe(true);
    expect(res.body.length).toEqual(1);
    expect(res.body[0].id).toEqual(sessions[0].id);
    expect(res.body[0].name).toEqual(sessions[0].name);
  });

  test("Should return an empty object if there are no open auctions", async () => {
    await clearDB();
    const res = await superagent.get(`${url}${endpoint}`);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(0);
  });
});

/**
 * PUT /session
 */
describe("Opening a new auction", () => {
  const endpoint = "/session";
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should return a 400 error if no session_name is provided in body", async () => {
    try {
      await superagent.put(`${url}${endpoint}`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, please provide a valid game session name"
      );
    }
  });

  test("It should return a 409 error if the name is already taken", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ session_name: sessions[0].name });
    } catch (err) {
      expect(err.status).toEqual(409);
      expect(err.response.text).toEqual(
        "Error, a session already exists with this name"
      );
    }
  });

  test("It should get back an session object on success", async () => {
    const res = await superagent
      .put(`${url}${endpoint}`)
      .send({ session_name: "My auction" });
    const body = JSON.parse(res.text);
    expect(res.status).toEqual(201);
    expect(body.name).toEqual("My auction");
    expect(body.status).toEqual("Open");
    expect(body).toHaveProperty("id");
    expect(uuidValidate(body.id)).toEqual(true);
  });
});

/**
 * GET /session/:session_id
 */
describe("Retrieving session public infos", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should get a 404 error if no session_id is provided", async () => {
    try {
      await superagent.get(`${url}/auction`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should get basic infos for an open auction", async () => {
    await superagent
      .put(`${url}/auction/${sessions[0].id}/register_user`)
      .send({ username: "User 1" });
    const res = await superagent.get(`${url}/auction/${sessions[0].id}`);
    expect(res.body.id).toEqual(sessions[0].id);
    expect(res.body.name).toEqual(sessions[0].name);
    expect(res.body.status).toEqual(sessions[0].status);
    expect(res.body.users).toEqual([{ name: "User 1", ready: false }]);
  });

  test("Should get basic infos for a running auction", async () => {
    const res = await superagent.get(`${url}/auction/${sessions[1].id}`);
    expect(res.body.id).toEqual(sessions[1].id);
    expect(res.body.name).toEqual(sessions[1].name);
    expect(res.body.status).toEqual("Running");
    expect(res.body.users).toEqual([
      { name: "User 1", ready: true },
      { name: "User 2", ready: true },
    ]);
    expect(res.body.step_no).toEqual(0);
  });
});
