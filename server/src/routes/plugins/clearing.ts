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
async function getAllBids(session_id: string): Promise<Bid[]> {
  const phase_no = await getCurrentPhaseNo(session_id);
  return (
    await db.query("SELECT * FROM bids WHERE session_id=$1 AND phase_no=$2", [
      session_id,
      phase_no,
    ])
  ).rows as Bid[];
}

/**
 * Sort the sell bids in ascending order.
 * @param bids List of bids
 */
function sortSellBids(bids: Bid[]): Bid[] {
  return bids
    .filter((bid) => bid.type === "sell")
    .sort((a, b) => a.price_eur_per_mwh - b.price_eur_per_mwh);
}

/**
 * Sort the buy bids in descending order.
 * @param bids List of bids
 */
function sortBuyBids(bids: Bid[]): Bid[] {
  return bids
    .filter((bid) => bid.type === "buy")
    .sort((a, b) => b.price_eur_per_mwh - a.price_eur_per_mwh);
}

interface ClearingStep {
  cum_vol: number;
  price: number;
}

function makeClearingFunction(bids: Bid[]): ClearingStep[] {
  const f = [];
  let sum = 0;
  for (let i = 0; i < bids.length; i++) {
    f.push({
      cum_vol: sum + bids[i].volume_mwh,
      price: bids[i].price_eur_per_mwh,
    });
    sum += bids[i].volume_mwh;
  }
  return f;
}

interface Clearing {
  price: number;
  volume: number;
}
/* 
function computeClearing(bids: Bid[]): Clearing {
  const clearing: Clearing = { price: 0, volume: 0 };
  const sell = makeClearingFunction(sortSellBids(bids));
  const buy = makeClearingFunction(sortBuyBids(bids));

  if (sell[0].price <= buy[0].price) {
    let cleared = false;
    const ib = 0;
    const is = 0;

    while (!cleared) {
      if( ib === buy.length) {
        cleared = true
        clearing = 
      }
      if (sell[is].cum_vol > buy[ib].cum_vol) {
        //
      }
    }
  }

  return clearing;
}
 */

export async function clearing(
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
