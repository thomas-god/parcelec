import * as clearing from "./clearing";

const bids_sell = [
  { type: "sell", volume_mwh: 3, price_eur_per_mwh: 1 },
  { type: "sell", volume_mwh: 6, price_eur_per_mwh: 3 },
  { type: "sell", volume_mwh: 5, price_eur_per_mwh: 5 },
];
const bids_buy = [
  { type: "buy", volume_mwh: 1, price_eur_per_mwh: 7 },
  { type: "buy", volume_mwh: 4, price_eur_per_mwh: 5 },
  { type: "buy", volume_mwh: 2, price_eur_per_mwh: 4 },
  { type: "buy", volume_mwh: 4, price_eur_per_mwh: 2 },
  { type: "buy", volume_mwh: 3, price_eur_per_mwh: 1 },
];

const bids = shuffle([...bids_sell, ...bids_buy]);

describe("Sorting the bids into sell and buy", () => {
  test("Should return 2 arrays with element sorted per volume_mwh", () => {
    const sorted_bids = clearing.sortBids(bids);
    expect(sorted_bids.length).toEqual(2);
    const [sell, buy] = sorted_bids;
    expect(sell).toEqual(bids_sell);
    expect(buy).toEqual(bids_buy);
  });

  test("Should merge bids with the same price", () => {
    const bids_sell = [
      { type: "sell", volume_mwh: 3, price_eur_per_mwh: 1 },
      { type: "sell", volume_mwh: 6, price_eur_per_mwh: 3 },
      { type: "sell", volume_mwh: 5, price_eur_per_mwh: 3 },
      { type: "sell", volume_mwh: 5, price_eur_per_mwh: 5 },
    ];
    const bids_buy = [
      { type: "buy", volume_mwh: 1, price_eur_per_mwh: 7 },
      { type: "buy", volume_mwh: 4, price_eur_per_mwh: 5 },
      { type: "buy", volume_mwh: 2, price_eur_per_mwh: 4 },
      { type: "buy", volume_mwh: 5, price_eur_per_mwh: 4 },
      { type: "buy", volume_mwh: 4, price_eur_per_mwh: 2 },
      { type: "buy", volume_mwh: 3, price_eur_per_mwh: 1 },
    ];
    const bids = shuffle([...bids_sell, ...bids_buy]);
    const [sell, buy] = clearing.sortBids(bids);
    expect(sell.length).toEqual(bids_sell.length - 1);
    expect(sell.find((b) => b.price_eur_per_mwh === 3).volume_mwh).toEqual(11);
    expect(buy.length).toEqual(bids_buy.length - 1);
    expect(buy.find((b) => b.price_eur_per_mwh === 4).volume_mwh).toEqual(7);
  });
});

describe("Computing the clearing function", () => {
  test("Should compute a function (array) with correct properties", () => {
    const [sell, buy] = clearing.sortBids(bids);
    const sell_fun = clearing.getBidFunction(sell);
    expect(sell_fun.length).toEqual(bids_sell.length);
    sell_fun.forEach((i) => {
      expect(i).toHaveProperty("vol_start");
      expect(i).toHaveProperty("vol_end");
      expect(i).toHaveProperty("price");
      expect(i.vol_start).toBeLessThan(i.vol_end);
    });
    expect(sell_fun[0].vol_start).toEqual(0);
    expect(sell_fun[sell_fun.length - 1].vol_end).toEqual(
      bids_sell.reduce((a, b) => a + b.volume_mwh, 0 as number)
    );
  });
});

describe("Computing the clearing value", () => {
  test("It should get the right value", () => {
    const [sell, buy] = clearing.sortBids(bids);
    const sell_fun = clearing.getBidFunction(sell);
    const buy_fun = clearing.getBidFunction(buy);
    const [clearing_value, internal_infos] = clearing.computeClearing(
      sell_fun,
      buy_fun
    );
    expect(clearing_value).toHaveProperty("volume");
    expect(clearing_value).toHaveProperty("price");
    expect(clearing_value.volume).toEqual(7);
    expect(clearing_value.price).toEqual(3);
  });
});

describe("Compute correct clearing values for various bids scenarios", () => {
  test("Bids scenario 1", () => {
    const bids = [
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 100, price_eur_per_mwh: 20 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(100);
    expect(clearing_value.price).toEqual(10);
  });

  test("Bids scenario 2", () => {
    const bids = [
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 50, price_eur_per_mwh: 20 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(50);
    expect(clearing_value.price).toEqual(10);
  });

  test("Bids scenario 3", () => {
    const bids = [
      { type: "sell", volume_mwh: 50, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 100, price_eur_per_mwh: 20 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(50);
    expect(clearing_value.price).toEqual(10);
  });

  test("Bids scenario 4", () => {
    const bids = [
      { type: "sell", volume_mwh: 10, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 10, price_eur_per_mwh: 10 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(10);
    expect(clearing_value.price).toEqual(10);
  });

  test("Bids scenario 5", () => {
    const bids = [
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 5 },
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 100, price_eur_per_mwh: 20 },
      { type: "buy", volume_mwh: 100, price_eur_per_mwh: 5 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(100);
    expect(clearing_value.price).toEqual(5);
  });

  test("Bids scenario 6", () => {
    const bids = [
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 5 },
      { type: "sell", volume_mwh: 100, price_eur_per_mwh: 10 },
      { type: "buy", volume_mwh: 100, price_eur_per_mwh: 20 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(100);
    expect(clearing_value.price).toEqual(5);
  });

  test("Bids scenario 7", () => {
    const bids = [
      { type: "sell", volume_mwh: 12, price_eur_per_mwh: 20 },
      { type: "buy", volume_mwh: 15, price_eur_per_mwh: 16 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(0);
    expect(clearing_value.price).toEqual(0);
  });

  test("Bids scenario 8", () => {
    const bids = [
      { type: "sell", volume_mwh: 400, price_eur_per_mwh: 25 },
      { type: "buy", volume_mwh: 200, price_eur_per_mwh: 25 },
      { type: "buy", volume_mwh: 200, price_eur_per_mwh: 40 },
    ];
    const clearing_value = computeClearing(bids);
    expect(clearing_value.volume).toEqual(400);
    expect(clearing_value.price).toEqual(25);
  });
});

/**
 * Helping function to do the clearing from a list of bids.
 * @param bids List of bids
 */
function computeClearing(bids) {
  const [sell, buy] = clearing.sortBids(bids);
  const sell_fun = clearing.getBidFunction(sell);
  const buy_fun = clearing.getBidFunction(buy);
  const [clearing_value, internal_infos] = clearing.computeClearing(
    sell_fun,
    buy_fun
  );
  return clearing_value;
}

/**
 * Shuffles an array in place.
 * @param {Array} a items An array containing the items.
 * Taken from https://stackoverflow.com/a/6274381 and inspired from
 * https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle#The_modern_algorithm
 */
function shuffle(a) {
  let j, x, i;
  for (i = a.length - 1; i > 0; i--) {
    j = Math.floor(Math.random() * (i + 1));
    x = a[i];
    a[i] = a[j];
    a[j] = x;
  }
  return a;
}
