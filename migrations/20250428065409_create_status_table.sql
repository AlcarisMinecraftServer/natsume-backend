-- Add migration script here
CREATE TABLE status (
    id SERIAL PRIMARY KEY,
    server_id TEXT NOT NULL,
    online BOOLEAN NOT NULL,
    latency INTEGER,
    players_online INTEGER,
    players_max INTEGER,
    timestamp BIGINT NOT NULL
);
