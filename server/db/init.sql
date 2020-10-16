CREATE TABLE scenarios_options
(
  id UUID PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT,
  difficulty TEXT CHECK (difficulty IN ('easy', 'medium', 'hard')),
  multi_game BOOLEAN DEFAULT FALSE,
  bids_duration_sec INT NOT NULL CHECK (bids_duration_sec > 0),
  plannings_duration_sec INT NOT NULL CHECK (plannings_duration_sec > 0),
  phases_number INT NOT NULL CHECK (phases_number > 0),
  conso_forecast_mwh INT[] CHECK (array_length(conso_forecast_mwh, 1) = phases_number),
  conso_price_eur REAL[] NOT NULL CHECK (array_length(conso_price_eur, 1) = phases_number),
  imbalance_costs_factor REAL[] NOT NULL CHECK (array_length(imbalance_costs_factor, 1) = phases_number)
);

CREATE TABLE scenarios_power_plants
(
  scenario_id UUID REFERENCES scenarios_options (id) ON DELETE CASCADE,
  type TEXT NOT NULL CHECK (type IN ('nuc', 'therm', 'hydro', 'ren', 'storage')),
  p_min_mw REAL NOT NULL,
  p_max_mw REAL NOT NULL,
  stock_max_mwh REAL NOT NULL CHECK (stock_max_mwh > 0 OR stock_max_mwh = -1),
  -- stock_max_mwh = -1 represents infinite stock
  price_eur_per_mwh REAL NOT NULL,
  CHECK (p_min_mw < p_max_mw)
);

CREATE TABLE scenarios_bids 
(
  scenario_id UUID REFERENCES scenarios_options (id) ON DELETE CASCADE,
  phase_no INT,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL
);

CREATE TABLE sessions 
(
  id UUID PRIMARY KEY,
  scenario_id UUID REFERENCES scenarios_options (id) ON DELETE CASCADE,
  name TEXT UNIQUE,
  status TEXT CHECK (status IN ('open','running', 'closed'))
);

CREATE TABLE options 
(
  session_id UUID REFERENCES sessions (id) ON DELETE CASCADE,
  scenario_id UUID REFERENCES scenarios_options (id) ON DELETE CASCADE,
  multi_game BOOLEAN DEFAULT FALSE,
  bids_duration_sec INT NOT NULL CHECK (bids_duration_sec > 0),
  plannings_duration_sec INT NOT NULL CHECK (plannings_duration_sec > 0),
  phases_number INT NOT NULL CHECK (phases_number > 0),
  conso_forecast_mwh INT[] CHECK (array_length(conso_forecast_mwh, 1) = phases_number),
  conso_price_eur REAL[] NOT NULL CHECK (array_length(conso_price_eur, 1) = phases_number),
  imbalance_costs_factor REAL[] NOT NULL CHECK (array_length(imbalance_costs_factor, 1) = phases_number)
);

CREATE TABLE users 
(
  id UUID PRIMARY KEY,
  name TEXT,
  session_id UUID REFERENCES sessions (id) ON DELETE CASCADE,
  game_ready BOOLEAN DEFAULT false,
  UNIQUE (name, session_id)
);

CREATE TABLE power_plants 
(
  id UUID PRIMARY KEY,
  session_id UUID REFERENCES sessions (id) ON DELETE CASCADE,
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  type TEXT NOT NULL CHECK (type IN ('nuc', 'therm', 'hydro', 'ren', 'storage')),
  p_min_mw REAL NOT NULL,
  p_max_mw REAL NOT NULL,
  stock_max_mwh REAL NOT NULL CHECK (stock_max_mwh > 0 OR stock_max_mwh = -1),
  -- stock_max_mwh = -1 represents infinite stock
  price_eur_per_mwh REAL NOT NULL,
  CHECK (p_min_mw < p_max_mw)
);

CREATE TABLE phases 
(
  session_id UUID REFERENCES sessions (id) ON DELETE CASCADE,
  phase_no INT DEFAULT 0,
  start_time TIMESTAMPTZ,
  clearing_time TIMESTAMPTZ,
  planning_time TIMESTAMPTZ,
  bids_allowed BOOLEAN DEFAULT true,
  clearing_available BOOLEAN DEFAULT false,
  plannings_allowed BOOLEAN DEFAULT true,
  results_available BOOLEAN DEFAULT false,
  status TEXT CHECK (status IN ('open', 'closed')),
  PRIMARY KEY (session_id, phase_no)
);

CREATE TABLE conso 
(
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
  value_mw REAL NOT NULL CHECK (value_mw > 0),
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE
);

CREATE TABLE bids 
(
  id UUID PRIMARY KEY NOT NULL,
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID NOT NULL,
  phase_no INT NOT NULL,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL
);

CREATE TABLE clearings 
(
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  volume_mwh REAL NOT NULL,
  price_eur_per_mwh REAL NOT NULL,
  internal_buy_last_bid_price REAL NOT NULL,
  internal_buy_last_bid_frac_volume REAL NOT NULL,
  internal_sell_last_bid_price REAL NOT NULL,
  internal_sell_last_bid_frac_volume REAL NOT NULL,
  PRIMARY KEY (session_id, phase_no)
);

CREATE TABLE exchanges 
(
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL
);

CREATE TABLE otc_exchanges 
(
  id UUID PRIMARY KEY NOT NULL,
  user_from_id UUID REFERENCES users (id) ON DELETE CASCADE,
  user_to_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL,
  status TEXT CHECK (status IN ('pending', 'accepted', 'rejected'))
);

CREATE TABLE production_plannings 
(
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  plant_id UUID REFERENCES power_plants (id),
  p_mw REAL NOT NULL,
  stock_start_mwh REAL NOT NULL,
  stock_end_mwh REAL NOT NULL,
  UNIQUE (plant_id, phase_no)
);

CREATE TABLE results
(
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  conso_mwh REAL NOT NULL,
  conso_eur REAL NOT NULL,
  prod_mwh REAL NOT NULL,
  prod_eur REAL NOT NULL,
  sell_mwh REAL NOT NULL,
  sell_eur REAL NOT NULL,
  buy_mwh REAL NOT NULL,
  buy_eur REAL NOT NULL,
  imbalance_mwh REAL NOT NULL,
  imbalance_costs_eur REAL NOT NULL,
  balance_eur REAL NOT NULL,
  ranking_current INT NOT NULL DEFAULT -1,
  ranking_overall INT NOT NULL DEFAULT -1
);

CREATE OR REPLACE FUNCTION get_url(session_name text, username text) 
RETURNS text
AS $$
#print_strict_params on
DECLARE
url text;
BEGIN
    SELECT CONCAT('http://localhost:8080/session/', sessions.id, '/user/', users.id) 
    INTO STRICT url
    FROM users, sessions 
    WHERE 
      users.session_id=sessions.id 
      AND users.name = get_url.username
      AND sessions.name = get_url.session_name;
    RETURN url;
END
$$ LANGUAGE plpgsql;
