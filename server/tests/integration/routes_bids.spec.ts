/**
 * Integration tests relative to bids routes.
 *
 * The tested routes are :
 *  POST /session/:session_id/user/:user_id/bid
 *  GET /session/:session_id/user/:user_id/bids
 *  DELETE /session/:session_id/user/:user_id/bid/:bid_id
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import {
  clearDB,
  getDefaultScenarioID,
  getUserPortfolio,
  insertNewSession,
  insertNewUser,
  insertRunningSession,
} from "./db_utils_new";

const url = process.env.API_URL;

/**
 * POST /session/:session_id/user/:user_id/bid
 */
describe("Posting a bid to a running session", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent
        .post(`${url}/session/${uuid()}/user/${uuid()}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent
        .post(`${url}/session/${session_id}/user/${uuid()}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the session is not running", async () => {
    try {
      await superagent
        .post(`${url}/session/${session_id}/user/${user_id}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
    } catch (error) {
      expect(error.response.text).toEqual("Error, the session is not running");
      expect(error.status).toEqual(400);
    }
  });

  test("should success when the session is running", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      const res = await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      expect(res.status).toEqual(201);
      expect(res.body).toHaveProperty("bid_id");
      expect(uuidValidate(res.body.bid_id)).toEqual(true);
    } catch (err) {
      fail(err.response.text);
    }
  });
});

/**
 * GET /session/:session_id/user/:user_id/bids
 */
describe("Getting a user's bids", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}/bids`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${session_id}/user/${uuid()}/bids`);
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the session is not running", async () => {
    try {
      await superagent.get(`${url}/session/${session_id}/user/${user_id}/bids`);
    } catch (error) {
      expect(error.response.text).toEqual("Error, the session is not running");
      expect(error.status).toEqual(400);
    }
  });

  test("should return empty list when no bids", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/bids`
      );
      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);
      expect(res.body.length).toEqual(0);
    } catch (err) {
      fail(err);
    }
  });

  test("should return the correct bid content", async () => {
    try {
      const { session_id: sid, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      await superagent
        .post(`${url}/session/${sid}/user/${user_id_1}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      const res = await superagent.get(
        `${url}/session/${sid}/user/${user_id_1}/bids`
      );
      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);
      expect(res.body.length).toEqual(1);
      expect(res.body[0].type).toEqual("buy");
      expect(res.body[0].volume_mwh).toEqual(10);
      expect(res.body[0].price_eur_per_mwh).toEqual(50);
      expect(uuidValidate(res.body[0].id)).toEqual(true);
    } catch (err) {
      console.log(err);

      fail(err);
    }
  });

  test("should return the correct number of bids", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/bids`
      );
      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);
      expect(res.body.length).toEqual(3);
    } catch (err) {
      fail(err);
    }
  });
});

/**
 * DELETE /session/:session_id/user/:user_id/bid/:bid_id
 */
describe("Deleting a user's bid", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent.delete(
        `${url}/session/${uuid()}/user/${uuid()}/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent.delete(
        `${url}/session/${session_id}/user/${uuid()}/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the bid does not exist", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      await superagent.delete(
        `${url}/session/${session_id}/user/${user_id_1}/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.response.text).toEqual("Error, no bid found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should delete the bid", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      // Add a bid
      const bid_id = (
        await superagent
          .post(`${url}/session/${session_id}/user/${user_id_1}/bid`)
          .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } })
      ).body.bid_id;

      // Check the bid has been inserted
      const res_bids_1 = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/bids`
      );
      expect(Array.isArray(res_bids_1.body)).toEqual(true);
      expect(res_bids_1.body.length).toEqual(1);
      expect(res_bids_1.body[0].id).toEqual(bid_id);

      // Delete the bid
      const res_delete = await superagent.delete(
        `${url}/session/${session_id}/user/${user_id_1}/bid/${bid_id}`
      );
      expect(res_delete.status).toEqual(200);

      // Check there are no longer bids for that user
      const res_bids_2 = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/bids`
      );
      expect(res_bids_2.body.length).toEqual(0);
    } catch (err) {
      fail(err);
    }
  });
});
