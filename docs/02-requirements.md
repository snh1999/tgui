# Requirements Specification: TGUI v0.1.0

### Update Log

- **05-12-2025**: Initial requirement specification

## 2.1 Non-Functional Requirements

| ID     | Requirement               | Target                                              |
|--------|---------------------------|-----------------------------------------------------|
| NFR-01 | **RAM usage**             | ≤ 50 MB Idle                                        |
| NFR-02 | **Startup time**          | Cold start < 3s, warm start < 500ms                 |
| NFR-03 | **Processe count**        | Handle 10-20 concurrent background processes        |
| NFR-04 | **Log buffer size**       | Keep last 10,000 lines per process in memory        |
| NFR-05 | **Compatibility**         | Ubuntu 22.04+, Fedora 42+, Arch, Windows 11+, macOS |
| NFR-06 | **Build size**            | Final Linux AppImage < 20 MB                        |
| NFR-07 | **Process spawn latency** | < 500 ms                                            |

## 2.2 Functional Requirements

| ID    | Requirement              | Priority | Target                                                                      |
|-------|--------------------------|----------|-----------------------------------------------------------------------------|
| FR-01 | Command Library          | Must     | Save commands with name, description, category, star favorites              |
| FR-02 | Execution Context        | Must     | Run command in specific directory                                           |
| FR-03 | Log Viewer               | Must     | Real-time stdout/stderr streaming                                           |
| FR-04 | Quick Search/Filtering   | Must     | Filter commands by name/category                                            |
| FR-05 | Templates and copying    | Must     | Create command sets (with context, commands)                                |
| FR-06 | Process Status Tracking  | Must     | Arguments parsing, Process starts, PID returned, logs stream in 1s          |
| FR-07 | Processe Termination     | Must     | both graceful and forced kill works, UI updates in <500ms                   |
| FR-08 | Background process       | Must     | Close window → process keeps running → reopen window → logs still streaming |
| FR-09 | Environment Variables    | Should   | Pass env vars with commands (e.g., `API_KEY=xxx`)                           |
| FR-10 | Command Chaining         | Should   | Run sequence of commands                                                    |
| FR-11 | System tray Integration  | Should   | Tray icon updates on spawn/kill, hover shows count                          |
| FR-12 | Category Management      | Should   | Orgaizing and customizations                                                |
| FR-13 | Multi-window Logs        | Could    | Can drag log window to second screen, close main window, logs persist       |
| FR-14 | Process Tree Termination | Could    | eg- kill entire process tree                                                |
| FR-15 | Log Search               | Could    | Ctrl+F opens search bar, filters in <500ms on 10k lines                     |
| FR-16 | Export/Import            | Could    | Share templates as JSON                                                     |
| FR-17 | Schedule Commands        | Could    | Cron-like scheduling                                                        |

### FR-01: Command Library: Save Commands with Execution Context

**Priority**: Must (MVP Core)  
**User Story**: TGUI-01  
**Description**: User can save a command with name, executable, arguments, working directory, category, and environment
variables.

**Acceptance Criteria**:

- Form accepts all required fields (name, command, directory)
- Arguments parsed correctly (handles spaces: `npm run "my script"`)
- Directory picker validates path exists
- Category dropdown with "Create New" option
- Command saved to SQLite within 200ms
- Saved command appears in list immediately

**Edge Cases**:

- Empty command name → show validation error
- Invalid directory → show error, suggest creating it
- Duplicate command name → warn user, allow save
- Special characters in arguments (`$`, `&`, `;`) → escaped correctly

---

### FR-02: Execution Context: Execute Commands in Correct Context

**Priority**: Must (MVP Core)  
**User Story**: TGUI-01, TGUI-06  
**Description**: User can click "Run" to execute command with correct working directory, arguments, and environment
variables.

**Acceptance Criteria**:

- Click Run → process spawns within 500ms (NFR-07)
- Process spawns in specified working directory
- Arguments passed correctly to executable
- Environment variables applied to process
- PID returned and displayed in UI
- Status updates: "Running" with PID

**Edge Cases**:

- Command not found → show error toast with helpful message
- Directory doesn't exist → show error, offer to create
- Permission denied → suggest checking file permissions
- Command already running → warn user, allow multiple instances

---

### FR-03: Log Viewer: Real-Time Log Streaming

**Priority**: Must (MVP Core)  
**User Story**: TGUI-02  
**Description**: stdout and stderr stream to UI with <200ms latency.

**Acceptance Criteria**:

- Logs render as they arrive (no buffering)
- Timestamp prefix: `[HH:MM:SS]`
- Color coding: stdout (white/gray), stderr (red/yellow)
- Auto-scroll enabled by default
- Auto-scroll stops when user scrolls up
- Auto-scroll resumes when scrolled to bottom
- Buffer maintains last 10,000 lines (NFR-04)

**Performance Requirements**:

- 10,000 line buffer: Search completes in <500ms

---

### FR-04: Search and Filter

**Priority**: Must  
**User Story**: TGUI-04  
**Description**: User can search commands by name, command text, or description.

**Acceptance Criteria**:

- Search box filters list in <200ms on 100+ commands
- Search matches: name, command, description, category
- Highlighting of matched text
- Case-insensitive search
- Clear search button (X icon)
- Empty result state: "No commands found. Try different keywords."

---

### FR-05: Command Templates and Copying

**Priority**: Should (Week 4)  
**User Story**: TGUI-05, TGUI-12  
**Description**: User can create templates with multiple commands that can be applied to a new directory.

**Acceptance Criteria**:

- Create template with 1+ commands
- Commands use `{{directory}}` placeholder
- Apply template prompts for directory
- All template commands created with substituted directory
- Template preview before applying
- Built-in templates: Python, Node.js, Docker, Git

**Template Variables**:

- `{{directory}}`: Base project directory
- `{{project_name}}`: Extracted from directory name
- `{{venv_name}}`: Virtual environment name (default: venv)

---

### FR-06: Process Status Tracking

**Priority**: Must (MVP Core)  
**User Story**: TGUI-01, TGUI-02  
**Description**: UI shows current status of each command's execution.

**Status States**:

- **Idle**: Never run or stopped
- **Running**: Process active, showing PID
- **Stopping**: SIGTERM sent, waiting for exit
- **Stopped**: Process exited cleanly (exit code 0)
- **Error**: Process exited with non-zero code

**UI Indicators**:

- Idle: Gray dot
- Running: Green pulsing dot + PID
- Stopping: Yellow dot
- Stopped: Gray dot
- Error: Red dot + exit code

---

### FR-07: Process Termination

**Priority**: Must (MVP Core)  
**User Story**: TGUI-03  
**Description**: User can stop running processes gracefully or forcefully.

**Acceptance Criteria**:

- "Stop" button sends SIGTERM (graceful)
- Right-click menu: "Force Kill" sends SIGKILL
- UI updates to "Stopping" within 500ms
- Process exits → UI shows "Stopped" with exit code
- No zombie processes after kill

**Confirmation**:

- SIGTERM: No confirmation (can be undone by restarting)
- SIGKILL: Confirmation dialog with warning

---

### FR-08: Background Process Persistence

**Priority**: Should (Week 3)  
**User Story**: TGUI-02, TGUI-06  
**Description**: Processes continue running when main window is closed.

**Acceptance Criteria**:

- Close main window → app minimizes to tray
- Processes keep running (verified with `ps aux`)
- Tray icon shows running process count
- Click tray → window reopens
- Reopen window → processes show correct status
- Logs show full history + new lines

**System Tray Menu**:

- Show/Hide Window
- Stop All Processes (with confirmation)
- Running processes count
- Quit (stops all processes, then exits)

---

### FR-09: Environment Variables

**Priority**: Could (Post-MVP)  
**User Story**: TGUI-07  
**Description**: User can define environment variables per command.

**Acceptance Criteria**:

- UI for adding key-value pairs
- Env vars stored with command in database
- Process spawned with correct environment
- Edit/delete individual env vars
- Warning: "Environment variables stored in plain text"

**Security Consideration**:

- Store in database unencrypted (document limitation)
- Display warning in UI: "Environment variables stored in plain text. Do not store sensitive data (passwords, API keys)
  unless you accept this risk."
- Future possibility: Encrypt with OS keyring integration

---

### FR-10: Command Sequences (Chaining)

**Priority**: Could (Post-MVP)  
**User Story**: TGUI-08  
**Description**: User can define sequences of commands to run automatically.

**Acceptance Criteria**:

- Define command order
- Sequential execution (wait for each to complete)
- Stop on first failure (exit code != 0)
- Option: "Continue on error" checkbox
- Logs show which command is currently running
- User can stop chain mid-execution

---

### FR-11: System Tray Integration

**Priority**: Should (Week 3)  
**User Story**: TGUI-06  
**Description**: App minimizes to system tray instead of closing.

**Acceptance Criteria**:

- Tray icon appears on app start
- Close window → app hides to tray
- Tray icon changes color based on process status:
    - Normal: Black/gray
    - Error: Red
    - Activity: Pulsing animation
- Right-click menu functional
- Click icon → toggle window visibility

---

### FR-12: Category Management

**Priority**: Should (Week 3)  
**User Story**: TGUI-04  
**Description**: User can organize commands into categories.

**Acceptance Criteria**:

- Create category with name and optional icon/color
- Assign category when creating command
- Filter commands by category
- Categories shown in sidebar or dropdown
- Edit/delete categories

---

### FR-13: Multi-Window Log Viewing

**Priority**: Could (Post-MVP)  
**User Story**: TGUI-09  
**Description**: User can open logs in separate windows.

**Acceptance Criteria**:

- Right-click command → "Open Logs in New Window"
- New window opens with logs for that command only
- Log window survives main window closure
- Close log window → process keeps running
- Each window has "Stop" button
- Window title shows command name

---

### FR-14: Process Tree Termination

**Priority**: Could (Post-MVP)  
**User Story**: TGUI-10  
**Description**: User can kill entire process tree (parent + children).

**Acceptance Criteria**:

- Checkbox: "Kill process tree"
    - Unix: Use process groups (`setpgid()`) to kill parent + children
    - Windows: Use Job Objects (requires additional implementation, otherwise might leave orphan processes)
- When enabled, sends signal to process group
- All child processes terminate
- Tested with: npm, docker-compose, python scripts

---

### FR-15: Log Search

**Priority**: Could (Post-MVP)  
**User Story**: TGUI-11  
**Description**: User can search logs by text or regex.

**Acceptance Criteria**:

- Ctrl+F opens search bar
- Highlights matches in real-time
- Filter mode: Shows only matching lines
- Search works on streaming logs
- Keyboard shortcuts: F3/(Ctrl + n) (next), Shift+F3/(Ctrl + Shift + n) (previous)

---

### FR-16: Export/Import

**Priority**: Could  
**User Story**: TGUI-12  
**Description**: User can export and share command templates as JSON.

**Acceptance Criteria**:

- Export template/group/command to JSON file
- Import template/group/command from JSON file
- Export logs log to text file
- Preview before import
- Validation: Check schema, warn on errors
- Option: Merge with existing or replace

---

### FR-17: Schedule Commands

**Priority**: Could
**Description**: User can add chron-job or auto-start certain commands on startup.

**Acceptance Criteria**:

- Schedule command execution
- Separte section/tag for scheduled tasks
- Set certain commands to run at application/OS startup
- Retry command on failure

---

## 2.3 User Personas

### Persona 1: Solo Developer "Alex"

- **Background**: Full-stack developer, works on 5+ projects simultaneously
- **Pain**: Constantly forgets exact commands for each project (npm vs pnpm, different env vars)
- **Goal**: One-click access to all dev commands organized by project
- **Usage Pattern**:
    - Saves 20-30 commands
    - Uses favorites heavily (5-10 commands)
    - Runs 3-5 commands daily
    - Needs quick search (types "docker" → sees all Docker commands)

### Persona 2: DevOps Engineer "Bob"

- **Background**: Manages microservices, monitors logs constantly
- **Pain**: Terminal tabs don't survive window closure, loses logs
- **Goal**: Keep servers running in background, monitor logs side-by-side
- **Usage Pattern**:
    - Runs 5-10 long-running processes (docker-compose, kubectl logs)
    - Uses multi-window logs on dual monitors
    - Relies on system tray to keep processes alive
    - Searches logs frequently (errors, warnings)

### Persona 3: Casual Linux User "Cat"

- **Background**: Uses Linux for daily tasks, not a developer
- **Pain**: Forgets command syntax for system maintenance (update, cleanup)
- **Goal**: GUI for common tasks without opening terminal
- **Usage Pattern**:
    - Saves 5-10 system commands (apt update, disk usage)
    - Uses templates for routine tasks
    - Rarely modifies commands after saving
    - Values simplicity over advanced features

---

## 2.4 Out of Scope (Not planned)

**Might get included in plan depending on feedback**

- Terminal emulator (uses system shell)
- Command history (shell history integration)
- Script editor (commands only, not full scripts)
- SSH command execution (local only)
- Cloud sync (commands stored locally only)
- Collaborative features (sharing via export/import only)
- Package manager integration (apt, brew, etc.)
