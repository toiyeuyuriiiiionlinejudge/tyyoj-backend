# AI Requirements & Project Specification

## MEMORY CONTRACT
- You have short memory and WILL forget. Therefore:
- TASK 0 (mandatory, before any other task): create `AI_REQUIREMENTS.md` at repo root and write EVERYTHING in this prompt into it as structured Markdown (headings: PROJECT CONTEXT, OWNER & LEGAL, TECH STACK, ENVIRONMENT, GIT IDENTITY, GITHUB, CONVENTIONS, AI POLICY, MVP SCOPE as `- [ ]` checklist, plus a "CHANGELOG of requirements" section).
- You MUST re-read `AI_REQUIREMENTS.md`:
  - (a) before every task 2–16;
  - (b) whenever context is compacted;
  - (c) when unsure about a requirement;
  - (d) before every git commit/push;
  - (e) before creating any issue/PR/repo.
- User changes requirement → UPDATE `AI_REQUIREMENTS.md` FIRST, append CHANGELOG, then code.
- Tick `- [x]` in `AI_REQUIREMENTS.md` after each task completes.
- If you acted without re-reading: STOP, re-read, diff, fix.

---

## PROJECT CONTEXT
- **Project:** Toi Yêu Yuri Online Judge (`tyyoj`) — A high-security, multi-repository Online Judge platform featuring Admin-controlled persistent Docker Judger provisioning, Markdown + KaTeX problem statements, and multi-mode competitive programming judging (Standard, Custom Checker, Interactive).
- **Purpose / users:** Competitive programmers, problem setters, and contest administrators needing an isolated, self-hosted online judge with manual/GUI-controlled Docker worker scaling and rich mathematical problem rendering.
- **Deadline:** None (continuous milestone-driven bootstrap).

---

## OWNER & LEGAL
- **Owner:** toiyeuyuriiiii (`toiyeuyuriiiii+github@gmail.com`)
- **Author credit:** toiyeuyuriiiii & nmkdeveloper
- **Copyright:** 2026 Toi Yêu Yuri Online Judge (toiyeuyuriiiii & nmkdeveloper)
- **License:** MIT

---

## TECH STACK
- **Language Selection Principle:** Strictly follows **Safety > Performance > Standard**.
- **Core Backend API & Docker Judger Orchestrator (`tyyoj-backend`):**
  - **Rust** (`axum`, `tokio`, `sqlx`, `bollard` Docker Engine API, `lettre` SMTP, `redis`/`fred`).
- **Shared Contracts & Schemas (`tyyoj-contracts`):**
  - **Rust** + OpenAPI specs.
- **Sandboxed Judger Engine (`tyyoj-judger`):**
  - **Rust** persistent worker daemon + **C++** (C++20) low-level Linux sandbox wrapper (namespaces, `seccomp-bpf`, `cgroups v2`, `rlimit`, bidirectional pipes for interactive tasks), bundling `g++`, `fpc` (Free Pascal), `python3`, and `testlib.h` inside worker image.
- **Debug Frontend (`tyyoj-frontend`):**
  - **Node.js (Active LTS) + TypeScript + React + Vite + TailwindCSS** (`react-markdown`, `remark-math`, `rehype-katex`, `katex`).
- **Database & Cache:**
  - **PostgreSQL** (current official LTS/stable major tag, isolated in dedicated Docker container with persistent volume and internal network).
  - **Redis** (current stable Docker tag for submission queues, worker heartbeats, and verification tokens).
- **Infra & Deployment:**
  - Docker & Docker Compose (`tyyoj` base repo); dynamic persistent `tyyoj-judger` containers provisioned on-demand via Admin GUI through Docker Engine socket.
- **Execution Constraints:**
  - Minimize/avoid Python in system infrastructure (strictly restricted to being a contestant submission language inside isolated judger containers).
  - Isolate untrusted contestant code execution with `NetworkMode: "none"`, strict cgroup CPU/memory/PID limits, read-only rootfs, and tmpfs scratch mounts.
- **Repository Architecture:**
  - Multi-repo under `toiyeuyuriiiiionlinejudge`: `tyyoj` (base orchestrator), plus `tyyoj-contracts`, `tyyoj-backend`, `tyyoj-judger`, `tyyoj-frontend`, and additional modular library repos if needed.
- **Pinning:**
  - Mandatory live web/registry lookup for current LTS & latest stable releases prior to pinning `.tool-versions`, `rust-toolchain.toml`, `.nvmrc`, Docker base image tags, `Cargo.toml`, and `package.json` + lockfiles (`Cargo.lock`, `package-lock.json`).

---

## ENVIRONMENT (dev-only — NOT production)
- **OS:** Debian 13 (Trixie) · **Arch:** x86_64 · **Sudo:** passwordless sudo · **Pkg mgrs:** apt, cargo, npm/pnpm, rustup
- **Runtimes:** None pre-installed (agent MUST actively search online for current LTS/latest stable versions before installing toolchains: Rust stable, Node.js active LTS, C++ g++/clang on Debian 13, Docker Engine + Compose plugin, Free Pascal `fpc` and `python3`/`pypy3` inside judger image).
- **Docker:** Docker Engine + Docker Socket API.
- **git:** pre-installed.
- **gh authed:** yes (authenticated GitHub CLI).
- **Network/proxy:** Direct internet access.
- **Disk:** Standard local SSD.
- **Forbidden:** Do NOT use Python for core backend services, judger daemons, or orchestration tooling (Python is strictly restricted to being a supported contestant submission language inside the isolated judger container).

---

## GIT IDENTITY
- **user.name:** `nmkdeveloper`
- **user.email:** `nguyenminhkhoi.nmk.dev@gmail.com`
- **GH user:** `nmkdeveloper`
- **GH email:** `nguyenminhkhoi.nmk.dev@gmail.com`
- **Co-authors:** `Co-authored-by: toiyeuyuriiiii <toiyeuyuriiiii+github@gmail.com>` (mandatory trailer on every commit).
- **SSH key:** pre-configured.
- **SSH uploaded:** yes.
- **GPG:** pre-configured commit signing (`Verified` badge enabled — preserve `commit.gpgsign = true`).
- **Default branch:** `main`.
- **Protocol:** SSH (`git@github.com:toiyeuyuriiiiionlinejudge/<repo>.git`).

---

## GITHUB
- **Org/user:** `toiyeuyuriiiiionlinejudge`
- **Repo:** `tyyoj` (base)
- **Visibility:** public (ALL repos MUST be public)
- **gh allowed:** yes
- **Extra repos:** `tyyoj-contracts` (public), `tyyoj-backend` (public), `tyyoj-judger` (public), `tyyoj-frontend` (public), plus any modular shared library repos deemed necessary (all public).
- **Branch protection:** Enabled on `main` (require status checks, block force pushes, require signed commits).
- **Labels:** standard + `judger`, `sandbox`, `frontend`, `backend`, `contracts`, `enhancement`, `bug`.
- **Templates:** `.github/ISSUE_TEMPLATE/{bug_report,feature_request,question}.yml`, `config.yml`, `PULL_REQUEST_TEMPLATE.md`, `dependabot.yml`.
- **CI:** `.github/workflows/{ci,codeql,release,stale}.yml`.
- **CODEOWNERS / FUNDING:** `CODEOWNERS` (`* @toiyeuyuriiiii @nmkdeveloper`), NO `FUNDING.yml`.

---

## CONVENTIONS
- **Formatter/linter:**
  - Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`
  - C++: `clang-format`, `clang-tidy`
  - TypeScript/React: `eslint`, `prettier`, `tsc --noEmit`
- **Commits:** Conventional Commits in English (`feat:`, `fix:`, `chore:`, `docs:`, `ci:`, `refactor:`, `test:`) with mandatory `Co-authored-by: toiyeuyuriiiii <toiyeuyuriiiii+github@gmail.com>` trailer.
- **Secrets:** `.env` gitignored, `.env.example` tracked with variable names only.
- **References:** `testlib.h` (latest upstream from MikeMirzayanov/testlib), KaTeX math rendering spec.
- **DO NOT:**
  - Never guess LTS or library version numbers from memory (always search live).
  - Never use Python for backend/judger infrastructure.
  - Never auto-spawn transient judgers without Admin GUI creation/control.
  - Never run contestant code with network access.
  - Never create private repos.
  - Never commit unverified commits or omit Co-authored-by.

---

## AI POLICY
- **TRACK:** `AI_REQUIREMENTS.md`.
- **IGNORE (verbatim in .gitignore):**
  ```gitignore
  AI_REQUIREMENTS.md.local
  AI_NOTES.md
  AI_SCRATCH.md
  .ai/
  .ai-cache/
  .claude/
  .cursor/
  .cursorrules
  .aider*
  .continue/
  .copilot/
  .copilot-instructions.md
  .gemini/
  .windsurf/
  .windsurfrules
  .codeium/
  .clinerules
  .roorules
  .rules
  .llm/
  .anthropic/
  .ollama/
  .llm-cache/
  prompt_history/
  *.prompt.md
  CLAUDE.md
  AGENTS.md
  GEMINI.md
  PROMPT.md
  PROMPTS.md
  ```

---

## EXTRA ARCHITECTURAL REQUIREMENTS
- **Mandatory Live Search for LTS & Latest Compatible Libraries:**
  Before installing runtimes or writing `Cargo.toml`, `package.json`, `.nvmrc`, `rust-toolchain.toml`, or `Dockerfile` base tags, query live sources (web search, `nodejs.org`, `crates.io` / `cargo search`, `registry.npmjs.org` / `npm view`, Docker Hub tags) to verify the exact current Active LTS versions (Node.js LTS, PostgreSQL stable, Debian 13 slim tags) and latest compatible stable versions of every crate (`axum`, `tokio`, `sqlx`, `bollard`, `lettre`, `redis`, `argon2`, `serde`, `utoipa`) and npm package (`react`, `vite`, `tailwindcss`, `react-markdown`, `remark-math`, `rehype-katex`, `katex`). Record all resolved versions and verification dates in `AI_REQUIREMENTS.md`.
- **Admin-Controlled Persistent Judger Lifecycle:**
  Unlike ephemeral one-shot containers, `tyyoj-judger` workers are explicitly created, scaled (count N), started, stopped, and removed by an Administrator via the Admin GUI (`tyyoj-frontend` -> `tyyoj-backend` -> Docker Engine API). Once created by Admin, each judger container runs continuously as a persistent daemon (`restart: unless-stopped`), sending periodic heartbeats to Redis and consuming submission jobs sequentially until Admin stops or deletes it.
- **Supported Contestant Languages & Judge Modes:**
  1. Languages inside `tyyoj-judger`: **C++** (`g++ -std=c++20 -O2`), **Pascal** (`fpc -O2`), and **Python** (`python3` / `pypy3`).
  2. Judge Modes:
     - **Standard:** Exact / whitespace-normalized token comparison.
     - **Custom Checker:** Compiles and runs problem-specific checker binary (supporting `testlib.h` arguments: `inf`, `ouf`, `ans`) inside a restricted sandbox.
     - **Interactive:** Spawns contestant process and interactor process simultaneously connected via bidirectional POSIX pipes (`stdin`/`stdout`), enforcing both CPU time limit and wall-clock idleness limit (`IDLENESS_LIMIT_EXCEEDED`).
- **Dynamic Per-Problem Execution & Resource Limits:**
  Memory limits (in MB) and CPU/wall time limits (in ms) are dynamically configured per problem (and optionally per testcase/language) in the problem definition/database and passed dynamically through the submission job payload into the sandbox arguments. Time and memory limits MUST NOT be hardcoded in the judger daemon or sandbox wrapper.

---

## MVP SCOPE CHECKLIST
- [ ] Multi-repo architecture under `toiyeuyuriiiiionlinejudge` (`tyyoj` base orchestrator + `tyyoj-contracts`, `tyyoj-backend`, `tyyoj-judger`, `tyyoj-frontend`).
- [ ] User authentication (`argon2id` + JWT/session) with SMTP email verification (configurable SMTP via `.env` + local Mailpit in Docker Compose for dev).
- [ ] Problem CRUD & preview supporting full Markdown + inline/block KaTeX math rendering (`$...$` and `$$...$$`), test case archive upload, and judge mode configuration.
- [ ] Admin GUI for Judger fleet management: Admin explicitly specifies/creates Judger instances via GUI; upon creation, `tyyoj-backend` provisions Docker containers via Docker Engine API that run continuously as persistent workers pulling jobs from Redis until stopped/deleted by Admin, with live worker status/telemetry shown in Admin GUI.
- [ ] Sandboxed Judger supporting 3 submission languages (**C++**, **Pascal**, **Python**) and 3 judging modes (**Standard** diff/token, **Custom Checker** with `testlib.h` support, and **Interactive** bidirectional pipe judge).
- [ ] Submission pipeline via Redis queue with live verdict updates (`PENDING`, `COMPILING`, `RUNNING`, `ACCEPTED`, `WRONG_ANSWER`, `TIME_LIMIT_EXCEEDED`, `MEMORY_LIMIT_EXCEEDED`, `RUNTIME_ERROR`, `COMPILATION_ERROR`, `IDLENESS_LIMIT_EXCEEDED`) and basic scoreboard.
- [ ] Functional debug-level frontend built with React + Vite + TailwindCSS covering auth, Markdown+KaTeX problems, code submission, Admin Judger control panel, and scoreboard.

---

## TASKS CHECKLIST
- [x] Task 0: Create `AI_REQUIREMENTS.md` with full prompt content. `cat AI_REQUIREMENTS.md` to verify. Report "requirements file created".
- [x] Task 1: [RE-READ] Live Version Audit & Toolchain Install. Verify git, gh auth, and install missing Debian 13 toolchains.
- [x] Task 2: [RE-READ] Configure git locally (user.name, user.email, init.defaultBranch, commit.template, preserve GPG/SSH commit signing). Verify SSH connection to GitHub.
- [x] Task 3: [RE-READ] Create multi-repo directory skeleton for `tyyoj` (base) and sibling repos (`tyyoj-contracts`, `tyyoj-backend`, `tyyoj-judger`, `tyyoj-frontend`).
- [x] Task 4: [RE-READ] Create `.gitignore` in all repos with mandatory blocks and AI-ignore list verbatim.
- [x] Task 5: [RE-READ] Create `.editorconfig`, `.gitattributes`, `.env.example`, `.nvmrc` / `rust-toolchain.toml` pinned to verified LTS/stable versions.
- [x] Task 6: [RE-READ] Create `.gitmessage` with Conventional Commits and pre-filled `Co-authored-by` trailer.
- [x] Task 7: [RE-READ] Create `LICENSE` (MIT), `README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `SUPPORT.md`, `CHANGELOG.md`, `CODEOWNERS`.
- [x] Task 8: [RE-READ] Create `.github/` templates, issue/PR templates, labels.yml, workflows (ci, codeql, release, stale).
- [x] Task 9: [RE-READ] Create `docs/` (`architecture.md`, `decisions/0001-...md`, `dev-setup.md`) and OpenAPI stub in `tyyoj-contracts`.
- [x] Task 10: [RE-READ] Init package manifests (`Cargo.toml`, `package.json`, `CMakeLists.txt`) using verified LTS/latest versions.
- [x] Task 11: [RE-READ] Setup lint+format+test+pre-commit hooks; wire into CI workflows.
- [ ] Task 12: [RE-READ] `git init -b main`; `git add -A`; first signed commit; verify `.gitignore` blocks secrets. Show status and signature.
- [ ] Task 13: [RE-READ] Run `gh repo create` for `tyyoj` and sibling repos (`tyyoj-contracts`, `tyyoj-backend`, `tyyoj-judger`, `tyyoj-frontend`). Push initial commits. Report URLs. STOP for owner confirmation.
- [ ] Task 14: [RE-READ] Apply branch protection on `main` across all repos. Push labels. Commit templates.
- [ ] Task 15: [RE-READ] Open initial issues — one per MVP item in `AI_REQUIREMENTS.md`.
- [ ] Task 16: [RE-READ] Final report: repo tree, URLs, git log, checklist diff, owner-review checklist.

---

## VERIFIED LIVE VERSIONS & TOOLCHAINS AUDIT LOG
- **Resolution Date:** 2026-11-12 (Queried live from official registries and release feeds)
- **Node.js Active LTS:** `v26.13.1` (Lithium, npm `11.20.0`)
- **Rust Stable:** `1.99.1` (channel `stable-x86_64-unknown-linux-gnu`, 2026-10-13)
- **PostgreSQL Stable Tag:** `17` / `17-alpine` (`17.11-alpine3.23`)
- **Redis Stable Tag:** `8` / `8-alpine` (`8.8.4-alpine3.23`)
- **Debian Toolchains:** `gcc` / `g++` 14.2.0, `clang-format` 19.1.7, `docker` 26.1.5, `fpc` 3.2.2, `pkg-config` 1.8.1
- **Rust Crates (latest verified compatible stable versions):**
  - `axum`: `0.8.9`
  - `tokio`: `1.54.1` (full features)
  - `sqlx`: `0.9.0` (postgres, runtime-tokio, tls-rustls, uuid, chrono)
  - `bollard`: `0.21.1` (Docker Engine API)
  - `lettre`: `0.11.23` (tokio1-rustls-tls, smtp-transport)
  - `redis`: `1.8.1` (tokio-comp, aio)
  - `argon2`: `0.6.0`
  - `serde`: `1.0.229` (derive)
  - `utoipa`: `6.0.0` (axum integration)
- **NPM Packages (latest verified compatible stable versions):**
  - `react`: `19.3.1`
  - `react-dom`: `19.3.1`
  - `vite`: `8.3.8`
  - `tailwindcss`: `4.3.5`
  - `react-markdown`: `10.1.0`
  - `remark-math`: `6.0.0`
  - `rehype-katex`: `7.0.1`
  - `katex`: `0.19.1`
  - `lucide-react`: `1.72.0`

---

## CHANGELOG of requirements
- 2026-11-12: Created .githooks (commit-msg enforcing Conventional Commits and Co-authored-by trailer, pre-commit blocking secrets and running linters) and tested validation. Completed Task 11.
- 2026-11-12: Initialized package manifests (Cargo.toml in contracts/backend/judger, CMakeLists.txt in judger sandbox with upstream testlib.h, package.json in frontend and tyyoj root) with author "toiyeuyuriiiii & nmkdeveloper", license "MIT", planned GitHub URLs, and verified live LTS versions. Built and tested all projects successfully. Completed Task 10.
- 2026-11-12: User requirement update: Enforced dynamic per-problem configuration for execution time limits and memory limits across the database, DTOs, judging pipeline, and sandbox parameters (strictly eliminating hardcoded limits).
- 2026-11-12: Created docs/architecture.md (ASCII system diagram + persistent judger provisioning flow + interactive pipe mode), docs/decisions/0001-record-architecture-decisions.md (ADR 0001), docs/dev-setup.md, and OpenAPI 3.1 specification (specs/openapi.yaml) in tyyoj-contracts. Completed Task 9.
- 2026-11-12: Created full .github configuration across all 5 repos: issue templates (bug_report, feature_request, question), config.yml, PULL_REQUEST_TEMPLATE.md, dependabot.yml, labels.yml, and GitHub Actions workflows (ci, codeql, release, stale). Confirmed NO FUNDING.yml. Completed Task 8.
- 2026-11-12: Created LICENSE (MIT), README.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md (Contributor Covenant 2.1), SECURITY.md, SUPPORT.md, CHANGELOG.md (Keep a Changelog 1.1.0), and CODEOWNERS across all 5 repos. Completed Task 7.
- 2026-11-12: User requirement change: Renamed base repository from `online_judge` to `tyyoj` and updated all relevant references across repository architecture, tasks, and documentation.
- 2026-11-12: Created .gitmessage template with Conventional Commits guide and pre-filled Co-authored-by trailer across all repos. Completed Task 6.
- 2026-11-12: Created .editorconfig, .gitattributes, .env.example (variable names only, no secrets), .nvmrc (26.13.1), and rust-toolchain.toml (1.99.1) pinned to verified LTS/stable versions across all repos. Completed Task 5.
- 2026-11-12: Created and verified .gitignore across all 5 repos with verbatim AI policy ignore block, OS/IDE/secrets/logs, and stack-specific rules. Verified .env and AI scratch files are ignored while AI_REQUIREMENTS.md and .env.example are tracked. Completed Task 4.
- 2026-11-12: Created multi-repo directory skeletons for online_judge, tyyoj-contracts, tyyoj-backend, tyyoj-judger, and tyyoj-frontend, and placed AI_REQUIREMENTS.md into each repo. Completed Task 3.
- 2026-11-12: Configured git identity, commit.template, gpg.format=ssh signing key, allowed_signers, and verified SSH connection to GitHub. Completed Task 2.
- 2026-11-12: Populated VERIFIED LIVE VERSIONS & TOOLCHAINS AUDIT LOG with live query results (Node.js v26.13.1, Rust 1.99.1, PostgreSQL 17, Redis 8, crates & npm packages). Completed Task 1.
- 2026-11-12: Initial creation of `AI_REQUIREMENTS.md` capturing all bootstrap requirements, architecture constraints, MVP checklist, and task sequence.
