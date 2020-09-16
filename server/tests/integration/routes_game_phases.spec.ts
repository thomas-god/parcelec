/**
 * Integration tests relative to game phases routes.
 *
 * The tested routes are :
 *  GET /session/:session_id/user/:user_id/portfolio
 *  GET /session/:session_id/user/:user_id/conso
 *  POST /session/:session_id/user/:user_id/bid
 *  GET /session/:session_id/user/:user_id/bids
 *  DELETE /session/:session_id/user/:user_id/bid/:bid_id
 *  PUT /session/:session_id/user/:user_id/planning
 *  GET /session/:session_id/clearing
 *  GET /session/:session_id/user/:user_id/clearing
 *  GET /session/:session_id/user/:user_id/results
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import { clearDB, prepareDB, sessions, users, power_plants } from "./db_utils";

const url = process.env.API_URL;

/**
 * GET /session/:session_id/user/:user_id/portfolio
 */
describe("Getting a user portfolio", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should error if no session_id is provided", async () => {
    try {
      await superagent.get(`${url}/session/user/${uuid()}/portfolio`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if not user_id is provided", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/portfolio`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the session_id does not match an existing one", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}/portfolio`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
    }
  });

  test("Should error if the user_id does not match an existing one", async () => {
    try {
      await superagent.get(
        `${url}/session/${sessions[0].id}/user/${uuid()}/portfolio`
      );
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual("Error, no user found with this ID");
    }
  });

  test("Should get a list of power plant object", async () => {
    const res = await superagent.get(
      `${url}/session/${users[0].session_id}/user/${users[0].id}/portfolio`
    );
    expect(res.status).toEqual(200);
    expect(Array.isArray(res.body)).toEqual(true);
    expect(res.body.sort((a, b) => (a.id < b.id ? 1 : -1))).toEqual(
      power_plants
        .filter((pp) => pp.user_id === users[0].id)
        .sort((a, b) => (a.id < b.id ? 1 : -1))
    );
  });

  test("Should get a portfolio for a newly created user", async () => {
    const user_id = (
      await superagent
        .put(`${url}/session/${sessions[0].id}/register_user`)
        .send({ username: "User" })
    ).body.user_id;
    const res = await superagent.get(
      `${url}/session/${sessions[0].id}/user/${user_id}/portfolio`
    );
    expect(res.status).toEqual(200);
    expect(Array.isArray(res.body)).toEqual(true);
  });
});

describe("Getting conso info for a running auction", () => {
  /**
   * Clean the DB and properly start a session by adding 2 users
   * and set them to ready
   */
  beforeEach(async () => {
    await prepareDB();
  });

  async function startSession(session_id: string): Promise<string[]> {
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

  test("Should error when the session does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}/conso`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent.get(
        `${url}/session/${sessions[1].id}/user/${uuid()}/conso`
      );
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual("Error, no user found with this ID");
    }
  });

  test("Should error when the session is not running", async () => {
    try {
      const user_id = (
        await superagent
          .put(`${url}/session/${sessions[0].id}/register_user`)
          .send({ username: "User" })
      ).body.user_id;
      await superagent.get(
        `${url}/session/${sessions[0].id}/user/${user_id}/conso`
      );
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the session is not running");
    }
  });

  test("Should get conso infos", async () => {
    const users_id = await startSession(sessions[0].id);
    const res = await superagent.get(
      `${url}/session/${sessions[0].id}/user/${users_id[0]}/conso`
    );
    expect(res.body).toHaveProperty("conso_mw");
  });
});
