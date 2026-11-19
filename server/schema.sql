-- Full schema for a NEW, empty database.
-- (If you already have the old tables with `jwt_token`, run
--  migrations/001_sessions_token_hash.sql instead.)
--C:\Users\annap\Desktop\desctop-projects\millida-demo\server\schema.sql
CREATE TABLE IF NOT EXISTS users (
                                     id            SERIAL PRIMARY KEY,
                                     username      VARCHAR(16) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email         TEXT NOT NULL UNIQUE
    );

CREATE TABLE IF NOT EXISTS sessions (
                                        id         SERIAL PRIMARY KEY,
                                        user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL
    );

CREATE INDEX IF NOT EXISTS sessions_expires_at_idx ON sessions (expires_at);

-- One skin per user.
CREATE TABLE IF NOT EXISTS skins (
                                     user_id  INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    skin_url TEXT NOT NULL,
    size     INTEGER NOT NULL
    );
