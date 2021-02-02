-- Add migration script here
CREATE TABLE IF NOT EXISTS stats (
    time TIMESTAMPTZ NOT NULL,
    apikey CHAR(64) NOT NULL,
    route TEXT NOT NULL,
    agent Text NOT NULL,
    api TEXT NOT NULL
);
SELECT create_hypertable('stats', 'time');