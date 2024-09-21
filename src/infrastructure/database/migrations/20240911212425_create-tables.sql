BEGIN;

CREATE TABLE users (
    tg_id BIGINT NOT NULL,
    sets_number INTEGER NOT NULL DEFAULT 0,
    created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tg_id)
);

CREATE TABLE sets (
    tg_id BIGINT NOT NULL,
    short_name TEXT NOT NULL,
    title TEXT NOT NULL,
    UNIQUE(short_name)
);

COMMIT;