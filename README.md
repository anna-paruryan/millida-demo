# Millida Launcher (Demo)

Desktop game launcher built with **Tauri 2 + React + TypeScript**: login/registration, multi-account switching, server list, skin upload/download, and Play button.

## Try it

- **Web demo (UI only):** [Open Millida Demo](https://millida-demo.vercel.app/)

  Sign in as `Anna123` with any password of 6+ characters. The web version uses mocked commands.

- **Desktop app (full version):** [⬇️ Download Millida Launcher for Windows](https://github.com/anna-paruryan/millida-demo/releases/latest/download/millida-demo_0.1.0_x64-setup.zip)

  Download the ZIP archive, extract it, run `millida-demo_0.1.0_x64-setup.exe`, and follow the installation instructions.

  Windows SmartScreen may warn about an unknown publisher. Only proceed if you trust the source of the installer.

## Features

| Feature | Web demo | Desktop app |
|---|---|---|
| Login / register | Mock | Real (Neon + JWT) |
| Skin validation | PNG only | PNG, 64x64, less than 1 MB |
| Play | Message | Real command |

## Stack

React 19, TypeScript, Vite, Tailwind CSS 4 · Tauri 2 (Rust, sqlx, argon2, jsonwebtoken) · Neon PostgreSQL · Vercel

## Run from source

Requirements: Node.js 20+, pnpm, Rust, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm dev            # web version in the browser (mocks)
pnpm tauri dev      # desktop app, needs src-tauri/.env with DATABASE_URL
pnpm tauri build    # build the installer
pnpm run tauri dev
cd .server/cargo check
cargo run

## Notes

`src/lib/tauri.ts` wraps `invoke()`: it calls the real Rust command inside Tauri and uses mocks in a normal browser.

Secrets should never be committed. `.env` is listed in `.gitignore`. The JWT secret and demo credentials are for demo purposes only.