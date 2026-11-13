# `tyyoj-backend`

High-performance, memory-safe backend API and Judger fleet orchestrator for **Toi Yêu Yuri Online Judge (`tyyoj`)**, written in Rust.

## Technology Stack

- **Web Framework:** [Axum](https://github.com/tokio-rs/axum) with [Tokio](https://tokio.rs/) async runtime.
- **Database Access:** [SQLx](https://github.com/launchbadge/sqlx) targeting PostgreSQL 17 with compile-time checked queries and migrations.
- **Podman Engine API:** [Bollard](https://github.com/fussybeaver/bollard) communicating with `/run/podman/podman.sock` to manage persistent judger worker containers.
- **Cache & Queue:** Redis 8 for submission job queues and worker heartbeats.
- **Email Service:** [Lettre](https://github.com/lettre/lettre) for SMTP user account verification.
- **Authentication:** Argon2id password hashing + JWT access and refresh tokens.

## Judger Fleet Management

Unlike conventional ephemeral runners, `tyyoj-backend` provides an administrative API allowing authorized Admins to:
- Create new persistent `tyyoj-judger` container instances via Podman Engine API.
- Monitor live heartbeats, CPU/memory usage, and queue consumption metrics from active judgers.
- Scale judger instances up or down.
- Gracefully stop and decommission judger instances.

## License

MIT © 2026 Toi Yêu Yuri Online Judge (toiyeuyuriiiii & nmkdeveloper)
