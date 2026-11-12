# `tyyoj` Architecture Specification

This document details the system design, communication protocols, sandboxing model, and execution workflows for **Toi Yêu Yuri Online Judge (`tyyoj`)**.

---

## 1. High-Level Architecture Diagram

```text
+----------------------------------------------------------------------------------------------------+
|                                         USER / CLIENT                                              |
|                                                                                                    |
|  [ Contestant Browser ]                                           [ Administrator Browser ]        |
|  - View Markdown + KaTeX Problem Statements                       - Problem & Contest Management   |
|  - Submit C++, Pascal, Python Code                                - Judger Fleet Provisioning GUI  |
|  - Real-time Verdict Stream                                       - Live Worker Telemetry Monitor  |
+---------------------------------+-------------------------------------------------+----------------+
                                  |                                                 |
                                  | HTTPS / WebSocket                               | HTTPS / WebSocket
                                  v                                                 v
+----------------------------------------------------------------------------------------------------+
|                                    tyyoj-frontend (React 19 + Vite)                                |
|                                                                                                    |
|  - Problem viewer (react-markdown, remark-math, rehype-katex, katex)                               |
|  - Monaco Code Editor (C++, Pascal, Python3)                                                       |
|  - Live Submissions & Standings Table                                                              |
|  - Admin Judger Management Dashboard                                                               |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  | REST API (OpenAPI 3.1) / WebSocket
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                      tyyoj-backend (Rust + Axum)                                   |
|                                                                                                    |
|  - Auth & Security: Argon2id password hashing, JWT auth, SMTP email verification (Lettre)           |
|  - Problem Management: Test case archives, Markdown statements, judge mode configuration          |
|  - Submission Ingestion: Validates language/mode, pushes submission jobs to Redis queue            |
|  - Admin Judger Fleet Orchestrator: Communicates with Docker Engine API via Bollard                |
|    * Explicitly creates, starts, scales, inspects, and deletes persistent Judger containers        |
+--------------------+----------------------------+---------------------------------+----------------+
                     |                            |                                 |
        SQL Queries  |               Redis Queue  |               Docker Engine API |
        (SQLx async) |               & Heartbeats |               (Unix Domain Sock)|
                     v                            v                                 v
+----------------------------+   +----------------------------+   +----------------------------------+
|   PostgreSQL 17 Database   |   |       Redis 8 Cache        |   |   Docker Daemon (dockerd API)    |
|                            |   |                            |   |                                  |
| - Users & credentials      |   | - Submission Job Queue     |   | Provisions & manages persistent  |
| - Problems & test metadata |   | - Worker Heartbeats & stats|   | tyyoj-judger worker containers   |
| - Submissions & verdicts   |   | - Live verdict pub/sub     |   |                                  |
+----------------------------+   +--------------+-------------+   +-----------------+----------------+
                                                |                                   |
                                                | Pop Job / Push Verdict            | Managed Container
                                                v                                   v
+----------------------------------------------------------------------------------------------------+
|                                tyyoj-judger Fleet (Persistent Workers)                             |
|                                                                                                    |
|  +----------------------------------------------------------------------------------------------+  |
|  | Persistent Worker Daemon (Rust):                                                             |  |
|  | - Continuously listens on Redis submission queue                                             |  |
|  | - Reports heartbeat & telemetry (CPU, RAM, container status) every 5 seconds                |  |
|  | - Fetches problem test cases & compiles contestant code                                      |  |
|  +----------------------------------------------+-----------------------------------------------+  |
|                                                 | Spawns & Enforces Constraints                    |
|                                                 v                                                  |
|  +----------------------------------------------------------------------------------------------+  |
|  | Low-Level Linux Sandbox (C++20 Sandbox Wrapper):                                             |  |
|  | - Isolation: Namespaces (PID, NET, IPC, Mount), cgroups v2 (CPU/RAM), rlimit, seccomp-bpf    |  |
|  | - Network: Strictly isolated (NetworkMode: "none")                                           |  |
|  | - Filesystem: Read-only rootfs + isolated tmpfs scratchpad                                   |  |
|  | - Supported Submission Languages: C++20 (g++), Pascal (fpc), Python (python3/pypy3)         |  |
|  +----------------------------------------------------------------------------------------------+  |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Multi-Repository Decomposition

1. **`tyyoj`** (Base Orchestrator):
   Contains root Docker Compose definitions (`docker-compose.yml`), environment profiles (`.env.example`), system documentation (`docs/`), and global build/deployment scripts.
2. **`tyyoj-contracts`**:
   Common Rust library containing models, request/response DTOs, database representations, submission verdict enums, language codes, and OpenAPI 3.1 YAML specifications.
3. **`tyyoj-backend`**:
   Core API server implemented in Rust using Axum. Integrates SQLx for PostgreSQL 17, Lettre for SMTP verification, Bollard for Docker Engine communication, and Redis for queues and heartbeats.
4. **`tyyoj-judger`**:
   Persistent containerized worker. The outer daemon is written in Rust, which invokes the C++20 sandbox wrapper to isolate untrusted user execution with high precision.
5. **`tyyoj-frontend`**:
   Modern SPA built with React 19, TypeScript, Vite, TailwindCSS, and KaTeX for LaTeX rendering of mathematics in competitive programming problem statements.

---

## 3. Persistent Admin-Controlled Judger Provisioning

Unlike platforms that spin up transient one-shot containers for every submission (incurring high container start overhead), `tyyoj` employs **persistent worker daemons explicitly created by Administrators**:

```text
Admin GUI                    tyyoj-backend               Docker Engine               Redis Queue
   |                              |                            |                          |
   | 1. Admin creates Judger      |                            |                          |
   |    (Count N, RAM, CPU)       |                            |                          |
   |----------------------------->|                            |                          |
   |                              | 2. bollard.create_container|                          |
   |                              |    (image: tyyoj-judger)   |                          |
   |                              |--------------------------->|                          |
   |                              | 3. bollard.start_container |                          |
   |                              |--------------------------->|                          |
   |                              |                            | 4. Spawns persistent     |
   |                              |                            |    judger daemon         |
   |                              |                            |                          |
   |                              |                            |====== tyyoj-judger =====>|
   |                              |                            | 5. Register & Heartbeat  |
   |                              |                            |    (worker_id, specs)    |
   |                              |                            |                          |
   | 6. Shows Online Status       | 7. Reads worker heartbeats |                          |
   |<-----------------------------|----------------------------|<-------------------------|
   |                              |                            |                          |
   |                              |                            | 8. Loop: BLPOP job       |
   |                              |                            |    from Redis queue      |
   |                              |                            |------------------------->|
```

- Each worker daemon runs continuously with `restart: unless-stopped`.
- Heartbeats are broadcast every 5 seconds to Redis (`judger:heartbeat:<worker_id>`).
- If an admin clicks "Stop" or "Delete" in the GUI, `tyyoj-backend` gracefully signals the container via Docker Engine API (`bollard`).

---

## 4. Judging Modes & Sandboxing Workflow

### Mode A: Standard Mode
The compiled executable runs inside the sandbox with input fed via `stdin` and output captured via `stdout`. The output is compared against the expected answer using an exact or whitespace-normalized token comparator.

### Mode B: Custom Checker Mode
For problems with multiple valid solutions or floating-point tolerances, a custom checker is provided. The checker binary is compiled against `testlib.h` and executed with arguments:
```text
./checker <input_file> <contestant_output_file> <answer_file>
```
The exit code determines the verdict (0: `ACCEPTED`, 1: `WRONG_ANSWER`, 2: `CRASH`, etc.).

### Mode C: Interactive Mode
For interactive tasks, the contestant process and problem interactor process run simultaneously, connected via bidirectional POSIX pipes:

```text
+-----------------------+              Pipe 1              +-----------------------+
|  Contestant Binary    |  stdout ----------------> stdin  |   Problem Interactor  |
|  (C++ / Pascal / Py)  |  stdin  <---------------- stdout |   (compiled C++)      |
+-----------------------+              Pipe 2              +-----------------------+
```

- **Deadlock Mitigation:** If both processes block on I/O, a wall-clock idleness monitor terminates the execution with `IDLENESS_LIMIT_EXCEEDED`.
- **Resource Measurement:** CPU time and memory are measured strictly for the contestant process.
