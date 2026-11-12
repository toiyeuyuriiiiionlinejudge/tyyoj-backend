# Contributing to Toi Yêu Yuri Online Judge (tyyoj)

Thank you for your interest in contributing to **Toi Yêu Yuri Online Judge (`tyyoj`)**! We welcome bug reports, feature discussions, and pull requests.

## Architecture & Repositories

`tyyoj` is organized as a multi-repository system under the [`toiyeuyuriiiiionlinejudge`](https://github.com/toiyeuyuriiiiionlinejudge) organization:

1. **`tyyoj`** (Base Orchestrator): Docker Compose environments, deployment manifests, shared documentation, and system integration.
2. **`tyyoj-contracts`**: Shared Rust schemas, DTOs, and OpenAPI specifications.
3. **`tyyoj-backend`**: Core REST API and Docker Judger fleet manager written in Rust (Axum, SQLx, Bollard, Lettre, Redis).
4. **`tyyoj-judger`**: Persistent judging worker daemon (Rust) + low-level Linux sandbox wrapper (C++20, cgroups v2, namespaces, seccomp-bpf, interactive pipe judge).
5. **`tyyoj-frontend`**: Web interface built with React 19, TypeScript, Vite, TailwindCSS, and KaTeX.

## Development Workflow & Standards

### 1. Git & Commit Conventions
- All commits MUST follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) in English:
  - `feat: <description>`
  - `fix: <description>`
  - `chore: <description>`
  - `docs: <description>`
  - `ci: <description>`
  - `refactor: <description>`
  - `test: <description>`
- Every commit MUST include the mandatory co-author trailer:
  ```text
  Co-authored-by: toiyeuyuriiiii <toiyeuyuriiiii+github@gmail.com>
  ```
- All commits MUST be GPG or SSH signed (`git commit -S` or `commit.gpgsign = true`).

### 2. Tooling & Linter Requirements
- **Rust**:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- **C++ (Sandbox)**:
  - `clang-format --dry-run -Werror`
  - `clang-tidy`
- **TypeScript / React**:
  - `npm run lint` / `eslint`
  - `npm run format:check` / `prettier --check`
  - `npm run typecheck` / `tsc --noEmit`

### 3. Security & Sandboxing Constraints
- Never use Python for backend services or judger infrastructure. Python is strictly reserved for contestant submissions inside isolated judger containers.
- Untrusted contestant code execution must always run with:
  - Network isolation (`NetworkMode: "none"`).
  - Strict CPU, memory, and process count limits via cgroups v2 and rlimit.
  - Read-only root filesystem with dedicated tmpfs scratch mounts.

## Reporting Issues & Requesting Features
- Please check existing issues before opening a new one.
- Use our GitHub issue templates (`bug_report.yml`, `feature_request.yml`, `question.yml`).

## License & Copyright
By contributing, you agree that your contributions will be licensed under the project's **MIT License** with copyright `2026 Toi Yêu Yuri Online Judge (toiyeuyuriiiii & nmkdeveloper)`.
