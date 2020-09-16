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
import { clearDB, prepareDB, sessions, users } from "./db_utils";

const url = process.env.API_URL;

/**
 * PUT /session/:session_id/register_user
 */
describe("Registering a new user to an open session", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should return error 404 when no credentials are provided", async () => {
    try {
      await superagent.put(`${url}/session/register_user`);
    } catch (err) {
      expect(err.status).toEqual(404);
    }
  });

  test("Should error when the session_id does not correspond to an existing session", async () => {
    try {
      await superagent
        .put(`${url}/session/${uuid()}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(404);
      expect(err.response.text).toEqual(
        "Error, the session_id does not correspond to an existing session"
      );
    }
  });

  test("Should error when the session_id correspond to a running session", async () => {
    try {
      await superagent
        .put(`${url}/session/${sessions[1].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, the session is not open for registration"
      );
    }
  });

  test("Should error when the session_id correspond to a closed session", async () => {
    try {
      await superagent
        .put(`${url}/session/${sessions[2].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, the session is not open for registration"
      );
    }
  });

  test("Should error when no username is provided", async () => {
    try {
      await superagent.put(`${url}/session/${sessions[0].id}/register_user`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual("Error, no username provided");
    }
  });

  test("Should return a uuid_v4 user_id on successful registration", async () => {
    const res = await superagent
      .put(`${url}/session/${sessions[0].id}/register_user`)
      .send({ username: "User" });
    expect(res.status).toEqual(201);
    expect(res.body).toHaveProperty("user_id");
    expect(uuidValidate(res.body.user_id)).toBe(true);
  });

  test("Should error when we try to register a user whose name already exists", async () => {
    try {
      await superagent
        .put(`${url}/session/${sessions[0].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(409);
      expect(err.response.text).toEqual(
        "Error, a user with this username is already registered to the session"
      );
    }
  });
});

/**
 * GET /session/:session_id/user/:user_id
 */
describe("Get informations about a given user", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should error if session does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, cannot find an user with these IDs"
      );
    }
  });

  test("Should error if the user does not exist on the session", async () => {
    try {
      await superagent.get(`${url}/session/${sessions[0].id}/user/${uuid()}`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, cannot find an user with these IDs"
      );
    }
  });

  test("Should get user infos", async () => {
    const res = await superagent.get(
      `${url}/session/${users[0].session_id}/user/${users[0].id}`
    );
    expect(res.body.session_id).toEqual(users[0].session_id);
    expect(res.body.name).toEqual(users[0].name);
    expect(res.body.ready).toEqual(users[0].game_ready);
  });
});

/**
 * PUT /session/:session_id/user/:user_id/ready
 */
describe("Marking an user as ready for a game session to start", () => {
  beforeAll(async () => {
    await prepareDB();
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
      await superagent.put(`${url}/session/${sessions[0].id}/user/ready`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the session does not exist", async () => {
    try {
      await superagent.put(`${url}/session/${uuid()}/user/${uuid()}/ready`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
    }
  });

  test("Should error if the user is not registered to the session", async () => {
    try {
      await superagent.put(
        `${url}/session/${sessions[0].id}/user/${uuid()}/ready`
      );
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, no user found with this ID");
    }
  });

  test("Should error if the session is running", async () => {
    try {
      await superagent.put(
        `${url}/session/${sessions[1].id}/user/${users[0].id}/ready`
      );
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the session is running");
    }
  });

  test("Should error if the session is closed", async () => {
    try {
      await superagent.put(
        `${url}/session/${sessions[2].id}/user/${users[2].id}/ready`
      );
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the session is closed");
    }
  });

  test("Should success with a registered user on an open session", async () => {
    const user_id = (
      await superagent
        .put(`${url}/session/${sessions[0].id}/register_user`)
        .send({ username: "User" })
    ).body.user_id;
    const res = await superagent.put(
      `${url}/session/${sessions[0].id}/user/${user_id}/ready`
    );
    expect(res.status).toEqual(201);
  });

  test("Should start the session when at least 2 users are ready", async () => {
    const user_id = (
      await superagent
        .put(`${url}/session/${sessions[0].id}/register_user`)
        .send({ username: "User 2" })
    ).body.user_id;
    await superagent.put(
      `${url}/session/${sessions[0].id}/user/${user_id}/ready`
    );
    await new Promise((r) => setTimeout(r, 150));
    const res = await superagent.get(`${url}/session/${sessions[0].id}`);
    expect(res.body.status).toEqual("running");
  });
});
