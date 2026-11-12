# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Multi-repository architecture under `toiyeuyuriiiiionlinejudge` (`tyyoj`, `tyyoj-contracts`, `tyyoj-backend`, `tyyoj-judger`, `tyyoj-frontend`).
- Live-verified toolchain integration (Node.js v26.13.1 Active LTS, Rust 1.99.1 stable, PostgreSQL 17, Redis 8).
- Repository metadata, `.gitignore`, `.editorconfig`, `.gitattributes`, `.gitmessage`, and governance documentation.
- Initial OpenAPI schemas and architecture documentation for persistent Admin-controlled Docker judger fleet provisioning and multi-mode judging (Standard, Custom Checker, Interactive).
