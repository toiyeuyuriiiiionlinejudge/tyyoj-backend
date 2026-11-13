# ADR 0001: System Architecture Decisions for `tyyoj`

## Status
Accepted

## Date
2026-11-12

## Context
Online Judge systems require executing arbitrary, potentially malicious user code while maintaining strict isolation, deterministic resource accounting, low latency, and ease of administrative scaling. Additionally, mathematical problem rendering (KaTeX) and diverse judging paradigms (Standard token comparison, `testlib.h` custom checkers, and interactive pipe judging) must be natively supported.

## Decision Drivers
1. **Safety > Performance > Standard**: High memory safety for core services; zero-overhead low-level systems programming for sandbox execution.
2. **Prohibition of Python in Core Infrastructure**: Python is prone to concurrency bottlenecks and runtime type hazards; it is strictly restricted to being a supported contestant submission language inside isolated containers.
3. **Admin-Controlled Persistent Workers**: Ephemeral one-shot containers introduce significant cold-start latency (1-3s per run). Persistent judger containers provisioned on-demand by Admins maintain instant readiness and pull jobs sequentially from Redis.

## Architectural Decisions

### 1. Multi-Repository Structure
- **Decision:** Split into modular public repositories: `tyyoj` (root orchestrator), `tyyoj-contracts` (shared schemas & OpenAPI), `tyyoj-backend` (Rust Axum API), `tyyoj-judger` (Rust daemon + C++20 sandbox), `tyyoj-frontend` (React + Vite + TailwindCSS).
- **Rationale:** Enables independent CI, versioning, modular deployments, and clear separation of concerns between web API and low-level Linux systems programming.

### 2. Backend & Judger Daemon: Rust
- **Decision:** Use Rust (`axum`, `tokio`, `sqlx`, `bollard`, `lettre`, `redis`) for backend services and the judger daemon.
- **Rationale:** Rust guarantees memory safety without garbage collection pauses, provides first-class async concurrency with Tokio, and integrates natively with Podman Socket API via `bollard`.

### 3. Linux Sandbox: C++20
- **Decision:** Low-level sandbox wrapper implemented in C++20 utilizing Linux kernel primitives: namespaces (`CLONE_NEWPID`, `CLONE_NEWNET`, `CLONE_NEWNS`, `CLONE_NEWIPC`), cgroups v2, `rlimit`, and `seccomp-bpf`.
- **Rationale:** Direct access to Linux kernel APIs, microsecond-accurate time measurement (`ptrace`/`wait4`), and seamless integration with the competitive programming standard `testlib.h`.

### 4. Database & Queue
- **Decision:** PostgreSQL 17 for persistent relational data; Redis 8 for submission job queues, worker heartbeats, and token caching.
- **Rationale:** PostgreSQL provides robust relational guarantees and ACID transactions for submissions and user records; Redis offers high-throughput atomic queue primitives (`LPUSH`/`BRPOP`).

### 5. Frontend & Math Rendering
- **Decision:** React 19 + TypeScript + Vite + TailwindCSS + KaTeX (`react-markdown`, `remark-math`, `rehype-katex`).
- **Rationale:** High-speed client-side rendering of complex mathematical statements (`$...$` and `$$...$$`) without server-side rendering latency, coupled with a responsive administration interface.

## Consequences
- **Positive:** Maximum system security, defense-in-depth isolation, sub-second judging response times, reproducible builds, and clear separation of responsibilities.
- **Negative:** Requires multi-language toolchain maintenance (Rust, C++, TypeScript) in developer environments.
