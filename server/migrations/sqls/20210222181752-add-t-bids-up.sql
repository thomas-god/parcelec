/* Replace with your SQL commands */
CREATE TABLE t_bids
(
  id UUID PRIMARY KEY NOT NULL,
  user_id UUID REFERENCES t_users (id) ON DELETE CASCADE,
  session_id UUID REFERENCES t_sessions (id) ON DELETE CASCADE,
  type TEXT CHECK (type IN ('buy', 'sell')),
  volume REAL NOT NULL CHECK (volume > 0),
  price REAL NOT NULL
);