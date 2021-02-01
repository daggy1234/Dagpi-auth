CREATE TABLE tokens (
	apikey CHAR(64) UNIQUE NOT NULL,
	userid BIGINT PRIMARY KEY  NOT NULL,
	uses  INT NOT NULL,
	totaluses BIGINT NOT NULL,
	ratelimit INT NOT NULL,
	enhanced BOOLEAN NOT NULL
);


CREATE TABLE stats (
    time  TIMESTAMPTZ  NOT NULL,
    apikey CHAR(64)  NOT NULL,
    route TEXT NOT NULL,
    agent Text NOT NULL
);

SELECT create_hypertable('stats','time');
