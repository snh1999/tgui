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

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

INSERT OR REPLACE INTO schema_version (version) VALUES (1);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_groups_position ON groups(parent_group_id, position);
CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_group_id);


CREATE INDEX IF NOT EXISTS idx_commands_category ON commands(category_id);
CREATE INDEX IF NOT EXISTS idx_commands_favorite ON commands(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_commands_position ON commands(group_id, position);
CREATE INDEX IF NOT EXISTS idx_commands_name ON commands(name);

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

CREATE TRIGGER IF NOT EXISTS settings_update_timestamp
AFTER UPDATE ON settings
BEGIN
UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
END;
