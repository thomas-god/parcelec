/**
 * The exported function holds all the logic when starting a new game
 * phase.
 */
import db from "../../db";
import { getConsoForecast, getSessionUsers } from "../utils";
import { sendUpdateToUsers } from "../websocket";

export async function endGame(
  session_id: string,
  phase_no: number
): Promise<void> {
  // Notify users that session is finished
  sendUpdateToUsers(session_id, "plannings-closed", {});

  // Mark plannings_allowed to false
  await db.query(
    "UPDATE phases SET plannings_allowed=false WHERE session_id=$1 AND phase_no=$2",
    [session_id, phase_no]
  );

  // Do the metering and results computation, it may take some time
  await computeResults(session_id, phase_no);

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
}

/**
 * Compute the energy end financial results for each user.
 * @param session_id Session ID
 * @param phase_no Phase number
 */
async function computeResults(
  session_id: string,
  phase_no: number
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
      results.conso_mwh = await getConsoForecast(session_id, user.id, phase_no);
      results.conso_eur = results.conso_mwh * 40;

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

      // Energy exchanges
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

      // Imbalance
      results.imbalance_mwh =
        results.prod_mwh +
        results.buy_mwh -
        results.conso_mwh -
        results.sell_mwh;
      results.imbalance_costs_eur = results.imbalance_mwh * 40;

      // Total financial balance
      results.balance_eur =
        results.conso_eur +
        results.sell_eur -
        results.prod_eur -
        results.buy_eur -
        results.imbalance_costs_eur;

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
        balance_eur
      )
      VALUES 
      ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)`,
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
        ]
      );
    })
  );
}
