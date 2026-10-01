# Test Rust — Slack-Type Workspace Application

A premium real-time collaboration platform built with Rust, Axum, SQLx, and PostgreSQL.

## Architecture

This project follows a vertical slice architecture with 11 crates:

| Crate | Purpose |
|---|---|
| `core` | Domain types, traits, errors — zero external deps |
| `db` | SQLx repositories, migrations, queries |
| `auth` | JWT, permissions, OAuth |
| `realtime` | WebSocket server, pub/sub, presence |
| `services` | Business logic services |
| `api` | HTTP handlers, middleware, DTOs, routes |
| `search` | Meilisearch integration |
| `storage` | S3/MinIO file handling |
| `email` | SMTP client |
| `whiteboard` | CRDT engine, sync, presence |
| `app` | Binary entry point, wiring |

## Prerequisites

- Rust 1.75+ (stable)
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+

## Quick Start

```bash
# 1. Clone and enter the project
cd test_rust

# 2. Copy environment configuration
cp .env.example .env

# 3. Start infrastructure
docker-compose up -d

# 4. Run database migrations
cargo sqlx migrate run

# 5. Build and run
cargo run --bin app
```

## Development

```bash# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p core
cargo test -p db
cargo test -p auth
```

## Project Structure

See `ARCHITECTURE_GUARDRAIL.md` for the binding architectural contract.
See `VERTICAL_SLICES.md` for the 45 vertical slices organized into 9 waves.
See `SLICE_DEPENDENCY_GRAPH.md` for the dependency graph and build order.

## License

MIT
