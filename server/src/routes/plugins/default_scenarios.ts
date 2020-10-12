/**
 * Generate the default scenarios.
 */
import { v4 as uuid } from "uuid";
import db from "../../db/index";

async function generateSoloScenario(): Promise<string> {
  const id = uuid();
  const default_options = {
    id: id,
    name: "Scénario de base (solo)",
    difficulty: "easy",
    description:
      "Ce scénario de base vous permet de prendre en main, en solo, les fonctionnalités de parcelec.",
    multi_game: false,
    bids_duration_sec: 120,
    plannings_duration_sec: 180,
    phases_number: 3,
    conso_forecast_mwh: [900, 1300, 2400],
    conso_price_eur: [35, 35, 35],
    imbalance_costs_factor: [1.08, 1.08, 1.08],
  };

  await db.query(
    `INSERT INTO scenarios_options
    (
      id,
      name,
      difficulty,
      description,
      multi_game,
      bids_duration_sec,
      plannings_duration_sec,
      phases_number,
      conso_forecast_mwh,
      conso_price_eur,
      imbalance_costs_factor
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);`,
    [
      default_options.id,
      default_options.name,
      default_options.difficulty,
      default_options.description,
      default_options.multi_game,
      default_options.bids_duration_sec,
      default_options.plannings_duration_sec,
      default_options.phases_number,
      default_options.conso_forecast_mwh,
      default_options.conso_price_eur,
      default_options.imbalance_costs_factor,
    ]
  );

  const default_power_plants = [
    {
      type: "nuc",
      p_min_mw: 400,
      p_max_mw: 1300,
      stock_max_mwh: -1,
      price_eur_per_mwh: 17,
    },
    {
      type: "therm",
      p_min_mw: 150,
      p_max_mw: 600,
      stock_max_mwh: -1,
      price_eur_per_mwh: 65,
    },
    {
      type: "hydro",
      p_min_mw: 50,
      p_max_mw: 500,
      stock_max_mwh: 500,
      price_eur_per_mwh: 0,
    },
  ];

  await Promise.all(
    default_power_plants.map(async (pp) => {
      await db.query(
        `INSERT INTO scenarios_power_plants
        (
          scenario_id,
          type,
          p_min_mw,
          p_max_mw,
          stock_max_mwh,
          price_eur_per_mwh
        )
        VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          id,
          pp.type,
          pp.p_min_mw,
          pp.p_max_mw,
          pp.stock_max_mwh,
          pp.price_eur_per_mwh,
        ]
      );
    })
  );

  const bids = [
    { phase_no: 0, type: "buy", volume_mwh: 200, price_eur_per_mwh: 25 },
    { phase_no: 1, type: "buy", volume_mwh: 200, price_eur_per_mwh: 20 },
    { phase_no: 2, type: "sell", volume_mwh: 500, price_eur_per_mwh: 70 },
  ];
  await Promise.all(
    bids.map(async (bid) => {
      await db.query(
        `INSERT INTO scenarios_bids
        (
          scenario_id,
          phase_no,
          type,
          volume_mwh,
          price_eur_per_mwh
        )
        VALUES ($1, $2, $3, $4, $5)`,
        [id, bid.phase_no, bid.type, bid.volume_mwh, bid.price_eur_per_mwh]
      );
    })
  );
  return id;
}

async function generateMultiScenario(): Promise<string> {
  const id = uuid();
  const default_options = {
    id: id,
    name: "Scénario de base (multi)",
    difficulty: "easy",
    description:
      "Ce scénario de base vous permet de prendre en main, à plusieurs, les fonctionnalités de parcelec.",
    multi_game: true,
    bids_duration_sec: 120,
    plannings_duration_sec: 180,
    phases_number: 3,
    conso_forecast_mwh: [900, 1300, 2400],
    conso_price_eur: [35, 35, 35],
    imbalance_costs_factor: [1.08, 1.08, 1.08],
  };

  await db.query(
    `INSERT INTO scenarios_options
    (
      id,
      name,
      difficulty,
      description,
      multi_game,
      bids_duration_sec,
      plannings_duration_sec,
      phases_number,
      conso_forecast_mwh,
      conso_price_eur,
      imbalance_costs_factor
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);`,
    [
      default_options.id,
      default_options.name,
      default_options.difficulty,
      default_options.description,
      default_options.multi_game,
      default_options.bids_duration_sec,
      default_options.plannings_duration_sec,
      default_options.phases_number,
      default_options.conso_forecast_mwh,
      default_options.conso_price_eur,
      default_options.imbalance_costs_factor,
    ]
  );

  const default_power_plants = [
    {
      type: "nuc",
      p_min_mw: 400,
      p_max_mw: 1300,
      stock_max_mwh: -1,
      price_eur_per_mwh: 17,
    },
    {
      type: "therm",
      p_min_mw: 150,
      p_max_mw: 600,
      stock_max_mwh: -1,
      price_eur_per_mwh: 65,
    },
    {
      type: "hydro",
      p_min_mw: 50,
      p_max_mw: 500,
      stock_max_mwh: 500,
      price_eur_per_mwh: 0,
    },
  ];

  await Promise.all(
    default_power_plants.map(async (pp) => {
      await db.query(
        `INSERT INTO scenarios_power_plants
        (
          scenario_id,
          type,
          p_min_mw,
          p_max_mw,
          stock_max_mwh,
          price_eur_per_mwh
        )
        VALUES ($1, $2, $3, $4, $5, $6)`,
        [
          id,
          pp.type,
          pp.p_min_mw,
          pp.p_max_mw,
          pp.stock_max_mwh,
          pp.price_eur_per_mwh,
        ]
      );
    })
  );

  const bids = [
    { phase_no: 0, type: "buy", volume_mwh: 200, price_eur_per_mwh: 25 },
    { phase_no: 1, type: "buy", volume_mwh: 200, price_eur_per_mwh: 20 },
    { phase_no: 2, type: "sell", volume_mwh: 500, price_eur_per_mwh: 70 },
  ];
  await Promise.all(
    bids.map(async (bid) => {
      await db.query(
        `INSERT INTO scenarios_bids
        (
          scenario_id,
          phase_no,
          type,
          volume_mwh,
          price_eur_per_mwh
        )
        VALUES ($1, $2, $3, $4, $5)`,
        [id, bid.phase_no, bid.type, bid.volume_mwh, bid.price_eur_per_mwh]
      );
    })
  );
  return id;
}

export default async function generateDefaultScenarios(): Promise<string[]> {
  return await Promise.all([
    await generateSoloScenario(),
    await generateMultiScenario(),
  ]);
}