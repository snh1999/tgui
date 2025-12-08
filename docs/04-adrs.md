# 4. Architecture Decision Records (ADRs)

## ADR-001: Framework Comparison (Tauri vs Wails)

### Context
Need a cross-platform desktop app that can spawns commands with native performance and multi-window log viewing with reliable background process management, shipping smaller binary size

**Preliminary Research**:
- **Tauri**: Mature (v2), Rust backend, excellent docs - slower in dev timing 
- **Wails**: Newer (v3 in alpha), Go backend, simpler for Go devs, smaller community
**Rejected Alternatives**
- **Electron**: RAM >200MB violates NFR-01
- **Flutter**: Platform channels for process signals are complex, no native PTY
- **Native (GTK+WinForms)**: Triples development time, no code sharing

### Decision**: Build **parallel prototypes** in both, decide after 1 week

### Evaluation Criteria:
| Criterion | Weight | Tauri | Wails | Notes |
|-----------|--------|-------|-------|-------|
| **Dev Experience** | High | ? | ? | Rebuild speed, hot reload quality |
| **Binary Size** | Medium | ? | ? | Target: < 15MB |
| **Process Spawning** | Critical | ? | ? | Must handle stdin/stdout streaming |
| **File Picker Integration** | High | ? | ? | Native directory selection |
| **Menu/Tray Support** | Low | ? | ? | Nice-to-have for quick access |
| **Multi window** | Medium | ? | ? | Multi window/monitor support (FR-13) |
| **Distribution** | High | ? | ? | Smoother packaging workflow |
| **Community/Docs** | Medium | ? | ? | Issue resolution speed |

### Test Plan (Week 1):
- Build same feature in both: "List commands + Execute one command + Show logs"
- Measure: Build time, bundle size, code clarity and UI update/responsiveness
- Test: Process spawning reliability, log streaming latency
- Decide: Keep one, document why in ADR-001-decision.md

### Consequences
- Must learn Rust and Golang for backend

---

## ADR-002: Frontend Framework - Vue 3

### Context
Need lightweight, fast-updating UI for log streaming with minimal bundle size.

### Decision: Vue 3 with Composition API + TypeScript

### Justification:
1. **Reactivity**: reactive statements reduce boilerplate for process state and list updates
2. **Component reuse**: CommandCard, LogViewer, TemplatePicker as separate components
3. **DevTools**: Vue DevTools helps debug state during development
4. **Learning**: Shows modern Vue skills on resume (Composition API is latest standard)
5. **Bundle size**: 2.5MB vs ~7.5MB (React) → smaller final binary (NFR-06)

**Alternatives Considered**:
- **React**: Too much boilerplate (`useState`/`useEffect` everywhere)
- **Svelte**: Smaller bundle, but Vue has better TypeScript integration
- **Vanilla JS**: Too much boilerplate for reactive lists

---

## ADR-003: Data Storage

### Context
Need fast and reliable persistance with query support. 

**Rejected Alternatives**:
- **JSON file**: Simple but no queries, hard to search 100+ commands
- **LocalStorage**: Browser limitation (5MB), not suitable for desktop app
- **Cloud**: Overkill for usecase, adds complexity (already has export option planned)

### **Decision**: SQLite database via Tauri/Wails SQL plugin

### Justification:
1. **Extensibility**: Better data storage and reusablility, ensures consistency
2. **Performance**: Faster query result and processing compared to alternatives  

**Schema** (draft):
```sql
CREATE TABLE commands (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  command TEXT NOT NULL,
  description TEXT,
  category TEXT,
  directory TEXT,
  env_vars TEXT, -- JSON string: {"KEY": "value"}
  is_favorite BOOLEAN DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE templates (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  commands TEXT -- JSON array of command objects
);

CREATE TABLE execution_history (
  id INTEGER PRIMARY KEY,
  command_id INTEGER,
  exit_code INTEGER,
  executed_at TIMESTAMP,
  FOREIGN KEY (command_id) REFERENCES commands(id)
);
```

---

## ADR-004: Log Storage Strategy

### Context
Need to buffer logs for display, but limit memory usage per process.

**Rejected Alternatives**
- **SQLite**: Adds 500KB binary size, I/O latency, complexity
- **Log files**: Users have the option `> output.log` if they want persistence
- **Infinite buffer**: Could violate NFR-01 (RAM target)

### **Decision**: In-memory buffer (upto last 10,000 lines per command by default) + optional export/persistance

### Justification:
1. **Performance**: No I/O latency, instant rendering
2. **Simplicity**: No log rotation logic
3. **Use case**: Users need recent logs, not full history from days ago
4. **Export option**: Users can save to file if needed
5. **Limit**: 10k lines × 100-500 chars ≈ estimated 2MB-10MB per command
6. **Search UX**: Higher number = slower search result

### Consequences
- User loses logs older than 10k lines (Custom limit can be added later)
- Must implement circular buffer logic in Rust


## ADR-005: Process Management Backend Pattern

### Context
Need reliable background process spawning with log streaming and signal handling.

### Decision
**Chosen**: Rust `std::process::Command` + `tokio` async runtime

### Justification
1. **Signal handling**: `nix` crate provides native SIGTERM/SIGKILL (FR-02)
2. **Process groups**: `setpgid()` for killing entire trees (FR-07)
3. **Memory safety**: Rust ownership prevents leaks in 24/7 background processes (NFR-01)
4. **Async**: `tokio::spawn` keeps UI responsive while streaming logs

### Rejected Alternatives
- **Synchronous spawning**: Blocks UI during long `wait()`, violates FR-03
- **Shell `&`**: Can't capture PID reliably, no signal control
- **Node.js `child_process`**: Requires IPC bridge, higher latency

### Consequences
- Must handle Rust async/await and `Pin<Box<dyn Future>>` complexity
- Need `Arc<Mutex<HashMap>>` for shared process state (learning curve)
