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

  });
});

