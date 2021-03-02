INSERT INTO t_sessions (id, name, status) VALUES
    ('a19bc943-4599-4782-a650-806b015f209a', 'Open session', 'open'),
    ('04c92e13-a42f-4381-aa67-94875798082e', 'Running session', 'running'),
    ('a196d524-ab6d-4adc-bcab-42e96f5ce547', 'Closed session', 'closed');

INSERT INTO t_phases (session_id, phase_no, status) VALUES
    ('04c92e13-a42f-4381-aa67-94875798082e', 1, 'open');