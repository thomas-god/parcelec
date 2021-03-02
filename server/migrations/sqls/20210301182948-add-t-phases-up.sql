/* Replace with your SQL commands */
CREATE TABLE t_phases
(
  session_id UUID REFERENCES t_sessions (id) ON DELETE CASCADE,
  phase_no INT NOT NULL,
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