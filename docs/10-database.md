# 10. Database Documentation

### Update Log

- **14-12-2025**: Initial security guidelines


## 13.1 Database Location

| Platform | Path |
|----------|------|
| **Linux** | `~/.local/share/tgui/tgui.db` |
| **Windows** | `%APPDATA%\tgui\tgui.db` |
| **macOS** | `~/Library/Application Support/tgui/tgui.db` |

**Permissions**: `0600` (owner read/write only) on Unix systems

---

## 13.2 Schema Overview

TGUI uses **separate tables** for different entity types:
- `categories`: Organize commands (tags)
- `groups`: Hierarchical folder/container (can be nested, commands are executable)
- `commands`: Executable commands (can be grouped or standalone)
- `templates`: Reusable blueprints (commands are not executable)
- `settings`: Key-value configuration
- `schema_version`: Track migrations

---

## 13.3 Entity Relationships

```
categories
    ↓ (1:many)
commands
    ↓ (many: 1)
groups (optional)
    ↓
groups (parent_group_id- optional)
```

**Key Design**:
- Commands can exist **without groups** (group_id = NULL)
- Groups can be **nested** (parent_group_id points to another group)
- Commands **inherit settings** from immediate parent group
- Templates store JSON structure (not relational)


---

## 13.4 Table Schemas

### categories

**Design**: Simple flat list of tags. Commands, groups, templates reference categories via foreign key.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-increment |
| name | TEXT | NOT NULL, UNIQUE | Category display name |
| icon | TEXT | NULL | Emoji or icon name (e.g., "🐳") |
| color | TEXT | NULL | Hex color (e.g., "#3b82f6") |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Creation time |

**Default row**: `id=0, name='Uncategorized', icon='📁'`

**Default category**: "Uncategorized" (ID: 0) ensures every command has a category

**Index**: `idx_commands_category` for fast filtering (Optional)

---

### groups

**Design**: Groups can contain nested groups.

**Inheritance Chain** (highest priority first):
- Command-specific value (non-NULL)
- Group's default value (provides default values that child commands inherit)
- Application setting (settings table)/default

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-increment |
| name | TEXT | NOT NULL | Group display name |
| description | TEXT | NULL | Group purpose |
| parent_group_id | INTEGER | NULL, FK → groups(id) CASCADE | NULL = top-level group |
| position | INTEGER | DEFAULT 0 | Order within parent |
| default_working_directory | TEXT | NULL | Inherited by child commands |
| default_env_vars | TEXT | NULL | JSON object, inherited |
| default_shell | TEXT | NULL | Shell path (e.g., "/bin/bash") |
| default_category_id | INTEGER | NULL, FK → categories(id) | Inherited by child commands |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Creation time |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Last modification |




**Indexes**:
<!--- `idx_groups_parent` on `(parent_group_id)`-->
- `idx_groups_position` on `(parent_group_id, position)`

**Triggers**: `groups_update_timestamp` updates `updated_at` on modification

---

### commands

**Design**: Stores everything needed to execute a command (FR-02).
- **Arguments Storage**: `arguments TEXT` as `JSON` array
- **Environment Variables**: `env_vars TEXT` as `JSON` object
  - `JSON` over separate table because of perfomance considerations (usually < 5 env_vars/command) 
  - Arguments are always fetched with commands, so no added benefit.
  - **Security consideration**: Stored plain text (see 12-security.md)
- **Group Deletion CASCADE**: Deleting group deletes all child commands for simplification


| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-increment |
| name | TEXT | NOT NULL | Command display name |
| command | TEXT | NOT NULL | Executable name or path |
| arguments | TEXT | NULL | JSON array: `["arg1", "arg2"]` |
| description | TEXT | NULL | Command purpose |
| group_id | INTEGER | NULL, FK → groups(id) CASCADE | NULL = ungrouped |
| position | INTEGER | DEFAULT 0 | Order within group or top-level |
| working_directory | TEXT | NULL | NULL = inherit from group or default to $HOME |
| env_vars | TEXT | NULL | JSON object, merged with group's |
| shell | TEXT | NULL | NULL = inherit from group or system default |
| category_id | INTEGER | NULL, FK → categories(id) | NULL = inherit from group |
| is_favorite | BOOLEAN | DEFAULT 0 | Star for quick access |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Creation time |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Last modification |

**Indexes**:
- `idx_commands_category` on `(category_id)`
<!--- `idx_commands_position` on `(group_id, position)`-->
- `idx_commands_name` on `(name)` for search
- `idx_commands_favorite` on `(is_favorite)`
- `idx_commands_group` on `(group_id)`

**Triggers**: `commands_update_timestamp` updates `updated_at` on modification

---

### templates

**Design**: Stores `JSON` structure in structure TEXT column (read heavy).

**Validation**: Application validates against JSON Schema (ADR-006) on read/write

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | INTEGER | PRIMARY KEY | Auto-increment |
| name | TEXT | NOT NULL | Template display name |
| description | TEXT | NULL | Template purpose |
| author | TEXT | NULL | Creator email or username |
| structure | TEXT | NOT NULL | JSON (see Template Spec doc) |
| created_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Creation time |

**No foreign keys** - templates are self-contained

---

### settings

**Design**: Simple key-value pairs for application-level configuration.
- Settings are sparse (separate columns are difficult to maintain)
- No need to ALTER TABLE for new settings
- Easy to add/remove settings without migration

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| key | TEXT | PRIMARY KEY | Setting name |
| value | TEXT | NOT NULL | Setting value (as string) |
| updated_at | DATETIME | DEFAULT CURRENT_TIMESTAMP | Last modification |

**Default rows**:
```sql
('default_shell', '/bin/bash')
('log_buffer_size', '10000')
('max_concurrent_processes', '20')
('auto_scroll_logs', 'true')
('warn_before_kill', 'true')
('kill_process_tree_by_default', 'false')
('theme', 'system')
```

---

### schema_version

**Design**: Single row storing current schema version.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| version | INTEGER | PRIMARY KEY | Current schema version |

```sql
-- 1. Check current version
SELECT version FROM schema_version;  -- Returns 1

-- 2. Backup
-- 3. Run migration SQL
-- 4. Update version
UPDATE schema_version SET version = 2;
```

**Idempotent migrations**: Use `CREATE TABLE IF NOT EXISTS` and `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`

**Downgrades**: Not yet supported. Users restore from backup to downgrade.

---

## 13.5 Common Query Patterns

### Get all top-level items (ungrouped commands + root groups)

```sql
-- Top-level groups
SELECT * FROM groups WHERE parent_group_id IS NULL ORDER BY position;

-- Ungrouped commands
SELECT * FROM commands WHERE group_id IS NULL ORDER BY position;
```

**Use**: Main view when no group selected


### Get all commands in a group (immediate children only)

```sql
SELECT * FROM commands 
WHERE group_id = ? 
ORDER BY position;
```


### Get full group hierarchy (all descendants)

```sql
WITH RECURSIVE group_tree AS (
    SELECT * FROM groups WHERE id = ?
    UNION ALL
    SELECT g.* FROM groups g
    JOIN group_tree gt ON g.parent_group_id = gt.id
)
SELECT * FROM group_tree;
```

**Use**: Expand group in UI


### Get all commands under a group (including nested groups)

```sql
WITH RECURSIVE group_tree AS (
    SELECT id FROM groups WHERE id = ?
    UNION ALL
    SELECT g.id FROM groups g
    JOIN group_tree gt ON g.parent_group_id = gt.id
)
SELECT c.* FROM commands c
WHERE c.group_id IN (SELECT id FROM group_tree)
ORDER BY c.group_id, c.position;
```

**Use**: Search within group and subgroups

### Search commands by name/command/description

```sql
SELECT * FROM commands 
WHERE name LIKE '%search_term%' 
   OR command LIKE '%search_term%' 
   OR description LIKE '%search_term%'
ORDER BY is_favorite DESC, updated_at DESC;
```

### Get favorite commands across all groups

```sql
SELECT * FROM commands 
WHERE is_favorite = 1
ORDER BY updated_at DESC;
```

### Delete group and all descendants (CASCADE)

```sql
-- SQLite CASCADE handles this automatically
DELETE FROM groups WHERE id = ?;
-- All child groups and commands are deleted via ON DELETE CASCADE
```

---

## 13.6 Resolve Inheritance 

**Not done in SQL** - handled by application code.

**Algorithm**:
```
For each command field (working_directory, env_vars, shell, category_id):
  1. If command's field is NOT NULL → use it
  2. Else if command has group_id:
     - Get group's corresponding default field
     - If NOT NULL → use it
  3. Else use application default:
     - working_directory → $HOME
     - shell → /bin/bash (or system default)
     - env_vars → empty map
     - category_id → 0 (Uncategorized)

For env_vars specifically (merge):
  1. Start with empty map
  2. If grouped, add group's default_env_vars
  3. Add command's env_vars (overwrites group values for same keys)
  4. Return merged map
```

**Example**:
```
Group "Python Projects":
  default_working_directory: /home/user/python
  default_env_vars: {"PYTHON_ENV": "dev", "DEBUG": "true"}

Command "Run Tests":
  group_id: 1 (Python Projects)
  working_directory: NULL (inherit)
  env_vars: {"DEBUG": "false"}

Resolved:
  working_directory: /home/user/python (inherited)
  env_vars: {"PYTHON_ENV": "dev", "DEBUG": "false"} (merged, command wins)
```

---

## 13.7 Migration Strategy

### Current Version: 1

**Version 1** (initial):
- Separate `commands`, `categories`, `templates`, `settings` tables
- `groups` table with nesting

### Future Migrations

When schema changes:
1. Increment `schema_version.version`
2. Write migration function
3. Run on app startup if version < current

### Future Schema Considerations
**Potential additions**:
1. `command_executions` table: Historical record (for analytics, optional)
2. Store sensitive values in OS keyring 
3. Track CPU/memory per process

**Example migration**:
```rust
fn migrate_to_v3(conn: &Connection) -> Result {
    // Add new column
    conn.execute("ALTER TABLE commands ADD COLUMN last_run_at DATETIME", [])?;
    // Create new index
    conn.execute("CREATE INDEX idx_commands_last_run ON commands(last_run_at DESC)", [])?;
    // Update version
    conn.execute("UPDATE schema_version SET version = 3", [])?;
    Ok(())
}
```

### Backup Before Migration

```rust
fn backup_database(db_path: &Path) -> Result {
    let backup_path = db_path.with_extension("db.backup");
    std::fs::copy(db_path, &backup_path)?;
    Ok(backup_path)
}
```

---

## 13.8 Performance Considerations

### WAL Mode (Enabled by default)

```sql
PRAGMA journal_mode = WAL;
```

**Benefits**:
- Readers don't block writers
- Better concurrency for log streaming + UI queries
- Faster commits

**Trade-off**: Creates `-wal` and `-shm` files alongside `.db`

### Query Optimization Tips

1. **Use indexes for filtered queries**:
   - Search by name → `idx_commands_name` helps
   - Filter by category → `idx_commands_category` helps
   - Filter favorites → `idx_commands_favorite` helps

2. **Avoid SELECT * in hot paths**:
   ```sql
   -- Bad (fetches all columns)
   SELECT * FROM commands WHERE group_id = ?;
   
   -- Good (only what you need)
   SELECT id, name, command FROM commands WHERE group_id = ?;
   ```

3. **Use prepared statements** (rusqlite does this automatically):
   ```rust
   let mut stmt = conn.prepare("SELECT * FROM commands WHERE id = ?")?;
   stmt.query_row(params![id], |row| { ... })?;
   ```

4. **Batch inserts in transactions**:
   ```rust
   let tx = conn.transaction()?;
   for cmd in commands {
       tx.execute("INSERT INTO commands ...", params![...])?;
   }
   tx.commit()?;
   ```

### Expected Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| Single command lookup | < 1ms | Indexed by PRIMARY KEY |
| Search 1000 commands | < 50ms | Uses `idx_commands_name` |
| List all commands in group | < 10ms | Uses `idx_commands_group` |
| Recursive group tree query | < 100ms | CTE for 10-level nesting |
| Apply template (create 20 commands) | < 200ms | Batched in transaction |

---

## 13.9 Data Integrity

### Foreign Key Constraints

**Enabled by default**:
```sql
PRAGMA foreign_keys = ON;
```

**Enforced relationships**:
- Delete group → CASCADE deletes all child groups and commands
- Delete category → Commands/groups with that category_id become NULL
- Invalid parent_group_id → Insert/update rejected

**Example**:
```sql
-- This will fail if parent_group_id=999 doesn't exist
INSERT INTO groups (name, parent_group_id) VALUES ('Child', 999);
-- Error: FOREIGN KEY constraint failed
```

### Circular Reference Prevention

**Not enforced by database** - must be checked in application code:

```rust
fn validate_parent_id(id: i64, parent_id: i64, conn: &Connection) -> Result {
    if id == parent_id {
        return Err("Group cannot be its own parent".into());
    }
    
    // Walk up the tree to detect cycles
    let mut current = parent_id;
    let mut visited = HashSet::new();
    
    while let Some(parent) = get_parent_id(current, conn)? {
        if visited.contains(&parent) || parent == id {
            return Err("Circular reference detected".into());
        }
        visited.insert(parent);
        current = parent;
        
        // Safety: prevent infinite loops
        if visited.len() > 100 {
            return Err("Group nesting too deep".into());
        }
    }
    
    Ok(())
}
```

### Data Validation

**Application-level checks** (not in database):
- Command name not empty
- Command executable exists (warn if not)
- Working directory exists (or default to $HOME)
- Environment variable keys valid (alphanumeric + underscore)
- Arguments array well-formed JSON

---

## 13.10 Backup & Recovery

### Manual Backup

```bash
# Simple file copy (stop app first)
cp ~/.local/share/tgui/tgui.db ~/.local/share/tgui/tgui.db.backup

# Or use SQLite backup command
sqlite3 ~/.local/share/tgui/tgui.db ".backup tgui.db.backup"
```

### Automatic Backup (Before Migration)

```rust
fn backup_before_migration(db_path: &Path) -> Result {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = db_path.with_file_name(
        format!("tgui.db.backup_{}", timestamp)
    );
    
    std::fs::copy(db_path, &backup_path)?;
    
    // Keep only last 5 backups
    cleanup_old_backups(db_path.parent().unwrap(), 5)?;
    
    Ok(backup_path)
}
```

### Recovery from Backup

```bash
# If database is corrupted
cd ~/.local/share/tgui/
mv tgui.db tgui.db.corrupted
cp tgui.db.backup tgui.db
```

### Export to JSON (User-Initiated)

```rust
#[tauri::command]
fn export_all_data() -> Result {
    let commands = db.list_commands()?;
    let groups = db.list_groups()?;
    let categories = db.list_categories()?;
    
    let export = json!({
        "version": "2.0.0",
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "commands": commands,
        "groups": groups,
        "categories": categories,
    });
    
    Ok(serde_json::to_string_pretty(&export)?)
}
```

---

## 13.11 Database Health Checks

### Integrity Check

```sql
-- Run on app startup
PRAGMA integrity_check;
-- Should return: ok
```

### Statistics

```sql
-- Check database size
SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size();

-- Count records per table
SELECT 'commands' as table_name, COUNT(*) as count FROM commands
UNION ALL
SELECT 'groups', COUNT(*) FROM groups
UNION ALL
SELECT 'categories', COUNT(*) FROM categories
UNION ALL
SELECT 'templates', COUNT(*) FROM templates;
```

### Vacuum (Reclaim Space)

```sql
-- Run occasionally to defragment
VACUUM;
```

**Note**: Vacuum locks database, so run when no commands are running

---

## 13.12 Troubleshooting

### Database Locked Error

**Cause**: Another connection has exclusive lock

**Solutions**:
1. Enable WAL mode (already done by default)
2. Reduce transaction duration
3. Retry with exponential backoff

```rust
fn execute_with_retry(mut op: F) -> Result
where
    F: FnMut() -> Result,
{
    for attempt in 1..=3 {
        match op() {
            Ok(result) => return Ok(result),
            Err(e) if e.to_string().contains("database is locked") => {
                std::thread::sleep(Duration::from_millis(100 * attempt));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err("Database locked after 3 retries".into())
}
```

### Corrupted Database

**Symptoms**:
- `PRAGMA integrity_check` returns errors
- Random query failures
- App crashes on startup

**Recovery**:
1. Restore from backup (see 13.10)
2. If no backup, try `.recover` command:
   ```bash
   sqlite3 tgui.db.corrupted ".recover" | sqlite3 tgui.db.recovered
   ```
3. Manually export/import data

### Missing Indexes

**Symptom**: Slow queries on large datasets

**Fix**: Recreate indexes
```sql
DROP INDEX IF EXISTS idx_commands_name;
CREATE INDEX idx_commands_name ON commands(name);
```



## 13.13 SQL Schema File

**Full schema available at**: `src-tauri/sql/schema.sql`

**Load schema**:
```rust
// In database initialization
conn.execute_batch(include_str!("../sql/schema.sql"))?;
```

**Generate schema dump**:
```bash
sqlite3 tgui.db .schema > schema_dump.sql
```

---

## 13.16 Summary

**Key Points**:
- Separate tables for clear entity separation
- Foreign keys enforced for data integrity
- WAL mode for concurrent access
- Indexes on all filtered columns
- Settings inheritance resolved in application layer
- Migrations tracked with `schema_version`
- Backups before schema changes

**File Size Estimate**:
- Empty database: ~50KB
- With 1000 commands + 50 groups: ~500KB
- With 10,000 commands: ~5MB
