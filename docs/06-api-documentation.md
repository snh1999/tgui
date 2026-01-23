# 10. API Documentation

### Update Log

- **12-12-2025**: Initial API documentation plan
- **23-01-2026**: Updated based on actual implementation

This document defines the IPC (Inter-Process Communication) contract between the Tauri Rust backend
and Vue frontend.

## 10.1 Command Management

## Command Management

### Create Command

`create_command(payload: CommandPayload) -> Result<CommandId>`

**Description**: Save a new command to the database.

**Payload**:

```rust
struct CommandPayload {
  name: String,
  command: String,              // Executable name/path
  arguments: Vec<String>,       // JSON array: ["arg1", "arg2"]
  description: Option<String>,
  group_id: Option<i64>,        // NULL = top-level/root command
  working_directory: Option<String>,
  env_vars: Option<HashMap<String, String>>, // Merged with group defaults
  shell: Option<String>,        // NULL = use group/system default
  category_id: Option<i64>,
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

- `Ok(id)`: Command ID (i64) on success
- `Err(msg)`: Error message

**Errors**:

- **Invalid directory**: Working directory doesn't exist
- **Duplicate name**: Command with same name exists (warning, not error)
- **Invalid data**: Empty name or command, invalid environment variable key
- **Database error**: SQLite error

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

**Errors**:

- **Not found**: Invalid command id
- Data validation issue same as `create_command`

---

### Delete Command

`delete_command(id: i64) -> Result<()>`

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

`get_command(id: CommandId) -> Result<Command>`

**Description**: Finds a single command by ID.

**Parameters**:

- `id`: Command ID

**Returns**:

- `Ok(command)`: Command object
- `Err("Command not found")`: Invalid ID

---

### List commands

`get_commands(group_id: Option<i64>, favorites_only: bool) -> Result<Vec<Command>>`

**Description**: Get commands with optional filtering.

Filter:

```rust
struct CommandFilter {
  category_id: Option<i64>,
  group_id: Option<i64>,
  is_favorite: Option<bool>,
}
```

**Parameters**:

- `group_id`: Filter by group
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

const topLevelCommands = await invoke('get_commands', {
  groupId: null,
  favoritesOnly: false
})

const favoriteCommands = await invoke('get_commands', {
  groupId: 1,
  favoritesOnly: true
})
```

---

### Search commands

`get_commands(search_term: String) -> Result<Vec<Command>>`

**Description**: Get commands with search term.

**Parameters:**

- `search_term`: Search string (case-insensitive, matches name/description/command)

---

### Move Command Between Positions

`
move_command_between(
    cmd_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>) -> Result<()>
`

**Description**: Reorder a command by placing it between two other commands using fractional
indexing.

**Parameters**:

- `cmd_id`: Command to move
- `prev_id`: Command before the new position (None = move to top)
- `next_id`: Command after the new position (None = move to bottom)

**Behavior Note**:

- Calculates midpoint between `prev` and `next` positions
- If gap exhausted (positions too close), automatically renumbers entire group
- After renumbering, positions are reset with `POSITION_GAP` spacing (default `1000`)

**Why this approach?**

- Efficient reordering without updating all items
- No need to shift positions of other items
- Automatic recovery when gaps run out

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Command ID not found

**Usage**:

```typescript
// Move command 5 between commands 2 and 7
await invoke('move_command_between', {
  cmdId: 5,
  prevId: 2,
  nextId: 7
})

// Move to top of list
await invoke('move_command_between', {
  cmdId: 5,
  prevId: null,
  nextId: 2  // first command
})

// Move to bottom
await invoke('move_command_between', {
  cmdId: 5,
  prevId: 7,  // last command
  nextId: null
})
```

---

### Toggle Favorite

`toggle_favorite(id: i64) -> Result<()>`

**Description**: Toggles the `is_favorite` flag for a command.

**Parameters**:

- `id`: Command ID

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Command not found

**Usage**:

```typescript
await invoke('toggle_favorite', { id: 5 })
```

---

## 10.2 Group Management

### Create Group

`create_group(group: Group) -> Result<GroupId>`

**Description**: Creates a new command group with optional parent hierarchy.

**Payload**:

```rust
struct Group {
  name: String,                 // Required, non-empty
  description: Option<String>,
  parent_group_id: Option<i64>, // NULL = top-level group
  position: i64,                // Auto-calculated, don't set manually
  working_directory: Option<String>,
  env_vars: Option<HashMap<String, String>>,
  shell: Option<String>,
  category_id: Option<i64>,
}
```

**Validation**:

- Name must not be empty
- Environment variable keys must be alphanumeric + underscore + dash

**Returns**: Group ID

**Usage**:

```typescript
const groupId = await invoke('create_group', {
  name: 'Docker Services',
  description: 'All Docker-related commands',
  parentGroupId: null,
  workingDirectory: '/home/user/docker',
  envVars: { DOCKER_HOST: 'unix:///var/run/docker.sock' },
  shell: '/bin/bash',
  categoryId: 2
})
```

---

### Get Group

`get_group(id: GroupId) -> Result<Group>`

**Description**: Retrieves a single group by ID.

**Returns**:

- `Ok(group)`: Group object
- `Err(NotFound { entity: "group", id })`: Invalid ID

---

### Get Groups

`get_groups(parent_id: Option<GroupId>) -> Result<Vec<Group>>`

**Description**: Get groups filtered by parent, ordered by position.

**Parameters**:

- `parent_id`: Filter by parent group (NULL for top-level groups)

**Returns**: Array of groups ordered by position

**Usage**:

```typescript
interface Group {
  id: number
  name: string
  description?: string
  parentGroupId?: number
  position: number
  workingDirectory?: string
  envVars?: Record<string, string>
  shell?: string
  categoryId?: number
  createdAt: string
  updatedAt: string
}

const topGroups = await invoke('get_groups', { parentId: null })

const childGroups = await invoke('get_groups', { parentId: 3 })
```

---

### Update Group

`update_group(group: Group) -> Result<(), Error>`

**Description**: Updates an existing group. Includes circular reference validation.

**Validation**:

- Name must not be empty
- Environment variable keys must be valid
- **Circular reference check**: Prevents setting a parent that would create a cycle

**Errors**:

- `NotFound { entity: "group", id }`: Group not found
- `CircularReference { group_id, parent_id }`: Would create circular hierarchy
- `InvalidData`: Validation failed

**Usage**:

```typescript
await invoke('update_group', {
  id: 5,
  name: 'Updated Group',
  description: 'New description',
  parentGroupId: 2,  // Moving to different parent
  // ... other fields
})
```

---

### Move Group Between Positions

`move_group_between(
    group_id: GroupId,
    prev_id: Option<GroupId>,
    next_id: Option<GroupId>) -> Result<()>`

**Description**: Reorder groups using fractional indexing (same logic as `move_command_between`).

**Parameters**:

- `group_id`: Group to move
- `prev_id`: Group before new position (None = move to top)
- `next_id`: Group after new position (None = move to bottom)

**Behavior**: Same as command reordering - calculates midpoint, auto-renumbers if gap exhausted.

---

### Delete Group

`delete_group(id: GroupId) -> Result<()>`

**Description**: Deletes a group and due to `ON DELETE CASCADE` in the schema

- Delete all child groups recursively
- Delete all commands in the group

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Group not found

**Frontend Recommendation**: Show confirmation dialog with count of affected items before deletion.

---

### Get Group Command Count

`get_group_command_count(id: GroupId) -> Result<i64>`

**Description**: Returns the number of commands directly in this group (not recursive).

**Usage**:

```typescript
const count = await invoke('get_group_command_count', { id: 5 })
```

---

### Get Group Tree

`get_group_tree(root_id: GroupId) -> Result<Vec<Group>>`

**Description**: Recursively retrieves a group and all its descendants using SQL recursive CTE.

**Returns**: Flattened array of groups in the tree, ordered by position

**Usage**:

```typescript
// Get entire subtree starting from group 3
const tree = await invoke('get_group_tree', { rootId: 3 })
// Returns [group_3, child_1, child_2, grandchild_1, ...]
```

---

### Get Group Path

`get_group_path(group_id: i64) -> Result<Vec<String>, Error>`

**Description**: Returns breadcrumb path from root to the specified group.

**Returns**: Array of group names from root to target (reversed to show hierarchy)

**Usage**:

```typescript
const path = await invoke('get_group_path', { groupId: 8 })
// Returns: ['Root', 'Docker', 'Production', 'Database']
// Useful for displaying: Root > Docker > Production > Database
```

---

## 10.3 Category Management

### Create Category

`fn create_category(name: String, icon: Option, color: Option) -> Result<CategoryId>`

**Description**: Creates a new category for organizing commands/groups.

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

**Errors**:

- **Invalid Name**: if name is empty string or whitespaces only

---

### Get Category

`fn get_categories(id: CategoryId) -> Result<Category>`

**Description**: Retrieves a single category by ID.

**Parameters**: Category Id

**Returns**:

- `Ok(category)`: Category object
- `Err(NotFound { entity: "category", id })`: Invalid ID

### List Categories

`fn get_categories() -> Result<Vec<Category>>`

**Description**: Get all categories, ordered alphabetically by name.

**Usage**

```typescript
interface Category {
  id: number
  name: string
  icon?: string
  color?: string
  createdAt: string
  updatedAt: string
}

const categories = await invoke('get_categories')
```

---

### Update Categories

`fn update_category(id: CategoryId, name: String, icon: Option, color: Option) -> Result`

Description: Updates an existing category.

**Returns**:

- `Ok(())`: Success
- `Err(NotFound { entity: "category", id })`: Category not found
- `Err(InvalidData{})`: Name is empty

---

### Delete Category

`fn delete_category(id: CategoryId) -> Result<()>`

**Description**: Deletes a category. (Commands/groups using this category will have category_id set
to NULL (FK ON DELETE SET NULL in schema))

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Category not found

---

### Get Category Command Count

`get_category_command_count(id: CategoryId) -> Result<i64, Error>`

**Description**: Returns the number of commands tagged with this category.

**Usage**:

```typescript
const count = await invoke('get_category_command_count', { id: 2 })
```

---

## 10.4 Settings

### Get Setting

`get_setting(key: String) -> Result<String, Error>`

**Description**: Retrieves a single setting value by key.

**Available Keys**:

- `theme`: "system" | "light" | "dark"
- `default_shell`: Shell path (e.g., "/bin/bash")
- `log_buffer_size`: Integer (max log lines to keep)
- `max_concurrent_processes`: Integer (max parallel processes)
- `auto_scroll_logs`: "true" | "false"
- `warn_before_kill`: "true" | "false"
- `kill_process_tree_by_default`: "true" | "false"

**Returns**:

- `Ok(value)`: Setting value as string
- `Err(NotFound { entity: "setting", id: 0 })`: Key not found

**Usage**:

```typescript
const theme = await invoke('get_setting', { key: 'theme' })
```

---

### Set Setting

`set_setting(key: String, value: String, state: State) -> Result<(), Error>`

**Description**: Updates or inserts a setting value (upsert operation).

**Validation**:

- Key must be one of the known settings
- Numeric settings must parse as valid integers
- Boolean settings must be "true" or "false"

**Errors**:

- `InvalidData { field: "key", reason: "Unknown setting: ..." }`: Invalid key
- `InvalidData { field: "value", reason: "" }`: Invalid value (type assertion)

**Usage**:

```typescript
await invoke('set_setting', {
  key: 'theme',
  value: 'dark'
})

await invoke('set_setting', {
  key: 'log_buffer_size',
  value: '20000'
})
```

---

### Get All Settings

`get_all_settings(state: State) -> Result<HashMap<String, String>, Error>`

**Description**: Retrieves all settings as a key-value map.

**Usage**:

```typescript
const settings = await invoke('get_all_settings')
// Returns: { theme: 'dark', default_shell: '/bin/bash', ... }
```

---

### Reset Settings

`reset_settings(state: State) -> Result<(), Error>`

**Description**: Resets all settings to their default values.

**Default Values**:

```json
{
  "theme": "system",
  "default_shell": "/bin/bash",
  "log_buffer_size": "10000",
  "max_concurrent_processes": "20",
  "auto_scroll_logs": "true",
  "warn_before_kill": "true",
  "kill_process_tree_by_default": "false"
}
```

**Usage**:

```typescript
await invoke('reset_settings')
```

---

## 10.5 Process Control

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

- `Ok(pid)`: Process ID (i64) on success
- `Err(msg)`: Error message

**Events Emitted**:

- process-started: `{ pid: number, commandId: number, command: string, timestamp: number }`
- log-line: `{ pid: number, line: string, source: 'stdout' | 'stderr', timestamp: number }` (
  repeated for each line)
- process-stopped: `{ pid: number, exitCode: number, signal?: string }`

**Errors**:

- **Command not found**: Command not in PATH, Invalid command ID
- **Directory not found**: Working directory doesn't exist
- **Permission denied**: Can't execute command
- **Already running**: Command is already running, Process with same PID exists

**Usage**:

```typescript
import { listen } from '@tauri-apps/api/event'

// Start listening for logs
const unlisten = await listen('log-line', (event) => {
  const { pid, line, source, timestamp } = event.payload
  console.log(`[${ pid }] ${ source }: ${ line }`)
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

- `pid`: Process ID (i64)
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
await invoke('kill_process', { pid: 12345, force: true })
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
  Running { pid: i64, start_time: u64 },
  Stopping,
  Stopped { exit_code: i32 },
  Error { exit_code: i32, message: String },
}
```

---

### Terminate all process

`stop_all_processes() -> Result<i64, Error>`

<!--TODO: consider passing a Optional array if needed-->

**Description**: Stops all running processes.

**Returns**:

- `OK(count)`: Count for killed process.
- `Err("Something went wrong")`

**Confirmation** : Always required (modal dialog)

**Events Emitted**: process-stopped for each process


---

## 10.6 Logs

### Event: `log-line`

**Description**: Emitted for each line of stdout/stderr.

**Payload**:

```rust
struct LogLine {
  pid: i64,
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
  pid: i64,
  old_status: ProcessStatus,
  new_status: ProcessStatus,
}
```

### Get logs

`get_log_buffer(pid: i64,offset: usize, limit: usize, ) -> Vec<LogLine>`

**Description**: Get log lines from circular buffer. (Allows to reopen window and populate logs from
memory)

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

## 10.7 Template Management

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

## 10.8 Others

### System Tray

`get_tray_status() -> TrayStatus`

**Description**: Returns status for tray icon tooltip.

**Returns**:

```rust
struct TrayStatus {
  running_count: i64,
  error_count: i64,
  total_commands: i64,
}
```

**Polling**: Frontend polls every 5 seconds

---

### Show main window

`show_main_window() -> Result<(), Error>`

**Description**: Shows/hides main window from tray.

### Error Types

All database operations return structured errors:

```rust
enum DatabaseError {
  NotFound {
    entity: &'static str,  // "command", "group", "category", "setting"
    id: i64
  },
  InvalidData {
    field: &'static str,
    reason: String
  },
  CircularReference {
    group_id: i64,
    parent_id: i64
  },
  DatabaseError(rusqlite::Error),
  SerializationError(serde_json::Error),
}
```

**Frontend Handling Recommendations**: Map to user-friendly toast messages

```typescript
try {
  await invoke('delete_command', { id: 5 })
} catch (error) {
  if (error.includes('NotFound')) {
    showToast('Command not found', 'error')
  } else if (error.includes('InvalidData')) {
    showToast('Validation failed: ' + error, 'error')
  } else if (error.includes('CircularReference')) {
    showToast('Cannot create circular group hierarchy', 'error')
  } else {
    showToast('Database error: ' + error, 'error')
  }
}
```

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
