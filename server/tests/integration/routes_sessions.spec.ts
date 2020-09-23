/**
 * Integration tests for session management routes.
 *
 * The tested routes are :
 *  GET /scenarios
 *  PUT /session
 *  GET /sessions/open
 *  GET /session/:session_id
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import {
  clearDB,
  getDefaultScenarioID,
  insertNewSession,
} from "./db_utils_new";

const url = process.env.API_URL;

/**
 * GET /scenarios
 */
describe("Getting a list of scenarios", () => {
  beforeEach(async () => {
    await clearDB();
  });

  it("should always return a scenario", async () => {
    try {
      const res = await superagent.get(`${url}/scenarios`);
      expect(res.status).toEqual(200);
      expect(res.body.length).toEqual(1);

      // Check ID
      expect(res.body[0]).toHaveProperty("id");
      expect(typeof res.body[0].id).toEqual("string");
      expect(uuidValidate(res.body[0].id)).toEqual(true);

      // Check scenario name
      expect(res.body[0]).toHaveProperty("name");
      expect(typeof res.body[0].name).toEqual("string");

      // Check scenario description
      expect(res.body[0]).toHaveProperty("description");
      expect(typeof res.body[0].description).toEqual("string");

      // Check difficulty property
      expect(res.body[0]).toHaveProperty("difficulty");
      expect(typeof res.body[0].difficulty).toEqual("string");
      expect(
        ["easy", "medium", "hard"].includes(res.body[0].difficulty)
      ).toEqual(true);

      // Check multi game property
      expect(res.body[0]).toHaveProperty("multi_game");
      expect(typeof res.body[0].multi_game).toEqual("boolean");
    } catch (error) {
      fail(error);
    }
  });
});

/**
 * PUT /session
 */
describe("Opening a new auction", () => {
  const endpoint = `${url}/session`;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
  });

  it("Should success using the default scenario ID", async () => {
    try {
      const scenario_id = await getDefaultScenarioID();
      const session = {
        session_name: "My session",
        scenario_id: scenario_id,
      };
      const res = await superagent.put(endpoint).send(session);
      expect(res.status).toEqual(201);

      // Check ID
      expect(res.body).toHaveProperty("id");
      expect(typeof res.body.id).toEqual("string");
      expect(uuidValidate(res.body.id)).toEqual(true);

      // Check session name
      expect(res.body.name).toEqual(session.session_name);

      // Check status
      expect(res.body.status).toEqual("open");
    } catch (error) {
      fail(error);
    }
  });

  it("Should success even if no scenario ID is provided", async () => {
    try {
      const session = {
        session_name: "My session",
      };
      const res = await superagent.put(endpoint).send(session);
      expect(res.status).toEqual(201);

      // Check ID
      expect(res.body).toHaveProperty("id");
      expect(typeof res.body.id).toEqual("string");
      expect(uuidValidate(res.body.id)).toEqual(true);

      // Check session name
      expect(res.body.name).toEqual(session.session_name);

      // Check status
      expect(res.body.status).toEqual("open");
    } catch (error) {
      fail(error);
    }
  });

  it("Should fail if no session_name is provided", async () => {
    try {
      await superagent.put(endpoint).send({});
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, please provide a valid game session name"
      );
      expect(err.status).toEqual(400);
    }
  });

  test("should fail if the name is already taken", async () => {
    try {
      await superagent.put(endpoint).send({ session_name: "Session" });
      await superagent.put(endpoint).send({ session_name: "Session" });
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, a session already exists with this name"
      );
      expect(err.status).toEqual(409);
    }
  });

  it("should fail if the scenario ID does not correspond to a valid scenario", async () => {
    try {
      await superagent
        .put(endpoint)
        .send({ session_name: "Session", scenario_id: uuid() });
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, no scenario found with this ID"
      );
      expect(err.status).toEqual(400);
    }
  });
});

/**
 * Get /sessions/open
 */
describe("Listing open game sessions", () => {
  const endpoint = `${url}/sessions/open`;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
  });

  test("Should list the available game sessions that are currently open", async () => {
    await insertNewSession("Session 1");
    await insertNewSession("Session 2");
    const res = await superagent.get(endpoint);
    expect(Array.isArray(res.body)).toBe(true);
    expect(res.body.length).toEqual(2);
    expect(res.body[0].name).toEqual("Session 1");
    expect(res.body[1].name).toEqual("Session 2");
  });

  test("Should return an empty object if there are no open auctions", async () => {
    const res = await superagent.get(endpoint);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(0);
  });
});

/**
 * GET /session/:session_id
 */
describe("Retrieving session public infos", () => {
  let session_id: string;
  beforeAll(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
  });

  test("Should get basic infos for an open session", async () => {
    const res = await superagent.get(`${url}/session/${session_id}`);
    expect(res.body.id).toEqual(session_id);
    expect(res.body.name).toEqual("Session");
    expect(res.body.users.length).toEqual(0);
  });
});
