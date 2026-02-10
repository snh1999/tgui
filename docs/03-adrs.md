# 3. Architecture Decision Records (ADRs)

### Update Log

- **08-12-2025**: Initial Decision Record (ADR-001 to ADR-006)
- **09-12-2025**: Decided on Tauri v2 because of better support, accepting potential compile time
  trade-off (ADR-001 update)
- **01-02-2026**: Updated decision to use shadcn-vue and veevalidate form (ADR-002 extension)
- **08-02-2026**: Adds Decision Record (ADR-007) and updates (ADR-004, ADR-005)

## ADR-001: Framework Choice - Tauri v2

**Updated**: 2025-12-09

### Context

Need a cross-platform desktop framework for TGUI with process spawning, system tray support and low
memory footprint.

**Requirements**:

- Cross-platform (Linux priority, Windows, macOS)
- Small binary size (<15MB)
- Native process spawning
- System tray support
- Good Vue.js integration

### Decision

**Chosen**: Tauri v2 with Vue 3 frontend

### Justification

1. **Binary size**: <10-20MB AppImage (meets NFR-06)
2. **Documentation**: Better official docs with Vue examples
3. **Process spawning**: Mature Rust ecosystem (tokio, nix crate)
4. **System tray**: Official plugin (`tauri-plugin-tray`)
5. **Security**: Rust's memory safety prevents common bugs
6. **Community**: 80k+ stars, actively maintained, corporate backing
7. **Vue support**: Official template with TypeScript

### Rejected Alternative:

- **Wails**:
    - Vue integration less mature (community-driven)
    - Documentation looked lacking in depth compared to Tauri
    - v3 is still in alpha
- **Electron**: RAM >200MB violates NFR-01
- **Flutter**: Platform channels for process signals are complex, no native PTY
- **Native (GTK+WinForms)**: Triples development time, no code sharing

### Implementation Details

**Stack**:

- **Backend**: Rust + tokio (async runtime)
- **Frontend**: Vue 3 + TypeScript + Vite
- **Database**: rusqlite (SQLite bindings)
- **Process management**: `std::process::Command` + `nix` crate
- **System tray**: `tauri-plugin-tray`

**Dev Workflow**:

```bash
# Frontend-only dev (instant hot reload)
npm run dev
# Full stack dev (2-3 min rebuild)
cargo tauri dev
# Production build
cargo tauri build
```

### Consequences

- Small binary (<15MB) meets NFR-06
- Low RAM usage (<40MB) meets NFR-01
- Strong type safety (Rust + TypeScript)
- Excellent documentation reduces risk
- Backend changes require patience (2-5 min compile)
- **Mitigation**:
    - Use frontend-only dev mode (`npm run dev`) for UI work
    - Use `cargo watch` for incremental builds
- Learning Rust adds 1-2 weeks to timeline

### Success Metrics

- [ ] Can spawn process and stream logs by Week 2
- [ ] Final binary <15MB (AppImage)
- [ ] No memory leaks after 24h runtime

---

## ADR-002: Frontend Framework - Vue 3

**Updated**: 2025-12-05

### Context

Need UI framework for command list, log viewer, and real-time updates with minimal bundle size.

### Decision: Vue 3 with Composition API + TypeScript

### Justification:

1. **Reactivity**: reactive statements reduce boilerplate for process state and list updates
2. **Component reuse**: CommandCard, LogViewer, TemplatePicker as Clean separate components
3. **DevTools**: Vue DevTools helps debug state during development
4. **TypeScript**: Full support with `<script setup lang="ts">`
5. **Ecosystem**: Vue Router (multi-window), Pinia (state management)
6. **Bundle size**: 2.5MB vs ~7.5MB (React) → smaller final binary (NFR-06)
7. **Easier to Style**: Vanilla css/tailwind and JS takes a lot of effort to get right

**Alternatives Considered**:

- **React**: Too much boilerplate/verbose (`useState`/`useEffect`)
- **Svelte**: Smaller bundle, but smaller community
- **Vanilla JS**: Would require to reinvent the wheel
- **Form library**:
    - `@tanstack/form`: Requires to fight excessive generic types declaration and Veevalidate has
      better Array input handling

### Consequences

- Great TypeScript integration
- Pinia for global state (running processes, settings)
- Smaller bundle size

#### UI Component Library: shadcn-vue

**Justification**:

1. **Copy-paste approach**: Components live in codebase, that Acts as modifiable base for design
   system,
2. **Customizable with tailwind/css**: Easy to modify styles without fighting framework (to achieve
   desktop look)
3. **TypeScript-first with Radix primitives**: Better DX than vanilla CSS/JS with well-tested base
   components

**Rejected**:

- **Vanilla CSS/JS**: Too time-consuming, would need to build everything from scratch
- **Element Plus/Vuetify**: Opinionated, harder to customize, larger bundle
- **Reka UI**: Good but less pre-styled/configured components, greater dependency on vanilla CSS

#### Form handling: VeeValidate

**Justification**:

1. **Array field support**: Critical for quite a few dynamic application inputs
2. **Vue-native and extensible**: Designed for Vue Composition API, works with Zod/Yup for type-safe
   validation
3. **Field arrays**: `useFieldArray()` handles dynamic command/step lists elegantly
4. **Less verbose**: Simpler than TanStack Form's excessive generic typing

**Rejected**:

- **TanStack Form**:
    - Generics are overly complex (upto 11-18 fields for some fields)
    - Real laggy documentation site, could not find vue documentation
    - Feels React first than headless
- **Manual validation**: Error-prone for nested/array fields
- **Formkit**: Less flexible, more opinionated

---

## ADR-003: Data Storage

### Context

Need fast and reliable persistence with query support for commands, categories, templates, and
settings.

### **Decision**: SQLite database (with rusqlite)

### Justification (SQLite):

1. **Extensibility**: Better data storage and reusability, ensures consistency
2. **Performance**: Faster query result and processing compared to alternatives
3. **Embedded**: No separate database server
4. **File-based**: Easy backup (just copy .db file)

### Rejected Alternatives (for SQLite):

- **JSON files**: Simple but no queries, hard to scale searching
- **PostgreSQL**: Overkill, requires separate server
- **NoSQL (MongoDB)**: Unnecessary complexity for structured data
- **LocalStorage**: Browser limitation (5MB), not suitable for desktop app
- **Cloud**: Overkill for use case, adds complexity (already has export option planned)

### Justification (rusqlite):

1. **Simpler than alternatives**:
    - `sqlx` compile-time checked queries need a database connection to compile the code, causing
      some
      edge cases [source](https://news.ycombinator.com/item?id=40994216) it has a offline mode but
      that
      evades the benefit [source](https://news.ycombinator.com/item?id=44690914)
    - `turso` is native rust but has too many features and more complicated setup
    - `diesel` can cause bigger binary size and slower compile time in favor of type safety
    - async overhead is not necessary for the use case
2. **Performance**: Faster in compilation and bulk data load (may not matter much, as there will not
   be that high load on database operations).
3. **Self-contained**: features = ["bundled"] means no OS dependencies
4. **File-based**: Easy backup (just copy .db file)

### Consequences

- Fast queries (<50ms for 1000 commands)
- Easy backup (copy tgui.db)
- Write raw SQL (no wrapper/ORM)
- Supports transactions (atomic template application)
- Need migration system for schema updates (use `schema_version` table)

### Database Location

- **Linux**: `~/.local/share/tgui/tgui.db`
- **Windows**: `%APPDATA%/tgui/tgui.db`
- **macOS**: `~/Library/Application Support/tgui/tgui.db`

`rusqlite` was chosen over `sqlx` or `disel` considering the faster complation and smaller bundle
size (the alternatives did not provide any significant benefit).

**draft schema and more explanation is at [database](10-database.md)**

### Template vs. Group vs. Category Distinction

**Template** (in `templates` table):

- Immutable blueprint for copying (if group is class/instance then template is interface)
- Contains placeholder commands (can not execute commands) and nested templates
- Used to generate executable commands/groups

**Group** (in `groups` table):

- Executable container for commands
- Can contain nested groups (hierarchy)
- **Commands inside groups can be executed**
- Supports inheritance of defaults (directory, env vars)

**Category** (in `categories` table):

- Simple tagging system for organization
- Flat structure (no nesting)
- Used for filtering and display grouping

### Why JSON Structure Instead of Database Rows?

**Decision**: Templates store command/group structure as JSON, not as separate `template_commands`
and `template_groups` tables.

- Templates stored as JSON text in single `structure` column
- Commands created in database only when template is applied

**Alternatives Considered**:
**Separate Tables**

- `template_groups` and `template_commands` tables with foreign keys
- Template commands exist as real database rows

**Justification**:

1. **Industry Standard**: Terraform, Docker Compose, Kubernetes all use text-based blueprints
2. **Export/Import**: Templates are portable JSON files (version control friendly)
3. **Safety**: Template commands can't be accidentally executed (not in `commands` table)
4. **Simplicity**: Templates are read-heavy, written rarely. One table vs three tables, simpler
   schema
5. **Extensibility**: JSON schema allows nested templates and variables (FR-05)
5. **Variables**: `{{variable}}` syntax works naturally in JSON strings

**Trade-offs**:

- Easy to share/version control
- Prevents accidental execution
- Preview requires parsing JSON (not a simple SELECT query)
- Can't reuse command CRUD logic directly

**Implementation**:
Template application follows this flow:

1. Parse `structure` JSON
2. Substitute `{{variables}}` with user values
3. Validate all paths (security check)
4. Create groups in database (in order)
5. Create commands with correct `group_id` references
6. Return list of created command IDs

This ensures atomic creation (transaction rollback on failure) and correct parent-child
relationships.


---

## ADR-004: Process Management Pattern

### Context

Need reliable background process spawning with log streaming and signal handling.

### Decision

**Chosen**: Backend manages processes in HashMap(Rust `std::process::Command` + `tokio` async
runtime), streams logs via events.

**Architecture**:

```
Backend (Rust)                    Frontend (Vue)
─────────────────                    ───────────────
AppState {                           ProcessStore {
  processes: HashMap<u32, Process>     processes: Map<number, Process>
}                                    }
    ↓                                    ↑
spawn_command() ────[event]────→ on("process-started")
    ↓                                    ↑
stream_logs() ──────[event]────→ on("log-line")
    ↓                                    ↑
kill_process() ─────[event]────→ on("process-stopped")
```

### Justification

1. **Signal handling**: `nix` crate provides native SIGTERM/SIGKILL (FR-02)
2. **Process groups**: `setpgid()` for killing entire trees (FR-14)
3. **Memory safety**: Rust ownership prevents leaks in 24/7 background processes (NFR-01)
4. **Async**: Non-blocking log streaming (Tokio/goroutines)
5. **HashMap storage**: O(1) lookup by PID
6. **Event-driven**: Frontend subscribes to log events, no polling

### Process Lifecycle

```
Idle → Running → Stopping → Stopped
                    ↓
                  Error
```

### Rejected Alternatives

- **Synchronous spawning**: Blocks UI during `wait()`
- **Shell & operator**: Can't capture PID reliably
- **Node.js child_process**: Requires IPC bridge, higher latency

### Consequences

- Non-blocking UI during execution
- Real-time log streaming
- Proper signal handling (graceful shutdown)
- Must handle async/await complexity
- Need thread-safe HashMap (Mutex in Rust) for shared process state

## Sequential Process Management (Workflow System)

Need ability to batch/sequence multiple commands with different execution contexts, error handling,
and conditional execution.

### Decision

**Chosen**: Workflows with steps stored in relational tables

**Architecture**:

```
workflows (parent container)
  ↓
workflow_steps (junction table with commands)
  ↓
commands (actual executable commands)
```

### Justification

1. **Standard junction table pattern**: Many-to-many relationship between workflows and commands
2. **Reusability**: Same command/process can be used in multiple workflows
3. **Flexibility**: Each step can have its own timeout, retry, conditions
4. **Execution modes**: Sequential (MVP), Parallel/Conditional (TODO: future)
5. **Error handling**: Continue-on-failure, stop-on-error per step

### Data Model

- **Workflows**: Container with execution mode (sequential/parallel/conditional)
- **Workflow Steps**: Position-ordered, references command_id
    - `condition`: always/on_success/on_failure
    - `timeout_seconds`: per-step timeout
    - `auto_retry_count`: automatic retries
    - `enabled`: can disable steps without deletion
    - `continue_on_failure`: keep executing after failure
- **Position system**: Uses fractional positioning (POSITION_GAP = 1000) for efficient reordering

### Consequences

- Commands can be reused across workflows with ability to disable/enable as steps
- TODO: Variables/templating system (e.g., `${PREVIOUS_OUTPUT}`)
- TODO: Rollback/cleanup commands on failure
- TODO: Pipe stdout between steps

---

## ADR-005: Logs

## Buffer Strategy

**Date**: 2025-12-05

### Context

Need to buffer logs in memory for display, but limit memory usage.

### Decision

**Chosen**: Circular buffer with 10,000 lines per process

### Justification

1. **Memory**: 10k lines × 100 chars ≈ 1MB per process
2. **Performance**: In-memory vector is O(1) append, Writing logs to DB would add latency (
   continuous write could overwhelm database)
3. **Use case**: It is assumed Users care about recent logs, not long-term storage
4. **Simplicity**: No file I/O, no rotation logic

### Rejected Alternatives

- **SQLite**: Adds I/O latency, file bloat
- **Log files**: Users can save to file if they want persistence
- **Infinite buffer**: May violate NFR-01 (RAM target)

### Consequences

- Predictable memory usage
- Logs older than 10k lines are lost (document limitation)
- Users who need full history can use `command > output.log 2>&1`
- Must implement circular buffer logic in Rust

## Audit Logging

Need secure audit logging that doesn't leak sensitive data and manages disk space.

### Decision

**Chosen**: Structured logging with automatic cleanup and sensitive data redaction

### Security Measures

1. **No sensitive data in logs**:
    - Avoid logging command text (might contain sensitive data)
    - Don't log arguments or environment variables
    - Log only metadata: name, id, length, count, boolean flags

2. **Automatic log cleanup**:
    - Delete logs older than 30 days on startup (Changeable from settings)
    - Daily log file rotation
    - Secure logs directory/file permissions: 0o700 on Unix (Files inherit directory permissions)

### Consequences

- Prevents credential leakage in logs
- Bounded disk usage
- User can manage audit logs via UI
- TODO: Log encryption for highly sensitive environments

---

## ADR-006: Template Format - JSON

### Context

Need a format for exporting/importing command templates.

### Decision

**Chosen**: JSON with schema validation

### Template Schema

```json
{
  "$schema": "https://tgui.dev/schemas/template-v1.json",
  "name": "Python Development",
  "description": "Common commands for Python projects",
  "version": "1.0.0",
  "variables": [
    {
      "key": "directory",
      "label": "Project Directory",
      "type": "path",
      "required": true
    },
    {
      "key": "venv_name",
      "label": "Virtual Environment Name",
      "type": "string",
      "default": "venv"
    }
  ],
  "commands": [
    {
      "name": "Create Virtual Environment",
      "command": "python",
      "arguments": [
        "-m",
        "venv",
        "{{venv_name}}"
      ],
      "working_directory": "{{directory}}",
      "category": "Python"
    },
    {
      "name": "Install Dependencies",
      "command": "{{directory}}/{{venv_name}}/bin/pip",
      "arguments": [
        "install",
        "-r",
        "requirements.txt"
      ],
      "working_directory": "{{directory}}",
      "category": "Python"
    }
  ]
}
```

### Justification

1. **Human-readable**: Easy to edit in text editor
2. **Validation**: JSON Schema prevents errors
3. **Version control**: Could be integrated with git
4. **Standard**: Wide tooling support

### Rejected Alternatives

- **YAML**: More prone to syntax errors (indentation)
- **TOML**: Less familiar to most users
- **Custom format**: Reinventing the wheel

### Consequences

- Sharable
- Validation catches errors before import
- Version field allows schema evolution

---

## ADR-007: Execution History for Audit Trail

### Context

Need to track command and workflow executions for debugging, statistics, and audit purposes.

### Decision

- Single `execution_history` table with CHECK constraints for type safety
- **No stdout/stderr storage**- Would bloat database, logs saved separately (ADR-005)

### Execution Types

1. **Command execution**: `command_id` set, others NULL
2. **Workflow execution**: `workflow_id` set, others NULL
3. **Workflow step execution**: All three IDs set

### Justification

1. **Single table**: Simpler queries than separate tables for each type
2. **Type safety**: CHECK constraint prevents invalid combinations

### Data Storage Decisions

**Outputs**: Not stored in database

- TODO: Option A - Save recents to `/executions/{execution_id}.log` files
- TODO: Option B - Don't save automatically, show live only during execution
- Rationale: Prevents database bloat, typical outputs can be large
- **Duration**: Calculated from `started_at` and `completed_at` timestamps
- TODO: Proper datetime parsing for duration calculation

### Consequences

- Efficient queries for execution history
- Can track workflow success/failure rates
- Cascade delete when command/workflow deleted
- Manual cleanup with configurable retention (default: keep last 100 per entity)
- TODO: Real-time stdout/stderr streaming during execution
- **Cleanup**: Keep last N executions per entity, or delete older than X days

### Triggers

- Auto-set `completed_at` when status becomes terminal
- Prevent updating completed executions
- Validate status transitions (e.g., can't go running → skipped)
