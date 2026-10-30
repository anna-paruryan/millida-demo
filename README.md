# Millida Launcher (demo)

A desktop game launcher built with **Tauri 2 + React + TypeScript**. It has account login/registration, multi-account switching, a server list, skin upload/download with validation, and a Play button.

The same UI also runs in a regular browser (web demo) using mocked backend commands, so it can be hosted on Vercel.

## Features

- Sign in / create account (Argon2 password hashing, JWT sessions, Neon Postgres)
- Up to 5 saved accounts, switch without re-entering a password
- Server list with selection and a Play button (`secure_launch`)
- Skin upload (PNG only, under 1 MB, must be 64x64) and skin download
- Dark UI, Tailwind CSS v4, toast notifications (sonner)

## Tech stack

| Layer      | Technology                                         |
| ---------- | -------------------------------------------------- |
| Frontend   | React 19, TypeScript, Vite, Tailwind CSS 4, sonner |
| Desktop    | Tauri 2 (Rust)                                     |
| Backend    | Rust commands: sqlx, argon2, jsonwebtoken          |
| Database   | Neon (PostgreSQL)                                  |
| Web hosting| Vercel (static build, mocked commands)             |

## Project structure

```
millida-demo/
├── src/
│   ├── components/
│   │   ├── Login.tsx                    # login / register form
│   │   ├── Launcher.tsx                 # servers, skin, Play
│   │   ├── ui/                          # button, card, input
│   │   └── features/auth/Accountmanager.tsx   # multi-account state helpers
│   └── lib/
│       └── tauri.ts                     # invoke() wrapper: real Tauri or browser mocks
├── src-tauri/
│   ├── src/lib.rs                       # Rust commands (login, register, launch, skin check)
│   ├── tauri.conf.json
│   └── .env                             # DATABASE_URL (NOT committed)
├── vite.config.ts
└── package.json
```

## How it works

`src/lib/tauri.ts` exports an `invoke()` function with one rule:

- **Inside the Tauri window** it calls the real Rust command.
- **In a normal browser** it uses mocks, because a browser cannot run Rust or reach the database.

| Mode             | Login                              | Skin check                      | Play                  |
| ---------------- | ---------------------------------- | ------------------------------- | --------------------- |
| Tauri app        | Real, checked against Neon         | Real (PNG, 64x64, size)         | Real command          |
| Web (Vercel)     | Mock, demo accounts only           | Mock (PNG signature only)       | Message only          |

## Test accounts

Use either of these with **any password of 6 or more characters**:

- `Anna123`
- `Petya2005`

In the Tauri app you can also register your own account.

## Requirements

For the web demo:

- Node.js 20+
- pnpm (`npm i -g pnpm`)

For the Tauri desktop app (on top of the above):

- Rust (install with [rustup](https://rustup.rs))
- Windows: Microsoft C++ Build Tools ("Desktop development with C++") and WebView2 (preinstalled on Windows 11)
- See the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for macOS and Linux

## Run the web demo

```bash
pnpm install
pnpm dev
```

Open http://localhost:1420 and sign in as `Anna123` with any password of 6+ characters.

## Run the desktop app (Tauri)

1. Install dependencies:

   ```bash
   pnpm install
   ```

2. Create `src-tauri/.env` with the database connection string (it is provided privately, never committed):

   ```
   DATABASE_URL=postgresql://USER:PASSWORD@HOST/DBNAME?sslmode=require
   ```

   A template is in `src-tauri/.env.example`.

3. Start the app:

   ```bash
   pnpm tauri dev
   ```

The first run compiles the Rust part and can take several minutes.

## Build an installer

```bash
pnpm tauri build
```

The installer appears in `src-tauri/target/release/bundle/`:

- `nsis/*.exe` (setup program)
- `msi/*.msi`

The installer is not code-signed, so Windows SmartScreen may show "Unknown publisher". Click **More info**, then **Run anyway**.

> The app reads `DATABASE_URL` when it starts. For a standalone installer, build it with the variable set so it is compiled in (see "Standalone build" below).

### Standalone build (connection string compiled in)

PowerShell:

```powershell
$env:DATABASE_URL="postgresql://USER:PASSWORD@HOST/DBNAME?sslmode=require"
pnpm tauri build
```

Anyone with the installer can extract the string from the binary. Use a separate Neon branch or role with limited access for demos, and rotate the credentials afterwards.

## Deploy the web demo to Vercel

1. Push the project to GitHub (make sure `.env`, `node_modules`, `dist` and `src-tauri/target` are in `.gitignore`).
2. On [vercel.com](https://vercel.com): **Add New, Project**, import the repository.
3. Settings:
  - Framework preset: **Vite**
  - Build command: `pnpm build`
  - Output directory: `dist`
4. Click **Deploy**.

No environment variables are needed: the web version uses mocks and never touches the database.

## Security notes

- Never commit `.env` or put `DATABASE_URL` in frontend code. Variables starting with `VITE_` are public.
- The JWT secret in `src-tauri/src/lib.rs` is a demo value. Move it to an environment variable before any real use.
- The `demo` password hash for `Anna123` / `Petya2005` accepts any password of 6+ characters. Remove this fallback before production.

## Scripts

| Command            | What it does                         |
| ------------------ | ------------------------------------ |
| `pnpm dev`         | Web dev server (browser, mocks)      |
| `pnpm build`       | Type-check and build the web bundle  |
| `pnpm tauri dev`   | Run the desktop app in dev mode      |
| `pnpm tauri build` | Build the desktop installer          |