import db from "../../db";
/**
 * Set of functions to work with production plannings.
 */

import { PowerPlantDispatch, ProductionPlanning } from "../types";
import { getCurrentPhaseNo, getPortfolio } from "../utils";

type UserPlanning = Omit<
  PowerPlantDispatch,
  "phase_no" | "stock_start_mwh" | "stock_end_mwh"
>[];

/**
 * Check and format
 * @param user_planning User provided production planning
 */
export async function formatUserPlanning(
  user_planning: UserPlanning
): Promise<ProductionPlanning> {
  if (user_planning.length === 0)
    throw "Error, user production planning is empty";

  // session_id, user_id must be the same across all planning items
  if (!user_planning.every((v, i, a) => v.session_id === a[0].session_id))
    throw "Error, session_id is not consistent across the planning";
  if (!user_planning.every((v, i, a) => v.user_id === a[0].user_id))
    throw "Error, user_id is not consistent across the planning";

  // Get IDs and phase_no
  const session_id = user_planning[0].session_id;
  const phase_no = await getCurrentPhaseNo(session_id);

  // Add the phase_no and stock_start_mwh keys
  return await Promise.all(
    user_planning.map(async (pp) => {
      let stock_start_mwh: number;
      if (phase_no === 0) {
        // First phase, stock_start is taken equal to stock_max
        stock_start_mwh = (
          await db.query("SELECT stock_max_mwh FROM power_plants WHERE id=$1", [
            pp.plant_id,
          ])
        ).rows[0].stock_max_mwh as number;
      } else {
        // Carry stock_end from previous phase
        stock_start_mwh = (
          await db.query(
            "SELECT stock_end_mwh FROM production_plannings WHERE plant_id=$1 AND phase_no=$2",
            [pp.plant_id, phase_no - 1]
          )
        ).rows[0].stock_end_mwh as number;
      }
      return {
        ...pp,
        phase_no: phase_no,
        stock_start_mwh: stock_start_mwh,
        stock_end_mwh: stock_start_mwh === -1 ? -1 : stock_start_mwh - pp.p_mw,
      };
    })
  );
}

/**
 * Check the planning against various constraints.
 *  - stock_end_mwh must be within plant's [0, stock_max_mwh],
 *    if stock_max_mwh != -1 (infinite energy)
 *  - p_mw must be within plant [p_min_mw, p_max_mw] or be zero.
 * @param planning a correctly formatted planning
 */
export async function checkPlanning(
  planning: ProductionPlanning
): Promise<boolean> {
  const portfolio = await getPortfolio(planning[0].user_id);
  // All plants must have a P0
  planning.map((dispatch) => {
    const pp = portfolio.find((item) => item.id === dispatch.plant_id);
    if (dispatch.p_mw !== 0 && dispatch.p_mw > pp.p_max_mw)
      throw `Error, dispatch too big for plant ${dispatch.plant_id}`;

    if (dispatch.p_mw !== 0 && dispatch.p_mw < pp.p_min_mw)
      throw `Error, dispatch too small for plant ${dispatch.plant_id}`;

    if (pp.stock_max_mwh !== -1 && dispatch.stock_end_mwh > pp.stock_max_mwh)
      throw `Error, dispatch will exceed max storage for plant ${dispatch.plant_id}`;

    if (pp.stock_max_mwh !== -1 && dispatch.stock_end_mwh < 0)
      throw `Error, not enough energy left for plant ${dispatch.plant_id}`;

    if (dispatch.p_mw === null)
      throw `Error, no set-point for plant ${dispatch.plant_id}`;
  });
  return true;
}

export async function insertPlanning(
  planning: ProductionPlanning
): Promise<void> {
  await Promise.all(
    planning.map(async (dispatch) => {
      await db.query(
        `INSERT INTO production_plannings 
          (user_id, session_id, phase_no, plant_id, p_mw, stock_start_mwh, stock_end_mwh)
          VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (plant_id, phase_no) 
          DO UPDATE SET (p_mw, stock_start_mwh, stock_end_mwh)
          = (excluded.p_mw, excluded.stock_start_mwh, excluded.stock_end_mwh)`,
        [
          dispatch.user_id,
          dispatch.session_id,
          dispatch.phase_no,
          dispatch.plant_id,
          dispatch.p_mw,
          dispatch.stock_start_mwh,
          dispatch.stock_end_mwh,
        ]
      );
    })
  );
}
