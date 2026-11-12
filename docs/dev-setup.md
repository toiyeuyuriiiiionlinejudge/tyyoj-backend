# Developer Setup Guide

This guide walks through setting up the **Toi Yêu Yuri Online Judge (`tyyoj`)** development environment from scratch on Debian 13 (Trixie) or compatible Linux systems.

---

## 1. Prerequisites & Toolchains

The following tools must be installed:
- **Rust Stable** (1.99.1+): `rustc`, `cargo`, `rustfmt`, `clippy`
- **Node.js Active LTS** (v26.13.1+): `node`, `npm`
- **C++ Compiler**: `g++` 14+, `clang-format` 19+
- **Free Pascal**: `fpc` 3.2+
- **Docker Engine & Compose**: Docker 26+, `docker-compose`
- **Git & GitHub CLI**: `git`, `gh`

---

## 2. Environment Configuration

Copy the example environment configuration in the root directory:
```bash
cp .env.example .env
```

Ensure the variables are set appropriately for local development:
- `POSTGRES_DB=tyyoj`
- `POSTGRES_USER=tyyoj_user`
- `POSTGRES_PASSWORD=tyyoj_dev_password`
- `REDIS_URL=redis://127.0.0.1:6379`
- `JWT_SECRET=super_secret_dev_jwt_key_32_bytes_min!`
- `ADMIN_INITIAL_EMAIL=admin@tyyoj.local`
- `ADMIN_INITIAL_PASSWORD=admin_password_123`

---

## 3. Launching Local Infrastructure (PostgreSQL, Redis, Mailpit)

From the `tyyoj` root directory, launch background dev services:
```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

This launches:
- **PostgreSQL 17** on port `5432`
- **Redis 8** on port `6379`
- **Mailpit** (local SMTP test server) on port `1025` (SMTP) and `8025` (Web UI for viewing verification emails).

---

## 4. Running Backend (`tyyoj-backend`)

```bash
cd tyyoj-backend
cargo run
```
The backend server listens on `http://127.0.0.1:8080`.

---

## 5. Running Frontend (`tyyoj-frontend`)

```bash
cd tyyoj-frontend
npm install
npm run dev
```
The frontend Vite server opens on `http://localhost:5173`.

---

## 6. Admin Judger Fleet Provisioning

1. Log into the web UI at `http://localhost:5173` with Admin credentials.
2. Navigate to the **Admin -> Judgers** dashboard.
3. Click **"Create Judger"**:
   - `tyyoj-backend` connects to `/var/run/docker.sock` and provisions a persistent `tyyoj-judger` container.
   - The worker container starts, registers with Redis, and displays green "Online" telemetry status in the Admin GUI.
