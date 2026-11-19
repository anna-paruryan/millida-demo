-- Sessions now store only the SHA-256 hash of the token.
-- Old sessions were created by the desktop client with raw tokens / JWTs,
-- so they are invalidated: users just log in again.

BEGIN;

DELETE FROM sessions;

ALTER TABLE sessions RENAME COLUMN jwt_token TO token_hash;

CREATE UNIQUE INDEX IF NOT EXISTS sessions_token_hash_idx ON sessions (token_hash);
CREATE INDEX IF NOT EXISTS sessions_expires_at_idx ON sessions (expires_at);

COMMIT;
