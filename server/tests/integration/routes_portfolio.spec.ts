/**
 * Integration tests relative to game phases routes.
 *
 * The tested routes are :
 *  GET /session/:session_id/user/:user_id/portfolio
 *  GET /session/:session_id/user/:user_id/conso
 *  PUT /session/:session_id/user/:user_id/planning
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
import { PowerPlant, PowerPlantDispatch } from "../../src/routes/types";

const url = process.env.API_URL;

/**
 * GET /session/:session_id/user/:user_id/portfolio
 */
describe("Getting a user portfolio", () => {
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
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error if the user_id does not match an existing one", async () => {
    try {
      await superagent.get(
        `${url}/session/${session_id}/user/${uuid()}/portfolio`
      );
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should get a list of power plant object", async () => {
    try {
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id}/portfolio`
      );
      expect(res.status).toEqual(200);
      expect(Array.isArray(res.body)).toEqual(true);

      // Check IDs
      expect(res.body[0]).toHaveProperty("id");
      expect(typeof res.body[0].id).toEqual("string");
      expect(uuidValidate(res.body[0].id)).toEqual(true);

      expect(res.body[0]).toHaveProperty("session_id");
      expect(typeof res.body[0].session_id).toEqual("string");
      expect(uuidValidate(res.body[0].session_id)).toEqual(true);
      expect(res.body[0].session_id).toEqual(session_id);

      expect(res.body[0]).toHaveProperty("user_id");
      expect(typeof res.body[0].user_id).toEqual("string");
      expect(uuidValidate(res.body[0].user_id)).toEqual(true);
      expect(res.body[0].user_id).toEqual(user_id);

      // Check power plant type
      expect(res.body[0]).toHaveProperty("type");
      expect(typeof res.body[0].type).toEqual("string");
      expect(
        ["nuc", "therm", "hydro", "ren", "storage"].includes(res.body[0].type)
      ).toEqual(true);

      // Check numeric values
      expect(res.body[0]).toHaveProperty("p_min_mw");
      expect(typeof res.body[0].p_min_mw).toEqual("number");

      expect(res.body[0]).toHaveProperty("p_max_mw");
      expect(typeof res.body[0].p_max_mw).toEqual("number");

      expect(res.body[0]).toHaveProperty("stock_max_mwh");
      expect(typeof res.body[0].stock_max_mwh).toEqual("number");

      expect(res.body[0]).toHaveProperty("price_eur_per_mwh");
      expect(typeof res.body[0].price_eur_per_mwh).toEqual("number");

      expect(res.body[0]).toHaveProperty("planning");
      expect(typeof res.body[0].planning).toEqual("number");
    } catch (err) {
      fail(err);
    }
  });
});

/**
 * GET /session/:session_id/user/:user_id/conso
 */
describe("Getting conso info for a running auction", () => {
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
      await superagent.get(`${url}/session/${uuid()}/user/${uuid()}/conso`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      await superagent.get(`${url}/session/${session_id}/user/${uuid()}/conso`);
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should get conso infos", async () => {
    try {
      const res = await superagent.get(
        `${url}/session/${session_id}/user/${user_id}/conso`
      );
      expect(res.body).toHaveProperty("conso_mw");
      expect(typeof res.body.conso_mw).toEqual("number");
    } catch (err) {
      fail(err);
    }
  });
});
/**
 * PUT /session/:session_id/user/:user_id/planning
 */
describe("Should post a complete production planning", () => {
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
      await superagent.put(`${url}/session/${uuid()}/user/${uuid()}/planning`);
    } catch (error) {
      expect(error.response.text).toEqual(
        "Error, no session found with this ID"
      );
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the user does not exist", async () => {
    try {
      const { session_id: sid } = await insertRunningSession("Running session");
      await superagent
        .put(`${url}/session/${sid}/user/${uuid()}/planning`)
        .send();
    } catch (error) {
      expect(error.response.text).toEqual("Error, no user found with this ID");
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when the session is not running", async () => {
    try {
      await superagent.put(
        `${url}/session/${session_id}/user/${user_id}/planning`
      );
    } catch (error) {
      expect(error.response.text).toEqual("Error, the session is not running");
      expect(error.status).toEqual(400);
    }
  });

  test("Should insert a planning", async () => {
    try {
      // Start session and get default planning
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running session"
      );
      const portfolio = await getUserPortfolio(session_id, user_id_1);
      const planning = portfolio.map((pp) => {
        return {
          user_id: pp.user_id,
          session_id: pp.session_id,
          plant_id: pp.id,
          p_mw: pp.p_max_mw,
        };
      });

      // Put the planning and check response status
      const res = await superagent
        .put(`${url}/session/${session_id}/user/${user_id_1}/planning`)
        .send(planning);
      expect(res.status).toEqual(201);

      // Check the planning via GET planning
      const res_get = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/planning`
      );
      expect(res_get.status).toEqual(200);
      expect(Array.isArray(res_get.body)).toEqual(true);
      expect(res_get.body.length).toEqual(planning.length);
      expect(
        res_get.body.sort((a, b) => (a.plant_id < b.plant_id ? 1 : -1))
      ).toEqual(planning.sort((a, b) => (a.plant_id < b.plant_id ? 1 : -1)));
    } catch (error) {
      fail(error);
    }
  });

  test("Should modify an existing planning", async () => {
    try {
      // Start session and get default planning
      const { session_id, user_id_1 } = await insertRunningSession(
        "Running session"
      );
      const portfolio = await getUserPortfolio(session_id, user_id_1);
      let planning = portfolio.map((pp) => {
        return {
          user_id: pp.user_id,
          session_id: pp.session_id,
          plant_id: pp.id,
          p_mw: pp.p_max_mw,
        };
      });

      // Put the planning and check response status
      const res = await superagent
        .put(`${url}/session/${session_id}/user/${user_id_1}/planning`)
        .send(planning);
      expect(res.status).toEqual(201);

      // PUT updated planning
      planning = portfolio.map((pp) => {
        return {
          user_id: pp.user_id,
          session_id: pp.session_id,
          plant_id: pp.id,
          p_mw: pp.p_min_mw,
        };
      });
      await superagent
        .put(`${url}/session/${session_id}/user/${user_id_1}/planning`)
        .send(planning);

      // Check the planning via GET planning
      const res_get = await superagent.get(
        `${url}/session/${session_id}/user/${user_id_1}/planning`
      );
      expect(res_get.status).toEqual(200);
      expect(Array.isArray(res_get.body)).toEqual(true);
      expect(res_get.body.length).toEqual(planning.length);
      expect(
        res_get.body.sort((a, b) => (a.plant_id < b.plant_id ? 1 : -1))
      ).toEqual(planning.sort((a, b) => (a.plant_id < b.plant_id ? 1 : -1)));
    } catch (error) {
      fail(error);
    }
  });
});
