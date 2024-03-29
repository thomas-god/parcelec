/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */
import db from "../../db";
import { SessionOptions } from "../types";
import {
  getCurrentConsoValue,
  getSession,
  getSessionOptions,
  getSessionUsers,
} from "../utils";
import { sendUpdateToUsers } from "../websocket";

export async function endGame(
  session_id: string,
  phase_no: number
): Promise<void> {
  const session = await getSession(session_id);
  const options = await getSessionOptions(session_id);
  // Notify users that session is finished
  sendUpdateToUsers(session_id, "plannings-closed", {});

  // Mark plannings_allowed to false
  await db.query(
    "UPDATE phases SET plannings_allowed=false WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );

  // Do the metering and results computation, it may take some time
  await computeResults(session_id, phase_no, options);

  // Close the phase and mark all users as not ready for the next phase
  await db.query(
    "UPDATE phases SET status='closed' WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  await db.query("UPDATE users SET game_ready=false WHERE session_id=$1", [
    session_id,
  ]);

  // When results are computed, notify the users and mark it the phase table
  await db.query(
    "UPDATE phases SET results_available=true WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );
  sendUpdateToUsers(session_id, "results-available", {});

  // If its the last phase, close the session
  if (phase_no === options.phases_number - 1) {
    await db.query("UPDATE sessions SET status=$1 WHERE id=$2", [
      "closed",
      session.id,
    ]);
    session.status = "closed";
    sendUpdateToUsers(session_id, "game-session-ended", {});
  }
}

/**
 * Compute the energy end financial results for each user.
 * @param session_id Session ID
 * @param phase_no Phase number
 */
async function computeResults(
  session_id: string,
  phase_no: number,
  options: SessionOptions
): Promise<void> {
  const users = await getSessionUsers(session_id);
  await Promise.all(
    users.map(async (user) => {
      const results: any = {
        user_id: user.id,
        session_id: session_id,
        phase_no: phase_no,
      };

      // Consumption
      results.conso_mwh = await getCurrentConsoValue(
        session_id,
        user.id,
        phase_no
      );
      results.conso_eur = results.conso_mwh * options.conso_price_eur[phase_no];

      // Production
      const prod = (
        await db.query(
          `
          SELECT 
            SUM(pl.p_mw) AS prod_mwh,
            SUM(pl.p_mw * pp.price_eur_per_mwh) AS prod_eur
          FROM production_plannings AS pl
          INNER JOIN power_plants AS pp
            on pl.plant_id=pp.id
          WHERE pl.session_id=$1 AND pl.user_id=$2 AND pl.phase_no=$3`,
          [session_id, user.id, phase_no]
        )
      ).rows;
      results.prod_mwh = prod.length === 1 ? prod[0].prod_mwh : 0;
      results.prod_eur = prod.length === 1 ? prod[0].prod_eur : 0;

      // Energy exchanges (via market)
      const exchanges = (
        await db.query(
          `
        SELECT 
          type, volume_mwh, volume_mwh * price_eur_per_mwh AS price_eur
        FROM exchanges
        WHERE 
          session_id=$1 AND user_id=$2 AND phase_no=$3`,
          [session_id, user.id, phase_no]
        )
      ).rows;
      const buy = exchanges.find((e) => e.type === "buy");
      results.buy_mwh = buy !== undefined ? buy.volume_mwh : 0;
      results.buy_eur = buy !== undefined ? buy.price_eur : 0;
      const sell = exchanges.find((e) => e.type === "sell");
      results.sell_mwh = sell !== undefined ? sell.volume_mwh : 0;
      results.sell_eur = sell !== undefined ? sell.price_eur : 0;

      // Energy exchanges (via OTC)
      const otcs = (
        await db.query(
          `SELECT
            user_from_id,
            user_to_id,
            type,
            volume_mwh,
            volume_mwh * price_eur_per_mwh AS price_eur
          FROM otc_exchanges
          WHERE 
            session_id=$1
            AND phase_no=$2
            AND (user_from_id=$3
            OR user_to_id=$3)
            AND status='accepted';`,
          [session_id, phase_no, user.id]
        )
      ).rows;
      otcs.forEach((otc) => {
        if (
          (otc.user_from_id === user.id && otc.type === "buy") ||
          (otc.user_to_id === user.id && otc.type === "sell")
        ) {
          results.buy_mwh += otc.volume_mwh;
          results.buy_eur += otc.price_eur;
        }
        if (
          (otc.user_from_id === user.id && otc.type === "sell") ||
          (otc.user_to_id === user.id && otc.type === "buy")
        ) {
          results.sell_mwh += otc.volume_mwh;
          results.sell_eur += otc.price_eur;
        }
      });

      // Imbalance
      results.imbalance_mwh =
        results.prod_mwh +
        results.buy_mwh -
        results.conso_mwh -
        results.sell_mwh;
      results.imbalance_costs_eur =
        results.imbalance_mwh > 0
          ? (results.imbalance_mwh * options.conso_price_eur[phase_no]) /
            options.imbalance_costs_factor[phase_no]
          : results.imbalance_mwh *
            options.conso_price_eur[phase_no] *
            options.imbalance_costs_factor[phase_no];

      // Total financial balance
      results.balance_eur =
        results.conso_eur +
        results.sell_eur -
        results.prod_eur -
        results.buy_eur +
        results.imbalance_costs_eur;

      // Overall balance
      results.balance_overall_eur = results.balance_eur;
      if (phase_no > 0) {
        const rows = (
          await db.query(
            `SELECT
              balance_overall_eur
            FROM results
            WHERE 
              session_id=$1
              AND user_id=$2
              AND phase_no=$3;`,
            [session_id, user.id, phase_no - 1]
          )
        ).rows;
        if (rows.length === 1)
          results.balance_overall_eur += rows[0].balance_overall_eur;
      }

      // Insert all into table
      await db.query(
        `
      INSERT INTO results (
        user_id,
        session_id,
        phase_no,
        conso_mwh,
        conso_eur,
        prod_mwh,
        prod_eur,
        sell_mwh,
        sell_eur,
        buy_mwh,
        buy_eur,
        imbalance_mwh,
        imbalance_costs_eur,
        balance_eur,
        balance_overall_eur
      )
      VALUES 
      ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)`,
        [
          user.id,
          session_id,
          phase_no,
          results.conso_mwh,
          results.conso_eur,
          results.prod_mwh,
          results.prod_eur,
          results.sell_mwh,
          results.sell_eur,
          results.buy_mwh,
          results.buy_eur,
          results.imbalance_mwh,
          results.imbalance_costs_eur,
          results.balance_eur,
          results.balance_overall_eur,
        ]
      );
    })
  );

  // All players' results have been computed, compute current phase and overall
  // ranking for each player
  console.log("computing ranking");
  const rankings = (
    await db.query(
      `WITH r AS (
      SELECT 
        SUM(balance_eur) AS total_eur,
        user_id AS user_id
      FROM results
      WHERE session_id=$1
      GROUP BY user_id
    ),
    rr AS (
      SELECT 
        balance_eur AS current_eur,
        user_id AS user_id
      FROM results
      WHERE 
        phase_no=$2
        AND session_id=$1
    ) 
    SELECT 
      r.user_id AS user_id,
      r.total_eur,
      rr.current_eur
    FROM r
    INNER JOIN rr 
      ON r.user_id=rr.user_id
    ;`,
      [session_id, phase_no]
    )
  ).rows;
  // Overall ranking
  rankings.sort((a, b) => b.total_eur - a.total_eur);
  for (let i = 0; i < rankings.length; i++) {
    if (i > 0 && rankings[i].total_eur === rankings[i - 1].total_eur) {
      rankings[i].overall_rank = rankings[i - 1].overall_rank;
    } else {
      rankings[i].overall_rank = i + 1;
    }
  }
  // Current phase ranking
  rankings.sort((a, b) => b.current_eur - a.current_eur);
  for (let i = 0; i < rankings.length; i++) {
    if (i > 0 && rankings[i].current_eur === rankings[i - 1].current_eur) {
      rankings[i].current_rank = rankings[i - 1].current_rank;
    } else {
      rankings[i].current_rank = i + 1;
    }
  }
  console.log(rankings);
  await Promise.all(
    rankings.map(async (rank) => {
      await db.query(
        `UPDATE results
        SET 
          ranking_current=$1,
          ranking_overall=$2
        WHERE 
          user_id=$3
          AND session_id=$4
          AND phase_no=$5;`,
        [
          rank.current_rank,
          rank.overall_rank,
          rank.user_id,
          session_id,
          phase_no,
        ]
      );
    })
  );
}
