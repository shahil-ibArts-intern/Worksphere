# Slack-Type Workspace Application — Rust Architecture & Implementation Plan

## 1. Missing Features Analysis

### Critical Missing Features

| Category | Missing Feature | Why It's Essential |
|---|---|---|
| **Communication** | Channels (public/private) | The backbone of org communication |
| | Direct Messages (1:1 & group) | Private conversations outside channels |
| | Message Threads | Organized replies without cluttering main channel |
| | Emoji Reactions | Lightweight acknowledgment, core to chat UX |
| | @Mentions | Notify specific users, drive engagement |
| | Typing Indicators | Real-time "user is typing..." feedback |
| | Message Edit/Delete | Content moderation and correction |
| | Pinned Messages | Surface important info at channel top |
| **Notifications** | Notification Center | Unified inbox for mentions, DMs, assignments |
| | Push Notifications | Mobile/desktop push for offline users |
| | Email Digests | Fallback for users who miss real-time |
| | Do-Not-Disturb / Snooze | User control over notification fatigue |
| **Collaboration** | File Sharing / Attachments | Images, documents, code snippets in messages |
| | Rich Text / Markdown | Code blocks, bold, italic, lists, links |
| **Organization** | User Roles & Permissions | Owner, Admin, Member, Guest |
| | Workspace Settings | Branding, retention policies, auth policies |
| | Invite Links | Shareable URLs for onboarding |
| | Guest Accounts | External collaborators with limited access |
| **Discovery** | Full-Text Search | Find messages, files, people, channels |
| | Channel Browser | Discover and join public channels |
| | User Directory | Find colleagues by name, role, department |
| **Presence** | Online/Away/Offline Status | See who's available in real-time |
| | Custom Status | "In a meeting", "On vacation" |
| **Integrations** | Incoming Webhooks | Post from external services |
| | Slash Commands | `/remind`, `/poll`, custom commands |
| | Bot Framework | Build custom workspace bots |
| **Reliability** | Read Receipts | Know when messages are seen |
| | Message Delivery Guarantees | At-least-once delivery with idempotency |
| | Multi-Device Sync | Seamless experience across devices |
| | Audit Logging | Compliance, security forensics |
| **Task Management** | Task Comments | Discussion on tasks |
| | Task Labels/Tags | Categorization |
| | Task Due Dates | Deadline tracking |
| | Task Activity Log | Audit trail of changes |
| | Board Views (Kanban/List/Calendar) | Multiple visualization modes |
| | Task Dependencies | Blocked-by / blocks relationships |

## 2. High-Level System Architecture

See the full architecture diagram in the conversation above. Key components:

- **Client Layer**: Web (Leptos/Yew), Desktop (Tauri), Mobile
- **API Layer**: Axum (REST + WebSocket)
- **Service Layer**: Domain logic (Auth, Org, Board, Task, Channel, Message, Notification, Search, File, Webhook)
- **Repository Layer**: SQLx-based data access
- **Data Layer**: PostgreSQL, Redis, Meilisearch, S3/MinIO

## 3. Rust Crate Recommendations

| Component | Crate | Version |
|---|---|---|
| Web Framework | axum | 0.7 |
| Async Runtime | tokio | 1.x |
| Database | sqlx (postgres) | 0.7 |
| Redis | fred | 8.x |
| Serialization | serde + serde_json | 1.x |
| Auth | jsonwebtoken + argon2 | 9.x / 0.5 |
| Validation | validator | 0.16 |
| Error Handling | thiserror + anyhow | 1.x |
| Logging | tracing + tracing-subscriber | 0.1 |
| Config | config + dotenvy | 0.14 |
| Time | chrono | 0.4 |
| IDs | uuid (v7) | 1.x |
| Email | lettre | 0.11 |
| HTTP Client | reqwest | 0.11 |
| Search | meilisearch-sdk | 0.27 |
| Rate Limiting | governor | 0.6 |
| Concurrency | dashmap + parking_lot | 0.5 / 0.12 |

## 4. Core Data Models

Full SQL schema with 17+ tables including:
- users, organizations, org_members, invitations
- boards, board_members, tasks, task_assignees
- channels, channel_members, messages, reactions
- attachments, notifications, pins, audit_logs, sessions

## 5. API Design

REST API with 60+ endpoints across:
- Auth, Organizations, Boards, Tasks, Channels, Messages
- Files, Notifications, Search, DMs, Webhooks

WebSocket protocol with typed events for:
- message.new, message.edited, message.deleted
- reaction.added, reaction.removed
- typing, presence.changed
- notification.new
- task.created, task.updated, task.assigned
- channel.created, channel.member_joined

## 6. Project Structure

Workspace with 10 crates:
- core (domain types, traits, errors)
- db (repositories, queries)
- auth (JWT, permissions, OAuth)
- realtime (WebSocket server, pub/sub, presence)
- services (business logic)
- api (HTTP handlers, middleware, DTOs)
- search (Meilisearch integration)
- storage (S3/MinIO file handling)
- email (SMTP client)
- app (binary entry point)

## 7. Challenges & Solutions

1. WebSocket scaling → Redis pub/sub fan-out
2. Message ordering → UUID v7 + cursor pagination + idempotency
3. Permission checking → Redis caching + batch checks + RLS
4. Presence tracking → Heartbeat + Redis SET with TTL
5. Search performance → Meilisearch + async indexing
6. File uploads → Streaming + pre-signed S3 URLs
7. Connection pool exhaustion → PgBouncer + read replicas
8. Multi-tenant isolation → org_id in every query + RLS
9. Graceful shutdown → Drain connections + flush messages
10. Frontend sync → Event sourcing + optimistic updates

## Implementation Roadmap

- Phase 1 (Weeks 1-2): Foundation — scaffolding, DB, auth, users, orgs
- Phase 2 (Weeks 3-4): Communication — channels, messages, WebSocket, presence
- Phase 3 (Weeks 5-6): Task Management — boards, tasks, kanban, comments
- Phase 4 (Weeks 7-8): Polish — notifications, files, search, rate limiting, email
- Phase 5 (Weeks 9-10): Production — observability, load testing, security, CI/CD
