# Architecture Guardrail — Non-Negotiable Rules for All Agents

**Status**: Binding Contract  
**Applies To**: All AI agents, developers, and contributors working in this workspace  
**Last Updated**: 2026-09-30  
**Related Documents**: `ARCHITECTURE.md`, `WHITEBOARD_DESIGN.md`, `VERTICAL_SLICES.md`

---

## Table of Contents

1. [Purpose & How to Use This Document](#1-purpose--how-to-use-this-document)
2. [Technology Stack — Locked](#2-technology-stack--locked)
3. [Crate Structure & Dependency Rules](#3-crate-structure--dependency-rules)
4. [Naming Conventions](#4-naming-conventions)
5. [Error Handling Patterns](#5-error-handling-patterns)
6. [Database Conventions](#6-database-conventions)
7. [API Design Patterns](#7-api-design-patterns)
8. [WebSocket Protocol Conventions](#8-websocket-protocol-conventions)
9. [Multi-Tenancy Rules — CRITICAL](#9-multi-tenancy-rules--critical)
10. [File & Module Organization Rules](#10-file--module-organization-rules)
11. [How to Add a New Feature (Process)](#11-how-to-add-a-new-feature-process)
12. [How to Add a New Crate](#12-how-to-add-a-new-crate)
13. [Testing Requirements](#13-testing-requirements)
14. [Code Style Rules](#14-code-style-rules)
15. [Documentation Requirements](#15-documentation-requirements)
16. [Prohibited Patterns — Never Do These](#16-prohibited-patterns--never-do-these)
17. [Pre-Submission Checklist](#17-pre-submission-checklist)

---

## 1. Purpose & How to Use This Document

This document is the **binding architectural contract** for the entire workspace. Every agent assigned to any task in this project MUST:

1. Read this document before writing any code
2. Read the relevant slice definition in `VERTICAL_SLICES.md`
3. Follow every rule here without exception
4. If a rule seems wrong for your task, **stop and ask** — do not unilaterally violate it

**When in doubt, follow this document.** If this document conflicts with any other document, this document wins.

---

## 2. Technology Stack — Locked

The following technology choices are **final**. Do not introduce alternatives without explicit written approval from the project lead.

| Layer | Technology | Version | Notes |
|---|---|---|---|
| Language | Rust | Edition 2021 | MSRV: 1.75+ |
| Web Framework | Axum | 0.7 | Only HTTP framework |
| Async Runtime | Tokio | 1.x | Multi-threaded, work-stealing |
| Database | PostgreSQL | 15+ | Via SQLx |
| Database Access | SQLx | 0.7 | Compile-time checked queries only |
| Redis Client | fred | 8.x | For cache, pub/sub, presence |
| Serialization | serde + serde_json | 1.x | No alternatives |
| Auth Tokens | jsonwebtoken | 9.x | JWT access + refresh tokens |
| Password Hashing | argon2 | 0.5 | Argon2id |
| Validation | validator | 0.16 | Derive-based validation |
| Error Handling | thiserror + anyhow | 1.x | thiserror for libs, anyhow for binaries |
| Logging | tracing + tracing-subscriber | 0.1 | Structured logging |
| Config | config + dotenvy | 0.14 | Layered config |
| Time | chrono | 0.4 | UTC everywhere |
| IDs | uuid | 1.x | **UUID v7 only** — time-ordered |
| Email | lettre | 0.11 | SMTP client |
| HTTP Client | reqwest | 0.11 | For outbound calls |
| Search | meilisearch-sdk | 0.27 | Full-text search |
| Rate Limiting | governor | 0.6 | Token bucket |
| Concurrency | dashmap + parking_lot | 0.5 / 0.12 | Concurrent maps, fast locks |
| CRDT | automerge | 0.5 | Whiteboard documents |
| CRDT Repo | automerge-repo | 0.1 | Sync management |
| Compression | zstd | 0.13 | Large document compression |
| Geometry | geo | 0.28 | Spatial operations |
| Graph | petgraph | 0.6 | Element dependency graphs |
| Binary Serde | bincode | 1.3 | Redis transport encoding |
| SVG/PNG Export | resvg + usvg + tiny-skia | 0.37 / 0.11 | Whiteboard export |
| Metrics | metrics | 0.22 | Prometheus exposition |
| TS Codegen | ts-rs | 0.10 | Generate TS types from Rust |

### Crate Versions in Workspace `Cargo.toml`

All external dependencies MUST be declared in the workspace root `Cargo.toml` under `[workspace.dependencies]`. Individual crates reference them with `{ workspace = true }`. **Never** specify versions in individual crate `Cargo.toml` files.

---

## 3. Crate Structure & Dependency Rules

### 3.1 The 11 Crates

```
test_rust/
├── Cargo.toml              # Workspace root
├── core/                   # Domain types, traits, errors — ZERO external deps
├── db/                     # SQLx repositories, migrations, queries
├── auth/                   # JWT, permissions, OAuth
├── realtime/               # WebSocket server, pub/sub, presence
├── services/               # Business logic services
├── api/                    # HTTP handlers, middleware, DTOs, routes
├── search/                 # Meilisearch integration
├── storage/                # S3/MinIO file handling
├── email/                  # SMTP client
├── whiteboard/             # CRDT engine, sync, presence (NEW)
└── app/                    # Binary entry point, wiring
```

### 3.2 Dependency Graph — The Law

```
                         ┌──────────┐
                         │   app    │  ← Binary, wires everything
                         └────┬─────┘
                              │ depends on ALL
          ┌───────────────────┼───────────────────┐
          │                   │                   │
     ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
     │   api   │        │realtime │        │services │
     └────┬────┘        └────┬────┘        └────┬────┘
          │                  │                   │
          │           ┌──────▼──────┐            │
          │           │ whiteboard  │◄───────────┘
          │           └──────┬──────┘
          │                  │
     ┌────▼────┐      ┌──────▼──────┐
     │ storage │      │     db      │
     └─────────┘      └──────┬──────┘
                             │
                      ┌──────▼──────┐
                      │    core     │  ← Foundation, no deps on other crates
                      └─────────────┘
```

### 3.3 Dependency Rules — Enforced

| Rule | Details |
|---|---|
| **R1: `core` is the foundation** | `core` depends on NO other workspace crate. All other crates MAY depend on `core`. |
| **R2: `db` depends only on `core`** | `db` contains all SQLx queries, migrations, and repository implementations. It depends on `core` for types and errors. |
| **R3: `auth` depends on `core` + `db`** | `auth` needs `db` to look up users, sessions, org memberships. |
| **R4: `realtime` depends on `core` + `db`** | `realtime` needs `db` for presence persistence and channel data. |
| **R5: `whiteboard` depends on `core` + `realtime`** | `whiteboard` uses `realtime` event types. Does NOT depend on `db` or `api` directly. |
| **R6: `services` depends on `core` + `db` + `auth`** | Business logic orchestrates across data and auth. |
| **R7: `api` depends on `core` + `services` + `auth` + `storage`** | HTTP layer wires services together. |
| **R8: `search` depends on `core` + `db`** | Search indexes data from the database. |
| **R9: `storage` depends on `core` only** | S3/MinIO client, no business logic. |
| **R10: `email` depends on `core` only** | SMTP client, no business logic. |
| **R11: `app` depends on ALL crates** | Binary entry point wires everything together. |
| **R12: No circular dependencies** | If you find yourself needing a circular dependency, extract the shared type into `core`. |
| **R13: No cross-crate direct DB access** | Only `db` crate may contain SQL queries. Services call repository traits, not raw SQL. |
| **R14: No cross-crate direct Redis access** | Only `realtime` and `whiteboard` crates may use Redis directly. Other crates go through service interfaces. |

### 3.4 What Goes in `core`

The `core` crate contains ONLY:
- Domain type definitions (structs, enums)
- Repository traits (interfaces, not implementations)
- Error types (`thiserror`)
- Shared constants and configuration structs
- ID types (`UserId`, `OrgId`, `ChannelId`, `MessageId`, etc.)
- Event type definitions shared across crates

**Never** put business logic, SQL queries, or HTTP handlers in `core`.

---

## 4. Naming Conventions

### 4.1 General Rust Naming

| Element | Convention | Example |
|---|---|---|
| Crate names | `snake_case` | `my_feature` |
| Modules | `snake_case` | `user_service` |
| Structs | `PascalCase` | `UserRepository` |
| Enums | `PascalCase` | `PermissionLevel` |
| Enum variants | `PascalCase` | `PermissionLevel::Admin` |
| Traits | `PascalCase` | `Repository` |
| Functions | `snake_case` | `create_user` |
| Variables | `snake_case` | `user_id` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_MESSAGE_SIZE` |
| Type aliases | `PascalCase` | `UserId` |
| Macros | `snake_case!` | `validate_email!` |

### 4.2 Crate-Specific Naming

| Crate | Pattern | Example |
|---|---|---|
| `core` | Domain types: `{Entity}{Property}` | `UserEmail`, `ChannelName` |
| `db` | Repositories: `{Entity}Repository` | `UserRepository` |
| `db` | Queries: `{action}_{entity}` | `insert_user`, `select_channel_by_id` |
| `auth` | Functions: `{verb}_{noun}` | `validate_token`, `hash_password` |
| `realtime` | Events: `{Entity}{Action}` | `MessageNew`, `PresenceChanged` |
| `services` | Services: `{Domain}Service` | `ChannelService`, `MessageService` |
| `api` | Handlers: `{verb}_{entity}` | `create_channel`, `list_messages` |
| `api` | DTOs: `{Entity}{Request/Response}` | `CreateChannelRequest` |
| `whiteboard` | Types: `Whiteboard{Thing}` | `WhiteboardDocument`, `WhiteboardPresence` |

### 4.3 Database Naming

| Element | Convention | Example |
|---|---|---|
| Tables | `snake_case` plural | `users`, `channel_members` |
| Columns | `snake_case` | `created_at`, `org_id` |
| Foreign Keys | `{table}_id` | `user_id`, `channel_id` |
| Indexes | `idx_{table}_{columns}` | `idx_messages_channel_created` |
| Migrations | `NNNN_{description}.sql` | `0001_initial.sql` |
| Enum types | `snake_case` | `permission_level` |

### 4.4 WebSocket Event Naming

Events use the pattern `{Entity}{Action}` in `PascalCase`:

```
MessageNew, MessageEdited, MessageDeleted
ReactionAdded, ReactionRemoved
TypingStarted, TypingStopped
PresenceChanged
ChannelCreated, ChannelMemberJoined
TaskCreated, TaskUpdated, TaskAssigned
WhiteboardJoin, WhiteboardSync, WhiteboardUpdate
```

---

## 5. Error Handling Patterns

### 5.1 Error Type Hierarchy

```rust
// core/src/error.rs

/// The single error type for the entire application.
/// Each variant wraps a domain-specific error.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication failed: {0}")]
    Authentication(#[from] AuthError),

    #[error("authorization failed: {0}")]
    Authorization(#[from] AuthError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("internal error: {0}")]
    Internal(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
}
```

### 5.2 Rules

| Rule | Details |
|---|---|
| **E1: Use `thiserror` for all error types** | Every crate's error type derives `thiserror::Error`. |
| **E2: Use `anyhow` only in `app` crate** | Binary entry point uses `anyhow::Result`. Libraries use concrete error types. |
| **E3: Never use `unwrap()` or `expect()` in library code** | Use `?` with proper error propagation. Only use `expect()` in tests. |
| **E4: Never panic in library code** | Return `Err(AppError::Internal(...))` instead. |
| **E5: Map errors at crate boundaries** | When calling another crate's function, map their error into your crate's error type. |
| **E6: Include context in errors** | Use `#[error("...")]` with descriptive messages. Include IDs where relevant. |
| **E7: HTTP errors map to status codes** | `AppError` implements `IntoResponse` for Axum, mapping to proper HTTP status codes. |
| **E8: Log errors before returning** | Use `tracing::error!` or `tracing::warn!` before returning errors from service layer. |

### 5.3 HTTP Status Code Mapping

| Error | HTTP Status |
|---|---|
| `Authentication` | 401 Unauthorized |
| `Authorization` | 403 Forbidden |
| `NotFound` | 404 Not Found |
| `Conflict` | 409 Conflict |
| `Validation` | 422 Unprocessable Entity |
| `RateLimited` | 429 Too Many Requests |
| `Database` | 500 Internal Server Error |
| `Internal` | 500 Internal Server Error |
| `ServiceUnavailable` | 503 Service Unavailable |

---

## 6. Database Conventions

### 6.1 SQLx Rules

| Rule | Details |
|---|---|
| **D1: Compile-time checked queries only** | Use `sqlx::query!` and `sqlx::query_as!` macros. Never use `sqlx::query()` (runtime string). |
| **D2: All queries go in `db` crate** | No SQL in `services`, `api`, or any other crate. |
| **D3: Use `query_as!` with structs** | Define row structs in `db` crate, map query results directly. |
| **D4: Transactions use `sqlx::Transaction`** | Begin, commit/rollback explicitly. Use `?` for early return on error. |
| **D5: Connection pool via `sqlx::PgPool`** | Configure in `app`, pass `PgPool` clones to services. |
| **D6: Migrations via `sqlx::migrate!`** | All migrations in `db/migrations/`. Run at startup in `app`. |

### 6.2 Schema Rules

| Rule | Details |
|---|---|
| **D7: Every table has `id UUID PRIMARY KEY DEFAULT uuid_generate_v7()`** | UUID v7 for time-ordering. |
| **D8: Every table has `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`** | Audit trail. |
| **D9: Every table has `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`** | Use a trigger or update in application code. |
| **D10: Multi-tenant tables have `org_id UUID NOT NULL`** | Every query MUST filter by `org_id`. |
| **D11: Soft delete pattern** | Use `deleted_at TIMESTAMPTZ` + `deleted_by UUID REFERENCES users(id)`. Never hard-delete user data. |
| **D12: Foreign keys use `ON DELETE CASCADE`** | Except for audit logs which use `ON DELETE SET NULL`. |
| **D13: Indexes on all foreign keys** | Every `*_id` column that's used in WHERE clauses must be indexed. |
| **D14: Partial indexes for soft-deleted rows** | `CREATE INDEX ... WHERE deleted_at IS NULL` for active-record queries. |
| **D15: Use `JSONB` for flexible schemas** | Element data, message metadata, etc. |
| **D16: Use `UUID[]` for arrays of IDs** | Not `TEXT[]` — proper type safety. |
| **D17: Use `BYTEA` for binary data** | CRDT documents, encrypted fields. |
| **D18: Use `DOUBLE PRECISION` for coordinates** | Whiteboard positions, viewport data. |

### 6.3 Query Patterns

```rust
// CORRECT: Always filter by org_id for multi-tenant isolation
sqlx::query_as!(
    Channel,
    r#"
    SELECT id, org_id, name, description, created_at, updated_at
    FROM channels
    WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL
    "#,
    channel_id,
    org_id
)

// WRONG: Missing org_id filter — SECURITY VULNERABILITY
sqlx::query_as!(
    Channel,
    r#"
    SELECT id, org_id, name, description, created_at, updated_at
    FROM channels
    WHERE id = $1
    "#,
    channel_id
)
```

---

## 7. API Design Patterns

### 7.1 REST Conventions

| Rule | Details |
|---|---|
| **A1: Base path `/api/v1`** | All endpoints prefixed with version. |
| **A2: Resource nesting max 3 levels** | `/api/v1/orgs/{org_id}/channels/{channel_id}/messages` |
| **A3: Use plural nouns for resources** | `/channels`, not `/channel` |
| **A4: Use HTTP verbs semantically** | GET (read), POST (create), PUT (replace), PATCH (update), DELETE (remove) |
| **A5: Return JSON always** | `Content-Type: application/json` |
| **A6: Use consistent response envelope** | `{ "data": T, "meta": Meta }` for lists, `{ "data": T }` for singles |
| **A7: Pagination via query params** | `?page=1&per_page=20` — return `{ data: [], meta: { page, per_page, total } }` |
| **A8: Filtering via query params** | `?sort=created_at&order=desc&status=active` |
| **A9: Return proper HTTP status codes** | 200, 201, 204, 400, 401, 403, 404, 409, 422, 429, 500 |
| **A10: Validate all input** | Use `validator` derive on request DTOs |

### 7.2 Request/Response DTOs

```rust
// api/src/dto/channel.rs

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: String,

    #[validate(length(max = 255))]
    pub description: Option<String>,

    pub is_private: bool,
}

#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 7.3 Handler Pattern

```rust
// api/src/handlers/channel.rs

pub async fn create_channel(
    State(state): State<ApiState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let channel = state
        .services
        .channel
        .create(org_id, req)
        .await?;

    Ok((StatusCode::CREATED, Json(channel)))
}
```

---

## 8. WebSocket Protocol Conventions

### 8.1 Connection

| Rule | Details |
|---|---|
| **W1: Single WS endpoint** | `WS /api/v1/ws?token={jwt}` — one connection for all real-time events. |
| **W2: JWT in query param** | Token passed in connection URL, validated on upgrade. |
| **W3: Heartbeat every 30s** | Client sends ping, server responds with pong. Timeout after 60s. |
| **W4: Max message size 1MB** | Reject larger messages. |
| **W5: Rate limit 100 msg/sec** | Per-connection rate limiting via Redis. |

### 8.2 Event Envelope

All WebSocket messages use a typed envelope:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RealtimeEvent {
    // Communication
    MessageNew(MessageNewEvent),
    MessageEdited(MessageEditedEvent),
    MessageDeleted(MessageDeletedEvent),
    ReactionAdded(ReactionAddedEvent),
    ReactionRemoved(ReactionRemovedEvent),
    TypingStarted(TypingStartedEvent),
    TypingStopped(TypingStoppedEvent),

    // Presence
    PresenceChanged(PresenceChangedEvent),

    // Channels
    ChannelCreated(ChannelCreatedEvent),
    ChannelMemberJoined(ChannelMemberMemberJoinedEvent),

    // Tasks
    TaskCreated(TaskCreatedEvent),
    TaskUpdated(TaskUpdatedEvent),
    TaskAssigned(TaskAssignedEvent),

    // Notifications
    NotificationNew(NotificationNewEvent),

    // Whiteboard
    WhiteboardJoin(WhiteboardJoinEvent),
    WhiteboardJoined(WhiteboardJoinedEvent),
    WhiteboardSync(WhiteboardSyncEvent),
    WhiteboardUpdate(WhiteboardUpdateEvent),
    WhiteboardPresenceUpdate(WhiteboardPresenceUpdateEvent),
    WhiteboardPresenceChanged(WhiteboardPresenceChangedEvent),
    WhiteboardLeave(WhiteboardLeaveEvent),
    WhiteboardUserLeft(WhiteboardUserLeftEvent),
    WhiteboardUndo(WhiteboardUndoEvent),
    WhiteboardRedo(WhiteboardRedoEvent),
    WhiteboardHistoryChanged(WhiteboardHistoryChangedEvent),
    WhiteboardCreateVersion(WhiteboardCreateVersionEvent),
    WhiteboardVersionCreated(WhiteboardVersionCreatedEvent),
    WhiteboardError(WhiteboardErrorEvent),

    // System
    Ping,
    Pong,
    Error(WsErrorEvent),
}
```

### 8.3 Event Rules

| Rule | Details |
|---|---|
| **W6: Events are typed** | Use `#[serde(tag = "type", content = "payload")]` — no raw JSON. |
| **W7: Events include `org_id`** | Every event payload includes the `org_id` for tenant isolation. |
| **W8: Events include `timestamp`** | Every event payload includes `chrono::DateTime<Utc>`. |
| **W9: Fan-out via Redis pub/sub** | Events published to `channel:{channel_id}` or `user:{user_id}` Redis channels. |
| **W10: Presence via Redis SET with TTL** | `presence:{org_id}` → SET of user IDs, TTL 60s, refreshed by heartbeat. |

---

## 9. Multi-Tenancy Rules — CRITICAL

**This is the most important section of this document. Violating multi-tenancy rules is a security breach.**

### 9.1 Rules

| Rule | Details |
|---|---|
| **M1: Every table has `org_id`** | All business data tables MUST have `org_id UUID NOT NULL`. |
| **M2: Every query filters by `org_id`** | No exceptions. Every SELECT, UPDATE, DELETE must include `WHERE org_id = $X`. |
| **M3: RLS policies on every table** | PostgreSQL Row Level Security policies enforce `org_id` isolation at the database level. |
| **M4: Extract `org_id` from JWT** | The org context comes from the authenticated user's JWT claims, never from client input. |
| **M5: Validate org membership** | Before any operation, verify the user belongs to the org. |
| **M6: No cross-org data leakage** | Never return data from one org in response to a request authenticated for another org. |
| **M7: Redis keys include org_id** | `org:{org_id}:channel:{channel_id}` — never global keys. |
| **M8: WebSocket events include org_id** | Clients only receive events for their authenticated org. |
| **M9: Search indexes include org_id** | Meilisearch index per org, or filter by `org_id` in every search query. |
| **M10: File paths include org_id** | S3 keys: `{org_id}/{file_id}/{filename}`. |

### 9.2 RLS Policy Template

```sql
-- Enable RLS on every multi-tenant table
ALTER TABLE messages ENABLE ROW LEVEL SECURITY;

-- Policy: users can only see messages in their org
CREATE POLICY messages_org_isolation ON messages
    USING (org_id = current_setting('app.current_org_id')::UUID);

-- Set org_id per transaction (done in application code)
SET LOCAL app.current_org_id = 'org-uuid-here';
```

---

## 10. File & Module Organization Rules

### 10.1 Crate Structure Template

Every crate follows this structure:

```
{crate}/
├── Cargo.toml              # Dependencies (use workspace = true)
├── src/
│   ├── lib.rs              # Public API, re-exports, module declarations
│   ├── error.rs            # Crate-specific error types (thiserror)
│   ├── {module}.rs         # Feature modules
│   ├── {module}/
│   │   ├── mod.rs          # Module declaration
│   │   ├── {submodule}.rs  # Sub-modules
│   │   └── ...
│   └── ...
├── tests/
│   ├── integration.rs      # Integration tests
│   └── {feature}_tests.rs  # Feature-specific tests
└── migrations/             # (db crate only) SQL migration files
```

### 10.2 Module Organization by Crate

| Crate | Modules |
|---|---|
| `core` | `types/`, `traits/`, `error.rs`, `ids.rs`, `config.rs`, `events.rs` |
| `db` | `repositories/`, `queries/`, `migrations/`, `pool.rs`, `transaction.rs` |
| `auth` | `jwt.rs`, `password.rs`, `permissions.rs`, `oauth.rs`, `session.rs` |
| `realtime` | `server.rs`, `connection.rs`, `events.rs`, `presence.rs`, `pubsub.rs`, `heartbeat.rs` |
| `services` | `channel.rs`, `message.rs`, `task.rs`, `notification.rs`, `search.rs`, `file.rs`, `webhook.rs`, `user.rs`, `org.rs` |
| `api` | `handlers/`, `middleware/`, `dto/`, `routes.rs`, `state.rs`, `error.rs` |
| `search` | `client.rs`, `indexer.rs`, `queries.rs` |
| `storage` | `client.rs`, `upload.rs`, `download.rs`, `presign.rs` |
| `email` | `client.rs`, `templates.rs`, `sender.rs` |
| `whiteboard` | `document.rs`, `element.rs`, `tool.rs`, `sync.rs`, `presence.rs`, `undo.rs`, `export.rs`, `spatial.rs`, `metrics.rs`, `error.rs` |
| `app` | `main.rs`, `config.rs`, `wiring.rs`, `shutdown.rs` |

### 10.3 Module Rules

| Rule | Details |
|---|---|
| **F1: One module per domain concept** | `channel.rs` contains all channel-related logic for that crate. |
| **F2: `mod.rs` declares sub-modules** | Every directory has a `mod.rs` that declares and re-exports. |
| **F3: `lib.rs` is the public API** | Only items marked `pub` in `lib.rs` are accessible from other crates. |
| **F4: Use `pub(crate)` for internal sharing** | Items shared within a crate but not external use `pub(crate)`. |
| **F5: Keep modules under 300 lines** | If a module grows beyond 300 lines, split it into sub-modules. |
| **F6: Handler functions in `api/handlers/`** | One file per resource: `channel.rs`, `message.rs`, etc. |
| **F7: DTOs in `api/dto/`** | Request/response types colocated with handlers. |

---

## 11. How to Add a New Feature (Process)

When adding a new feature, follow this exact process:

### Step 1: Define the Slice
- Identify which vertical slice the feature belongs to (see `VERTICAL_SLICES.md`)
- If it's a new slice, document it with: name, description, crates touched, dependencies, acceptance criteria

### Step 2: Database Changes
1. Create a new migration file in `db/migrations/NNNN_{feature}.sql`
2. Follow all schema rules (D7-D18)
3. Add RLS policies for any new multi-tenant tables
4. Run `cargo sqlx migrate run` to apply

### Step 3: Core Types
1. Add domain types to `core/src/types/{feature}.rs`
2. Add repository traits to `core/src/traits/{feature}_repository.rs`
3. Add error variants to `core/src/error.rs` if needed
4. Re-export from `core/src/lib.rs`

### Step 4: Repository Implementation
1. Implement repository traits in `db/src/repositories/{feature}.rs`
2. Use `sqlx::query_as!` macros (compile-time checked)
3. Always filter by `org_id`
4. Write unit tests with `#[sqlx::test]`

### Step 5: Service Layer
1. Add service to `services/src/{feature}.rs`
2. Implement business logic
3. Call repository traits (not raw SQL)
4. Emit real-time events via `realtime` crate
5. Write unit tests with mock repositories

### Step 6: API Layer
1. Add DTOs to `api/src/dto/{feature}.rs`
2. Add handlers to `api/src/handlers/{feature}.rs`
3. Register routes in `api/src/routes.rs`
4. Add middleware (auth, validation, rate limiting)
5. Write integration tests

### Step 7: Real-Time Events (if applicable)
1. Add event types to `realtime/src/events.rs`
2. Implement event broadcasting in service layer
3. Add WebSocket handlers if needed

### Step 8: Wire Up in App
1. Initialize service in `app/src/wiring.rs`
2. Register routes in `app/src/main.rs`
3. Add configuration to `app/src/config.rs`

### Step 9: Testing
1. Unit tests for all new modules
2. Integration tests for all new endpoints
3. Property-based tests for complex logic
4. Load tests for performance-critical paths

### Step 10: Documentation
1. Update `VERTICAL_SLICES.md` with implementation notes
2. Update API documentation
3. Add inline doc comments to all public items

---

## 12. How to Add a New Crate

If a new crate is needed (e.g., `whiteboard` was added):

### Step 1: Justify
- Explain why the functionality cannot live in an existing crate
- Document which existing crates it will depend on
- Confirm no circular dependencies

### Step 2: Scaffold
```bash
cargo new {crate_name} --lib
```

### Step 3: Register in Workspace
Add to root `Cargo.toml`:
```toml
[workspace]
members = [
    # ... existing ...
    "{crate_name}",
]

[workspace.dependencies]
{crate_name} = { path = "{crate_name}" }
```

### Step 4: Add Dependencies
In `{crate_name}/Cargo.toml`, add dependencies using `{ workspace = true }` for external crates and `{ path = "../{crate}" }` for internal crates.

### Step 5: Follow Structure
Follow the crate structure template in Section 10.1.

### Step 6: Wire Up
Add to `app/src/wiring.rs` and `app/src/main.rs`.

### Step 7: Document
Update `ARCHITECTURE_GUARDRAIL.md` crate list and dependency graph.
Update `VERTICAL_SLICES.md` with new slice.

---

## 13. Testing Requirements

### 13.1 Minimum Test Coverage

| Crate | Unit Tests | Integration Tests | Property Tests |
|---|---|---|---|
| `core` | 90%+ | N/A | For complex types |
| `db` | 80%+ | 100% of repositories | N/A |
| `auth` | 90%+ | 100% of auth flows | Password hashing |
| `realtime` | 80%+ | 100% of event types | N/A |
| `services` | 85%+ | 100% of service methods | Business logic |
| `api` | 70%+ | 100% of endpoints | N/A |
| `search` | 80%+ | 100% of search queries | N/A |
| `storage` | 80%+ | 100% of upload/download | N/A |
| `email` | 80%+ | Template rendering | N/A |
| `whiteboard` | 90%+ | CRDT sync, reconnection | Concurrent edits |
| `app` | N/A | Smoke tests | N/A |

### 13.2 Test Rules

| Rule | Details |
|---|---|
| **T1: Use `#[tokio::test]` for async tests** | All async test functions use this attribute. |
| **T2: Use `#[sqlx::test]` for DB tests** | Automatically creates test database with migrations. |
| **T3: Mock external services** | Use `mockall` or manual trait impls for Redis, S3, Meilisearch. |
| **T4: Test error paths** | Every `Result::Err` branch must have a test. |
| **T5: Test multi-tenancy** | Every repository test must verify `org_id` filtering. |
| **T6: Property tests for CRDT** | Use `proptest` for whiteboard concurrent edit scenarios. |
| **T7: Benchmark critical paths** | Use `criterion` for hot paths (message send, search, sync). |
| **T8: Integration tests hit real endpoints** | Use `tower::ServiceExt` for in-process HTTP testing. |

### 13.3 Test Naming

```rust
#[tokio::test]
async fn test_create_channel_success() { ... }

#[tokio::test]
async fn test_create_channel_duplicate_name_returns_conflict() { ... }

#[tokio::test]
async fn test_create_channel_unauthorized_returns_401() { ... }

#[tokio::test]
async fn test_create_channel_cross_org_access_forbidden() { ... }
```

---

## 14. Code Style Rules

### 14.1 Formatting

| Rule | Details |
|---|---|
| **S1: Use `rustfmt` default style** | Run `cargo fmt` before every commit. |
| **S2: Max line length 100** | `rustfmt` default. |
| **S2: Use `clippy` with `-D warnings`** | Zero clippy warnings allowed. |
| **S3: Import ordering** | `std`, external crates, `crate::`, `super::` — alphabetical within groups. |
| **S4: No unused imports** | `#[deny(unused_imports)]` at crate level. |

### 14.2 Idioms

| Rule | Details |
|---|---|
| **S5: Prefer `?` over `match` for error propagation** | Use `?` operator. |
| **S6: Use `impl Trait` for return types** | `fn get_user(id: Uuid) -> Result<User, AppError>` not `fn get_user(id: Uuid) -> Result<User, Box<dyn Error>>`. |
| **S7: Use `Arc<T>` for shared state** | Not `Rc<T>` — we're multi-threaded. |
| **S8: Use `tracing::instrument` on public functions** | Automatic span creation for observability. |
| **S9: Prefer `Cow<str>` for string parameters** | Avoid unnecessary allocations. |
| **S10: Use `NonZeroU32`/`NonZeroU64` for IDs** | When zero is not a valid value. |

### 14.3 Derive Macros

Always derive in this order:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyType { ... }
```

---

## 15. Documentation Requirements

### 15.1 Inline Documentation

| Rule | Details |
|---|---|
| **Doc1: All public items have doc comments** | `/// Description` on every `pub` struct, enum, trait, fn, mod. |
| **Doc2: Complex logic has inline comments** | Explain WHY, not WHAT. |
| **Doc3: Examples in doc comments** | Use ` ```rust ` blocks for non-trivial types. |
| **Doc4: Document error conditions** | Use `# Errors` section in doc comments for fallible functions. |
| **Doc5: Document panics** | Use `# Panics` section if a function can panic. |

### 15.2 External Documentation

| Document | When to Update |
|---|---|
| `ARCHITECTURE_GUARDRAIL.md` | When adding crates, changing dependency rules, adding conventions |
| `VERTICAL_SLICES.md` | When adding slices, changing dependencies, completing slices |
| `ARCHITECTURE.md` | When high-level architecture changes |
| `WHITEBOARD_DESIGN.md` | When whiteboard design changes |
| API docs | When adding/changing endpoints |
| `README.md` | When setup instructions change |

---

## 16. Prohibited Patterns — Never Do These

| # | Pattern | Why |
|---|---|---|
| P1 | **Raw SQL outside `db` crate** | Breaks compile-time checking, leaks DB concerns |
| P2 | **Missing `org_id` filter in queries** | Security vulnerability — cross-tenant data leak |
| P3 | **Using `unwrap()` in non-test code** | Panics in production |
| P4 | **Circular crate dependencies** | Compilation issues, architectural decay |
| P5 | **Business logic in `api` handlers** | Handlers are thin — delegate to services |
| P6 | **Direct Redis access from `services`** | Only `realtime` and `whiteboard` use Redis directly |
| P7 | **Stringly-typed events** | Use typed `RealtimeEvent` enum |
| P8 | **UUID v4 for entity IDs** | Use UUID v7 for time-ordering |
| P9 | **Hard deletes on user data** | Use soft delete pattern |
| P10 | **Synchronous I/O in async context** | Use `tokio::fs`, `tokio::io` |
| P11 | **Blocking the async runtime** | No `std::thread::sleep`, no blocking locks across `.await` |
| P12 | **Storing secrets in code** | Use environment variables or secret manager |
| P13 | **Ignoring `clippy` warnings** | Zero warnings policy |
| P14 | **Committing without running tests** | All tests must pass |
| P15 | **Modifying migrations after they've been applied** | Create new migrations |

---

## 17. Pre-Submission Checklist

Before submitting any code for review, verify:

- [ ] `cargo fmt` has been run
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes for the entire workspace
- [ ] `cargo build` succeeds in release mode
- [ ] All new public items have doc comments
- [ ] All new database queries include `org_id` filtering
- [ ] All new tables have RLS policies
- [ ] All new endpoints have integration tests
- [ ] All new services have unit tests
- [ ] `VERTICAL_SLICES.md` is updated if a slice was completed
- [ ] No prohibited patterns (Section 16) are present
- [ ] No new dependencies without justification
- [ ] Migration files are named correctly (`NNNN_description.sql`)

---

*End of Architecture Guardrail Document*
