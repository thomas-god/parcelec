/* Replace with your SQL commands */
CREATE TABLE t_sessions
(
  id UUID PRIMARY KEY,
  scenario_id UUID,
  name TEXT,
  status TEXT CHECK (status IN ('open','running', 'closed')) DEFAULT 'open'
);