/**
 * List the different types needed accros the server app.
 */

export interface Session {
  name: string;
  id: string;
  status: "open" | "running" | "closed";
}

export interface SessionOptions {
  multi_game: boolean;
  bids_duration_sec: number;
  plannings_duration_sec: number;
  phases_number: number;
  conso_forecast_mwh: number[];
  conso_price_eur: number;
  imbalance_costs_eur: number;
}

export interface User {
  id: string;
  name: string;
  session_id: string;
  game_ready: boolean;
}

export interface PowerPlant {
  id: string;
  session_id: string;
  user_id: string;
  type: "nuc" | "therm" | "hydro" | "ren" | "storage";
  p_min_mw: number;
  p_max_mw: number;
  stock_max_mwh: number;
  price_eur_per_mwh: number;
}

export interface PowerPlantWithPlanning extends PowerPlant {
  planning: number;
}

export type PowerPlantTemplate = Omit<
  PowerPlant,
  "session_id" | "user_id" | "id"
>;

export interface GamePhase {
  session_id: string;
  phase_no: number;
  start_time: Date;
  clearing_time: Date;
  planning_time: Date;
  bids_allowed: boolean;
  clearing_available: boolean;
  plannings_allowed: boolean;
  results_available: boolean;
}

export interface ConsoForecast {
  user_id: string;
  session_id: string;
  phase_no: number;
  value_mw: number;
}

export interface Bid {
  id: string;
  user_id: string;
  session_id: string;
  phase_no: number;
  type: "buy" | "sell";
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export interface Clearing {
  session_id: string;
  phase_no: number;
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export interface EnergyExchange {
  user_id: string;
  session_id: string;
  phase_no: number;
  type: "buy" | "sell";
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export interface PowerPlantDispatch {
  user_id: string;
  session_id: string;
  phase_no: number;
  plant_id: string;
  p_mw: number;
  stock_start_mwh: number;
  stock_end_mwh: number;
}

export type ProductionPlanning = PowerPlantDispatch[];

export interface PhaseResults {
  user_id: string;
  session_id: string;
  phase_no: number;
  conso_mwh: number;
  conso_eur: number;
  prod_mwh: number;
  prod_eur: number;
  sell_mwh: number;
  sell_eur: number;
  buy_mwh: number;
  buy_eur: number;
  imbalance_mwh: number;
  imbalance_costs_eur: number;
  balance_eur: number;
}

export interface ClientMessage {
  username: string;
  date: Date;
  reason: "message" | "handshake";
  credentials: {
    session_id: string;
    user_id: string;
  };
  data: any;
}
