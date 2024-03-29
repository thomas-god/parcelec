/**
 * Functions to perform the clearing of the bids and compute
 * the corresponding energy exchanges.
 */

import db from "../../db";
import { Bid } from "../types";
import { getSessionUsers, getUserBids } from "../utils";
import { sendUpdateToUsers } from "../websocket";
import logger from "../../utils/log";

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
      await db.query(
        `SELECT 
          id, 
          user_id, 
          session_id, 
          phase_no, 
          type, 
          volume_mwh, 
          price_eur_per_mwh
        FROM bids 
        WHERE 
          session_id=$1 
          AND phase_no=$2;`,
        [session_id, phase_no]
      )
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

  // Sort sell in ascending order and merge same prices
  bids_sell.sort((a, b) => a.price_eur_per_mwh - b.price_eur_per_mwh);
  mergeSamePriceBids(bids_sell);

  // Sort buy in ascending order, merge same prices, and finally sort
  // in descending order
  bids_buy.sort((a, b) => a.price_eur_per_mwh - b.price_eur_per_mwh);
  mergeSamePriceBids(bids_buy);
  bids_buy.sort((a, b) => b.price_eur_per_mwh - a.price_eur_per_mwh);

  return [bids_sell, bids_buy];
}

/**
 * Merge in place bids that are the same price into a single bid with the
 * corresponding total volume.
 * @param bids List of ordered bids
 */
function mergeSamePriceBids(bids: Bid[]): Bid[] {
  let i_previous = 0;
  let n_removal = 0;
  for (let i = 1; i < bids.length; i++) {
    if (
      bids[i - n_removal].price_eur_per_mwh ===
      bids[i_previous].price_eur_per_mwh
    ) {
      bids[i_previous].volume_mwh += bids[i - n_removal].volume_mwh;
      bids.splice(i - n_removal, 1);
      n_removal++;
    } else {
      i_previous = i - n_removal;
    }
  }
  return bids;
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

export interface ClearingInternalInfos {
  buy_last_bid_price: number;
  buy_last_bid_frac_volume: number;
  sell_last_bid_price: number;
  sell_last_bid_frac_volume: number;
}

/**
 * Compute the intersection between the offer and demand curves.
 *
 * @param sell Offer curve
 * @param buy Demand curve
 */
export function computeClearing(
  sell: ClearingFunction,
  buy: ClearingFunction
): [Clearing, ClearingInternalInfos] {
  const clearing = {
    price: 0,
    volume: 0,
  };
  const internal_infos = {
    buy_last_bid_price: 0,
    buy_last_bid_frac_volume: 0,
    sell_last_bid_price: 0,
    sell_last_bid_frac_volume: 0,
  };

  if (sell.length !== 0 && buy.length !== 0) {
    const ns = sell.length;
    const nb = buy.length;
    let is = 0;
    let ib = 0;
    let flag = true;
    while (is < ns && ib < nb && flag) {
      if (sell[is].vol_end < buy[ib].vol_end) {
        const p = findPrice(sell[is].vol_end, buy);
        if (is + 1 < ns) {
          if (sell[is].price <= p && p < sell[is + 1].price) {
            // Clearing if sell curve intersects buy curve
            clearing.price = p;
            clearing.volume = sell[is].vol_end;
            flag = false;
          } else {
            is++;
          }
        } else {
          if (sell[is].price <= p) {
            // Clearing if sell curve is finished and lesser than buy curve
            clearing.price = sell[is].price;
            clearing.volume = sell[is].vol_end;
            flag = false;
          } else {
            is++;
          }
        }
      } else if (buy[ib].vol_end < sell[is].vol_end) {
        const p = findPrice(buy[ib].vol_end, sell);
        if (ib + 1 < nb) {
          if (buy[ib + 1].price < p && p <= buy[ib].price) {
            // Clearing if buy curve intersects sell curve
            clearing.price = p;
            clearing.volume = buy[ib].vol_end;
            flag = false;
          } else {
            ib++;
          }
        } else {
          if (buy[ib].price >= p) {
            // Clearing if the buy curve is finished and greater than sell curve
            clearing.price = p;
            clearing.volume = buy[ib].vol_end;
            flag = false;
          } else {
            ib++;
          }
        }
      } else {
        if (is + 1 < ns && ib + 1 < nb) {
          if (sell[is + 1].price >= buy[ib + 1].price) {
            // Clearing if curves cross each other
            clearing.price = sell[is].price;
            clearing.volume = sell[is].vol_end;
            flag = false;
          } else {
            is++;
            ib++;
          }
        } else {
          if (sell[is].price <= buy[ib].price) {
            // Clearing if buy curve is still above
            clearing.price = sell[is].price;
            clearing.volume = Math.min(sell[is].vol_end, buy[ib].vol_end);
            flag = false;
          } else {
            is++;
            ib++;
          }
        }
      }
    }
    // Compute internal information based on last bids index if
    // clearing is successful
    if (is !== ns && ib !== nb) {
      internal_infos.buy_last_bid_price = buy[ib].price;
      internal_infos.buy_last_bid_frac_volume =
        (clearing.volume - buy[ib].vol_start) /
        (buy[ib].vol_end - buy[ib].vol_start);
      internal_infos.sell_last_bid_price = sell[is].price;
      internal_infos.sell_last_bid_frac_volume =
        (clearing.volume - sell[is].vol_start) /
        (sell[is].vol_end - sell[is].vol_start);
    }
  }

  return [clearing, internal_infos];
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
): Promise<[Clearing, ClearingInternalInfos]> {
  logger.info("starting clearing procedure", {
    session_id,
    phase_no,
  });
  const bids = await getAllBids(session_id, phase_no);
  const [sell, buy] = sortBids(bids);
  const sell_fun = getBidFunction(sell);
  const buy_fun = getBidFunction(buy);
  const [clearing_value, internal_infos] = computeClearing(sell_fun, buy_fun);
  logger.info(
    `clearing done at volume: ${clearing_value.volume} MWh, price ${clearing_value.price} €/MWh`,
    {
      session_id,
      phase_no,
    }
  );
  return [clearing_value, internal_infos];
}

export async function computeAndInsertEnergyExchanges(
  session_id: string,
  phase_no: number,
  clearing_value: Clearing,
  clearing_infos: ClearingInternalInfos
): Promise<void> {
  if (clearing_value.volume > 0) {
    const users = await getSessionUsers(session_id);
    await Promise.all(
      users.map(async (user) => {
        const bids = await getUserBids(session_id, user.id, phase_no);
        const [sell, buy] = sortBids(bids);

        // Sell exchange
        const sell_ok_vol = sell
          .map((bid) => {
            if (bid.price_eur_per_mwh < clearing_infos.sell_last_bid_price) {
              return bid.volume_mwh;
            } else if (
              bid.price_eur_per_mwh === clearing_infos.sell_last_bid_price
            ) {
              return bid.volume_mwh * clearing_infos.sell_last_bid_frac_volume;
            } else {
              return 0;
            }
          })
          .reduce((a, b) => a + b, 0);
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

        // Buy exchange
        const buy_ok_vol = buy
          .map((bid) => {
            if (bid.price_eur_per_mwh > clearing_infos.buy_last_bid_price) {
              return bid.volume_mwh;
            } else if (
              bid.price_eur_per_mwh === clearing_infos.buy_last_bid_price
            ) {
              return bid.volume_mwh * clearing_infos.buy_last_bid_frac_volume;
            } else {
              return 0;
            }
          })
          .reduce((a, b) => a + b, 0);
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
  const [clearing_value, internal_infos] = await doClearingProcedure(
    session_id,
    phase_no
  );
  await db.query(
    `INSERT INTO clearings 
      (
        session_id, 
        phase_no, 
        volume_mwh, 
        price_eur_per_mwh, 
        internal_buy_last_bid_price,
        internal_buy_last_bid_frac_volume,
        internal_sell_last_bid_price,
        internal_sell_last_bid_frac_volume
      ) 
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    `,
    [
      session_id,
      phase_no,
      clearing_value.volume,
      clearing_value.price,
      internal_infos.buy_last_bid_price,
      internal_infos.buy_last_bid_frac_volume,
      internal_infos.sell_last_bid_price,
      internal_infos.sell_last_bid_frac_volume,
    ]
  );

  // After clearing, compute the energy exchanges for each user
  await computeAndInsertEnergyExchanges(
    session_id,
    phase_no,
    clearing_value,
    internal_infos
  );

  // When clearing is done, notify the users and mark clearing available as true
  await db.query(
    "UPDATE phases SET clearing_available=true WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  sendUpdateToUsers(session_id, "clearing-finished", {});
}
