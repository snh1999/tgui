-- Categories table
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  icon TEXT,
  color TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO categories (id, name, icon) VALUES (0, 'Uncategorized', '📁');

-- Groups table
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    parent_group_id INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    position INTEGER DEFAULT 0,
    working_directory TEXT,
    env_vars TEXT,
    shell TEXT,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CHECK (parent_group_id IS NULL OR parent_group_id != id),
);

-- Commands table
CREATE TABLE IF NOT EXISTS commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    arguments TEXT,
    description TEXT,
    group_id INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    position INTEGER DEFAULT 0,
    working_directory TEXT,
    env_vars TEXT,
    shell TEXT,

     INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    is_favorite BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Templates table
CREATE TABLE IF NOT EXISTS templates (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     name TEXT NOT NULL,
     description TEXT,
     author TEXT,
     structure TEXT NOT NULL,
     updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO settings (key, value) VALUES
('default_shell', '/bin/bash'),
('log_buffer_size', '10000'),
('max_concurrent_processes', '20'),
('auto_scroll_logs', 'true'),
('warn_before_kill', 'true'),
('kill_process_tree_by_default', 'false'),
('theme', 'system');

-- Schema version table
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

INSERT OR IGNORE INTO schema_version (version) VALUES (1);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_groups_position ON groups(parent_group_id, position);
CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_group_id);

CREATE INDEX IF NOT EXISTS idx_commands_category ON commands(category_id);
CREATE INDEX IF NOT EXISTS idx_commands_position ON commands(group_id, position);
CREATE INDEX IF NOT EXISTS idx_commands_name ON commands(name);
CREATE INDEX IF NOT EXISTS idx_commands_group ON commands(group_id);

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

-- to protect default category
CREATE TRIGGER IF NOT EXISTS prevent_uncategorized_delete
BEFORE DELETE ON categories
WHEN OLD.id = 0
BEGIN
SELECT RAISE(ABORT, 'Cannot delete Uncategorized category');
END;