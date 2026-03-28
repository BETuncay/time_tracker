# Time Tracker — Agent Notes

## Goal

Build a desktop time tracking app to replace an Excel sheet. The user logs work sessions by clicking named task buttons, adds descriptions, and can review or manually add past entries.

## Stack

- **Language:** Rust
- **GUI framework:** iced (Elm architecture, GPU-accelerated via wgpu)
- **Storage:** SQLite via `rusqlite` (simple, single-file, no server)
- **Build:** Cargo, single binary output

## Design Principles

- Simple in the Casey Muratori sense: no unnecessary abstraction, direct data flow, explicit state
- Modern, minimalistic UI — clean typography, tight layout, intentional color use
- Small binary, fast startup, low idle CPU

## Core Features (in order of implementation)

1. **Task buttons** — a configurable grid of named tasks; clicking one starts a timer
2. **Active timer display** — shows current task name and elapsed time
3. **Stop / switch** — stopping saves the entry; clicking a new task auto-stops the current one
4. **Entry log** — list of today's entries (task, description, start, end, duration)
5. **Manual entry** — add a past time block with task, description, start, and end
6. **Edit entry** — modify description or times on any logged entry
7. **Task management** — add, rename, remove task buttons
8. **Basic reporting** — daily and weekly totals per task

## Data Model (initial)

```
entries
  id          INTEGER PRIMARY KEY
  task        TEXT NOT NULL
  description TEXT
  started_at  INTEGER NOT NULL  -- unix timestamp
  ended_at    INTEGER           -- null if still running
```

## App Architecture (iced Elm model)

```
Message
  StartTask(task_name)
  StopCurrent
  Tick(Instant)
  EditDescription(id, text)
  AddManualEntry { task, description, start, end }
  DeleteEntry(id)
  ...

Model
  tasks: Vec<String>
  active: Option<ActiveTimer>
  entries: Vec<Entry>
  view_state: ViewState  -- which panel is shown
```

## Working Approach

- Build incrementally: get a running timer working first, then persistence, then full UI polish
- Keep all state in one flat `Model` struct — no nested state machines unless clearly needed
- Persist to SQLite on every write operation (no in-memory cache that can diverge)
- UI theming done once after core logic works
- Each feature gets its own commit when complete

## Project Tracking

Two files are used to track work across sessions:

- `TODO.md` — current tasks and in-progress work. Treated as the active sprint. Each item has a status: `[ ]` open, `[~]` in progress, `[x]` done.
- `IDEAS.md` — backlog of future ideas, nice-to-haves, and things to revisit. Not prioritized, just captured so nothing is lost.

New ideas that come up during work go into `IDEAS.md`, not `TODO.md`, to avoid scope creep.

## Agent Work Loop

Each session follows this exact sequence:

1. **Read** `TODO.md` — identify the next open `[ ]` item
2. **Mark it** `[~]` in progress
3. **Implement** the feature or task
4. **Mark it** `[x]` done in `TODO.md`
5. **Commit** all changes (source + updated `TODO.md`) with a clear message describing what was done
6. **Repeat** from step 1 until told to stop or no open items remain

If a new idea surfaces during implementation, append it to `IDEAS.md` before continuing — do not act on it immediately.
