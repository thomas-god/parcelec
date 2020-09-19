/**
 * Functions to perform the clearing of the bids and compute
 * the corresponding energy exchanges.
 */

import db from "../../db";
import { Bid } from "../types";
import { getSessionUsers, getUserBids } from "../utils";
import { sendUpdateToUsers } from "../websocket";

// Gather all bids
// Sort the bids, desc. for the buyers and asc. for the sellers
// Compute the intersection (what if the intersection if a segment?)

/**
 * Get the list of bids.
 * @param session_id Session ID
 * @param phase_no Number of the game phase
 */
export async function getAllBids(
  session_id: string,
  phase_no: number
): Promise<Bid[]> {
  let bids = [];
  if (phase_no !== null) {
    bids = (
      await db.query("SELECT * FROM bids WHERE session_id=$1 AND phase_no=$2", [
        session_id,
        phase_no,
      ])
    ).rows as Bid[];
  }
  return bids;
}

/**
 * Sort bids by type and sort them accordingly. First item of returned array
 * is sell bids and second item is buy bids.
 * @param bids List of bids
 */
export function sortBids(bids: Bid[]): Bid[][] {
  const bids_sell: Bid[] = [];
  const bids_buy: Bid[] = [];

  // Split bids depending on type
  bids.forEach((bid) => {
    if (bid.type === "buy") {
      bids_buy.push(bid);
    } else {
      bids_sell.push(bid);
    }
  });

  // Sort sell in ascending order and buy in descending order
  // TODO: merge the bids with equal prices
  bids_sell.sort((a, b) => a.price_eur_per_mwh - b.price_eur_per_mwh);
  bids_buy.sort((a, b) => b.price_eur_per_mwh - a.price_eur_per_mwh);

  return [bids_sell, bids_buy];
}

export interface ClearingFunctionItem {
  vol_start: number;
  vol_end: number;
  price: number;
}

export type ClearingFunction = ClearingFunctionItem[];

/**
 * Return the data structure to perform the clearing.
 * @param bids List of sorted bids
 */
export function getBidFunction(bids: Bid[]): ClearingFunction {
  const bids_fmt: ClearingFunctionItem[] = [];
  if (bids.length > 0) {
    for (let i = 0; i < bids.length; i++) {
      const vol_start = i > 0 ? bids_fmt[i - 1].vol_end : 0;
      bids_fmt.push({
        vol_start: vol_start,
        vol_end: vol_start + bids[i].volume_mwh,
        price: bids[i].price_eur_per_mwh,
      });
    }
  }
  return bids_fmt;
}

export interface Clearing {
  price: number;
  volume: number;
}

/**
 * Compute the intersection between the offer and demand curves.
 * @param sell Offer curve
 * @param buy Demand curve
 */
export function computeClearing(
  sell: ClearingFunction,
  buy: ClearingFunction
): Clearing {
  const clearing = {
    price: 0,
    volume: 0,
  };
  if (sell.length !== 0 && buy.length !== 0) {
    const ns = sell.length;
    const nb = buy.length;
    if (sell[0].price <= buy[0].price) {
      // Can do the clearing
      console.log("can do the clearing");

      if (sell[ns - 1].price <= buy[nb - 1].price) {
        // Demand curve is always above supply curve
        clearing.price = (sell[ns - 1].price + buy[nb - 1].price) / 2;
        clearing.volume = Math.min(sell[ns - 1].vol_end, buy[nb - 1].vol_end);
      } else {
        let is = 0;
        let ib = 0;
        while (is < ns && ib < nb) {
          if (sell[is].vol_end < buy[ib].vol_end) {
            const vol = sell[is].vol_end;
            const price_b = findPrice(vol, buy);
            if (sell[is].price < price_b && price_b < sell[is + 1].price) {
              clearing.volume = vol;
              clearing.price = price_b;
              is = ns;
            }
            is++;
          } else {
            const vol = buy[ib].vol_end;
            const price_s = findPrice(vol, sell);
            if (buy[ib + 1].price < price_s && price_s < buy[ib].price) {
              clearing.volume = vol;
              clearing.price = price_s;
              ib = nb;
            }
            ib++;
          }
        }
      }
    }
  }

  return clearing;
}

/**
 * Find the corresponding price on the opposite curve.
 * @param vol Target volume
 * @param fun Supply or demand curve
 */
export function findPrice(vol: number, fun: ClearingFunction): number {
  let i = 0;
  let price = 0;
  while (i < fun.length) {
    if (fun[i].vol_start <= vol && vol < fun[i].vol_end) {
      price = fun[i].price;
      i = fun.length;
    }
    i++;
  }
  return price;
}

/**
 * Do the chaining of the different utils functions for the clearing.
 * @param session_id Session ID
 * @param phase_no Phase number
 */
export async function doClearingProcedure(
  session_id: string,
  phase_no: number
): Promise<Clearing> {
  const bids = await getAllBids(session_id, phase_no);
  const [sell, buy] = sortBids(bids);
  const sell_fun = getBidFunction(sell);
  const buy_fun = getBidFunction(buy);
  const clearing_value = computeClearing(sell_fun, buy_fun);
  return clearing_value;
}

export async function computeAndInsertEnergyExchanges(
  session_id: string,
  phase_no: number,
  clearing_value: Clearing
): Promise<void> {
  // TODO: handle the case where several multiple bids (from different users or not)
  // TODO: have the same price, especially if it is the clearing price as the volume
  // TODO: should then be prorated.
  const users = await getSessionUsers(session_id);
  await Promise.all(
    users.map(async (user) => {
      const bids = await getUserBids(session_id, user.id, phase_no);
      const [sell, buy] = sortBids(bids);
      const sell_ok_vol = sell
        .filter((bid) => bid.price_eur_per_mwh <= clearing_value.price)
        .reduce((a, b) => a + b.volume_mwh, 0 as number);
      if (sell_ok_vol > 0) {
        await db.query(
          `
        INSERT INTO exchanges 
          (user_id, session_id, phase_no, type, volume_mwh, price_eur_per_mwh)
        VALUES
         ($1, $2, $3, 'sell', $4, $5)`,
          [user.id, session_id, phase_no, sell_ok_vol, clearing_value.price]
        );
      }
      const buy_ok_vol = buy
        .filter((bid) => bid.price_eur_per_mwh >= clearing_value.price)
        .reduce((a, b) => a + b.volume_mwh, 0 as number);
      if (buy_ok_vol > 0) {
        await db.query(
          `
        INSERT INTO exchanges 
          (user_id, session_id, phase_no, type, volume_mwh, price_eur_per_mwh)
        VALUES
         ($1, $2, $3, 'buy', $4, $5)`,
          [user.id, session_id, phase_no, buy_ok_vol, clearing_value.price]
        );
      }
    })
  );
}

/**
 * Define the timeline of the clearing process.
 * @param session_id Session ID
 * @param phase_no Number of the current phase
 */
export default async function clearing(
  session_id: string,
  phase_no: number
): Promise<void> {
  // Notify users that auction is closing
  sendUpdateToUsers(session_id, "clearing-started", {});

  // Mark bids_allowed to false
  await db.query(
    "UPDATE phases SET bids_allowed=false WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );

  // Do the actual clearing
  const clearing_value = await doClearingProcedure(session_id, phase_no);
  await db.query(
    `INSERT INTO clearings 
      (session_id, phase_no, volume_mwh, price_eur_per_mwh) 
      VALUES ($1, $2, $3, $4)
    `,
    [session_id, phase_no, clearing_value.volume, clearing_value.price]
  );

  // After clearing, compute the energy exchanges for each user
  await computeAndInsertEnergyExchanges(session_id, phase_no, clearing_value);

  // When clearing is done, notify the users and mark clearing available as true
  await db.query(
    "UPDATE phases SET clearing_available=true WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  sendUpdateToUsers(session_id, "clearing-finished", {});
}
