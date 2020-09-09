import db from "../../src/db/index";
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";
import { json } from "express";

const url = process.env.API_URL;

const auctions = [
  { id: uuid(), name: "Open auction", status: "Open" },
  { id: uuid(), name: "Running auction", status: "Running" },
  { id: uuid(), name: "Close auction", status: "Close" },
];

const users = [
  { auction_id: auctions[1].id, name: "User 1", id: uuid() },
  { auction_id: auctions[1].id, name: "User 2", id: uuid() },
];

function mapAuction(auction) {
  return [auction.id, auction.name, auction.status];
}

async function clearDb() {
  await db.query("DELETE FROM bids CASCADE", []);
  await db.query("DELETE FROM auctions_steps CASCADE", []);
  await db.query("DELETE FROM users CASCADE", []);
  await db.query("DELETE FROM auctions CASCADE", []);
}
async function populateDb() {
  // Insert auctions
  await Promise.all(
    auctions.map(async (auction) => {
      await db.query(
        "INSERT INTO auctions (id, name, status) VALUES ($1, $2, $3)",
        mapAuction(auction)
      );
    })
  );

  // Insert Users
  await Promise.all(
    users.map(async (user) => {
      await db.query(
        "INSERT INTO users (id, name, auction_id) VALUES ($1, $2, $3)",
        [user.id, user.name, user.auction_id]
      );
    })
  );

  // Insert one auction step for the running auction
  await db.query(
    "INSERT INTO auctions_steps (auction_id, step_no, status) VALUES ($1, $2, $3)",
    [auctions[1].id, 0, "open"]
  );
}

async function prepareDB() {
  await clearDb();
  await populateDb();
}

/**
 * Get /auction/list_open
 */
describe("Listing open auctions", () => {
  const endpoint = "/auction/list_open";
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should list the available auctions that are currently open", async () => {
    const res = await superagent.get(`${url}${endpoint}`);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(1);
    expect(body[0].id).toEqual(auctions[0].id);
    expect(body[0].name).toEqual("Open auction");
  });

  test("Should return an empty object if there are no open auctions", async () => {
    await clearDb();
    const res = await superagent.get(`${url}${endpoint}`);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(0);
  });
});

/**
 * PUT /auction/open
 */
describe("Opening a new auction", () => {
  const endpoint = "/auction/open";
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should return a 400 error if no auction_name is provided in body", async () => {
    try {
      await superagent.put(`${url}${endpoint}`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, please provide a valid session name"
      );
    }
  });

  test("It should return a 409 error if the name is already taken", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ auction_name: "Open auction" });
    } catch (err) {
      expect(err.status).toEqual(409);
      expect(err.response.text).toEqual(
        "Error, a session already exists with this name"
      );
    }
  });

  test("It should get back an auction object on success", async () => {
    const res = await superagent
      .put(`${url}${endpoint}`)
      .send({ auction_name: "My auction" });
    const body = JSON.parse(res.text);
    expect(res.status).toEqual(201);
    expect(body.name).toEqual("My auction");
    expect(body.status).toEqual("Open");
    expect(body).toHaveProperty("id");
  });
});

/**
 * PUT /auction/:auction_id/register_user
 */
describe("Registering a new user to an auction", () => {
  const endpoint = "/auction/register_user";
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should return error 404 when no credentials are provided", async () => {
    try {
      await superagent.put(`${url}/auction/register_user`);
    } catch (err) {
      expect(err.status).toEqual(404);
    }
  });

  test("Should error when the auction_id does not correspond to an existing auction", async () => {
    try {
      await superagent
        .put(`${url}/auction/${uuid()}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(404);
      expect(err.response.text).toEqual(
        "Error, the auction_id does not correspond to an existing auction"
      );
    }
  });

  test("Should error when the auction_id correspond to a running auction", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[1].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, the auction is not open for registration"
      );
    }
  });

  test("Should error when the auction_id correspond to a closed auction", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[2].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, the auction is not open for registration"
      );
    }
  });

  test("Should error when no username is provided", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[0].id}/register_user`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual("Error, no username provided");
    }
  });

  test("Should return a uuid_v4 user_id on successful registration", async () => {
    const res = await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User" });
    const body = JSON.parse(res.text);
    expect(res.status).toEqual(201);
    expect(body).toHaveProperty("user_id");
    expect(uuidValidate(body.user_id)).toBe(true);
  });

  test("Should error when we try to register a user whose name already exists", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[0].id}/register_user`)
        .send({ username: "User" });
    } catch (err) {
      expect(err.status).toEqual(409);
      expect(err.response.text).toEqual(
        "Error, a user with this username is already registered to the auction"
      );
    }
  });
});

/**
 * PUT /auction/start
 */
describe("Starting an auction", () => {
  const endpoint = "/auction/start";
  beforeEach(async () => {
    await prepareDB();
  });

  test("Should error if no auction_id is provided", async () => {
    try {
      await superagent.put(`${url}/auction/start`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("should error if no auction is found", async () => {
    try {
      await superagent.put(`${url}/auction/${uuid()}/start`);
    } catch (error) {
      expect(error.status).toEqual(404);
      expect(error.response.text).toEqual(
        "Error, the auction ID does not match an existing auction"
      );
    }
  });

  test("Should error if the auction is already running", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[1].id}/start`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, the auction is already running"
      );
    }
  });

  test("should error if the auction is closed", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[2].id}/start`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, the auction is closed");
    }
  });

  test("Should error if there is no users registered to the auction", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[0].id}/start`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, not enough users registered to start the session"
      );
    }
  });

  test("Should error if only 1 user is registered to the auction", async () => {
    await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User 1" });
    try {
      await superagent.put(`${url}/auction/${auctions[0].id}/start`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, not enough users registered to start the session"
      );
    }
  });

  test("Should return a 200 on success", async () => {
    await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User 1" });
    await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User 2" });
    const res = await superagent.put(`${url}/auction/${auctions[0].id}/start`);
    expect(res.status).toEqual(200);
  });
});

/**
 * GET /auction/:auction_id
 */
describe("Retrieving auction infos", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should get a 404 error if no auction_id is provided", async () => {
    try {
      await superagent.get(`${url}/auction`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should get basic infos for an open auction", async () => {
    await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User 1" });
    const res = await superagent.get(`${url}/auction/${auctions[0].id}`);
    const body = JSON.parse(res.text);
    expect(body.id).toEqual(auctions[0].id);
    expect(body.name).toEqual(auctions[0].name);
    expect(body.status).toEqual(auctions[0].status);
    expect(body.n_users).toEqual(1);
  });

  test("Should get basic infos for a running auction", async () => {
    await superagent
      .put(`${url}/auction/${auctions[0].id}/register_user`)
      .send({ username: "User 2" });
    await superagent.put(`${url}/auction/${auctions[0].id}/start`);
    const res = await superagent.get(`${url}/auction/${auctions[0].id}`);
    const body = JSON.parse(res.text);
    expect(body.id).toEqual(auctions[0].id);
    expect(body.name).toEqual(auctions[0].name);
    expect(body.status).toEqual("Running");
    expect(body.n_users).toEqual(2);
    expect(body.step_no).toEqual(0);
  });
});

/**
 * PUT /auction/:auction_id/bid
 */
describe("Submitting a bid to an open auction", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should error if no auction id is provided", async () => {
    try {
      await superagent.put(`${url}/auction/bid`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when no matching auction is found", async () => {
    try {
      await superagent.put(`${url}/auction/${uuid()}/bid`);
    } catch (error) {
      expect(error.status).toEqual(404);
    }
  });

  test("Should error when trying to bid on an open auction", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[0].id}/bid`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, the auction is not running and bids cannot be received"
      );
    }
  });

  test("Should error when trying to bid on a closed auction", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[2].id}/bid`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, the auction is not running and bids cannot be received"
      );
    }
  });

  test("Should error when no user id is provided", async () => {
    try {
      await superagent.put(`${url}/auction/${auctions[1].id}/bid`);
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, no user_id specified");
    }
  });

  test("Should error when a non registered user submit a bid", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[1].id}/bid`)
        .send({ user_id: uuid() });
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual(
        "Error, no registered user found with this ID"
      );
    }
  });

  test("Should error when no bid value is provided", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[1].id}/bid`)
        .send({ user_id: users[0].id });
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, no bid value provided");
    }
  });

  test("Should success when a registered user tries to bid", async () => {
    const res = await superagent
      .put(`${url}/auction/${auctions[1].id}/bid`)
      .send({ user_id: users[0].id, bid: 10 });
    expect(res.status).toEqual(201);
  });

  test("Should error when a registered user tries to bid again", async () => {
    try {
      await superagent
        .put(`${url}/auction/${auctions[1].id}/bid`)
        .send({ user_id: users[0].id, bid: 10 });
    } catch (error) {
      expect(error.status).toEqual(400);
      expect(error.response.text).toEqual("Error, this user has already bid");
    }
  });
});
