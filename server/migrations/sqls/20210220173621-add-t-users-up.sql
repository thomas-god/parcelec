/* Replace with your SQL commands */
CREATE TABLE t_users
(
  id UUID PRIMARY KEY,
  name TEXT,
  session_id UUID REFERENCES t_sessions (id) ON DELETE CASCADE,
  is_ready BOOLEAN DEFAULT false,
  UNIQUE (name, session_id)
);