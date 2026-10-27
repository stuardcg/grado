-- Timestamps are unix seconds (UTC). Secrets are stored only as SHA-256 hex.

CREATE TABLE users (
    id          TEXT PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE TABLE sessions (
    id_hash     TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL
);

-- Bearer tokens Claude Code uses to reach the MCP endpoint.
CREATE TABLE access_tokens (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    TEXT NOT NULL UNIQUE,
    label         TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    last_used_at  INTEGER,
    revoked_at    INTEGER
);

CREATE TABLE level_progress (
    user_id          TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    level_id         TEXT NOT NULL,
    started_at       INTEGER NOT NULL,
    completed_at     INTEGER,
    -- content hash the level was started and completed on
    started_version  TEXT NOT NULL,
    completed_version TEXT,
    PRIMARY KEY (user_id, level_id)
);

-- Every piece of evidence the agent submitted, pass or fail.
CREATE TABLE criterion_attempts (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    level_id      TEXT NOT NULL,
    criterion_id  TEXT NOT NULL,
    evidence      TEXT NOT NULL,
    passed        INTEGER NOT NULL,
    feedback      TEXT NOT NULL,
    grader        TEXT NOT NULL,
    created_at    INTEGER NOT NULL
);
CREATE INDEX criterion_attempts_by_level ON criterion_attempts (user_id, level_id, created_at);

-- One-time checkpoint stamps. Issuing a new one for a level voids the old one.
CREATE TABLE checkpoint_stamps (
    stamp_hash   TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    level_id     TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,
    redeemed_at  INTEGER,
    voided_at    INTEGER
);

CREATE TABLE handoffs (
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    level_id    TEXT NOT NULL,
    body        TEXT NOT NULL,
    updated_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, level_id)
);

CREATE TABLE guide_unlocks (
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    level_id     TEXT NOT NULL,
    tier         INTEGER NOT NULL,
    unlocked_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, level_id, tier)
);

CREATE TABLE ideas (
    id                TEXT PRIMARY KEY,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    raised_in_level   TEXT NOT NULL,
    belongs_to_level  TEXT,
    body              TEXT NOT NULL,
    created_at        INTEGER NOT NULL
);
