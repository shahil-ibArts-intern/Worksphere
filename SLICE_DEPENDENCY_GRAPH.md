# Slice Dependency Graph — Build Order & Parallelization

**Status**: Active  
**Last Updated**: 2026-09-30  
**Related**: `VERTICAL_SLICES.md`, `ARCHITECTURE_GUARDRAIL.md`

---

## 1. How to Use This Document

This document answers three questions for any agent:

1. **What must be built before my slice?** — See "Depends On" column
2. **What can be built in parallel with my slice?** — See "Parallel Group"
3. **What is the overall build order?** — See the wave diagrams below

**Rule**: You may only start work on a slice when ALL of its dependencies are marked complete in `VERTICAL_SLICES.md`.

---

## 2. Complete Dependency Matrix

| Slice | Name | Depends On | Parallel Group | Wave |
|---|---|---|---|---|
| VS-01 | Workspace Scaffolding | — | A | 1 |
| VS-02 | Database Foundation | VS-01 | B | 1 |
| VS-03 | Core Types & Traits | VS-01 | B | 1 |
| VS-04 | Authentication | VS-02, VS-03 | C | 1 |
| VS-05 | User Management | VS-04 | D | 1 |
| VS-06 | Organization Management | VS-04, VS-05 | D | 1 |
| VS-07 | WebSocket Infrastructure | VS-04 | E | 2 |
| VS-08 | Channels | VS-06, VS-07 | F | 2 |
| VS-09 | Messages | VS-08 | G | 2 |
| VS-10 | Presence | VS-07 | F | 2 |
| VS-11 | Reactions | VS-09 | H | 2 |
| VS-12 | Typing Indicators | VS-09 | H | 2 |
| VS-13 | Message Threads | VS-09 | H | 2 |
| VS-14 | Direct Messages | VS-08 | H | 2 |
| VS-15 | Pinned Messages | VS-09 | H | 2 |
| VS-16 | Message Edit/Delete History | VS-09 | H | 2 |
| VS-17 | File Attachments | VS-09 | I | 3 |
| VS-18 | Rich Text / Markdown | VS-09 | I | 3 |
| VS-19 | @Mentions | VS-09 | I | 3 |
| VS-20 | Boards | VS-06 | J | 4 |
| VS-21 | Tasks | VS-20 | K | 4 |
| VS-22 | Task Comments | VS-21 | L | 4 |
| VS-23 | Task Labels | VS-21 | L | 4 |
| VS-24 | Task Due Dates | VS-21 | L | 4 |
| VS-25 | Board Views | VS-20 | K | 4 |
| VS-26 | Task Dependencies | VS-21 | L | 4 |
| VS-27 | Notification Center | VS-09, VS-11 | M | 5 |
| VS-28 | Push Notifications | VS-27 | N | 5 |
| VS-29 | Email Digests | VS-27 | N | 5 |
| VS-30 | Do-Not-Disturb | VS-27 | N | 5 |
| VS-31 | Full-Text Search | VS-09 | O | 6 |
| VS-32 | Channel Browser | VS-08 | O | 6 |
| VS-33 | User Directory | VS-05 | O | 6 |
| VS-34 | Incoming Webhooks | VS-09 | P | 7 |
| VS-35 | Slash Commands | VS-09 | P | 7 |
| VS-36 | Bot Framework | VS-34 | Q | 7 |
| VS-37 | Whiteboard Foundation | VS-07, VS-06 | R | 8 |
| VS-38 | Whiteboard Real-Time Collab | VS-37 | S | 8 |
| VS-39 | Whiteboard Tools & UX | VS-38 | T | 8 |
| VS-40 | Whiteboard Export & Versions | VS-39 | U | 8 |
| VS-41 | Whiteboard Performance | VS-40 | V | 8 |
| VS-42 | Rate Limiting | VS-09 | W | 9 |
| VS-43 | Audit Logging | VS-09 | W | 9 |
| VS-44 | Observability | VS-01 | W | 9 |
| VS-45 | CI/CD & Deployment | VS-44 | X | 9 |

---

## 3. Wave-by-Wave Build Order

### Wave 1: Foundation (Sequential — 2 weeks)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        WAVE 1: FOUNDATION                          │
│                                                                     │
│  Week 1                                                             │
│  ┌──────────┐                                                       │
│  │  VS-01   │ Workspace Scaffolding                                 │
│  └────┬─────┘                                                       │
│       │                                                             │
│       ├──────────────────┐                                          │
│       │                  │                                          │
│  ┌────▼─────┐      ┌─────▼────┐                                    │
│  │  VS-02   │      │  VS-03   │  ← PARALLEL GROUP B               │
│  │ Database │      │  Core    │                                    │
│  └────┬─────┘      └────┬─────┘                                    │
│       │                  │                                          │
│       └────────┬─────────┘                                          │
│                │                                                    │
│  ┌─────────────▼──────────┐                                        │
│  │         VS-04          │  ← PARALLEL GROUP C                   │
│  │    Authentication       │                                        │
│  └─────────────┬──────────┘                                        │
│                │                                                    │
│       ┌────────┴────────┐                                          │
│       │                 │                                          │
│  ┌────▼─────┐     ┌────▼─────┐                                    │
│  │  VS-05   │     │  VS-06   │  ← PARALLEL GROUP D               │
│  │  Users   │     │   Orgs   │                                    │
│  └──────────┘     └──────────┘                                    │
│                                                                     │
│  Week 2 (continued)                                                 │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-01 → (VS-02 ∥ VS-03) → VS-04 → (VS-05 ∥ VS-06)

---

### Wave 2: Communication (2 weeks)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WAVE 2: COMMUNICATION                          │
│                                                                     │
│  Week 3                                                             │
│  ┌──────────┐                                                       │
│  │  VS-07   │ WebSocket Infrastructure                              │
│  └────┬─────┘                                                       │
│       │                                                             │
│       ├──────────────────┐                                          │
│       │                  │                                          │
│  ┌────▼─────┐      ┌─────▼────┐                                    │
│  │  VS-08   │      │  VS-10   │  ← PARALLEL GROUP F               │
│  │ Channels │      │ Presence │                                    │
│  └────┬─────┘      └──────────┘                                    │
│       │                                                             │
│  ┌────▼─────┐                                                       │
│  │  VS-09   │ Messages                                              │
│  └────┬─────┘                                                       │
│       │                                                             │
│  Week 4                                                             │
│       ├────┬────┬────┬────┬────┐                                   │
│       │    │    │    │    │    │                                   │
│  ┌────▼┐┌──▼──┐┌▼───┐┌▼───┐┌▼───┐                                │
│  │VS-11││VS-12││VS-13││VS-14││VS-15│  ← PARALLEL GROUP H         │
│  │React││Typng││Thrds││ DMs ││Pins │                              │
│  └─────┘└─────┘└─────┘└─────┘└─────┘                                │
│       │                                                             │
│  ┌────▼─────┐                                                       │
│  │  VS-16   │ Message Edit/Delete History                           │
│  └──────────┘                                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-07 → (VS-08 ∥ VS-10) → VS-09 → (VS-11 ∥ VS-12 ∥ VS-13 ∥ VS-14 ∥ VS-15 ∥ VS-16)

---

### Wave 3: Collaboration (1 week — fully parallel)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WAVE 3: COLLABORATION                          │
│                                                                     │
│  Week 5                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │  VS-17   │  │  VS-18   │  │  VS-19   │  ← PARALLEL GROUP I     │
│  │  Files   │  │ RichText │  │ @Mentions│                          │
│  └──────────┘  └──────────┘  └──────────┘                          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-17 ∥ VS-18 ∥ VS-19 (all parallel)

---

### Wave 4: Task Management (2 weeks)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    WAVE 4: TASK MANAGEMENT                          │
│                                                                     │
│  Week 5                                                             │
│  ┌──────────┐                                                       │
│  │  VS-20   │ Boards                                                │
│  └────┬─────┘                                                       │
│       │                                                             │
│       ├──────────────────┐                                          │
│       │                  │                                          │
│  ┌────▼─────┐      ┌─────▼────┐                                    │
│  │  VS-21   │      │  VS-25   │  ← PARALLEL GROUP K               │
│  │  Tasks   │      │  Views   │                                    │
│  └────┬─────┘      └──────────┘                                    │
│       │                                                             │
│  Week 6                                                             │
│       ├────┬────┬────┐                                              │
│       │    │    │    │                                              │
│  ┌────▼┐┌──▼──┐┌▼───┐┌▼───┐                                       │
│  │VS-22││VS-23││VS-24││VS-26│  ← PARALLEL GROUP L                │
│  │Comm ││Label││Due  ││Deps │                                     │
│  └─────┘└─────┘└─────┘└─────┘                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-20 → (VS-21 ∥ VS-25) → (VS-22 ∥ VS-23 ∥ VS-24 ∥ VS-26)

---

### Wave 5: Notifications (1 week)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WAVE 5: NOTIFICATIONS                          │
│                                                                     │
│  Week 6                                                             │
│  ┌──────────┐                                                       │
│  │  VS-27   │ Notification Center                                   │
│  └────┬─────┘                                                       │
│       │                                                             │
│       ├────┬────┬────┐                                              │
│       │    │    │    │                                              │
│  ┌────▼┐┌──▼──┐┌▼───┐┌▼───┐                                       │
│  │VS-28││VS-29││VS-30│     │  ← PARALLEL GROUP N                 │
│  │Push ││Email││ DND │     │                                      │
│  └─────┘└─────┘└─────┘     │                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-27 → (VS-28 ∥ VS-29 ∥ VS-30)

---

### Wave 6: Discovery (1 week — fully parallel)

```
┌─────────────────────────────────────────────────────────────────────┐
│                       WAVE 6: DISCOVERY                             │
│                                                                     │
│  Week 7                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │  VS-31   │  │  VS-32   │  │  VS-33   │  ← PARALLEL GROUP O     │
│  │  Search  │  │ Channel  │  │  User    │                          │
│  │          │  │ Browser  │  │Directory │                          │
│  └──────────┘  └──────────┘  └──────────┘                          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-31 ∥ VS-32 ∥ VS-33 (all parallel)

---

### Wave 7: Integrations (1 week)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WAVE 7: INTEGRATIONS                           │
│                                                                     │
│  Week 7                                                             │
│  ┌──────────┐  ┌──────────┐                                        │
│  │  VS-34   │  │  VS-35   │  ← PARALLEL GROUP P                   │
│  │ Webhooks │  │  Slash   │                                        │
│  └────┬─────┘  │ Commands │                                        │
│       │        └──────────┘                                        │
│  ┌────▼─────┐                                                       │
│  │  VS-36   │ Bot Framework                                         │
│  └──────────┘                                                       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: (VS-34 ∥ VS-35) → VS-36

---

### Wave 8: Whiteboard (4 weeks — sequential)

```
┌─────────────────────────────────────────────────────────────────────┐
│                       WAVE 8: WHITEBOARD                            │
│                                                                     │
│  Week 7:  [VS-37] Whiteboard Foundation                             │
│              │                                                      │
│  Week 8:  [VS-38] Real-Time Collaboration                           │
│              │                                                      │
│  Week 9:  [VS-39] Tools & UX                                        │
│              │                                                      │
│  Week 10: [VS-40] Export & Versions                                 │
│              │                                                      │
│           [VS-41] Performance & Hardening                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: VS-37 → VS-38 → VS-39 → VS-40 → VS-41 (strictly sequential)

---

### Wave 9: Production (2 weeks)

```
┌─────────────────────────────────────────────────────────────────────┐
│                       WAVE 9: PRODUCTION                            │
│                                                                     │
│  Week 10                                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │  VS-42   │  │  VS-43   │  │  VS-44   │  ← PARALLEL GROUP W     │
│  │  Rate    │  │  Audit   │  │Observabil│                          │
│  │  Limit   │  │  Logging │  │   ity    │                          │
│  └──────────┘  └──────────┘  └────┬─────┘                          │
│                                   │                                  │
│  Week 11                          │                                  │
│                            ┌──────▼─────┐                           │
│                            │   VS-45    │                           │
│                            │  CI/CD &   │                           │
│                            │ Deployment │                           │
│                            └────────────┘                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Build Order**: (VS-42 ∥ VS-43 ∥ VS-44) → VS-45

---

## 4. Cross-Wave Parallelization Opportunities

With multiple agents, waves can overlap:

```
Week:     1    2    3    4    5    6    7    8    9   10   11
         ├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤
Wave 1:  ████████
Wave 2:            ████████
Wave 3:                      ████
Wave 4:                      ████████
Wave 5:                           ████
Wave 6:                                ████
Wave 7:                                ████
Wave 8:                           ████████████████████████
Wave 9:                                          ████████

         ↑                                ↑
    1 agent sequential              3 agents parallel
    (~11 weeks)                      (~6 weeks)
```

### Optimal 3-Agent Schedule

```
Agent A (Backend Core):  VS-01 → VS-02 → VS-04 → VS-06 → VS-08 → VS-09 → VS-11 → VS-13 → VS-14 → VS-15 → VS-16 → VS-17 → VS-18 → VS-19 → VS-20 → VS-21 → VS-22 → VS-23 → VS-24 → VS-25 → VS-26 → VS-27 → VS-28 → VS-29 → VS-30 → VS-31 → VS-32 → VS-33 → VS-34 → VS-35 → VS-36 → VS-42 → VS-43 → VS-44 → VS-45

Agent B (Real-Time):     VS-01 → VS-03 → VS-04 → VS-07 → VS-10 → VS-37 → VS-38 → VS-39 → VS-40 → VS-41

Agent C (Infrastructure): VS-01 → VS-05 → VS-06 → VS-12 → VS-44 → VS-45
```

---

## 5. Critical Path Analysis

The **critical path** (longest chain of dependencies) determines minimum project duration:

```
VS-01 → VS-02 → VS-04 → VS-06 → VS-08 → VS-09 → VS-11 → VS-27 → VS-28
  2  +  3   +  4   +  4   +  3   +  4   +  2   +  3   +  3   = 28 days

Alternative critical path (whiteboard):
VS-01 → VS-02 → VS-04 → VS-07 → VS-37 → VS-38 → VS-39 → VS-40 → VS-41
  2  +  3   +  4   +  4   +  5   +  5   +  5   +  4   +  4   = 36 days
```

**The whiteboard path is the critical path** at ~36 person-days. This means:
- Whiteboard work should start as early as possible (VS-37 can start after VS-07 + VS-06)
- Delays in whiteboard slices delay the entire project
- Consider assigning a dedicated agent to whiteboard work

---

## 6. Dependency Rules for Agents

### Rule 1: Check Dependencies Before Starting
Before writing any code, verify that all slices in the "Depends On" column are marked complete in `VERTICAL_SLICES.md`.

### Rule 2: No Forward References
Do not write code that depends on a slice that hasn't been built yet. If you need a type or interface from a future slice, define a stub in `core` and implement it when the dependent slice is built.

### Rule 3: Parallel Slices Must Not Conflict
If two slices are in the same parallel group, coordinate to avoid:
- Modifying the same file simultaneously
- Creating conflicting migrations (use different migration numbers)
- Defining conflicting route paths

### Rule 4: Migration Numbering
When creating database migrations in parallel:
- Odd-numbered migrations (0001, 0003, ...) for Agent A
- Even-numbered migrations (0002, 0004, ...) for Agent B
- Or better: coordinate and assign unique numbers before starting

### Rule 5: Shared Type Changes
If you need to modify a type in `core` that another parallel agent is also using:
- Add new fields (don't remove or rename)
- Use `Option<T>` for new fields to maintain backwards compatibility
- Coordinate with the other agent before breaking changes

---

## 7. Visual Summary: All Slices at a Glance

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           ALL 45 VERTICAL SLICES                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  WAVE 1: FOUNDATION (6 slices)                                               │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                          │
│  │VS-01│ │VS-02│ │VS-03│ │VS-04│ │VS-05│ │VS-06│                          │
│  │Scaff│ │ DB  │ │Core │ │Auth │ │Users│ │Orgs │                          │
│  └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘                          │
│     │       │       │       │       │       │                                │
│  WAVE 2: COMMUNICATION (10 slices)                                          │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │
│  │VS-07│ │VS-08│ │VS-09│ │VS-10│ │VS-11│ │VS-12│ │VS-13│ │VS-14│ │VS-15│  │
│  │ WS  │ │Chan │ │Msg  │ │Pres │ │React│ │Typng│ │Thrds│ │ DMs │ │Pins │  │
│  └──┬──┘ └──┬──┘ └──┬──┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │
│     │       │       │       ┌─────┐                                         │
│     │       │       │       │VS-16│                                         │
│     │       │       │       │Hist │                                         │
│     │       │       │       └─────┘                                         │
│     │       │       │                                                       │
│  WAVE 3: COLLABORATION (3 slices)                                           │
│     │       │       ├─────┬─────┬─────┐                                    │
│     │       │       │VS-17│VS-18│VS-19│                                    │
│     │       │       │Files│RichT│@Ment│                                    │
│     │       │       └─────┴─────┴─────┘                                    │
│     │       │                                                               │
│  WAVE 4: TASK MANAGEMENT (7 slices)                                         │
│     │       ├─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐      │
│     │       │VS-20│ │VS-21│ │VS-22│ │VS-23│ │VS-24│ │VS-25│ │VS-26│      │
│     │       │Board│ │Task │ │TCom │ │Label│ │Due  │ │View │ │Deps │      │
│     │       └─────┘ └──┬──┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘      │
│     │                  │                                                    │
│  WAVE 5: NOTIFICATIONS (4 slices)                                           │
│     │                  ├─────┐ ┌─────┐ ┌─────┐ ┌─────┐                    │
│     │                  │VS-27│ │VS-28│ │VS-29│ │VS-30│                    │
│     │                  │Notif│ │Push │ │Email│ │ DND │                    │
│     │                  └─────┘ └─────┘ └─────┘ └─────┘                    │
│     │                                                                      │
│  WAVE 6: DISCOVERY (3 slices)                                               │
│     │                  ├─────┬─────┬─────┐                                │
│     │                  │VS-31│VS-32│VS-33│                                │
│     │                  │Srch ││Brws ││Dir  │                                │
│     │                  └─────┴─────┴─────┘                                │
│     │                                                                      │
│  WAVE 7: INTEGRATIONS (3 slices)                                            │
│     │                  ├─────┬─────┐ ┌─────┐                              │
│     │                  │VS-34│VS-35│ │VS-36│                              │
│     │                  │Webhk││Slash│ │Bots │                              │
│     │                  └─────┴─────┘ └─────┘                              │
│     │                                                                      │
│  WAVE 8: WHITEBOARD (5 slices)                                              │
│     ├─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                       │
│     │VS-37│ │VS-38│ │VS-39│ │VS-40│ │VS-41│                              │
│     │WBFnd│ │WBRT │ │WBTls│ │WBExp│ │WBPerf                              │
│     └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                              │
│                                                                              │
│  WAVE 9: PRODUCTION (4 slices)                                              │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                                          │
│  │VS-42│ │VS-43│ │VS-44│ │VS-45│                                          │
│  │Rate │ │Audit│ │Obsv │ │CICD │                                          │
│  └─────┘ └─────┘ └─────┘ └─────┘                                          │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Quick Reference: "Can I Start My Slice?"

Use this decision tree:

```
START: What is your slice ID?
  │
  ├─ VS-01 through VS-06 → Wave 1
  │   └─ Are ALL dependencies complete? → YES: Start work
  │                                      → NO:  Wait for dependencies
  │
  ├─ VS-07 through VS-16 → Wave 2
  │   └─ Is VS-04 complete? (auth)
  │       └─ YES: Is VS-06 complete? (orgs, for channels)
  │           └─ YES: Is VS-07 complete? (WS, for channels)
  │               └─ YES: Is VS-08 complete? (channels, for messages)
  │                   └─ YES: Start work
  │                   → NO:  Wait for VS-08
  │
  ├─ VS-17 through VS-19 → Wave 3
  │   └─ Is VS-09 complete? (messages) → YES: Start work
  │
  ├─ VS-20 through VS-26 → Wave 4
  │   └─ Is VS-06 complete? (orgs) → YES: Start work on VS-20
  │       └─ Is VS-20 complete? (boards) → YES: Start work on VS-21, VS-25
  │           └─ Is VS-21 complete? (tasks) → YES: Start work on VS-22-26
  │
  ├─ VS-27 through VS-30 → Wave 5
  │   └─ Are VS-09 AND VS-11 complete? → YES: Start work
  │
  ├─ VS-31 through VS-33 → Wave 6
  │   └─ Is the relevant dependency complete? → YES: Start work
  │
  ├─ VS-34 through VS-36 → Wave 7
  │   └─ Is VS-09 complete? (messages) → YES: Start work
  │
  ├─ VS-37 through VS-41 → Wave 8 (Whiteboard)
  │   └─ Are VS-07 AND VS-06 complete? → YES: Start VS-37
  │       └─ Is previous whiteboard slice complete? → YES: Start next
  │
  └─ VS-42 through VS-45 → Wave 9
      └─ Is the relevant dependency complete? → YES: Start work
```

---

*End of Slice Dependency Graph*
