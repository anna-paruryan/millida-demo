# Millida Launcher (Demo)

A Windows desktop Minecraft launcher demo built with **Tauri 2, React 19, TypeScript, and Rust**.

The project demonstrates desktop application architecture, user authentication, session management, multi-account UI, server selection, and skin file validation.

## Download for Windows

[![Download Millida Launcher](https://img.shields.io/badge/⬇️_Download-Millida_Launcher-blue?style=for-the-badge&logo=windows)](https://github.com/anna-paruryan/millida-demo/releases/download/v0.1.1/millida-demo_0.1.0_x64-setup.exe)

**Windows 10/11 · x64**

Download the installer and follow the installation instructions.

> **Note:** This is a development demo. The authentication backend currently requires local setup, so login and registration will not work on another computer yet.

## Preview

**Web Demo (UI only):** [Open Millida Demo](https://millida-demo.vercel.app/)

The web version uses mocked Tauri commands. It is intended for previewing the interface and does not connect to the authentication backend.

## Features

| Feature | Web Demo | Desktop App |
|---|---|---|
| Login / Registration | Mock | Backend API |
| Authentication | Mock | Opaque session tokens |
| Database | None | Neon PostgreSQL |
| Multi-account UI | Demo | Supported |
| Skin validation | Basic PNG check | PNG, 64×64, less than 1 MB |
| Skin upload | Mock | Local validation only |
| Play button | Demo message | Authentication check and simulated launch |

### Current limitations

- Minecraft is not launched yet.
- Skin files are validated locally but are not uploaded to remote storage.
- Windows Credential Manager integration through Keyring is not implemented yet.
- Account token persistence between application launches is not implemented yet.
- The authentication backend is currently intended for local development.
- A public HTTPS backend deployment is required before the desktop application can be used on other computers.

## Tech Stack

### Frontend
- React 19
- TypeScript
- Vite
- Tailwind CSS 4

### Desktop
- Tauri 2
- Rust
- Reqwest
- Keyring (integration pending)

### Backend
- Rust
- Axum 0.8
- SQLx 0.8
- Argon2 password hashing
- SHA-256 session token hashing
- Neon PostgreSQL

## Architecture

The project consists of three main parts:

1. **React frontend** — UI, login forms, account management, and launcher screens.
2. **Tauri desktop client** — communicates with the backend over HTTP using Rust commands.
3. **Authentication backend** — handles registration, login, session validation, logout, and PostgreSQL access.

The desktop client does not connect directly to PostgreSQL.

### Authentication flow

`React → Tauri command → Rust HTTP client → Axum API → Neon PostgreSQL`

The backend generates random opaque session tokens. Only SHA-256 hashes of session tokens are stored in the database.

## Run Locally

### Requirements

- Node.js 20+
- pnpm
- Rust toolchain
- Neon PostgreSQL database
- [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### 1. Configure the backend

Create `server/.env`:

```env
DATABASE_URL=your_neon_database_connection_string
```

Never commit this file.

### 2. Start the authentication backend

Open a terminal:

```bash
cd server
cargo run
```

The backend listens on:

`http://127.0.0.1:8080`

### 3. Start the desktop application

Open another terminal in the project root:

```bash
pnpm install
pnpm tauri dev
```

The desktop development build connects to the local backend.

### 4. Run the web demo

```bash
pnpm dev
```

The browser version uses mocked commands.

### 5. Build the Windows installer

```bash
pnpm run tauri build
```

Generated installers:

- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`

A production desktop build requires a publicly accessible HTTPS authentication API. The default local API address is intended for development only.

## Project Structure

```text
millida-demo/
├── server/
│   ├── migrations/
│   ├── src/
│   ├── Cargo.toml
│   └── schema.sql
├── src/
│   ├── components/
│   └── lib/
├── src-tauri/
│   ├── src/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```

## Security

- Passwords are hashed with Argon2 on the backend.
- Authentication uses random opaque session tokens.
- Only SHA-256 hashes of session tokens are stored in PostgreSQL.
- The desktop client communicates with the backend through Rust HTTP commands.
- Environment variables and secrets must never be committed to Git.

## License

This project is a demonstration and portfolio project.