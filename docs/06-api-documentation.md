# 10. API Documentation

### Update Log

- **12-12-2025**: Initial API documentation plan 


This document defines the IPC (Inter-Process Communication) contract between the Tauri Rust backend and Vue frontend.

## 10.1 Command Management

## Command Management

### Create Command 
```rust
#[tauri::command]
create_command(payload: CommandPayload) -> Result<CommandId, Error>
```

**Description**: Save a new command to the database.

**Payload**:
```rust
struct CommandPayload {
    name: String,
    command: String,              // Executable name/path
    arguments: Vec<String>,       // JSON array: ["arg1", "arg2"]
    description: Option<String>,
    group_id: Option<u32>,        // NULL = top-level command
    working_directory: Option<String>,
    env_vars: Option<HashMap<String, String>>, // Merged with group defaults
    shell: Option<String>,        // NULL = use group/system default
    category_id: Option<u32>,
    is_favorite: bool,
    state: State
}
```

**Parameters**:
- `name`: Display name for the command
- `command`: Executable name or path
- `arguments`: Array of command arguments
- `description`: Optional description
- `working_directory`: Directory where command runs
- `category_id`: Optional category ID
- `is_favorite`: Star the command
- `env_vars`: Optional environment variables

**Returns**:
- `Ok(id)`: Command ID (u32) on success
- `Err(msg)`: Error message

**Errors**:
- `"Invalid directory"`: Working directory doesn't exist
- `"Duplicate name"`: Command with same name exists (warning, not error)
- `"Database error"`: SQLite error

**Usage**:

```typescript
import { invoke } from '@tauri-apps/api/tauri'

const commandId = await invoke('create_command', {
  name: 'Start Dev Server',
  command: 'npm',
  arguments: ['run', 'dev'],
  description: 'Starts Vite dev server',
  workingDirectory: '/home/user/project',
  categoryId: 1,
  isFavorite: false,
  envVars: { NODE_ENV: 'development' }
})
```

---

### Update Command
`update_command(id: CommandId, payload: CommandPayload) -> Result<(), Error>`

**Description**: Update an existing command.

**Parameters**: Same as `save_command` plus `id`

**Returns**:
- `Ok(())`: Success
- `Err(msg)`: Error message

---

### Delete Command

`delete_command(id: i64, state: State) -> Result`

**Description**: Deletes a command and stops if running.

**Parameters**:
- `id`: Command ID

**Returns**:
- `Ok(())`: Success
- `Err("Command not found")`: Invalid ID

**Edge Cases**:
- If command is running → stop it first (SIGTERM), 
- else Returns error: "Cannot delete running command. Stop it first."

**Events Emitted**: `command-deleted { id }`

---

### Get Command

`get_command(id: CommandId) -> Result<Command, Error>`

**Description**: Finds a single command by ID.

**Parameters**:
- `id`: Command ID

**Returns**:
- `Ok(command)`: Command object
- `Err("Command not found")`: Invalid ID

---

### List commands

`get_commands(filter: Option<CommandFilter>) -> Vec<Command>`

**Description**: Get commands with optional filtering.

Filter: 
```rust
struct CommandFilter {
    category_id: Option<u32>,
    group_id: Option<u32>,
    search_term: Option<String>,  // Searches name, command, description
    is_favorite: Option<bool>,
}
```

**Parameters**:
- `search`: Filter by name/command/description
- `category_id`: Filter by category
- `favorites_only`: Show only favorites

**Returns**:
- `Ok(commands)`: Array of commands with resolved inheritance (settings applied)
- `Err(msg)`: Database error


**Usage**:

```typescript
interface Command {
  id: number
  name: string
  command: string
  arguments: string[]
  description?: string
  workingDirectory: string
  categoryId?: number
  isFavorite: boolean
  envVars?: Record
  createdAt: string
  updatedAt: string
}

const commands = await invoke('get_commands', {
  search_term: 'docker',
  category_id: null,
  is_favorite: false
})
```

---

## 10.2 Process Control

### Command Execution

`spawn_command(command_id: CommandId) -> Result<Pid, Error>`

**Description**: Spawns process by running a saved command and start streaming logs.

**Process**:
1. Validate command exists
2. Resolve working directory (inheritance chain: cmd → group → settings → $HOME)
3. Merge environment variables (cmd overrides group)
4. Spawn async process
5. Start log streaming thread
6. Update UI status

**Parameters**:
- `id`: Command ID from database
- `window`: Tauri window handle (for emitting events), Optional

**Returns**:
- `Ok(pid)`: Process ID (u32) on success
- `Err(msg)`: Error message

**Events Emitted**:
- process-started: `{ pid: number, commandId: number, command: string, timestamp: number }`
- log-line: `{ pid: number, line: string, source: 'stdout' | 'stderr', timestamp: number }` (repeated for each line)
- process-stopped: `{ pid: number, exitCode: number, signal?: string }`

**Errors**:
- **Command not found**: Command not in PATH, Invalid command ID
- **Directory not found**: Working directory doesn't exist
- **Permission denied**: Can't execute command
- **Already running**: Command is already running,  Process with same PID exists


**Usage**:

```typescript
import { listen } from '@tauri-apps/api/event'

// Start listening for logs
const unlisten = await listen('log-line', (event) => {
  const { pid, line, source, timestamp } = event.payload
  console.log(`[${pid}] ${source}: ${line}`)
})

// Execute command
const pid = await invoke('spawn_command', { id: commandId })

// Clean up listener when done
unlisten()
```

---

### Process Termination

`termination_process(pid: Pid, force: bool) -> Result<(), Error>`

**Description**: Stop and send signal to running process.

**Parameters**:
- `pid`: Process ID (u32)
- `force`: boolean
  - false: Send `SIGTERM` (graceful)
  - true: Send `SIGKILL` (immediate, Show dialog for confirmation)
  
**Events Emitted**: 
- process-stopped { pid, exit_code, timestamp }

**Returns**:
- `Ok(())`: Signal sent successfully
- `Err(msg)`: Error message

**Usage**:

```typescript
// Graceful stop
await invoke('kill_process', { pid: 12345, force: true })

// Force kill (after confirmation)
await invoke('kill_process', { pid: 12345, force: true  })
```

---

### Running Processes

`get_running_processes(state: State) -> Result<Vec, String>`

**Description**: Get all currently running processes.

**Returns**:

```typescript
interface ProcessInfo {
  pid: number
  commandId: number
  commandName: string
  command: string
  status: 'Running' | 'Stopping' | 'Stopped' | 'Error'
  startTime: number // Unix timestamp
  exitCode?: number
}

const processes = await invoke('get_running_processes')
```

---

### Get process Status
`get_process_status(pid: Pid) -> ProcessStatus`

**Description**: Returns current process state by Id.

**Returns**:

```rust
enum ProcessStatus {
    Idle,
    Running { pid: u32, start_time: u64 },
    Stopping,
    Stopped { exit_code: i32 },
    Error { exit_code: i32, message: String },
}
```

---

### Terminate all process

`stop_all_processes() -> Result<u32, Error>`

<!--TODO: consider passing a Optional array if needed-->

**Description**: Stops all running processes. 

**Returns**:
- `OK(count)`: Count for killed process.
- `Err("Something went wrong")`

**Confirmation** : Always required (modal dialog)

**Events Emitted**: process-stopped for each process

---

## 10.2 Logs

### Event: `log-line`
**Description**: Emitted for each line of stdout/stderr.

**Payload**:
```rust
struct LogLine {
    pid: u32,
    timestamp: u64,        // Unix millis
    line: String,
    is_stderr: bool,
}
```

**Performance**: Batched every 50ms max to reduce IPC overhead

### **Event**: `process-status-changed`
**Description**: Emitted when process state changes.

**Payload**:
```rust
struct ProcessStatusEvent {
    pid: u32,
    old_status: ProcessStatus,
    new_status: ProcessStatus,
}
```


### Get logs
`get_log_buffer(pid: u32,offset: usize, limit: usize, ) -> Vec<LogLine>`

**Description**: Get log lines from circular buffer. (Allows to reopen window and populate logs from memory)

**Parameters**:
- `pid`: Process ID
- `limit`: Max number of lines (default: 10000)

**Returns**: Slice of log buffer (newest first if offset = 0)

**Usage**:
```typescript
interface LogLine {
  line: string
  source: 'stdout' | 'stderr'
  timestamp: number
}

const logs = await invoke('get_log_buffer', { 
  pid: 12345,
  offset: 0,
  limit: 1000, 
})
```

### Clear logs
`clear_log_buffer(pid: Pid) -> Result<(), Error>`

**Description**:Clears log buffer for a process.

**Use Case**: User wants to reset logs

---

## 10.3 Category Management

### Create Category

```rust
#[tauri::command]
fn create_category(
    name: String,
    icon: Option,
    color: Option,
    state: State,
) -> Result
```

**Parameters**:
- `name`: Category name (unique)
- `icon`: Emoji or icon name (e.g., "🐳", "docker")
- `color`: Hex color (e.g., "#3b82f6")

**Returns**: Category ID

**Usage**:
```typescript
const categoryId = await invoke('create_category', {
  name: 'Docker',
  icon: '🐳',
  color: '#2496ed'
})
```

---

### Get Categories

```rust
#[tauri::command]
fn get_categories(state: State) -> Result<Vec, String>
```

**Usage**

```typescript
interface Category {
  id: number
  name: string
  icon?: string
  color?: string
  commandCount: number // How many commands in this category
  createdAt: string
}

const categories = await invoke('get_categories')
```

---

### Update Categories

```rust
#[tauri::command]
fn update_category(
    id: i64,
    name: String,
    icon: Option,
    color: Option,
    state: State,
) -> Result
```

---

### Delete Category

```rust
#[tauri::command]
fn delete_category(
    id: i64,
    move_to_uncategorized: bool,
    state: State,
) -> Result
```

**Parameters**:
- `id`: Category ID
- `move_to_uncategorized`: If true, moves commands to null category; if false, deletes commands

---

## 10.4 Template Management

### Create Template
`create_template(payload: TemplatePayload) -> Result<TemplateId, Error>`

**Description**: Creates a reusable template blueprint.

Payload:
```rust
struct TemplatePayload {
    name: String,
    description: Option<String>,
    structure: TemplateStructure, // JSON matching schema from ADR-006
}
```

**Usage**:
```typescript
interface TemplateCommand {
  name: string
  command: string
  arguments: string[]
  workingDirectory: string // Can contain {{variables}}
  categoryName?: string
  envVars?: Record
}

interface TemplateVariable {
  key: string
  label: string
  type: 'string' | 'path' | 'number'
  default?: string
  required: boolean
}

const templateId = await invoke('create_template', {
  name: 'Python Development',
  description: 'Common Python project commands',
  commands: [
    {
      name: 'Create venv',
      command: 'python',
      arguments: ['-m', 'venv', '{{venv_name}}'],
      workingDirectory: '{{directory}}',
      categoryName: 'Python'
    },
    {
      name: 'Install deps',
      command: '{{directory}}/{{venv_name}}/bin/pip',
      arguments: ['install', '-r', 'requirements.txt'],
      workingDirectory: '{{directory}}',
      categoryName: 'Python'
    }
  ],
  variables: [
    {
      key: 'directory',
      label: 'Project Directory',
      type: 'path',
      required: true
    },
    {
      key: 'venv_name',
      label: 'Virtual Environment Name',
      type: 'string',
      default: 'venv',
      required: false
    }
  ]
})
```

---

### Apply template

`apply_template(template_id: TemplateId, variables: Map<String, String>) -> Result<Vec<CommandId>, Error>`

**Description**: Apply template by creating commands with substituted variables.

**Parameters**:
- `template_id`: Template ID
- `variable_values`: Map of variable keys to values

**Process**:
- Load template structure
- Substitute all {{variables}}
- Create groups and commands in database
- Return list of created command IDs

**Events Emitted**: `commands-created { ids }`

**Returns**: Array of created command IDs

**Usage**:

```typescript
const commandIds = await invoke('apply_template', {
  templateId: 1,
  variableValues: {
    directory: '/home/user/my-project',
    venv_name: 'venv'
  }
})
```

---

### Export Template

`export_template(template_id: i64) -> Result`

**Description**: Export template as JSON string.

**Returns**: JSON string (see ADR-006 for format)

---

### Import Template

`import_template(json: String) -> Result`

**Description**: Import template from JSON.

**Parameters**:
- `json`: Template JSON string

**Returns**: Template ID

**Errors**:
- `"Invalid JSON"`: Malformed JSON
- `"Schema validation failed"`: Missing required fields
- `"Dangerous commands detected"`: Contains risky commands (return as warning, not error)

---

## 10.5 Settings

### Get Settings

`get_setting(key: Option<String>) -> Option<String>`

**Description**: Load settings/single setting from database.

---

### Update Settings

`set_setting(key: String, value: String) -> Result<(), Error>`

**Description**: Save settings to database.

**Events Emitted**: `settings-changed { key, value }`

---


## 10.6 Others

### System Tray
`get_tray_status() -> TrayStatus`

**Description**: Returns status for tray icon tooltip.

**Returns**:
```rust
struct TrayStatus {
    running_count: u32,
    error_count: u32,
    total_commands: u32,
}
```

**Polling**: Frontend polls every 5 seconds

---

### Show main window
`show_main_window() -> Result<(), Error>`

**Description**: Shows/hides main window from tray.


### Error Types
All API endpoints return structured errors:
```rust
enum TguiError {
    CommandNotFound { command: String, suggestion: String },
    PermissionDenied { path: String },
    ProcessAlreadyRunning { pid: u32 },
    DirectoryNotFound { path: String },
    TemplateValidationFailed { errors: Vec<String> },
    DatabaseError { message: String },
    IoError { message: String },
}
```

**Frontend Handling**: Map to user-friendly toast messages

### Event Subscription Pattern
Frontend subscribes to events via Tauri's listen:

```typeScript
import { listen } from '@tauri-apps/api/event'

listen('log-line', (event) => {
  const { pid, line, is_stderr } = event.payload
  logStore.append(pid, line, is_stderr)
})

listen('process-status-changed', (event) => {
  const { pid, new_status } = event.payload
  processStore.updateStatus(pid, new_status)
})
```

## Versioning
- **API Version**: v1 (increments on breaking changes)
  - **All commands prefixed**: `api:v1:create_command`
  - **Events include version**: `api:v1:log-line`
