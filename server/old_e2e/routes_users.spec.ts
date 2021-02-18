/**
 * Integration tests for users management routes.
 *
 * The tested routes are :
 *  PUT /session/:session_id/register_user
 *  GET /session/:session_id/user/:user_id
 *  PUT /session/:session_id/user/:user_id/ready
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import {
  clearDB,
  getDefaultScenarioID,
  insertNewSession,
  insertNewUser,
} from "./db_utils_new";

const url = process.env.API_URL;

/**
 * PUT /session/:session_id/register_user
 */
describe("Registering a new user to an open session", () => {
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
  });

  test("Should fail when no credentials are provided", async () => {
    try {
      await superagent.put(`${url}/session/register_user`);
    } catch (err) {
      expect(err.status).toEqual(404);
    }
  });

  test("Should fail when the session_id does not correspond to an existing session", async () => {
    try {
      await superagent
        .put(`${url}/session/${uuid()}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, the session_id does not correspond to an existing session"
      );
      expect(err.status).toEqual(404);
    }
  });

  test("Should error when the session_id correspond to a running session", async () => {
    try {
      // TODO start session
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, the session is not open for registration"
      );
      expect(err.status).toEqual(400);
    }
  });

  test("Should error when the session_id correspond to a closed session", async () => {
    try {
      // TODO close session
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, the session is not open for registration"
      );
      expect(err.status).toEqual(400);
    }
  });

  test("Should error when no username is provided", async () => {
    try {
      const session_id = await insertNewSession("Session");
      await superagent.put(`${url}/session/${session_id}/register_user`);
    } catch (err) {
      expect(err.response.text).toEqual("Error, no username provided");
      expect(err.status).toEqual(400);
    }
  });

  test("Should return a uuid_v4 user_id on successful registration", async () => {
    try {
      const session_id = await insertNewSession("Session");
      const res = await superagent
        .put(`${url}/session/${session_id}/register_user`)
        .send({ username: "User" });
      expect(res.status).toEqual(201);
      expect(res.body).toHaveProperty("user_id");
      expect(uuidValidate(res.body.user_id)).toBe(true);
    } catch (err) {
      fail(err);
    }
  });

  test("Should error when we try to register a user whose name already exists", async () => {
    try {
      const session_id = await insertNewSession("Session");
      await superagent
        .put(`${url}/session/${session_id}/register_user`)
        .send({ username: "User" });
      await superagent
        .put(`${url}/session/${session_id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.response.text).toEqual(
        "Error, a user with this username is already registered to the session"
      );
      expect(err.status).toEqual(409);
    }
  });
});

/**
 * GET /session/:session_id/user/:user_id
 */
describe("Get informations about a given user", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should error if session does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, cannot find an user with these IDs"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the user does not exist on the session", async () => {
    try {
      await superagent.get(`${url}/session/${session_id}/user/${uuid()}`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, cannot find an user with these IDs"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should get user infos", async () => {
    try {
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id}`
      );
      expect(res.body.session_id).toEqual(session_id);
      expect(res.body.name).toEqual("User");
      expect(res.body.ready).toEqual(false);
    } catch (err) {
      fail(err);
    }
  });
});

/**
 * PUT /session/:session_id/user/:user_id/ready
 */
describe("Marking an user as ready for a game session to start", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should error if no session_id is provided", async () => {
    try {
      await superagent.put(`${url}/session/user/${uuid()}/ready`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if no user_id is provided", async () => {
    try {
      await superagent.put(`${url}/session/${session_id}/user/ready`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the session does not exist", async () => {
    try {
      await superagent.put(`${url}/session/${uuid()}/user/${uuid()}/ready`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the user is not registered to the session", async () => {
    try {
      await superagent.put(`${url}/session/${session_id}/user/${uuid()}/ready`);
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(400);
    }
  });

  test("Should success with a registered user on an open session", async () => {
    try {
      const res = await superagent.put(
        `${url}/session/${session_id}/user/${user_id}/ready`
      );
      expect(res.status).toEqual(201);
    } catch (err) {
      fail(err);
    }
  });
});
