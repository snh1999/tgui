# 4. Architecture Decision Records (ADRs)

### Update Log
- **08-12-2025**: Initial Decision Record (ADR-001 to ADR-006)


## ADR-001: Framework Choice - Tauri v2

**Updated**: 2025-12-09 

### Context
Need a cross-platform desktop framework for TGUI with process spawning, system tray support and low memory footprint.

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

### Update Log
- 09-12-2025: Decided on Tauri v2 because of better support, accepting potential compile time trade-off


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

**Alternatives Considered**:
- **React**: Too much boilerplate/verbose (`useState`/`useEffect`)
- **Svelte**: Smaller bundle, but smaller community
- **Vanilla JS**: Would require to reinvent the wheel

### Consequences
- Great TypeScript integration
- Pinia for global state (running processes, settings)
- Smaller bundle size

---

## ADR-003: Data Storage

**Date**: 2025-12-05  

### Context
Need fast and reliable persistance with query support for commands, categories, templates, and settings. 

### **Decision**: SQLite database

### Justification:
1. **Extensibility**: Better data storage and reusablility, ensures consistency
2. **Performance**: Faster query result and processing compared to alternatives
3. **Embedded**: No separate database server
4. **File-based**: Easy backup (just copy .db file)


### Rejected Alternatives
- **JSON files**: Simple but no queries, hard to scale searching
- **PostgreSQL**: Overkill, requires separate server
- **NoSQL (MongoDB)**: Unnecessary complexity for structured data
- **LocalStorage**: Browser limitation (5MB), not suitable for desktop app
- **Cloud**: Overkill for usecase, adds complexity (already has export option planned)


### Consequences
- Fast queries (<50ms for 1000 commands)
- Easy backup (copy tgui.db)
- Supports transactions (atomic template application)
- Need migration system for schema updates (use `schema_version` table)

### Database Location
- **Linux**: `~/.local/share/tgui/tgui.db`
- **Windows**: `%APPDATA%/tgui/tgui.db`
- **macOS**: `~/Library/Application Support/tgui/tgui.db`

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

draft schema is at [database](14-database.md)


---

## ADR-004: Process Management Pattern (AI generated)

**Date**: 2025-12-05  

### Context
Need reliable background process spawning with log streaming and signal handling.

### Decision
**Chosen**: Backend manages processes in HashMap(Rust `std::process::Command` + `tokio` async runtime), streams logs via events. 

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


---


## ADR-005: Log Buffer Strategy
 
**Date**: 2025-12-05  

### Context
Need to buffer logs in memory for display, but limit memory usage.

### Decision
**Chosen**: Circular buffer with 10,000 lines per process

### Justification
1. **Memory**: 10k lines × 100 chars ≈ 1MB per process
2. **Performance**: In-memory vector is O(1) append
3. **Use case**: Users want recent logs, not 3-day history
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

---

## ADR-006: Template Format - JSON

**Status**: Accepted  
**Date**: 2025-12-05  

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
      "arguments": ["-m", "venv", "{{venv_name}}"],
      "working_directory": "{{directory}}",
      "category": "Python"
    },
    {
      "name": "Install Dependencies",
      "command": "{{directory}}/{{venv_name}}/bin/pip",
      "arguments": ["install", "-r", "requirements.txt"],
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
