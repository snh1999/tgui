# 06. API Documentation

### Update Log

- **12-12-2025**: Initial API documentation plan
- **23-01-2026**: Updated based on actual implementation
- **13-02-2026**: Added Workflow Management API (section 6.5)
- **20-02-2026**: Added Execution History API (section 6.6)

This document defines the IPC (Inter-Process Communication) contract between the Tauri Rust backend
and Vue frontend.

## 6.1 Command Management

## Common

### Row Deserialization Edge Cases

**Arguments Parsing**:

- Invalid JSON in arguments field returns empty `Vec::new()` (graceful degradation)
- Warning logged: "Failed to parse arguments, using default"

**Environment Variables Deserialization**:

- Invalid JSON in `env_vars` field returns `None` (graceful degradation)
- Warning logged: "Failed to parse env_vars, using None"
- Empty JSON object `{}` deserializes to empty HashMap, not None

**Execution Status Deserialization**:

- Unknown status strings default to `ExecutionStatus::Completed`
- Safe fallback prevents application crashes on data corruption

**TriggeredBy Deserialization**:

- If `workflow_id` is present → `TriggeredBy::Workflow`
- Otherwise → `TriggeredBy::Manual`


**Status Deserialization**: Unknown status strings in DB default to `ExecutionStatus::Completed`
with a warning log. This is a safety fallback, not a valid state to set intentionally.



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
    - **keys handling**
        - Environment variable keys must match `^[a-zA-Z_][a-zA-Z0-9_]*$`
        - Must start with letter or underscore
        - Subsequent characters: alphanumeric + underscore only
        - Spaces, dots, unicode, and other special characters are rejected with
          `InvalidData { field: "env_vars" }`
    - **Value Handling**:
        - Empty string values are allowed
        - Special characters in values are preserved (shell injection patterns stored as-is)
        - Values are JSON-serialized with HashMap structure

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
  envVars: {NODE_ENV: 'development'}
})
```

---

### Update Command

`update_command(id: CommandId, payload: CommandPayload) -> Result<(), Error>`

- Changing `group_id` automatically recalculates `position` and assigns the last position of the
  group.

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

`get_commands(group_id: GroupFilter, category_id: CategoryFilter, favorites_only: bool) -> Result<Vec<Command>>`

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

- `parent_id`: `GroupFilter`
    - `Group(id)` filters to children of that parent;
    - `None` returns root-level groups (NULL parent);
    - `All` returns all groups regardless of parent
- `category_id`: `CategoryFilter`
    - `Category(id)` filters by category;
    - `None` returns uncategorized groups;
    - `All` ignores category
- `favorites_only`: When `true`, only groups with `is_favorite = 1` are returned

- **Pagination Parameters**:
    - `limit`: Maximum number of commands to return (None = unlimited)
    - `offset`: Number of commands to skip (None = start from beginning)

**History Join/Retrieval Behavior**:

- Returns `WithHistory<Command>` where `history` contains the most recent non-workflow execution
- Workflow-associated history (where `workflow_id IS NOT NULL`) is excluded from the join
- History is ordered by `started_at DESC` with `ROW_NUMBER() = 1` to get latest entry per command

**Returns**:

- `Ok(commands)`: Array of commands with latest execution history entryo (settings applied)
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

### List Recent Commands

`get_recent_commands(limit: i64) -> Result<Vec<WithHistory<Command>>>`

**Description**: Returns commands that have been executed recently, ordered by their most recent
execution time.

- Only commands with at least one execution history entry are included (Returns empty vector if none
  found)
- Commands with multiple executions only appear once (most recent)
- Workflow-associated executions are included (unlike get_commands filter)

**Parameters**:

- `limit`: Maximum number of commands to return (required, `i64`)

**Returns**: Array of commands wrapped with their most recent execution history, ordered by
`started_at DESC` (most recent executions first).

**Key Differences from `get_commands`**:

- Only returns commands that have been executed at least once
- Results ordered by execution time, not by command position
- No filtering by group, category, or favorites

**SQL Implementation Notes**:

- Uses `ROW_NUMBER() OVER (PARTITION BY command_id ORDER BY started_at DESC, id DESC)` to select
  most recent history per command
- The `id DESC` tie-breaker ensures deterministic ordering when timestamps are identical
- Uses `WHERE EXISTS` to filter only commands with history entries
- `NULLS LAST` option requires SQLite 3.30.0+ (2019-10-04)

**Edge Cases**:

- **No executions exist**: Returns an empty vector;
- **Command executed multiple times**: Appears exactly once, with the most recent history entry
  attached
- Identical `started_at` timestamps: Tie-broken by `id`, ensuring deterministic ordering
- Workflow executions: Unlike `get_commands`, workflow-associated executions (
  `workflow_id IS NOT NULL`) are included here; if a command was only ever run via a workflow it
  will still appear
- `limit = 0`: Returns an empty vector without error

**Usage**:

```typescript
const recent = await invoke('get_recent_commands', {limit: 10})
```

---

### Get Commands Count

`get_commands_count(group_id: Option<i64>, category_id: Option<i64>, favorites_only: bool) -> Result<i64>`

**Description**: Returns the count of commands matching the specified filters.

**Parameters**:

- `group_id`: Filter by group (None for root-level commands)
- `category_id`: Filter by category (None for all categories)
- `favorites_only`: Count only favorited commands when true

**Returns**: Count of matching commands (i64)

**Usage**:

```typescript
const totalCount = await invoke('get_commands_count', {
  groupId: 1,
  categoryId: null,
  favoritesOnly: false
})
```

---

### Search commands

`get_commands(search_term: String) -> Result<Vec<Command>>`

**Description**: Get commands with search term.

**Parameters:**

- `search_term`: Search string (case-insensitive, matches name/description/command)

**Behavior Note**:

- Case-insensitive matching using SQLite `LIKE`
- Searches across `name`, `command`, and `description` fields
- Pattern: `%search_term%` (substring match)
- Ordering: `is_favorite DESC, updated_at DESC` (favorites first, then most recently updated)
- Special characters (`%`, `_`) in search term are treated as wildcards by SQLite
- Empty Search Term returns all commands ordered by favorites and updated_at

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

- Calculates midpoint between `prev` and `next` positions (between adjacent items)
- `POSITION_GAP` constant (default: 1000) used for initial positioning and renumbering
- If gap exhausted (difference in gap <= 1), automatic renumbering triggers
- After renumbering, positions are reset with `POSITION_GAP` spacing

**Edge Cases**:

- Moving to same position (prev/next unchanged) is idempotent
- When gap ≤ 1 between adjacent items, automatic renumbering triggers with `POSITION_GAP` spacing
- Renumbering preserves relative order of all items in group

**Error Cases**:

- Returns `InvalidData { field: "item_id" }` if **both** `prev_id` and `next_id` are `None`
- Returns `InvalidData { field: "parent_id" }` if `prev_id` or `next_id` belong to a different
  parent group than `group_id`
- Returns `NotFound` if `group_id`, `prev_id`, or `next_id` don't exist

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
await invoke('toggle_favorite', {id: 5})
```

---

## 6.2 Group Management

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
    icon: Option<String>,
    color: Option<String>,
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
  envVars: {DOCKER_HOST: 'unix:///var/run/docker.sock'},
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

`get_groups(parent_id: GroupFilter, category_id: CategoryFilter, favorites_only: bool) -> Result<Vec<Group>>`

**Description**: Get groups filtered by parent, ordered by position.

**Parameters**:

- same as `get_commands`

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
  isFavorite: boolean
  icon?: string
  color?: string
  createdAt: string
  updatedAt: string
}

const topGroups = await invoke('get_groups', {
  parentId: null,
  categoryId: null,
  favoritesOnly: false
})

const childGroups = await invoke('get_groups', {
  parentId: 3,
  categoryId: null,
  favoritesOnly: false
})
```

---

### Update Group

`update_group(group: Group) -> Result<(), Error>`

**Description**: Updates an existing group. Includes circular reference validation.

- Changing `parent_group_id` automatically recalculates `position` and assigns the last position of
  the group.

**Validation**:

- Name must not be empty
- Environment variable keys must be valid
- **Circular reference check**: Prevents setting a parent that would create a cycle
    - Fires when `parent_group_id` is set and either:
        - `parent_group_id == group.id` (self-reference), or
        - Walking the ancestor chain of `parent_group_id` upward reaches `group.id` (multi-level
          cycle, e.g. A→B→C then setting A's parent to C)
        - Duplicate IDs detected in the ancestor chain (data corruption guard)
        - Returns `CircularReference { group_id, parent_id }`
    - Setting `parent_group_id` to `None` (promoting to root) skips the check entirely

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

- behavior is same as `move_group_between`, both uses same helper function

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
const count = await invoke('get_group_command_count', {id: 5})
```

---

### Get Group Tree

`get_group_tree(root_id: GroupId) -> Result<Vec<Group>>`

**Description**: Recursively retrieves a group and all its descendants using SQL recursive CTE.

**Returns**: Flattened array of groups ordered by `depth ASC, position ASC`

- root is always first,
- then all depth-1 children by position,
- then depth-2, and so on. A single-node tree returns just the root.

**Usage**:

```typescript
// Get entire subtree starting from group 3
const tree = await invoke('get_group_tree', {rootId: 3})
// Returns [group_3, child_1, child_2, grandchild_1, ...]
```

---

### Get Group Path

`get_group_path(group_id: i64) -> Result<Vec<String>, Error>`

**Description**: Returns breadcrumb path from root to the specified group.

**Returns**: Array of group names from root to target (reversed to show hierarchy)

**Usage**:

```typescript
const path = await invoke('get_group_path', {groupId: 8})
// Returns: ['Root', 'Docker', 'Production', 'Database']
// Useful for displaying: Root > Docker > Production > Database
```

---
---

### Search Groups

`search_groups(search_term: String) -> Result<Vec<Group>>`

**Description**: Search groups by name or description.

**Parameters**:

- `search_term`: Substring to match (case-insensitive). Empty string returns all groups.

**Behavior**:

- Pattern: `%search_term%` applied to both `name` and `description` fields
- Results ordered by `name ASC`
- Special characters (`%`, `_`) are treated as SQLite wildcards
- Empty `search_term` (`""`) returns all groups ordered by name.

**Returns**: Matching groups ordered alphabetically by name

**Usage**:

```typescript
const groups = await invoke('search_groups', {searchTerm: 'docker'})
```

---

### Get Groups Count

`get_groups_count(group_id: Option<i64>, category_id: Option<i64>, favorites_only: bool) -> Result<i64>`

**Description**: Returns the count of groups matching the specified filters. Uses the same filter
logic as `get_groups`.

**Parameters**:

- `group_id`: Filter by parent group (`None` = root-level groups)
- `category_id`: Filter by category (`None` = uncategorised)
- `favorites_only`: Count only favorited groups when `true`

**Returns**: Count of matching groups (`i64`)

**Usage**:

```typescript
const count = await invoke('get_groups_count', {
  groupId: null,
  categoryId: null,
  favoritesOnly: false
})
```

---

### Get Group Ancestor Chain

`get_group_ancestor_chain(group_id: i64) -> Result<Vec<Group>>`

**Description**: Returns the group itself followed by all its ancestors walking up the parent chain,
closest-first (direct parent before grandparent, root last).

**Returns**: Array ordered closest-first. A root group returns a single-element array containing
itself.

**SQL Implementation**: Recursive CTE walking `parent_group_id` upward from `group_id`.

**Usage**:

```typescript
// For a group at path Root > Docker > Production:
const chain = await invoke('get_group_ancestor_chain', {groupId: productionId})
// Returns: [Production, Docker, Root]
```

---

### Toggle Group Favorite

`toggle_group_favorite(id: i64) -> Result<()>`

**Description**: Toggles the `is_favorite` flag for a group.

**Parameters**:

- `id`: Group ID

**Returns**:

- `Ok(())`: Success
- `Err(NotFound { entity: "groups", id })`: Group not found

**Usage**:

```typescript
await invoke('toggle_group_favorite', {id: 5})
```

---

## 6.3 Category Management

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
const count = await invoke('get_category_command_count', {id: 2})
```

---

## 6.4 Settings

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
const theme = await invoke('get_setting', {key: 'theme'})
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

## 6.5 Workflow Management

### Create Workflow

`create_workflow(workflow: Workflow) -> Result<i64>`

**Description**: Creates a new workflow that can contain multiple workflow steps.

**Payload**:

```rust
struct Workflow {
    name: String,
    description: Option<String>,
    category_id: Option<i64>,
    is_favorite: bool,
    position: i64,  // Auto-calculated using fractional indexing
}
```

**Parameters**:

- `name`: Display name for the workflow
- `description`: Optional description
- `category_id`: Optional category ID
- `is_favorite`: Star the workflow
- `position`: Auto-calculated position (don't set manually)

**Returns**:

- `Ok(id)`: Workflow ID (i64) on success
- `Err(msg)`: Error message

**Errors**:

- **Invalid data**: Empty name
- **Database error**: SQLite error

**Usage**:

```typescript
import { invoke } from '@tauri-apps/api/tauri'

const workflowId = await invoke('create_workflow', {
  workflow: {
    name: 'Deploy to Production',
    description: 'Full deployment workflow',
    categoryId: 2,
    isFavorite: true,
    position: 0
  }
})
```

---

### Get Workflow

`get_workflow(id: i64) -> Result<Workflow>`

**Description**: Retrieves a single workflow by ID.

**Parameters**:

- `id`: Workflow ID

**Returns**:

- `Ok(workflow)`: Workflow object
- `Err("Workflow not found")`: Invalid ID

---

### Get Workflows

`get_workflows(category_id: Option<i64>, favorites_only: bool) -> Result<Vec<Workflow>>`

**Description**: Get workflows with optional filtering.

**Parameters**:

- `category_id`: Filter by category (None for all categories)
- `favorites_only`: Show only favorited workflows

**Returns**:

- `Ok(workflows)`: Array of workflows ordered by position
- `Err(msg)`: Database error

**Usage**:

```typescript
interface Workflow {
  id: number
  name: string
  description?: string
  categoryId?: number
  isFavorite: boolean
  position: number
  createdAt: string
  updatedAt: string
}

// Get all workflows
const allWorkflows = await invoke('get_workflows', {
  categoryId: null,
  favoritesOnly: false
})

// Get favorite workflows in category 2
const favoriteWorkflows = await invoke('get_workflows', {
  categoryId: 2,
  favoritesOnly: true
})
```

---

### Update Workflow

`update_workflow(workflow: Workflow) -> Result<()>`

**Description**: Updates an existing workflow.

**Parameters**: Workflow object with updated fields

**Returns**:

- `Ok(())`: Success
- `Err(msg)`: Error message

**Errors**:

- **Not found**: Invalid workflow ID
- **Invalid data**: Empty name

---

### Delete Workflow

`delete_workflow(id: i64) -> Result<()>`

**Description**: Deletes a workflow and all associated workflow steps.

**Parameters**:

- `id`: Workflow ID

**Returns**:

- `Ok(())`: Success
- `Err("Workflow not found")`: Invalid ID

**Note**: This will cascade delete all workflow steps associated with this workflow.

---

### Toggle Favorite Workflow

`toggle_favorite_workflow(id: i64) -> Result<()>`

**Description**: Toggles the `is_favorite` flag for a workflow.

**Parameters**:

- `id`: Workflow ID

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Workflow not found

**Usage**:

```typescript
await invoke('toggle_favorite_workflow', {id: 3})
```

---

### Get Workflow Count for Category

`get_workflow_count_for_category(category_id: Option<i64>) -> Result<i64>`

**Description**: Returns the count of workflows in a category.

**Parameters**:

- `category_id`: Category ID (None for uncategorized workflows)

**Returns**: Count of workflows

---

### Move Workflow Between Positions

`move_workflow_between(workflow_id: i64, prev_id: Option<i64>, next_id: Option<i64>) -> Result<()>`

**Description**: Reorder a workflow by placing it between two other workflows using fractional
indexing.

**Parameters**:

- `workflow_id`: Workflow to move
- `prev_id`: Workflow before the new position (None = move to top)
- `next_id`: Workflow after the new position (None = move to bottom)

**Behavior**: Same fractional indexing logic as command reordering

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Workflow ID not found

**Usage**:

```typescript
// Move workflow 5 between workflows 2 and 7
await invoke('move_workflow_between', {
  workflowId: 5,
  prevId: 2,
  nextId: 7
})
```

---

## 6.5.1 Workflow Steps

### Create Workflow Step

`create_workflow_step(flow_steps: WorkflowStep) -> Result<i64>`

**Description**: Creates a new step in a workflow that references a command.

**Payload**:

```rust
struct WorkflowStep {
    workflow_id: i64,
    command_id: i64,
    position: i64,  // Auto-calculated using fractional indexing
    enabled: bool,
    wait_for_completion: bool,
    delay_seconds: Option<i64>,
}
```

**Parameters**:

- `workflow_id`: Parent workflow ID
- `command_id`: Command to execute in this step
- `position`: Auto-calculated position (don't set manually)
- `enabled`: Whether this step is active
- `wait_for_completion`: If true, wait for this command to complete before proceeding
- `delay_seconds`: Optional delay before executing this step

**Returns**:

- `Ok(id)`: Workflow step ID (i64) on success
- `Err(msg)`: Error message

**Usage**:

```typescript
const stepId = await invoke('create_workflow_step', {
  flowSteps: {
    workflowId: 1,
    commandId: 5,
    position: 0,
    enabled: true,
    waitForCompletion: true,
    delaySeconds: 5
  }
})
```

---

### Get Workflow Step

`get_workflow_step(id: i64) -> Result<WorkflowStep>`

**Description**: Retrieves a single workflow step by ID.

**Parameters**:

- `id`: Workflow step ID

**Returns**:

- `Ok(step)`: WorkflowStep object
- `Err("Workflow step not found")`: Invalid ID

---

### Get Workflow Steps

`get_workflow_steps(workflow_id: Option<i64>, command_id: Option<i64>, enabled_only: bool) -> Result<Vec<WorkflowStep>>`

**Description**: Get workflow steps with optional filtering.

**Parameters**:

- `workflow_id`: Filter by workflow (None for all workflows)
- `command_id`: Filter by command (None for all commands)
- `enabled_only`: Show only enabled steps

**Returns**: Array of workflow steps ordered by position

**Usage**:

```typescript
interface WorkflowStep {
  id: number
  workflowId: number
  commandId: number
  position: number
  enabled: boolean
  waitForCompletion: boolean
  delaySeconds?: number
  createdAt: string
  updatedAt: string
}

// Get all steps for workflow 1
const steps = await invoke('get_workflow_steps', {
  workflowId: 1,
  commandId: null,
  enabledOnly: false
})

// Get enabled steps that use command 5
const enabledSteps = await invoke('get_workflow_steps', {
  workflowId: null,
  commandId: 5,
  enabledOnly: true
})
```

---

### Get Workflow Steps with Command Data

`get_workflow_steps_command_populated(workflow_id: i64, enabled_only: bool) -> Result<Vec<(WorkflowStep, Command)>>`

**Description**: Get workflow steps with their associated command objects populated. Useful for
displaying workflow execution details.

**Parameters**:

- `workflow_id`: Workflow ID
- `enabled_only`: Show only enabled steps

**Returns**: Array of tuples containing (WorkflowStep, Command)

**Usage**:

```typescript
const stepsWithCommands = await invoke('get_workflow_steps_command_populated', {
  workflowId: 1,
  enabledOnly: true
})

// Returns: Array<[WorkflowStep, Command]>
stepsWithCommands.forEach(([step, command]) => {
  console.log(`Step ${step.id}: ${command.name}`)
})
```

---

### Update Workflow Step

`update_workflow_step(workflow: WorkflowStep) -> Result<()>`

**Description**: Updates an existing workflow step.

**Parameters**: WorkflowStep object with updated fields

**Returns**:

- `Ok(())`: Success
- `Err(msg)`: Error message

---

### Delete Workflow Step

`delete_workflow_step(id: i64) -> Result<()>`

**Description**: Deletes a workflow step.

**Parameters**:

- `id`: Workflow step ID

**Returns**:

- `Ok(())`: Success
- `Err("Workflow step not found")`: Invalid ID

---

### Move Workflow Step Between Positions

`move_workflow_step_between(workflow_id: i64, prev_id: Option<i64>, next_id: Option<i64>) -> Result<()>`

**Description**: Reorder a workflow step within its workflow using fractional indexing.

**Parameters**:

- `workflow_id`: Workflow step to move
- `prev_id`: Step before the new position (None = move to top)
- `next_id`: Step after the new position (None = move to bottom)

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Workflow step ID not found

**Usage**:

```typescript
// Move step 10 between steps 8 and 12
await invoke('move_workflow_step_between', {
  workflowId: 10,
  prevId: 8,
  nextId: 12
})
```

---

### Toggle Workflow Step Enabled

`toggle_workflow_step_enabled(id: i64) -> Result<()>`

**Description**: Toggles the `enabled` flag for a workflow step.

**Parameters**:

- `id`: Workflow step ID

**Returns**:

- `Ok(())`: Success
- `Err(NotFound)`: Workflow step not found

**Usage**:

```typescript
await invoke('toggle_workflow_step_enabled', {id: 15})
```

---

### Get Workflow Step Count

`get_workflow_step_count(id: i64) -> Result<i64>`

**Description**: Returns the count of steps in a workflow.

**Parameters**:

- `id`: Workflow ID

**Returns**: Count of workflow steps

**Usage**:

```typescript
const stepCount = await invoke('get_workflow_step_count', {id: 1})
```

---

## 6.6 Execution history management

```rust
// status, exit_code, started_at, completed_at — those are set by the DB/process layer.
struct ExecutionHistory {
    id: i64,
    command_id: Option<i64>,
    workflow_id: Option<i64>,
    workflow_step_id: Option<i64>,
    pid: Option<i64>,        // set after spawn via update_execution_pid()
    status: Status,             // default: Status::Running on create
    exit_code: Option<i32>,
    started_at: String,             // DATETIME, set by DB DEFAULT
    completed_at: Option<String>,     // set by DB trigger on terminal transition
    triggered_by: TriggeredBy,
    context: Option<String>,     // optional JSON metadata
}

enum Status {
    Running,
    Success,
    Failed,
    TimedOut,
    Cancelled,
    Skipped,
    Paused,
    // Completed is used as a safe fallback when deserialising unknown status values from DB;
    Completed,
}

enum TriggeredBy { Manual, Workflow, Schedule }
```

- Valid command_id / workflow_id / workflow_step_id combinations (enforced by both DB `CHECK` and
  rust validation):
- Valid types

| command_id | workflow_id | workflow_step_id | 
|------------|-------------|------------------|
| Some       | None        | None             |
| None       | Some        | None             |   
| Some       | Some        | Some             |

Passing any other combination returns `DatabaseError::InvalidData`.

### Create Execution History

`create_execution_history(history: &ExecutionHistory) -> Result<i64>`

**Description**: Insert a new row with `status = 'running'`.

- Validates that all referenced IDs exist in the database before inserting.

**Returns**: Ok(id) — the new execution_history.id, used as the key in ProcessManager.

**Errors**:

- `NotFound` — `command_id`, `workflow_id`, or `workflow_step_id` does not exist
- `InvalidData` — invalid ID combination (see table above)
- `InvalidData { field: "command" }` — `command_id` references a command that already has a`Running`
  execution
-

### Get Execution History

`get_execution_history(id: i64) -> Result<ExecutionHistory>`

**Description**: Fetch a single history row by ID.

**Errors**: `NotFound` if the row doesn't exist.

### Get Command Execution History

`get_command_execution_history(command_id: i64, limit: Option<i64>) -> Result<Vec<ExecutionHistory>>`

**Description**: All history rows for a command, most-recent first.

**Parameters**:

- `limit` — max rows to return. Defaults to `EXECUTION_HISTORY_LIMIT= 100` if None.

### Get Workflow Execution History

`get_workflow_execution_history(workflow_id: i64, limit: Option<i64>) -> Result<Vec<ExecutionHistory>>`

**Description**: All history rows for a workflow (includes step-level rows), most-recent first.

**Parameters**:
`limit` — max rows to return. Defaults to `EXECUTION_HISTORY_LIMIT=100` if None.

### Get Running Commands

`get_running_commands() -> Result<Vec<ExecutionHistory>>`

**Description**: Returns all execution history rows with `status = 'running'` for standalone command
executions only.

- `command_id IS NOT NULL` — excludes pure workflow-level rows
- `workflow_id IS NULL` — excludes workflow-step executions; use `get_workflow_execution_history`for
  those
- Used at app startup for orphan detection and by the frontend's active-process list

**Returns**: All currently running standalone command executions, unordered.

### Update Execution PID

`update_execution_pid(id: i64, pid: u32) -> Result<()>`

**Description**: Stores the OS process ID once `child.spawn()` has succeeded. Called immediately
after spawn before any log streaming begins.

**Why separate from create**: The PID is only available after the OS process actually starts, which
happens after the DB row is created.

**Errors**:

- `NotFound` — no row with that `id`
- `InvalidData { field: "pid" }` — the row exists but its status is not `Running`; the PID can only
  be stored on a live execution

### Update Execution History Status

`update_execution_history_status(id: i64, status: Status, exit_code: Option<i32>) -> Result<()>`

**Description**: Transition a running execution to a terminal state. The completed_at timestamp is
set automatically by the execution_history_timestamps DB trigger.

**Valid terminal transitions from Running:**

- `Status::Success` — process exited with code 0
- `Status::Failed` — process exited with non-zero code
- `Status::TimedOut` — process exceeded step timeout_seconds
- `Status::Cancelled` — process was stopped by user (`SIGTERM` or `SIGKILL`)

**Parameters**:

- `status` — target terminal state (not Running or Skipped)
- `exit_code` — OS exit code; None for signals/cancellation

**Errors**:

- `InvalidData { field: "status" }` — if the current status is already terminal (not `Running`), or
  if the requested `status` is `Running` (re-entering running is never valid)
- `NotFound` — row does not exist

### Cancel Execution History

`cancel_execution_history(id: i64) -> Result<()>`

**Description**: Convenience wrapper — calls
`update_execution_history_status(id, Status::Cancelled, None).`

- Used when a spawn fails after the history row was already created (e.g. build_exec returned
  an error).

### Delete Execution History

`delete_execution_history(id: i64) -> Result<()>`

**Description**: Hard-delete a single history row.

(Deleting the parent command or workflow cascades to delete all associated history automatically).

### Cleanup Command History

`cleanup_command_history(command_id: i64, keep_last: i64) -> Result<()>`

**Description**: Retains only the most recent keep_last entries for a command; older rows are
deleted. Implements the ADR-007 retention strategy.
- Only standalone executions (`workflow_id IS NULL`) are considered for deletion; workflow-step history for the same command is never removed by this function.
- **Default**: `keep_last = 100` (matches `EXECUTION_HISTORY_LIMIT`).

**Edge Case**: 
- When `days = 0`, deletes all non-running history regardless of timestamp
- except running entries which are always preserved

**TODO**: Add equivalent cleanup for workflow and workflow-step history.

### Cleanup History Older Than

`cleanup_history_older_than(days: i64) -> Result<()>`

**Description**: Deletes all history rows where `started_at < NOW - days`.

- Skips rows with `status = 'running'` to avoid deleting live sessions.
- Called on app startup based on the `log_retention_days` setting.

### Get Execution Stats

`get_execution_stats(target: StatsTarget, days: Option<i64>) -> Result<ExecutionStats>`

**Description**: Returns aggregate execution statistics for a command, workflow, or globally.
```rust
enum StatsTarget {
    Command(i64),   // standalone executions only (workflow_id IS NULL)
    Workflow(i64),  // top-level workflow rows only (command_id IS NULL, workflow_step_id IS NULL)
    Global,         // all rows where workflow_step_id IS NULL
}

struct ExecutionStats {
    total_count: i64,
    success_count: i64,
    failed_count: i64,
    cancelled_count: i64,
    timeout_count: i64,
    running_count: i64,
    paused_count: i64,
    skipped_count: i64,
    success_rate: f64,           // success_count / total_count, rounded to 2dp; 0.0 if total = 0
    average_duration_ms: Option<i64>,  // None if no completed executions
    last_executed_at: Option<String>,
    first_executed_at: Option<String>,
}
```

**Parameters**:
- `target` — scope of the query (see `StatsTarget` above)
- `days` — when `Some(n)`, only rows where `started_at >= NOW - n days` are counted; `None` = all time

**Edge cases**:
- No matching rows → all counts zero, `success_rate = 0.0`, `average_duration_ms = None`
- `average_duration_ms` is `None` if no rows have a `completed_at` value (e.g. all still running)

**Usage**:
```typescript
const stats = await invoke('get_execution_stats', { target: { Command: commandId }, days: 30 })
```

---

## 6.7 Process Control

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
  const {pid, line, source, timestamp} = event.payload
  console.log(`[${pid}] ${source}: ${line}`)
})

// Execute command
const pid = await invoke('spawn_command', {id: commandId})

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
await invoke('kill_process', {pid: 12345, force: true})

// Force kill (after confirmation)
await invoke('kill_process', {pid: 12345, force: true})
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

## 6.8 Logs

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

## 6.9 Template Management

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

## 6.10 Others

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
  await invoke('delete_command', {id: 5})
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
  const {pid, line, is_stderr} = event.payload
  logStore.append(pid, line, is_stderr)
})

listen('process-status-changed', (event) => {
  const {pid, new_status} = event.payload
  processStore.updateStatus(pid, new_status)
})
```

## Versioning

- **API Version**: v1 (increments on breaking changes)
    - **All commands prefixed**: `api:v1:create_command`
    - **Events include version**: `api:v1:log-line`
