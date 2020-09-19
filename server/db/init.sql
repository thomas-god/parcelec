CREATE TABLE sessions 
(
  id UUID PRIMARY KEY,
  name TEXT UNIQUE,
  status TEXT CHECK (status IN ('open','running', 'closed'))
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
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users (id) ON DELETE CASCADE,
  session_id UUID,
  phase_no INT,
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
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL,
  PRIMARY KEY (session_id, phase_no)
);

CREATE TABLE exchanges 
(
  user_id UUID REFERENCES users (id),
  session_id UUID,
  phase_no INT,
  FOREIGN KEY (session_id, phase_no) REFERENCES phases (session_id, phase_no) ON DELETE CASCADE,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume_mwh REAL NOT NULL CHECK (volume_mwh > 0),
  price_eur_per_mwh REAL NOT NULL
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
  net_conso REAL NOT NULL,
  net_prod REAL NOT NULL,
  costs_eur REAL NOT NULL,
  revenues_eur REAL NOT NULL
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
