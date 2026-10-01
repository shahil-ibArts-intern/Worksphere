# Collaborative Real-Time Whiteboard — Architecture & Design

**Status**: Design Document  
**Integrates with**: ARCHITECTURE.md (10-crate workspace)  
**Date**: 2026-09-30

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Technology Choices & Rationale](#2-technology-choices--rationale)
3. [New Crate: `whiteboard`](#3-new-crate-whiteboard)
4. [Data Models & Schema](#4-data-models--schema)
5. [CRDT Architecture](#5-crdt-architecture)
6. [WebSocket Protocol Extensions](#6-websocket-protocol-extensions)
7. [Storage Strategy](#7-storage-strategy)
8. [API Endpoints](#8-api-endpoints)
9. [Project Structure Updates](#9-project-structure-updates)
10. [Challenges & Solutions](#10-challenges--solutions)
11. [Phased Implementation Plan](#11-phased-implementation-plan)
12. [Frontend Integration Notes](#12-frontend-integration-notes)

---

## 1. Executive Summary

### What We're Building

A Figma/Miro-style collaborative whiteboard embedded in every workspace, supporting:

- **Multi-user real-time drawing** with sub-50ms sync latency
- **Rich toolset**: pen, shapes (rect, ellipse, line, arrow), text, sticky notes, eraser, laser pointer
- **Infinite canvas** with pan/zoom
- **Undo/redo** at both individual and collaborative levels
- **Presence**: live cursors, user avatars, "who's viewing" indicators
- **Persistence**: full state survives restarts, version history
- **Export**: PNG, SVG, PDF

### Design Principles

| Principle | Approach |
|---|---|
| **Conflict-free** | CRDTs (Automerge) — no central lock, no lost updates |
| **Low latency** | Optimistic local rendering + async CRDT sync |
| **Scalable** | Operation log + periodic snapshots, lazy loading |
| **Backwards-compatible** | Extends existing WebSocket protocol, no breaking changes |
| **Multi-tenant** | `org_id` + `workspace_id` isolation on every operation |

---

## 2. Technology Choices & Rationale

### 2.1 CRDT Library: **Automerge** (`automerge` crate)

| Candidate | Verdict | Reason |
|---|---|---|
| **Automerge** ✅ | **Selected** | Mature Rust CRDT, binary document format, excellent performance with `automerge-repo`, supports text + arbitrary JSON, used by Figma-adjacent tools |
| `crdt-rs` | Rejected | Unmaintained (last commit 2022), limited to specific CRDT types, no active ecosystem |
| `yrs` (via FFI) | Alternative | Very fast (used by Yjs ecosystem), but requires C FFI bindings — adds build complexity and safety concerns in a pure-Rust stack |
| `diamond-types` | Alternative | Excellent for text sequences, but whiteboards need 2D spatial data + rich element types, not just text |
| Custom OT | Rejected | Operational Transformation is notoriously hard to get right; CRDTs eliminate an entire class of conflict bugs |

**Why Automerge wins for whiteboards:**

1. **Document model fits naturally**: A whiteboard is a JSON document with maps (elements by ID) and lists (z-order). Automerge handles both.
2. **Binary sync protocol**: `automerge::sync` produces compact binary messages — perfect for WebSocket transport.
3. **Undo/redo built-in**: Automerge tracks actor IDs and change history — we can implement undo by reverting to a previous change hash.
4. **No server authority needed**: Clients merge changes peer-to-peer; the server is a persistence relay, not a bottleneck.
5. **Active ecosystem**: `automerge-repo` provides storage adapters, network transports.

### 2.2 Supporting Crates

| Crate | Purpose | Version |
|---|---|---|
| `automerge` | CRDT document operations | 0.5 |
| `automerge-repo` | Sync management, storage adapters | 0.1 |
| `tokio-tungstenite` | WebSocket client (for sync) | 0.21 |
| `resvg` / `usvg` | SVG rendering for export | 0.37 |
| `tiny-skia` | PNG rasterization | 0.11 |
| `geo` | Geometric operations (hit testing, bounds) | 0.28 |
| `petgraph` | Element dependency graph (for undo ordering) | 0.6 |
| `dashmap` | Concurrent in-memory element cache | 0.5 (already in workspace) |
| `bincode` | Fast binary serialization for Redis transport | 1.3 |

### 2.3 Why Not Yjs/Yrs?

Yjs is the most popular CRDT for collaborative apps (used by Notion, Figma's multiplayer). However:

- `yrs` requires C FFI — adds `cmake`, `cc` build dependencies, complicates cross-compilation
- Yjs ecosystem is JS-first; Rust bindings lag behind JS feature parity
- Automerge's Rust implementation is first-class, not a binding
- For a pure-Rust backend, Automerge avoids the safety/performance tax of FFI

**Migration path**: If Yjs compatibility is ever needed (e.g., for a JS-based whiteboard widget), we can add a `yjs-bridge` service that translates Automerge ops ↔ Yjs ops. This is a Phase 4+ concern.

---

## 3. New Crate: `whiteboard`

### 3.1 Workspace Integration

```
test_rust/
├── Cargo.toml          # workspace root (add whiteboard member)
├── core/               # ← no changes (shared types)
├── db/                 # ← add whiteboard migrations + repositories
├── auth/               # ← no changes
├── realtime/           # ← extend with whiteboard event types
├── services/           # ← add WhiteboardService
├── api/                # ← add whiteboard REST + WS handlers
├── search/             # ← no changes
├── storage/            # ← no changes
├── email/              # ← no changes
├── whiteboard/         # ← NEW: CRDT engine, sync, presence
└── app/                # ← wire up whiteboard module
```

### 3.2 Crate Dependency Graph

```
core ← db ← services ← api ← app
  ↑                              ↑
  └──── whiteboard ──────────────┘
         ↑
       realtime (event types)
```

The `whiteboard` crate depends on `core` (for `WorkspaceId`, `UserId`, error types) and `realtime` (for WebSocket event definitions). It does NOT depend on `db` or `api` — keeping it decoupled and testable.

### 3.3 Crate Structure

```
whiteboard/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API
│   ├── document.rs               # Automerge document wrapper
│   ├── element.rs                # Whiteboard element types
│   ├── tool.rs                   # Drawing tool definitions
│   ├── sync.rs                   # CRDT sync protocol
│   ├── presence.rs               # User presence tracking
│   ├── undo.rs                   # Undo/redo stack
│   ├── export.rs                 # PNG/SVG/PDF export
│   ├── spatial.rs                # Spatial indexing (R-tree)
│   └── metrics.rs                # Prometheus metrics
└── tests/
    ├── crdt_concurrent.rs        # Concurrent edit tests
    ├── sync_reconnect.rs         # Reconnection tests
    └── export.rs                 # Export tests
```

---

## 4. Data Models & Schema

### 4.1 PostgreSQL Tables

```sql
-- ============================================================
-- WHITEBOARD SCHEMA (Migration 0018_whiteboard.sql)
-- ============================================================

-- One whiteboard per workspace (1:1 relationship)
CREATE TABLE whiteboards (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT NOT NULL DEFAULT 'Untitled Board',
    description     TEXT DEFAULT '',
    
    -- CRDT document state (binary Automerge document)
    -- Stored as bytea for efficiency; ~1-10MB typical
    document_state  BYTEA,
    
    -- Snapshot for fast loading (JSON representation of current state)
    -- Generated periodically; used for initial load without replaying ops
    snapshot        JSONB,
    snapshot_version BIGINT DEFAULT 0,
    
    -- Version tracking
    version         BIGINT NOT NULL DEFAULT 0,  -- monotonically increasing
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Soft delete
    deleted_at      TIMESTAMPTZ,
    deleted_by      UUID REFERENCES users(id),
    
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(workspace_id, id)
);

CREATE INDEX idx_whiteboards_workspace ON whiteboards(workspace_id) 
    WHERE deleted_at IS NULL;

-- Operation log: append-only, used for replay and sync
-- Partitioned by workspace_id for query performance
CREATE TABLE whiteboard_operations (
    id              BIGSERIAL,
    board_id        UUID NOT NULL REFERENCES whiteboards(id) ON DELETE CASCADE,
    workspace_id    UUID NOT NULL,
    
    -- CRDT change hash from Automerge
    change_hash     BYTEA NOT NULL,
    
    -- Binary Automerge change data
    change_data     BYTEA NOT NULL,
    
    -- Who made the change
    user_id         UUID NOT NULL REFERENCES users(id),
    
    -- Client session that produced this op
    session_id      UUID NOT NULL,
    
    -- Timestamp from the client (for ordering within a session)
    client_timestamp TIMESTAMPTZ NOT NULL,
    
    -- Server timestamp (for global ordering)
    server_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Element(s) affected (for UI filtering / partial sync)
    affected_elements UUID[] DEFAULT '{}',
    
    PRIMARY KEY (board_id, id)
) PARTITION BY HASH (board_id);

-- Create 16 partitions for parallel writes
CREATE TABLE whiteboard_operations_p0 PARTITION OF whiteboard_operations
    FOR VALUES WITH (MODULUS 16, REMAINDER 0);
-- ... p1 through p15

CREATE INDEX idx_wb_ops_board_version ON whiteboard_operations(board_id, id);
CREATE INDEX idx_wb_ops_session ON whiteboard_operations(session_id);
CREATE INDEX idx_wb_ops_user ON whiteboard_operations(user_id, server_timestamp DESC);

-- Active editing sessions (for presence)
CREATE TABLE whiteboard_sessions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    board_id        UUID NOT NULL REFERENCES whiteboards(id) ON DELETE CASCADE,
    workspace_id    UUID NOT NULL,
    user_id         UUID NOT NULL REFERENCES users(id),
    
    -- Connection info
    connection_id   TEXT NOT NULL,  -- WebSocket connection ID
    
    -- Viewport state (for "follow user" feature)
    viewport_x      DOUBLE PRECISION DEFAULT 0,
    viewport_y      DOUBLE PRECISION DEFAULT 0,
    viewport_zoom   DOUBLE PRECISION DEFAULT 1.0,
    
    -- Cursor position (for live cursors)
    cursor_x        DOUBLE PRECISION,
    cursor_y        DOUBLE PRECISION,
    cursor_tool     TEXT,           -- current tool
    
    -- User display
    display_name    TEXT NOT NULL,
    avatar_url      TEXT,
    color           TEXT NOT NULL,  -- assigned cursor color
    
    -- Timing
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(board_id, user_id, connection_id)
);

CREATE INDEX idx_wb_sessions_board ON whiteboard_sessions(board_id) 
    WHERE last_activity_at > NOW() - INTERVAL '5 minutes';

-- Version history (named snapshots for "restore to version")
CREATE TABLE whiteboard_versions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    board_id        UUID NOT NULL REFERENCES whiteboards(id) ON DELETE CASCADE,
    workspace_id    UUID NOT NULL,
    
    name            TEXT NOT NULL,  -- "Before redesign", "v2", etc.
    description     TEXT DEFAULT '',
    
    -- Full document state at this version
    document_state  BYTEA NOT NULL,
    version         BIGINT NOT NULL,
    
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(board_id, name)
);

-- Element metadata (for search, filtering, and Meilisearch indexing)
-- Denormalized from CRDT document for query performance
CREATE TABLE whiteboard_elements (
    id              UUID PRIMARY KEY,
    board_id        UUID NOT NULL REFERENCES whiteboards(id) ON DELETE CASCADE,
    workspace_id    UUID NOT NULL,
    
    -- Element type: 'path' | 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'text' | 'sticky' | 'image'
    element_type    TEXT NOT NULL,
    
    -- Spatial index for viewport-based queries
    bbox_x          DOUBLE PRECISION NOT NULL,
    bbox_y          DOUBLE PRECISION NOT NULL,
    bbox_width      DOUBLE PRECISION NOT NULL,
    bbox_height     DOUBLE PRECISION NOT NULL,
    
    -- Z-order (for layering)
    z_index         INTEGER NOT NULL DEFAULT 0,
    
    -- Creator
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Soft delete (for undo support)
    deleted_at      TIMESTAMPTZ,
    deleted_by      UUID REFERENCES users(id),
    
    -- CRDT metadata
    crdt_key        TEXT NOT NULL,  -- key in the Automerge document
    
    -- Full element data (JSON, for search indexing)
    data            JSONB NOT NULL
);

CREATE INDEX idx_wb_elements_board ON whiteboard_elements(board_id) 
    WHERE deleted_at IS NULL;
CREATE INDEX idx_wb_elements_type ON whiteboard_elements(board_id, element_type) 
    WHERE deleted_at IS NULL;
CREATE INDEX idx_wb_elements_spatial ON whiteboard_elements USING GIST (
    point(bbox_x + bbox_width/2, bbox_y + bbox_height/2)
);
CREATE INDEX idx_wb_elements_gin ON whiteboard_elements USING GIN(data);

-- Permissions (reuse existing permission system)
CREATE TABLE whiteboard_permissions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    board_id        UUID NOT NULL REFERENCES whiteboards(id) ON DELETE CASCADE,
    workspace_id    UUID NOT NULL,
    
    -- Can be user or role
    grantee_type    TEXT NOT NULL CHECK (grantee_type IN ('user', 'role')),
    grantee_id     UUID NOT NULL,
    
    -- Permission level
    permission      TEXT NOT NULL CHECK (permission IN ('view', 'comment', 'edit', 'admin')),
    
    granted_by      UUID NOT NULL REFERENCES users(id),
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(board_id, grantee_type, grantee_id)
);
```

### 4.2 Redis Data Structures

```
# Active board sessions (TTL: 5 min, refreshed by heartbeat)
wb:session:{board_id}:{user_id}:{conn_id}  →  JSON { viewport, cursor, tool, color, ... }
wb:sessions:{board_id}  →  SET of session keys

# Operation buffer (for batching writes to Postgres)
wb:ops:{board_id}  →  LIST of serialized operations (capped at 1000)

# Board metadata cache
wb:meta:{board_id}  →  JSON { version, snapshot_version, last_activity }

# Presence pub/sub channel
wb:presence:{board_id}  →  Redis pub/sub channel

# Sync channel (for broadcasting CRDT changes)
wb:sync:{board_id}  →  Redis pub/sub channel

# Rate limiting per user
wb:ratelimit:{board_id}:{user_id}  →  INCR with EXPIRES (100 ops/sec)
```

### 4.3 Core Rust Types

```rust
// whiteboard/src/lib.rs

use automerge::{AutoCommit, transaction::Transactable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a whiteboard element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub Uuid);

/// The CRDT document wrapper for a whiteboard
pub struct WhiteboardDocument {
    doc: AutoCommit,
    board_id: Uuid,
    workspace_id: Uuid,
}

/// All element types supported by the whiteboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Element {
    Path(PathElement),
    Rectangle(RectangleElement),
    Ellipse(EllipseElement),
    Line(LineElement),
    Arrow(ArrowElement),
    Text(TextElement),
    StickyNote(StickyNoteElement),
    Image(ImageElement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathElement {
    pub id: ElementId,
    pub points: Vec<Point>,
    pub stroke_color: Color,
    pub stroke_width: f64,
    pub stroke_style: StrokeStyle,  // solid, dashed, dotted
    pub opacity: f64,
    pub bbox: BBox,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub z_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectangleElement {
    pub id: ElementId,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f64,
    pub corner_radius: f64,
    pub opacity: f64,
    pub rotation: f64,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub z_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    pub id: ElementId,
    pub position: Point,
    pub content: String,
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub color: Color,
    pub align: TextAlign,
    pub width: f64,  // bounding box for wrapping
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub z_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickyNoteElement {
    pub id: ElementId,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    pub color: StickyColor,  // yellow, pink, blue, green, purple
    pub text: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub z_index: i32,
}

#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,  // alpha 0-255
}

/// Drawing tools available in the toolbar
#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Pen,
    Highlighter,
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Text,
    StickyNote,
    Eraser,
    LaserPointer,  // temporary, doesn't persist
    Hand,          // pan tool
}

/// User presence on a whiteboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardPresence {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: String,           // hex color for cursor
    pub cursor: Option<Point>,
    pub tool: Tool,
    pub viewport: Viewport,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}
```

---

## 5. CRDT Architecture

### 5.1 Why CRDTs Are the Right Fit

Whiteboard editing is a **multi-writer, partition-tolerant** problem:

- Multiple users draw simultaneously on the same canvas
- Network partitions happen (mobile, flaky WiFi)
- Users expect **their** edits to never be lost
- No single server should be the "source of truth" bottleneck

CRDTs provide **strong eventual consistency** — all replicas converge to the same state without coordination. This is exactly what we need.

### 5.2 Automerge Document Structure

The whiteboard is modeled as an Automerge document with this structure:

```json
{
  "elements": {
    "element-uuid-1": { "type": "path", "points": [...], "stroke": "#000", ... },
    "element-uuid-2": { "type": "rectangle", "x": 100, "y": 200, ... },
    "element-uuid-3": { "type": "text", "content": "Hello", ... }
  },
  "z_order": ["element-uuid-1", "element-uuid-2", "element-uuid-3"],
  "metadata": {
    "board_id": "...",
    "workspace_id": "...",
    "version": 42,
    "background": "#ffffff",
    "grid_enabled": true
  }
}
```

**Key design decisions:**

| Decision | Rationale |
|---|---|
| `elements` as an Automerge **Map** | Concurrent edits to different elements never conflict. Each element is an independent key. |
| `z_order` as an Automerge **List** | Concurrent z-order changes (e.g., two users reordering) are resolved by CRDT merge rules — last-writer-wins per position, but no lost elements. |
| Element IDs are **UUID v7** | Time-ordered, unique across clients, no coordination needed. |
| Each element stores `created_by` + `created_at` | For attribution, even after CRDT merge. |

### 5.3 Concurrent Edit Scenarios

#### Scenario 1: Two users draw different shapes

```
User A: creates Rectangle R1 at (100, 100)
User B: creates Circle C1 at (300, 300)

Automerge result: Both R1 and C1 exist in the document.
No conflict — different keys in the elements map.
```

#### Scenario 2: Two users move the same element

```
User A: moves Element E1 to (500, 500)
User B: moves Element E1 to (200, 200)

Automerge result: Last-writer-wins based on actor ID + timestamp.
Both users see the same final position after sync.
No data loss — the element still exists.
```

#### Scenario 3: User A deletes while User B edits

```
User A: deletes Element E1
User B: changes E1's color to red

Automerge result: E1 is deleted (delete wins over modify in our config).
User B's color change is lost — this is acceptable UX (user B can undo).
Alternative: We could implement "soft delete" where deleted elements are 
  marked with `deleted_at` and can be restored.
```

#### Scenario 4: Concurrent z-order changes

```
User A: moves E1 to top of z_order
User B: moves E2 to top of z_order

Automerge result: Both E1 and E2 are at the top, 
  with deterministic ordering based on actor IDs.
Both users see the same final z_order after sync.
```

### 5.4 Sync Protocol

```
┌──────────┐                    ┌──────────┐                    ┌──────────┐
│ Client A │                    │  Server  │                    │ Client B │
└────┬─────┘                    └────┬─────┘                    └────┬─────┘
     │                               │                               │
     │  1. Connect + send sync msg   │                               │
     │ ─────────────────────────────>│                               │
     │                               │                               │
     │  2. Server responds with      │                               │
     │     missing changes           │                               │
     │ <─────────────────────────────│                               │
     │                               │                               │
     │  3. Client applies changes,   │                               │
     │     sends its own changes     │                               │
     │ ─────────────────────────────>│                               │
     │                               │  4. Server persists +          │
     │                               │     broadcasts to B            │
     │                               │ ─────────────────────────────>│
     │                               │                               │
     │  5. Both clients now have     │                               │
     │     identical state           │                               │
     │                               │                               │
     │  6. Ongoing: incremental      │                               │
     │     sync via Redis pub/sub    │                               │
     │ <────────────────────────────>│<─────────────────────────────>│
     │                               │                               │
```

**Sync message format** (using Automerge's built-in sync):

```rust
// whiteboard/src/sync.rs

/// Initial sync message sent when a client joins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJoinMessage {
    pub board_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    /// Automerge sync state — tells server what we already have
    pub sync_state: Vec<u8>,
}

/// Server response with missing changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub board_id: Uuid,
    /// Binary Automerge sync message
    pub sync_message: Vec<u8>,
    /// Current server version
    pub version: u64,
    /// Active users on this board
    pub presence: Vec<WhiteboardPresence>,
}

/// Incremental update broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUpdate {
    pub board_id: Uuid,
    /// Binary Automerge change
    pub change: Vec<u8>,
    /// Who made the change
    pub user_id: Uuid,
    /// New version number
    pub version: u64,
    /// Affected element IDs (for client-side optimization)
    pub affected_elements: Vec<ElementId>,
}
```

### 5.5 Undo/Redo Implementation

Automerge doesn't have built-in undo, so we implement it at the application level:

```rust
// whiteboard/src/undo.rs

/// Per-user undo stack (stored in memory, not persisted)
pub struct UndoStack {
    /// Stack of (change_hash, inverted_operations) for undo
    undo: Vec<UndoEntry>,
    /// Stack of (change_hash, operations) for redo
    redo: Vec<UndoEntry>,
    /// Maximum stack depth
    max_depth: usize,
}

struct UndoEntry {
    /// The Automerge change hash this entry corresponds to
    change_hash: ChangeHash,
    /// Human-readable description ("Drew rectangle", "Moved text")
    description: String,
    /// Timestamp for grouping rapid changes (e.g., a single stroke)
    timestamp: Instant,
    /// Whether this can be grouped with the previous entry
    groupable: bool,
}
```

**Undo semantics:**

- **Individual undo**: Each user undoes their own actions. This is the default and most intuitive.
- **Collaborative undo** (optional, Phase 3): Undo any user's action. This is complex because it may conflict with other users' subsequent edits. We implement this as a "revert to version" operation that creates a new CRDT change reverting the target change.

**Grouping logic:**

- A pen stroke (pointer down → move → up) is ONE undo step
- Rapid text edits within 500ms are grouped
- Each shape creation is ONE undo step

---

## 6. WebSocket Protocol Extensions

### 6.1 New Event Types

The existing WebSocket protocol uses typed events. We extend it with whiteboard-specific events:

```rust
// realtime/src/events.rs (additions)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RealtimeEvent {
    // ... existing events ...
    
    // ============================================================
    // Whiteboard Events
    // ============================================================
    
    /// Client wants to join a whiteboard session
    WhiteboardJoin(WhiteboardJoinEvent),
    
    /// Server confirms join + sends initial state
    WhiteboardJoined(WhiteboardJoinedEvent),
    
    /// Client sends CRDT sync message
    WhiteboardSync(WhiteboardSyncEvent),
    
    /// Server broadcasts CRDT update to other clients
    WhiteboardUpdate(WhiteboardUpdateEvent),
    
    /// Client updates their presence (cursor, viewport, tool)
    WhiteboardPresenceUpdate(WhiteboardPresenceUpdateEvent),
    
    /// Server broadcasts presence change
    WhiteboardPresenceChanged(WhiteboardPresenceChangedEvent),
    
    /// User leaves the whiteboard
    WhiteboardLeave(WhiteboardLeaveEvent),
    
    /// Server broadcasts user left
    WhiteboardUserLeft(WhiteboardUserLeftEvent),
    
    /// Client requests undo
    WhiteboardUndo(WhiteboardUndoEvent),
    
    /// Client requests redo
    WhiteboardRedo(WhiteboardRedoEvent),
    
    /// Server broadcasts undo/redo result
    WhiteboardHistoryChanged(WhiteboardHistoryChangedEvent),
    
    /// Client creates a named version snapshot
    WhiteboardCreateVersion(WhiteboardCreateVersionEvent),
    
    /// Server confirms version created
    WhiteboardVersionCreated(WhiteboardVersionCreatedEvent),
    
    /// Error specific to whiteboard operations
    WhiteboardError(WhiteboardErrorEvent),
}

// ============================================================
// Event payloads
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardJoinEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
    /// Automerge sync state from client
    pub sync_state: Option<Vec<u8>>,
    /// Last known version (for delta sync)
    pub last_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardJoinedEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
    /// Full sync message (if client had no state)
    pub sync_message: Option<Vec<u8>>,
    /// Delta changes since client's last version (if provided)
    pub delta: Option<Vec<u8>>,
    /// Current version
    pub version: u64,
    /// All active users on this board
    pub presence: Vec<WhiteboardPresence>,
    /// Board metadata
    pub metadata: WhiteboardMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardSyncEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
    /// Binary Automerge sync message
    pub sync_message: Vec<u8>,
    /// Elements affected by this sync
    pub affected_elements: Vec<ElementId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardUpdateEvent {
    pub board_id: Uuid,
    /// User who made the change
    pub user_id: Uuid,
    /// Binary Automerge change
    pub change: Vec<u8>,
    /// New version number
    pub version: u64,
    /// Affected elements
    pub affected_elements: Vec<ElementId>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardPresenceUpdateEvent {
    pub board_id: Uuid,
    pub cursor: Option<Point>,
    pub tool: Option<Tool>,
    pub viewport: Option<Viewport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardPresenceChangedEvent {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub cursor: Option<Point>,
    pub tool: Option<Tool>,
    pub viewport: Option<Viewport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardLeaveEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardUserLeftEvent {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardUndoEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardRedoEvent {
    pub board_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardHistoryChangedEvent {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub can_undo: bool,
    pub can_redo: bool,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardCreateVersionEvent {
    pub board_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardVersionCreatedEvent {
    pub board_id: Uuid,
    pub version_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardErrorEvent {
    pub board_id: Uuid,
    pub code: WhiteboardErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WhiteboardErrorCode {
    BoardNotFound,
    PermissionDenied,
    InvalidOperation,
    SyncConflict,
    RateLimited,
    SessionExpired,
}
```

### 6.2 Message Flow: Joining a Whiteboard

```
Client                              Server
  │                                   │
  │  WS Connect                       │
  │ ─────────────────────────────────>│
  │                                   │
  │  WhiteboardJoin {                 │
  │    board_id,                      │
  │    session_id,                    │
  │    sync_state: <automerge state>, │
  │    last_version: 42               │
  │  }                                │
  │ ─────────────────────────────────>│
  │                                   │
  │                          ┌────────┴────────┐
  │                          │ 1. Validate     │
  │                          │    permissions  │
  │                          │ 2. Load board   │
  │                          │    from DB      │
  │                          │ 3. Automerge    │
  │                          │    sync with    │
  │                          │    client state │
  │                          │ 4. Get presence │
  │                          │    from Redis   │
  │                          └────────┬────────┘
  │                                   │
  │  WhiteboardJoined {               │
  │    sync_message: <delta since 42>, │
  │    version: 57,                   │
  │    presence: [...],               │
  │    metadata: {...}                │
  │  }                                │
  │ <─────────────────────────────────│
  │                                   │
  │  Apply delta to local Automerge   │
  │  document, render canvas          │
  │                                   │
```

### 6.3 Message Flow: Drawing a Stroke

```
Client A                            Server                    Client B
  │                                   │                          │
  │  User draws pen stroke            │                          │
  │  → local Automerge commit         │                          │
  │  → optimistic render              │                          │
  │                                   │                          │
  │  WhiteboardSync {                 │                          │
  │    sync_message: <automerge msg>  │                          │
  │  }                                │                          │
  │ ─────────────────────────────────>│                          │
  │                                   │                          │
  │                          ┌────────┴────────┐                 │
  │                          │ 1. Apply to     │                 │
  │                          │    server copy  │                 │
  │                          │ 2. Persist to   │                 │
  │                          │    op log       │                 │
  │                          │ 3. Broadcast    │                 │
  │                          └────────┬────────┘                 │
  │                                   │                          │
  │                                   │  WhiteboardUpdate {      │
  │                                   │    change: <binary>,     │
  │                                   │    version: 58           │
  │                                   │  }                       │
  │                                   │ ────────────────────────>│
  │                                   │                          │
  │                                   │                 Apply to │
  │                                   │                 local    │
  │                                   │                 doc +    │
  │                                   │                 render   │
```

### 6.4 Presence Flow

Presence is sent at most every 100ms (throttled) and uses Redis pub/sub:

```
Client A                           Redis                         Client B
  │                                 │                              │
  │  WhiteboardPresenceUpdate {     │                              │
  │    cursor: {x: 500, y: 300},    │                              │
  │    tool: Pen,                   │                              │
  │    viewport: {x: 0, y: 0, z: 1} │                              │
  │  }                              │                              │
  │ ─────────────────────────────── >│                              │
  │                                 │                              │
  │                        PUBLISH wb:presence:{board_id}          │
  │                                 │ ────────────────────────────>│
  │                                 │                              │
  │                                 │                    Update    │
  │                                 │                    cursor    │
  │                                 │                    overlay   │
```

### 6.5 Reconnection Handling

```
Client                              Server
  │                                   │
  │  Connection lost                  │
  │  ──────────────────── X           │
  │                                   │
  │  [Client buffers local changes    │
  │   in Automerge doc during         │
  │   disconnect]                     │
  │                                   │
  │  Reconnect                        │
  │ ─────────────────────────────────>│
  │                                   │
  │  WhiteboardJoin {                 │
  │    sync_state: <full state>,      │
  │    last_version: 58               │
  │  }                                │
  │ ─────────────────────────────────>│
  │                                   │
  │                          ┌────────┴────────┐
  │                          │ 1. Server has   │
  │                          │    version 65   │
  │                          │ 2. Client has   │
  │                          │    version 58   │
  │                          │ 3. Send delta   │
  │                          │    58 → 65      │
  │                          │ 4. Client also  │
  │                          │    has local    │
  │                          │    changes 59-61│
  │                          │ 5. Automerge    │
  │                          │    merges both  │
  │                          └────────┬────────┘
  │                                   │
  │  WhiteboardJoined {               │
  │    sync_message: <delta 58→65>    │
  │    version: 65                    │
  │  }                                │
  │ <─────────────────────────────────│
  │                                   │
  │  Apply delta, merge with local    │
  │  changes, render                  │
  │                                   │
```

**Key insight**: Automerge's sync protocol is designed for this. The client sends its sync state, the server responds with missing changes, and both converge. No data loss, no conflicts.

---

## 7. Storage Strategy

### 7.1 Hybrid: Operation Log + Snapshots

We use a **hybrid approach** combining the best of both worlds:

```
┌─────────────────────────────────────────────────────────┐
│                    Storage Architecture                  │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────┐ │
│  │  Operation   │    │  Snapshot    │    │  Live     │ │
│  │  Log         │    │  (Periodic)  │    │  Cache    │ │
│  │  (Postgres)  │    │  (Postgres)  │    │  (Redis)  │ │
│  │              │    │              │    │           │ │
│  │  Append-only │    │  Every 1000  │    │  Current  │ │
│  │  CRDT changes│    │  ops or 1hr  │    │  doc +    │ │
│  │              │    │              │    │  presence │ │
│  │  Source of   │    │  Fast load   │    │  Fastest  │ │
│  │  truth       │    │  for new     │    │  read     │ │
│  │              │    │  connections │    │           │ │
│  └──────────────┘    └──────────────┘    └───────────┘ │
│                                                          │
│  Write path: Client → Redis buffer → Postgres (batched)  │
│  Read path:  Redis cache → Postgres snapshot → Replay    │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Write Path

```
1. Client sends CRDT change via WebSocket
2. Server applies to in-memory Automerge doc
3. Server appends to Redis operation buffer (LIST)
4. Server broadcasts to other clients via Redis pub/sub
5. Background worker flushes buffer to Postgres every 5 seconds
   (or when buffer reaches 100 ops)
```

**Why buffer writes?**
- Reduces Postgres write pressure by 10-100x
- CRDT changes are small (100-500 bytes each)
- Batching 100 ops into one INSERT is 100x more efficient

### 7.3 Read Path (New Connection)

```
1. Check Redis for cached document state
   → If found and fresh (< 5s old): return immediately
   
2. Load latest snapshot from Postgres
   → Deserialize Automerge document from binary
   
3. Replay operations since snapshot
   → Fetch from operation log WHERE board_id = ? AND id > snapshot_version
   → Apply each operation to the document
   
4. Cache result in Redis with 5s TTL
   
5. Return to client
```

### 7.4 Snapshot Strategy

```rust
// whiteboard/src/document.rs

/// Snapshot configuration
pub struct SnapshotConfig {
    /// Create snapshot every N operations
    pub ops_threshold: usize,      // default: 1000
    /// Create snapshot every N seconds
    pub time_threshold: Duration,  // default: 1 hour
    /// Maximum snapshots to keep per board
    pub max_snapshots: usize,      // default: 10
}

/// When to snapshot:
/// - Every 1000 operations (prevents long replay)
/// - Every 1 hour (time-based safety net)
/// - Before major operations (export, version create)
/// - On graceful shutdown
```

**Snapshot storage**: The snapshot is the full Automerge document in binary form, stored in the `whiteboards.snapshot` column. Typical size: 1-10MB for a board with 1000 elements.

### 7.5 Data Retention

| Data | Retention | Reason |
|---|---|---|
| Operation log | Indefinite | Full history, enables time-travel |
| Snapshots | Last 10 per board | Enough for recovery |
| Sessions | 5 minutes after last activity | Auto-cleanup via TTL |
| Versions | Indefinite | User-created, explicit |
| Deleted boards | 30 days | Soft delete, then hard delete |

### 7.6 Storage Size Estimates

| Scenario | Elements | Op Log | Snapshot | Total/Board |
|---|---|---|---|---|
| Small team, light use | 100 | 50 MB | 2 MB | ~52 MB |
| Medium team, active | 1,000 | 500 MB | 10 MB | ~510 MB |
| Large team, heavy use | 10,000 | 5 GB | 100 MB | ~5.1 GB |

**Mitigation for large boards:**
- **Viewport-based loading**: Only load elements in the current viewport + margin
- **Element compression**: Use `zstd` on the binary document state
- **Archival**: Boards inactive for 90 days are archived to S3

---

## 8. API Endpoints

### 8.1 REST Endpoints

```
# ============================================================
# Whiteboard REST API
# Base: /api/v1/workspaces/{workspace_id}/whiteboards
# ============================================================

# List all whiteboards in a workspace
GET    /api/v1/workspaces/{workspace_id}/whiteboards
       Query: ?page=1&per_page=20&sort=last_activity&order=desc
       Response: { data: WhiteboardSummary[], meta: PaginationMeta }

# Create a new whiteboard
POST   /api/v1/workspaces/{workspace_id}/whiteboards
       Body: { name: "Sprint Planning", description: "..." }
       Response: Whiteboard

# Get whiteboard metadata
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}
       Response: Whiteboard (with metadata, no element data)

# Update whiteboard metadata
PATCH  /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}
       Body: { name: "...", description: "..." }
       Response: Whiteboard

# Delete whiteboard (soft delete)
DELETE /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}
       Response: 204

# Get full board state (for initial load / reconnection)
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/state
       Query: ?version=42 (optional, for delta sync)
       Response: {
         board_id, version, 
         sync_message: <base64>,  // Automerge binary
         delta: <base64>,         // only if version param provided
         elements: Element[],     // decoded elements (for non-CRDT clients)
         presence: WhiteboardPresence[]
       }

# Get elements in a viewport (for lazy loading)
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/elements
       Query: ?x=0&y=0&width=1920&height=1080&zoom=1.0
       Response: { elements: Element[], total: usize }

# Export board as image
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/export
       Query: ?format=png|svg|pdf&x=0&y=0&width=1920&height=1080&zoom=1.0
       Response: Binary file (image/svg/pdf)

# ============================================================
# Version History
# ============================================================

# List versions
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/versions
       Response: { data: WhiteboardVersion[] }

# Create named version
POST   /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/versions
       Body: { name: "Before redesign", description: "..." }
       Response: WhiteboardVersion

# Get version state
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/versions/{version_id}
       Response: { version_id, name, document_state, version, created_at }

# Restore to version
POST   /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/versions/{version_id}/restore
       Response: Whiteboard

# ============================================================
# Permissions
# ============================================================

# List permissions
GET    /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/permissions
       Response: { data: WhiteboardPermission[] }

# Grant permission
POST   /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/permissions
       Body: { grantee_type: "user", grantee_id: "...", permission: "edit" }
       Response: WhiteboardPermission

# Revoke permission
DELETE /api/v1/workspaces/{workspace_id}/whiteboards/{board_id}/permissions/{perm_id}
       Response: 204
```

### 8.2 WebSocket Endpoint

```
# WebSocket connection for real-time sync
WS /api/v1/ws/whiteboard?token={jwt}&board_id={board_id}

# All communication uses the RealtimeEvent types defined in Section 6
```

### 8.3 Request/Response Examples

```json
// GET /api/v1/workspaces/{ws_id}/whiteboards/{board_id}/state
{
  "board_id": "550e8400-e29b-41d4-a716-446655440000",
  "version": 157,
  "sync_message": "base64-encoded-automerge-binary...",
  "elements": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "type": "rectangle",
      "x": 100,
      "y": 200,
      "width": 300,
      "height": 200,
      "fill": "#4f46e5",
      "stroke": "#312e81",
      "stroke_width": 2,
      "corner_radius": 8,
      "opacity": 1.0,
      "rotation": 0,
      "created_by": "user-uuid",
      "created_at": "2026-09-30T10:00:00Z",
      "z_index": 0
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "type": "text",
      "x": 120,
      "y": 220,
      "content": "Sprint Goals",
      "font_family": "Inter",
      "font_size": 24,
      "font_weight": "bold",
      "color": "#ffffff",
      "align": "left",
      "width": 260,
      "created_by": "user-uuid",
      "created_at": "2026-09-30T10:01:00Z",
      "z_index": 1
    }
  ],
  "presence": [
    {
      "user_id": "user-uuid-2",
      "display_name": "Alice Chen",
      "avatar_url": "https://...",
      "color": "#ef4444",
      "cursor": { "x": 450, "y": 320 },
      "tool": "pen",
      "viewport": { "x": 0, "y": 0, "zoom": 1.0 },
      "last_activity": "2026-09-30T10:05:00Z"
    }
  ],
  "metadata": {
    "board_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Sprint Planning",
    "background": "#ffffff",
    "grid_enabled": true,
    "version": 157
  }
}
```

---

## 9. Project Structure Updates

### 9.1 Updated Workspace `Cargo.toml`

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "core",
    "db",
    "auth",
    "realtime",
    "services",
    "api",
    "search",
    "storage",
    "email",
    "whiteboard",    # ← NEW
    "app",
]

[workspace.dependencies]
# ... existing dependencies ...
automerge = "0.5"
automerge-repo = "0.1"
geo = "0.28"
petgraph = "0.6"
bincode = "1.3"
zstd = "0.13"
```

### 9.2 `whiteboard/Cargo.toml`

```toml
[package]
name = "whiteboard"
version = "0.1.0"
edition = "2021"

[dependencies]
# Internal
core = { path = "../core" }
realtime = { path = "../realtime" }

# CRDT
automerge = { workspace = true }
automerge-repo = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
bincode = { workspace = true }

# Async
tokio = { workspace = true }
tokio-tungstenite = "0.21"
futures = "0.3"

# Data structures
dashmap = { workspace = true }
parking_lot = { workspace = true }
geo = { workspace = true }
petgraph = { workspace = true }

# Compression
zstd = { workspace = true }

# IDs
uuid = { workspace = true }

# Time
chrono = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }

# Metrics
metrics = "0.22"

[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
proptest = "1.4"
```

### 9.3 Updated Crate Dependency Graph

```
                    ┌─────────┐
                    │   app   │
                    └────┬────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
     ┌────▼────┐   ┌────▼────┐   ┌────▼────┐
     │   api   │   │realtime │   │ services│
     └────┬────┘   └────┬────┘   └────┬────┘
          │              │              │
          │         ┌────▼────┐         │
          │         │whiteboard│◄────────┘
          │         └────┬────┘
          │              │
     ┌────▼────┐   ┌────▼────┐
     │ storage │   │   db    │
     └─────────┘   └────┬────┘
                        │
                   ┌────▼────┐
                   │  core   │
                   └─────────┘
```

### 9.4 Module Registration in `app/`

```rust
// app/src/main.rs

mod whiteboard;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... existing setup ...
    
    // Initialize whiteboard module
    let whiteboard_service = whiteboard::WhiteboardService::new(
        db_pool.clone(),
        redis_client.clone(),
        realtime_server.clone(),
    ).await?;
    
    // Register whiteboard routes
    let app = api::create_router()
        .nest("/api/v1/workspaces/:workspace_id/whiteboards", 
              whiteboard::routes(whiteboard_service.clone()))
        // ... existing routes ...
    
    // ... start server ...
}
```

---

## 10. Challenges & Solutions

### 10.1 Concurrent Editing Conflicts

| Challenge | Solution |
|---|---|
| Two users edit same element | CRDT last-writer-wins; both clients converge |
| Two users delete same element | Delete wins; element is soft-deleted (recoverable) |
| Z-order conflicts | Automerge List merge; deterministic ordering |
| Simultaneous board creation | Database UNIQUE constraint on (workspace_id, name) |
| Split-brain during partition | Automerge sync protocol handles reconnection automatically |

### 10.2 Large Canvas State Sync

| Challenge | Solution |
|---|---|
| Board with 10,000+ elements | Viewport-based lazy loading; only sync visible elements |
| Initial load too slow | Snapshot + delta sync; don't replay entire history |
| Memory pressure | LRU cache for board documents; evict inactive boards |
| Network bandwidth | Binary Automerge format; zstd compression for large payloads |
| Mobile clients | Send simplified representation; reduce presence frequency |

**Viewport-based sync optimization:**

```rust
// Only sync elements within the client's viewport + 20% margin
pub fn get_viewport_elements(
    doc: &WhiteboardDocument,
    viewport: &Viewport,
    margin: f64,  // 0.2 = 20% margin
) -> Vec<Element> {
    let expanded = viewport.expanded(margin);
    doc.query_elements_in_bbox(expanded.bbox())
}
```

### 10.3 Performance with Many Elements

| Challenge | Solution |
|---|---|
| Rendering 10,000 elements | Spatial index (R-tree) for viewport culling |
| Hit testing | R-tree query instead of linear search |
| Automerge doc size | Periodic compaction (Automerge saves space by merging changes) |
| Search across elements | GIN index on `whiteboard_elements.data` JSONB |
| Export performance | Render only visible viewport; use `resvg` for GPU-accelerated SVG |

**Spatial indexing:**

```rust
// whiteboard/src/spatial.rs

use rstar::RTree;

pub struct SpatialIndex {
    tree: RTree<IndexedElement>,
}

struct IndexedElement {
    bbox: BBox,
    element_id: ElementId,
}

impl SpatialIndex {
    pub fn query_viewport(&self, viewport: &Viewport) -> Vec<ElementId> {
        let bbox = viewport.to_bbox();
        self.tree
            .locate_in_envelope_intersecting(&bbox.to_aabb())
            .map(|e| e.element_id)
            .collect()
    }
    
    pub fn rebuild(&mut self, elements: &[Element]) {
        let indexed: Vec<_> = elements.iter()
            .map(|e| IndexedElement { bbox: e.bbox(), element_id: e.id() })
            .collect();
        self.tree = RTree::bulk_load(indexed);
    }
}
```

### 10.4 Reconnection Handling

| Challenge | Solution |
|---|---|
| Missed changes during disconnect | Automerge sync state exchange on reconnect |
| Duplicate changes | Idempotent CRDT operations; applying same change twice is no-op |
| Stale presence | Redis TTL on session keys; heartbeat every 30s |
| Out-of-order changes | Automerge handles out-of-order delivery natively |
| Large delta after long disconnect | Fall back to full snapshot if delta > 10MB |

### 10.5 Security & Permissions

| Challenge | Solution |
|---|---|
| Unauthorized board access | JWT validation + permission check on every WS message |
| Rate limiting | Redis-based token bucket: 100 ops/sec per user per board |
| Board data isolation | `workspace_id` in every query + RLS policies |
| Malicious payloads | Validate all CRDT changes; size limit per message (1MB) |
| Audit logging | Log all operations to `audit_logs` table |

### 10.6 Scalability

| Challenge | Solution |
|---|---|
| Many boards per workspace | Partition operation log by `board_id` |
| Many concurrent users per board | Redis pub/sub fan-out; no server-side bottleneck |
| Many workspaces | Shard by `workspace_id`; consistent hashing |
| Server restart | Reload from snapshot + replay ops; < 1s recovery |
| Global distribution | Multi-region with CRDT sync between regions |

---

## 11. Phased Implementation Plan

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic whiteboard with single-user drawing and persistence.

| Task | Crate | Est. |
|---|---|---|
| Set up `whiteboard` crate with Automerge | `whiteboard` | 1 day |
| Create DB migration for all tables | `db` | 1 day |
| Implement `WhiteboardDocument` wrapper | `whiteboard` | 2 days |
| Implement basic element types (path, rect, ellipse, text) | `whiteboard` | 2 days |
| REST API: CRUD for whiteboards | `api` | 1 day |
| REST API: Get board state | `api` | 1 day |
| Basic WebSocket: join/leave/sync | `realtime` + `whiteboard` | 2 days |
| Write operation log to Postgres | `db` | 1 day |
| Snapshot creation and loading | `whiteboard` | 1 day |
| **Milestone**: Single user can draw, reload, and see their work | | |

### Phase 2: Real-Time Collaboration (Weeks 3-4)

**Goal**: Multi-user real-time drawing with presence.

| Task | Crate | Est. |
|---|---|---|
| Implement CRDT sync protocol | `whiteboard` | 3 days |
| WebSocket: bidirectional sync | `realtime` | 2 days |
| Presence tracking (cursors, viewports) | `whiteboard` | 2 days |
| Redis pub/sub for fan-out | `realtime` | 1 day |
| Operation buffering and batch writes | `db` | 2 days |
| Reconnection handling | `whiteboard` | 2 days |
| Rate limiting | `api` | 1 day |
| Permission checking | `api` | 1 day |
| **Milestone**: Multiple users can draw simultaneously with live cursors | | |

### Phase 3: Tools & UX (Weeks 5-6)

**Goal**: Full toolset, undo/redo, and polished UX.

| Task | Crate | Est. |
|---|---|---|
| Implement all tools (pen, shapes, text, sticky, eraser) | `whiteboard` | 3 days |
| Undo/redo stack | `whiteboard` | 2 days |
| Laser pointer (non-persistent) | `whiteboard` | 1 day |
| Element selection and manipulation | `whiteboard` | 2 days |
| Spatial indexing for performance | `whiteboard` | 2 days |
| Viewport-based lazy loading | `api` + `whiteboard` | 2 days |
| **Milestone**: Feature-complete whiteboard with good performance | | |

### Phase 4: Export & Versions (Week 7)

**Goal**: Export to images, version history.

| Task | Crate | Est. |
|---|---|---|
| SVG export | `whiteboard` | 2 days |
| PNG export (via resvg/tiny-skia) | `whiteboard` | 2 days |
| PDF export | `whiteboard` | 1 day |
| Version history UI API | `api` | 1 day |
| Restore to version | `api` + `whiteboard` | 1 day |
| **Milestone**: Production-ready whiteboard with export | | |

### Phase 5: Performance & Hardening (Week 8)

**Goal**: Load testing, optimization, edge cases.

| Task | Crate | Est. |
|---|---|---|
| Load testing (100+ concurrent users) | all | 2 days |
| Automerge document compaction | `whiteboard` | 1 day |
| zstd compression for large documents | `whiteboard` | 1 day |
| Monitoring and alerting | `app` | 1 day |
| Security audit | all | 1 day |
| Documentation | all | 1 day |
| **Milestone**: Production deployment | | |

### Timeline Summary

```
Week:  1    2    3    4    5    6    7    8
       ├────┼────┼────┼────┼────┼────┼────┤
Phase 1 ████████
Phase 2      ████████
Phase 3           ████████
Phase 4                ████
Phase 5                     ████
```

---

## 12. Frontend Integration Notes

### 12.1 Recommended Frontend Stack

Since the existing architecture mentions Leptos/Yew for the web client:

| Option | Pros | Cons |
|---|---|---|
| **Leptos + WASM** | Rust end-to-end, shared types | CRDT in WASM is slower; smaller ecosystem |
| **React + TypeScript** | Best ecosystem, Automerge JS is mature | Separate codebase, type duplication |
| **Canvas API + Rust backend** | Best performance for rendering | More complex frontend |

**Recommendation**: Use **React + TypeScript** for the whiteboard frontend, with the Rust backend handling persistence and sync. The whiteboard is a complex enough UI that React's ecosystem (react-konva, fabric.js, excalidraw) provides significant advantages.

### 12.2 Shared Types

Generate TypeScript types from Rust using `ts-rs`:

```rust
// In whiteboard crate
#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct Element {
    // ...
}
```

This generates `.ts` files that the frontend imports, ensuring type safety across the WebSocket boundary.

### 12.3 Rendering Approach

```
┌─────────────────────────────────────────────┐
│              Frontend Architecture           │
│                                              │
│  ┌──────────────┐    ┌──────────────────┐   │
│  │  React UI    │    │  Canvas Renderer │   │
│  │  (toolbar,   │    │  (HTML5 Canvas   │   │
│  │   panels)    │    │   or WebGL)      │   │
│  └──────┬───────┘    └────────┬─────────┘   │
│         │                     │              │
│  ┌──────┴─────────────────────┴──────────┐   │
│  │         Automerge Document             │   │
│  │  (source of truth for element state)   │   │
│  └────────────────┬───────────────────────┘   │
│                   │                            │
│  ┌────────────────┴───────────────────────┐   │
│  │         WebSocket Sync Manager         │   │
│  │  (sends/receives CRDT changes)         │   │
│  └────────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 12.4 Optimistic Rendering

```typescript
// Pseudocode for optimistic local rendering
function onPenStroke(points: Point[]) {
  // 1. Immediately render locally (optimistic)
  const elementId = uuidv7();
  canvasRenderer.drawPath(elementId, points);
  
  // 2. Apply to local Automerge doc
  automerge.change(doc => {
    doc.elements[elementId] = createPathElement(points);
  });
  
  // 3. Sync to server (async)
  syncManager.sendSyncMessage(automerge.getSyncMessage());
  
  // 4. Server will broadcast to other clients
  // 5. When we receive our own change back, it's a no-op (idempotent)
}
```

---

## Appendix A: Complete Crate File Listing

```
whiteboard/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API, re-exports
│   ├── document.rs               # WhiteboardDocument (Automerge wrapper)
│   ├── element.rs                # Element enum + all element types
│   ├── tool.rs                   # Tool enum + tool configuration
│   ├── sync.rs                   # SyncJoinMessage, SyncResponse, SyncUpdate
│   ├── presence.rs               # WhiteboardPresence, presence management
│   ├── undo.rs                   # UndoStack, UndoEntry
│   ├── export.rs                 # SVG/PNG/PDF export
│   ├── spatial.rs                # R-tree spatial index
│   ├── metrics.rs                # Prometheus metrics
│   └── error.rs                  # WhiteboardError types
├── tests/
│   ├── crdt_concurrent.rs        # Property-based concurrent edit tests
│   ├── sync_reconnect.rs         # Reconnection and delta sync tests
│   ├── undo_redo.rs              # Undo/redo correctness tests
│   ├── export.rs                 # Export format tests
│   └── benchmarks.rs             # Criterion benchmarks
└── examples/
    └── demo.rs                   # CLI demo of CRDT operations
```

## Appendix B: Migration File

```sql
-- migrations/0018_whiteboard.sql
-- (Full SQL from Section 4.1 above)
```

## Appendix C: Environment Variables

```bash
# Whiteboard-specific configuration
WHITEBOARD_SNAPSHOT_OPS_THRESHOLD=1000
WHITEBOARD_SNAPSHOT_TIME_THRESHOLD=3600
WHITEBOARD_MAX_SNAPSHOTS=10
WHITEBOARD_OP_BUFFER_SIZE=100
WHITEBOARD_OP_FLUSH_INTERVAL_MS=5000
WHITEBOARD_PRESENCE_HEARTBEAT_INTERVAL_SECS=30
WHITEBOARD_RATE_LIMIT_PER_SEC=100
WHITEBOARD_MAX_MESSAGE_SIZE_BYTES=1048576
WHITEBOARD_COMPRESSION_ENABLED=true
WHITEBOARD_COMPRESSION_LEVEL=3
```

---

*End of Whiteboard Design Document*
