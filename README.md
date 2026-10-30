# Millida Launcher (demo)

Desktop game launcher built with **Tauri 2 + React + TypeScript**: login/registration, multi-account switching, server list, skin upload/download, Play button.

## Try it

- **Web demo (UI only):** (https://millida-demo.vercel.app/)
  Sign in as `Anna123` with any password of 6+ characters. The web version uses mocked commands.
- **Desktop app (full version):** download the installer from [Releases](https://github.com/anna-paruryan/millida-demo), run it, and sign in with the same account (or register a new one).
  Windows SmartScreen may warn about an unknown publisher: click **More info**, then **Run anyway**.

| Feature            | Web demo | Desktop app        |
| ------------------ | -------- | ------------------ |
| Login / register   | Mock     | Real (Neon + JWT)  |
| Skin validation    | PNG only | PNG, 64x64, <1 MB  |
| Play               | Message  | Real command       |

## Stack

React 19, TypeScript, Vite, Tailwind CSS 4 · Tauri 2 (Rust, sqlx, argon2, jsonwebtoken) · Neon PostgreSQL · Vercel

## Run from source

Requirements: Node.js 20+, pnpm, Rust, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm dev            # web version in the browser (mocks)
pnpm tauri dev      # desktop app, needs src-tauri/.env with DATABASE_URL
pnpm tauri build    # build the installer
```

## Notes

`src/lib/tauri.ts` wraps `invoke()`: it calls the real Rust command inside Tauri and uses mocks in a normal browser.
Secrets are never committed: `.env` is in `.gitignore`. The JWT secret and the demo `Anna123` / `Petya2005` fallback are for demo purposes only.