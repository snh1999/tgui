# 11. Template Specification

### Update Log
- **13-12-2025**: Initial template specifications

## 11.1 Overview

Templates are **reusable blueprints** for creating sets of commands and groups. They use variable substitution to customize commands for different projects.

**Use Cases**:
- **New project setup**: Apply "Python Dev" template to `/home/user/my-ml-project`
- **Team sharing**: Export your workflow as JSON, teammates import it
- **Multi-environment**: Create "Deploy to Prod" template with environment-specific variables

---

## 11.2 Template Structure

Templates are stored as JSON with this structure:

```json
{
  "$schema": "https://tgui.dev/schemas/template-v1.json",
  "name": "Python Development",
  "version": "1.0.0",
  "description": "Common commands for Python projects",
  "author": "user@example.com",
  "variables": [
    {
      "key": "project_dir",
      "label": "Project Directory",
      "type": "path",
      "required": true,
      "description": "Root directory of your Python project"
    },
    {
      "key": "venv_name",
      "label": "Virtual Environment Name",
      "type": "string",
      "default": "venv",
      "required": false
    }
  ],
  "groups": [
    {
      "name": "Backend",
      "description": "Backend server commands",
      "default_working_directory": "{{project_dir}}/backend",
      "default_env_vars": {
        "PYTHON_ENV": "development",
        "DEBUG": "true"
      },
      "commands": [
        {
          "name": "Install Dependencies",
          "command": "pip",
          "arguments": ["install", "-r", "requirements.txt"],
          "description": "Install Python packages"
        }
      ],
      "groups": []
    }
  ],
  "commands": [
    {
      "name": "Create Virtual Environment",
      "command": "python",
      "arguments": ["-m", "venv", "{{venv_name}}"],
      "working_directory": "{{project_dir}}",
      "category": "Python",
      "env_vars": null
    }
  ]
}
```

---

## 11.3 Field Definitions

### Template-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `$schema` | string | No | JSON Schema URL for validation |
| `name` | string | Yes | Template display name |
| `version` | string | Yes | Semantic version (e.g., "1.0.0") |
| `description` | string | No | What this template is for |
| `author` | string | No | Creator email or username |
| `variables` | array | Yes | List of customizable variables |
| `groups` | array | No | Nested group structures |
| `commands` | array | No | Top-level commands (not in any group) |

### Variable Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | string | Yes | Variable name (used in `{{key}}`) |
| `label` | string | Yes | Human-readable label for UI |
| `type` | enum | Yes | `"string"`, `"path"`, or `"number"` |
| `default` | string | No | Default value if user doesn't provide |
| `required` | boolean | Yes | If true, user must provide value |
| `description` | string | No | Help text shown in UI |

**Variable Types**:
- `"string"`: Free-form text input
- `"path"`: Directory picker (validates path exists)
- `"number"`: Numeric input (validates is number)

### Group Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Group display name |
| `description` | string | No | Group purpose |
| `default_working_directory` | string | No | Default for child commands (can use `{{variables}}`) |
| `default_env_vars` | object | No | Key-value pairs inherited by children |
| `default_shell` | string | No | Shell path (e.g., "/bin/zsh") |
| `commands` | array | No | Commands in this group |
| `groups` | array | No | Nested child groups (recursive) |

### Command Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Command display name |
| `command` | string | Yes | Executable name or path |
| `arguments` | array | No | List of string arguments |
| `description` | string | No | Command purpose |
| `working_directory` | string | No | Override group's directory |
| `category` | string | No | Category name (created if doesn't exist) |
| `env_vars` | object | No | Additional env vars (merged with group's) |

---

## 11.4 Variable Substitution

Variables use `{{key}}` syntax and are replaced when template is applied.

**Substitution Rules**:
1. Variables can appear in any string field
2. Multiple variables in one field: `"{{dir}}/{{subdir}}/file.txt"`
3. If variable is required but missing → error before creating any commands
4. If variable has default and user doesn't provide → use default
5. After substitution, paths are validated (see Security section)

**Example**:
```json
{
  "variables": [
    {"key": "project_dir", "type": "path", "required": true},
    {"key": "port", "type": "number", "default": "8000"}
  ],
  "commands": [{
    "name": "Start Server",
    "command": "python",
    "arguments": ["-m", "http.server", "{{port}}"],
    "working_directory": "{{project_dir}}/public"
  }]
}
```

**User applies with**:
- `project_dir = "/home/user/myapp"`
- `port = "3000"` (overrides default)

**Result**:
```json
{
  "name": "Start Server",
  "command": "python",
  "arguments": ["-m", "http.server", "3000"],
  "working_directory": "/home/user/myapp/public"
}
```

---

## 11.5 Nested Groups

Groups can contain other groups (unlimited depth):

```json
{
  "groups": [
    {
      "name": "Web Project",
      "default_working_directory": "{{project_dir}}",
      "groups": [
        {
          "name": "Frontend",
          "default_working_directory": "{{project_dir}}/frontend",
          "commands": [
            {"name": "npm install", "command": "npm", "arguments": ["install"]}
          ]
        },
        {
          "name": "Backend",
          "default_working_directory": "{{project_dir}}/backend",
          "commands": [
            {"name": "pip install", "command": "pip", "arguments": ["install", "-r", "requirements.txt"]}
          ]
        }
      ]
    }
  ]
}
```

**Application result**:
```
Group: Web Project (/home/user/myapp)
  ├─ Group: Frontend (/home/user/myapp/frontend)
  │   └─ Command: npm install
  └─ Group: Backend (/home/user/myapp/backend)
      └─ Command: pip install
```

---

## 11.6 Built-in Templates

TGUI ships with these templates (in `templates/` directory):

### Python Development
```json
{
  "name": "Python Development",
  "variables": [
    {"key": "project_dir", "label": "Project Directory", "type": "path", "required": true},
    {"key": "venv_name", "label": "Virtual Environment", "type": "string", "default": "venv"},
    {"key": "python_version", "label": "Python Version", "type": "string", "default": "python3"}
  ],
  "commands": [
    {
      "name": "Create venv",
      "command": "{{python_version}}",
      "arguments": ["-m", "venv", "{{venv_name}}"],
      "working_directory": "{{project_dir}}",
      "category": "Python"
    },
    {
      "name": "Activate venv",
      "command": "source",
      "arguments": ["{{venv_name}}/bin/activate"],
      "working_directory": "{{project_dir}}",
      "category": "Python"
    },
    {
      "name": "Install deps",
      "command": "{{project_dir}}/{{venv_name}}/bin/pip",
      "arguments": ["install", "-r", "requirements.txt"],
      "working_directory": "{{project_dir}}",
      "category": "Python"
    },
    {
      "name": "Run tests",
      "command": "{{project_dir}}/{{venv_name}}/bin/pytest",
      "arguments": [],
      "working_directory": "{{project_dir}}",
      "category": "Python"
    }
  ]
}
```

### Node.js Development
```json
{
  "name": "Node.js Development",
  "variables": [
    {"key": "project_dir", "type": "path", "required": true},
    {"key": "package_manager", "type": "string", "default": "npm"}
  ],
  "commands": [
    {
      "name": "Install deps",
      "command": "{{package_manager}}",
      "arguments": ["install"],
      "working_directory": "{{project_dir}}",
      "category": "Node.js"
    },
    {
      "name": "Start dev server",
      "command": "{{package_manager}}",
      "arguments": ["run", "dev"],
      "working_directory": "{{project_dir}}",
      "category": "Node.js"
    },
    {
      "name": "Build production",
      "command": "{{package_manager}}",
      "arguments": ["run", "build"],
      "working_directory": "{{project_dir}}",
      "category": "Node.js"
    }
  ]
}
```

### Docker Compose
```json
{
  "name": "Docker Compose",
  "variables": [
    {"key": "project_dir", "label": "Directory with docker-compose.yml", "type": "path", "required": true}
  ],
  "commands": [
    {
      "name": "Build containers",
      "command": "docker-compose",
      "arguments": ["build"],
      "working_directory": "{{project_dir}}",
      "category": "Docker"
    },
    {
      "name": "Start services",
      "command": "docker-compose",
      "arguments": ["up", "-d"],
      "working_directory": "{{project_dir}}",
      "category": "Docker"
    },
    {
      "name": "View logs",
      "command": "docker-compose",
      "arguments": ["logs", "-f"],
      "working_directory": "{{project_dir}}",
      "category": "Docker"
    },
    {
      "name": "Stop services",
      "command": "docker-compose",
      "arguments": ["down"],
      "working_directory": "{{project_dir}}",
      "category": "Docker"
    }
  ]
}
```

---

## 11.7 Template vs Group Export

**Key Difference**:
- **Template**: Has `variables` → user must substitute before use
- **Group Export**: No `variables` → ready to import directly

**Same JSON format**, just:
```json
// Template (with variables)
{
  "name": "Python Dev Template",
  "variables": [{"key": "project_dir", ...}],
  "commands": [{"working_directory": "{{project_dir}}", ...}]
}

// Group Export (no variables)
{
  "name": "My ML Project Commands",
  "variables": [],
  "commands": [{"working_directory": "/home/user/ml-project", ...}]
}
```

**Import behavior**:
- If `variables` is empty → import directly (no substitution step)
- If `variables` has items → prompt user for values before import

---

## 11.8 Validation

Templates are validated on import:

**Schema Validation**:
- All required fields present
- Field types correct (string vs array vs object)
- Version format is semantic versioning

**Security Validation**:
- No commands with dangerous patterns (`rm -rf`, `:(){ :|:& };:`)
- Paths don't escape user space after substitution
- No `sudo` or privilege escalation attempts

**Business Logic Validation**:
- Variable keys are valid identifiers (alphanumeric + underscore)
- All `{{variables}}` used in template are defined
- Group nesting depth < 10 (prevent deep recursion)

**Validation Errors**:
```json
{
  "valid": false,
  "errors": [
    "Variable 'project-dir' has invalid key (use underscores, not hyphens)",
    "Command 'Install' uses undefined variable '{{python_ver}}' (did you mean '{{python_version}}'?)",
    "Group 'Backend' contains dangerous command: 'rm -rf /'"
  ]
}
```

---

## 11.9 Migration & Versioning

**Version 1.0.0** (current):
- Initial template format

**Future versions**:
- `1.1.0`: Add `default_shell` to groups
- `2.0.0`: Breaking change (if needed)

**Backward compatibility**:
- TGUI will support templates from v1.x indefinitely
- When importing older version, auto-upgrade to current format
- Export always uses latest version

**Example migration** (if v2.0.0 changes structure):
```rust
fn migrate_template(json: &str) -> Template {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let version = value["version"].as_str().unwrap();
    
    match version {
        "1.0.0" | "1.1.0" => {
            // Auto-upgrade to 2.0.0 format
            upgrade_v1_to_v2(value)
        },
        "2.0.0" => {
            // Use directly
            serde_json::from_value(value)?
        },
        _ => Err("Unsupported template version")
    }
}
```
