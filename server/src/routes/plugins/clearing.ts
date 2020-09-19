/**
 * Functions to perform the clearing of the bids and compute
 * the corresponding energy exchanges.
 */

import db from "../../db";
import { Bid } from "../types";
import { getCurrentPhaseNo } from "../utils";
import { sendUpdateToUsers } from "../websocket";

// Gather all bids
// Sort the bids, desc. for the buyers and asc. for the sellers
// Compute the intersection (what if the intersection if a segment?)

/**
 * Get the list of bids.
 * @param session_id Session ID
 */
export async function getAllBids(session_id: string): Promise<Bid[]> {
  const phase_no = await getCurrentPhaseNo(session_id);
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

  // Do the actual clearing, it may take some time
  await new Promise((r) => setTimeout(r, 10000));

  // When clearing is done, notify the users and mark clearing available as true
  await db.query(
    "UPDATE phases SET clearing_available=true WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  sendUpdateToUsers(session_id, "clearing-finished", {});
}
