import db from "../../src/db/index";
import { v4 as uuid, validate as uuidValidate } from "uuid";
import superagent from "superagent";

const url = process.env.API_URL;

const auctions = [
  { id: uuid(), name: "Open auction", status: "Open" },
  { id: uuid(), name: "Running auction", status: "Running" },
  { id: uuid(), name: "Close auction", status: "Close" },
];

function mapAuction(auction) {
  return [auction.id, auction.name, auction.status];
}

async function clearDb() {
  await db.query("DELETE FROM users CASCADE", []);
  await db.query("DELETE FROM auctions CASCADE", []);
}
async function populateDb() {
  await Promise.all(
    auctions.map((auction) => {
      db.query(
        "INSERT INTO auctions (id, name, status) VALUES ($1, $2, $3)",
        mapAuction(auction)
      );
    })
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
 * PUT /auction/register_user
 */
describe("Registering a new user to an auction", () => {
  const endpoint = "/auction/register_user";
  beforeAll(async () => {
    await prepareDB();
  });

  it("Should return error 400 when no credentials are provided", async () => {
    try {
      await superagent.put(`${url}${endpoint}`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual("Error, no auction key provided");
    }
  });

  it("Should return error 404 when the auction_id does not correspond to an existing auction", async () => {
    try {
      await superagent.put(`${url}${endpoint}`).send({ auction_id: "toto" });
    } catch (err) {
      expect(err.status).toEqual(404);
      expect(err.response.text).toEqual(
        "Error, the auction_id does not correspond to an existing auction"
      );
    }
  });

  it("Should return error 403 when the auction_id correspond to a running auction", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ auction_id: auctions[1].id });
    } catch (err) {
      expect(err.status).toEqual(403);
      expect(err.response.text).toEqual(
        "Error, the auction_id corresponds to a running auction"
      );
    }
  });

  it("Should return error 403 when the auction_id correspond to a closed auction", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ auction_id: auctions[2].id });
    } catch (err) {
      expect(err.status).toEqual(403);
      expect(err.response.text).toEqual(
        "Error, the auction_id corresponds to a closed auction"
      );
    }
  });

  it("Should a return a 400 error when no username is provided", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ auction_id: auctions[0].id, username: "User" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual("Error, no username provided");
    }
  });

  it("Should a return a uuid_v4 user_id on successful registration", async () => {
    const res = await superagent
      .put(`${url}${endpoint}`)
      .send({ auction_id: auctions[0].id, username: "User" });
    const body = JSON.parse(res.text);
    expect(res.status).toEqual(201);
    expect(body).toHaveProperty("user_id");
    expect(uuidValidate(body.user_id)).toBe(true);
  });

  it("Should return error 409 when we try to a register a user whose name already exists", async () => {
    try {
      await superagent
        .put(`${url}${endpoint}`)
        .send({ auction_id: auctions[0].id, username: "User" });
    } catch (err) {
      expect(err.status).toEqual(409);
      expect(err.response.text).toEqual(
        "Error, a user with this username is already registered to the auction"
      );
    }
  });
});
