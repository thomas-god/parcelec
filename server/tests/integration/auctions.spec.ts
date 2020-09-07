import db from "../../src/db/index";
import { v4 as uuid } from "uuid";
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

describe("Auctions listing", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should list the available auctions that are currently open", async () => {
    const res = await superagent.get(`${url}/auction/list_open`);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(1);
    expect(body[0].id).toEqual(auctions[0].id);
    expect(body[0].name).toEqual("Open auction");
  });

  test("Should return an empty object if there are no open auctions", async () => {
    await clearDb();
    const res = await superagent.get(`${url}/auction/list_open`);
    const body = JSON.parse(res.text);
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toEqual(0);
  });
});

describe("Opening a new auction", () => {
  beforeAll(async () => {
    await prepareDB();
  });

  test("Should return a 400 error if no auction_name is provided in body", async () => {
    try {
      await superagent.put(`${url}/auction/open`);
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, please provide a valid session name"
      );
    }
  });

  test("It should return a 400 error if the name is already taken", async () => {
    try {
      await superagent
        .put(`${url}/auction/open`)
        .send({ auction_name: "Open auction" });
    } catch (err) {
      expect(err.status).toEqual(400);
      expect(err.response.text).toEqual(
        "Error, a session already exists with this name"
      );
    }
  });

  test("It should get back an auction object on success", async () => {
    const res = await superagent
      .put(`${url}/auction/open`)
      .send({ auction_name: "My auction" });
    const body = JSON.parse(res.text);
    expect(res.status).toEqual(201);
    expect(body.name).toEqual("My auction");
    expect(body.status).toEqual("Open");
    expect(body).toHaveProperty("id");
  });
});
