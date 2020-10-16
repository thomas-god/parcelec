/**
 * Integration tests relative to OTC routes.
 *
 * The tested routes are :
 *  GET /session/:session_id/user/:user_id/otc
 *  POST /session/:session_id/user/:user_id/otc/new
 *  PUT /session/:session_id/user/:user_id/otc/:otc_id/accept
 *  PUT /session/:session_id/user/:user_id/otc/:otc_id/reject
 *  PUT /session/:session_id/user/:user_id/otc/:otc_id/counter_offer
 *
 */
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import {
  clearDB,
  getDefaultScenarioID,
  insertNewSession,
  insertNewUser,
  insertRunningSession,
} from "./db_utils_new";

const url = process.env.API_URL;

/**
 * POST /session/:session_id/user/:user_id/otc
 */
describe("Posting an OTC offer to another user", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should post a simple OTC offer", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      const res = await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/otc`)
        .send({
          type: "sell",
          user_to: "User 2",
          volume_mwh: 100,
          price_eur_per_mwh: 50,
        });
      expect(res.status).toEqual(201);
      expect(res.body).toHaveProperty("otc_id");
      expect(uuidValidate(res.body.otc_id)).toEqual(true);
    } catch (error) {
      fail(error);
    }
  });
});

/**
 * GET /session/:session_id/user/:user_id/otc
 */
describe("Posting an OTC offer to another user", () => {
  let session_id: string;
  let user_id: string;
  beforeEach(async () => {
    await clearDB();
    await getDefaultScenarioID();
    session_id = await insertNewSession("Session");
    user_id = await insertNewUser(session_id, "User");
  });

  test("Should post a simple OTC offer", async () => {
    try {
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running Session"
      );
      await superagent
        .post(`${url}/session/${session_id}/user/${user_id_1}/otc`)
        .send({
          type: "sell",
          user_to: "User 2",
          volume_mwh: 100,
          price_eur_per_mwh: 50,
        });
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/otc`
      );

      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);
      expect(res.body.length).toEqual(1);

      expect(uuidValidate(res.body[0].id)).toEqual(true);
      expect(uuidValidate(res.body[0].user_from)).toEqual(false);
      expect(uuidValidate(res.body[0].user_to)).toEqual(false);
    } catch (error) {
      fail(error);
    }
  });
});
