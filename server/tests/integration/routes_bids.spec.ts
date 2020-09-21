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
import { prepareDB, sessions, startSession, users } from "./db_utils";

const url = process.env.API_URL;

/**
 * POST /session/:session_id/user/:user_id/bid
 */
describe("Posting a bid to a running session", () => {
  beforeEach(async () => {
    await prepareDB();
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent
        .post(`${url}/session/${uuid()}/user/${uuid()}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent
        .post(`${url}/session/${sessions[1].id}/user/${uuid()}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
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
      await superagent
        .post(`${url}/session/${sessions[0].id}/user/${user_id}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the session is not running");
    }
  });

  test("should success when the session is running", async () => {
    try {
      const users_id = await startSession(url, sessions[0].id);
      const res = await superagent
        .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
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
  beforeEach(async () => {
    await prepareDB();
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}/bids`);
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
        `${url}/session/${sessions[1].id}/user/${uuid()}/bids`
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
        `${url}/session/${sessions[0].id}/user/${user_id}/bids`
      );
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the session is not running");
    }
  });

  test("should return empty list when no bids", async () => {
    try {
      const users_id = await startSession(url, sessions[0].id);
      const res = await superagent.get(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bids`
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
      const users_id = await startSession(url, sessions[0].id);
      await superagent
        .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      const res = await superagent.get(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bids`
      );
      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);
      expect(res.body.length).toEqual(1);
      expect(res.body[0].type).toEqual("buy");
      expect(res.body[0].volume_mwh).toEqual(10);
      expect(res.body[0].price_eur_per_mwh).toEqual(50);
      expect(uuidValidate(res.body[0].id)).toEqual(true);
    } catch (err) {
      fail(err);
    }
  });

  test("should return the correct number of bids", async () => {
    try {
      const users_id = await startSession(url, sessions[0].id);
      await superagent
        .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      await superagent
        .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      await superagent
        .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
        .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } });
      const res = await superagent.get(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bids`
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
  beforeEach(async () => {
    await prepareDB();
  });

  test("Should error when the session does not exist", async () => {
    try {
      await superagent.delete(
        `${url}/session/${uuid()}/user/${uuid()}/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent.delete(
        `${url}/session/${sessions[1].id}/user/${uuid()}/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual("Error, no user found with this ID");
    }
  });

  test("Should error when the bid does not exist", async () => {
    try {
      await superagent.delete(
        `${url}/session/${users[0].session_id}/user/${
          users[0].id
        }/bid/${uuid()}`
      );
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual("Error, no bid found with this ID");
    }
  });

  test("Should delete the bid", async () => {
    try {
      const users_id = await startSession(url, sessions[0].id);
      // Add a bid
      const bid_id = (
        await superagent
          .post(`${url}/session/${sessions[0].id}/user/${users_id[0]}/bid`)
          .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } })
      ).body.bid_id;

      // Check the bid has been inserted
      const res_bids_1 = await superagent.get(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bids`
      );
      expect(Array.isArray(res_bids_1.body)).toEqual(true);
      expect(res_bids_1.body.length).toEqual(1);
      expect(res_bids_1.body[0].id).toEqual(bid_id);

      // Delete the bid
      const res_delete = await superagent.delete(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bid/${bid_id}`
      );
      expect(res_delete.status).toEqual(200);

      // Check there are no longer bids for that user
      const res_bids_2 = await superagent.get(
        `${url}/session/${sessions[0].id}/user/${users_id[0]}/bids`
      );
      expect(res_bids_2.body.length).toEqual(0);
    } catch (err) {
      fail(err);
    }
  });
});
