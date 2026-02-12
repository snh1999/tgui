-- Migration script not required for now, as app is not released yet
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    icon TEXT,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK (length(trim(name)) > 0)
);

CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    parent_group_id INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    working_directory TEXT,
    env_vars TEXT,
    shell TEXT,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT 0 CHECK(is_favorite IN (0,1)),
    icon TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK (parent_group_id IS NULL OR parent_group_id != id),
    CHECK (length(trim(name)) > 0),
    CHECK (env_vars IS NULL OR json_valid(env_vars))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_groups_position_unique ON groups(COALESCE(parent_group_id, -1), position);

CREATE TABLE IF NOT EXISTS commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    arguments TEXT,
    description TEXT,
    group_id INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    working_directory TEXT,
    env_vars TEXT,
    shell TEXT,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT 0 CHECK(is_favorite IN (0,1)),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK (length(trim(name)) > 0),
    CHECK (length(trim(command)) > 0),
    CHECK (env_vars IS NULL OR json_valid(env_vars)),
    CHECK (arguments IS NULL OR json_valid(arguments))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_commands_position_unique ON commands(COALESCE(group_id, -1), position);

CREATE TABLE IF NOT EXISTS templates (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     name TEXT NOT NULL,
     description TEXT,
     author TEXT,
     structure TEXT NOT NULL,
     created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
     updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS workflows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT 0 CHECK(is_favorite IN (0,1)),
    execution_mode TEXT NOT NULL,
    position INTEGER NOT NULL, -- for rearranging
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK (length(trim(name)) > 0),
    CHECK(execution_mode IN ('sequential', 'parallel', 'conditional'))
);

CREATE TABLE IF NOT EXISTS workflow_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    command_id INTEGER NOT NULL REFERENCES commands(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    condition TEXT NOT NULL DEFAULT 'always',
    timeout_seconds INTEGER,
    auto_retry_count INTEGER DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
    continue_on_failure BOOLEAN NOT NULL DEFAULT 0 CHECK(continue_on_failure IN (0,1)),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK(condition IN ('always', 'on_success', 'on_failure'))
);

CREATE TABLE IF NOT EXISTS execution_history (
    id INTEGER PRIMARY KEY,
    command_id INTEGER REFERENCES commands(id) ON DELETE CASCADE,
    workflow_id INTEGER REFERENCES workflows(id) ON DELETE CASCADE,
    workflow_step_id INTEGER REFERENCES workflow_steps(id) ON DELETE CASCADE,
    status TEXT,
    exit_code INTEGER,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    triggered_by TEXT,
    context TEXT,
    stop_reason TEXT,
    CHECK (
        (command_id IS NOT NULL AND workflow_id IS NULL AND workflow_step_id IS NULL) OR
        (command_id IS NULL AND workflow_id IS NOT NULL AND workflow_step_id IS NULL) OR
        (command_id IS NOT NULL AND workflow_id IS NOT NULL AND workflow_step_id IS NOT NULL)
    ),
    CHECK(status IN ('running', 'success', 'failed', 'timeout', 'cancelled', 'skipped')),
    CHECK(triggered_by IN ('manual', 'workflow', 'schedule')),
    CHECK(stop_reason IN ('error', 'user_cancelled', 'timeout', 'completed'))
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

INSERT OR REPLACE INTO schema_version (version) VALUES (1);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_groups_position ON groups(parent_group_id, position);
CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_group_id);
CREATE INDEX IF NOT EXISTS idx_groups_name ON groups(name);

CREATE INDEX IF NOT EXISTS idx_commands_category ON commands(category_id);
CREATE INDEX IF NOT EXISTS idx_commands_favorite ON commands(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_commands_position ON commands(group_id, position);
CREATE INDEX IF NOT EXISTS idx_commands_name ON commands(name);

CREATE INDEX IF NOT EXISTS idx_workflows_name ON workflows(name);
CREATE INDEX IF NOT EXISTS idx_workflows_category ON workflows(category_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow ON workflow_steps(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_command ON workflow_steps(command_id);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_position ON workflow_steps(workflow_id, command_id, position);

CREATE INDEX IF NOT EXISTS idx_execution_history_command ON execution_history(command_id);
CREATE INDEX IF NOT EXISTS idx_execution_history_workflow ON execution_history(workflow_id);
CREATE INDEX IF NOT EXISTS idx_execution_history_workflow_step ON execution_history(command_id, workflow_id, workflow_step_id);
CREATE INDEX IF NOT EXISTS idx_execution_history_status ON execution_history(status);
CREATE INDEX IF NOT EXISTS idx_execution_history_command_status ON execution_history(command_id, status);
CREATE INDEX IF NOT EXISTS idx_execution_history_workflow_status ON execution_history(workflow_id, status);
CREATE INDEX IF NOT EXISTS idx_execution_history_time ON execution_history(completed_at);

-- Triggers
-- Updated At time
CREATE TRIGGER IF NOT EXISTS groups_update_timestamp
AFTER UPDATE ON groups
BEGIN
UPDATE groups SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS commands_update_timestamp
AFTER UPDATE ON commands
BEGIN
UPDATE commands SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS templates_update_timestamp
AFTER UPDATE ON templates
BEGIN
UPDATE templates SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS workflows_update_timestamp
AFTER UPDATE ON workflows
BEGIN
UPDATE workflows SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS workflow_steps_update_timestamp
AFTER UPDATE ON workflow_steps
BEGIN
UPDATE workflow_steps SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS execution_history_timestamps
AFTER UPDATE OF status ON execution_history
BEGIN
UPDATE execution_history
SET
    started_at = CASE
        WHEN NEW.status = 'running' AND OLD.status != 'running'
            THEN CURRENT_TIMESTAMP
        ELSE started_at
    END,
    completed_at = CASE
       WHEN NEW.status IN ('success', 'failed', 'timeout', 'cancelled')
           AND OLD.status = 'running'
           THEN CURRENT_TIMESTAMP
       ELSE completed_at
    END
WHERE id = NEW.id;
END;
