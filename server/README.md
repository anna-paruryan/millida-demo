# Millida auth server

The only component with database access. The desktop app talks to it over HTTPS.

## Run locally
1. `cp .env.example .env` and put your Neon DATABASE_URL in it.
2. Apply the migration once (Neon SQL Editor or psql): `migrations/001_sessions_token_hash.sql`
3. `cargo run --release`  → listens on http://0.0.0.0:8080

## API
- POST /api/register  {username, password, email} → 201 {username, token}
- POST /api/login     {username, password}        → 200 {username, token}
- GET  /api/me        Authorization: Bearer <token> → {username}
- POST /api/logout    Authorization: Bearer <token> → 204
- GET  /health        → "ok"
Errors: {"error": "..."} with 400 / 401 / 409 / 500.
