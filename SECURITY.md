# Security Policy

## Supported Versions

Security updates are actively maintained for the current milestone releases:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

The Toi Yêu Yuri Online Judge (`tyyoj`) project takes the security of our platform and sandbox environments very seriously. Because the judger executes untrusted, arbitrary user-submitted code in multiple languages (C++, Pascal, Python), defense-in-depth isolation is critical.

If you believe you have discovered a vulnerability, security flaw, or sandbox breakout:

1. **Do NOT open a public GitHub issue.**
2. Send an email immediately to the project owners:
   - **toiyeuyuriiiii+github@gmail.com**
   - **nguyenminhkhoi.nmk.dev@gmail.com**
3. Include detailed information:
   - Description of the vulnerability and attack vector.
   - Exact steps or proof-of-concept (PoC) code to reproduce.
   - Affected repository or component (`tyyoj-judger`, `tyyoj-backend`, sandbox cgroups/seccomp, etc.).
   - Potential impact.

## Sandbox Isolation Guarantees & Constraints

Our security architecture enforces:
- **No Network Access:** Contestant processes execute inside containers with network mode set strictly to `none`.
- **Resource Constraints:** Strict cgroups v2 memory limits (OOM killed on exceed), wall/CPU time limits, and PID limits to mitigate fork bombs.
- **Filesystem Protections:** Read-only root filesystem with isolated ephemeral tmpfs mounts.
- **Syscall Filtering:** seccomp-bpf filters blocking unauthorized syscalls (e.g., ptrace, socket, clone/unshare without permitted flags).
