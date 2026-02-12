# 8. Security Considerations

### Update Log

- **12-12-2025**: Initial security guidelines

---

**Threat Model**: TGUI runs locally on user machine with full user privileges. Primary risks are
accidental data loss, command injection, unintended execution and insecure storage of sensitive
data. So security is critical even for personal use.

---

## 1. Environment Variables (FR-09)

### Risk: Plain Text Storage

- **Issue**: Environment variables stored in SQLite without encryption
- **Impact**: Malware or unauthorized users could read sensitive data (API keys, tokens)
- **Mitigation**:
    - **UI Warning**: Display prominent warning when adding env vars:
      > "Security Notice: Environment variables are stored in plain text. Avoid storing passwords,
      API keys, or other sensitive data unless you accept this risk."
    - **Future**: v0.2.0 may integrate OS keyring (`keyring` crate) for sensitive values
    - **Alternative**: Users can source env files: `source .env && mycommand` (not stored in DB)

### Recommended Usage Pattern

```bash
# Good - API key not stored
echo "Enter API key:"
read API_KEY
export API_KEY
mycommand

# Acceptable - Non-sensitive
export NODE_ENV=development
mycommand

# Bad (for MVP) - Sensitive data
export AWS_SECRET_ACCESS_KEY=xxx  # Will be stored plain text
```

### 1.1 Group Inheritance Security

**Risk**: default_env_vars is set to dangerous values

**Mitigation**:

- Display inherited settings clearly in UI ("This command inherits: ...")
- Warn if group has env vars with "sudo", "password", "token" in names
- Track which commands inherit from which groups (for debugging)

**Example warning**:

```
Group "System Utils" has environment variable "LD_PRELOAD"
This can be used to inject malicious code. Only use if you trust this group.

[Cancel] [I Understand, Save Anyway]
```

---

## 2. Command Injection

### Risk: Malicious command arguments

- **Issue**: User could accidentally paste dangerous commands
- **Impact**: Data loss (`rm -rf /`), privilege escalation
- **Mitigation**:
    - **Denylist**: Block commands with these patterns (show warning):
      ```regex
      rm\s+-rf\s+/.*       # Recursive root deletion
      :\(\)\{.*\}          # Fork bomb
      mkfs\..*             # Filesystem formatting
      ```
    - **Escape Arguments**: Use `std::process::Command` safely (avoids shell injection)
    - **No Shell Interpretation**: Execute commands directly, not via `sh -c`
    - **Confirmation**: Show warning for dangerous commands:
      ```
      ⚠️ This command looks dangerous. Are you sure?
      Command: rm -rf /tmp/*
      ```

### Implementation (Rust)

```rust
fn is_dangerous_command(cmd: &str) -> bool {
    let dangerous = [
        r"rm\s+-rf\s+/",
        r":\(\)\{",
        r"mkfs\.",
    ];
    dangerous.iter().any(|pattern| {
        regex::Regex::new(pattern).unwrap().is_match(cmd)
    })
}
// usage
if is_dangerous_command( & command) {
  return Err(TguiError::DangerousCommand(
        "This command may be destructive. Please confirm.".to_string()
  ));
}
```

### 2.1 Warning on Root Execution

```rust
// In main.rs
fn main() {
    #[cfg(unix)]
    {
        if nix::unistd::geteuid().is_root() {
            eprintln!("⚠️  WARNING: Running as root is dangerous and not recommended.");
            eprintln!("This will allow spawned commands to modify system files.");
            std::process::exit(1);
        }
    }
    // ...
}
```

---

## 3. File System Access

### 3.1 Path traversal attacks

- **Issue**: Malicious working directory could escape sandbox
- **Impact**: Read/write arbitrary files, execute from untrusted location
- **Mitigation**:
    - **Validate Paths**: Ensure working directory is absolute and exists
    - **No Relative Paths**: Convert relative paths to absolute on save
    - **Sanitize**: Reject paths with `..` that escape intended root
    - **Permissions**: Check directory is readable/executable before spawning

**Path Validation**:

```rust
fn validate_working_directory(path: &str) -> Result<PathBuf, TguiError> {
    let path = PathBuf::from(path);

    // Must be absolute
    if !path.is_absolute() {
        return Err(TguiError::InvalidPath(
            "Working directory must be absolute".to_string()
        ));
    }

    // Check for path traversal
    if path.components().any(|c| c == Component::ParentDir) {
        return Err(TguiError::InvalidPath(
            "Path traversal not allowed".to_string()
        ));
    }

    Ok(path)
}
```

### 3.2 Log File Security

```bash
# If you ever write logs to disk (not planned, but if you do):
chmod 600 ~/.config/yourapp/logs/*.log
chown $USER:$USER ~/.config/yourapp/logs/
```

### 3.3 Template Variable Validation

**Risk**: User imports template with `{{directory}}` → `/etc/passwd`

**Attack Scenario**:

1. Attacker creates template: `"directory": "{{project_dir}}/../../etc"`
2. User applies template, enters `/home/user/project`
3. Path resolves to `/etc` (via `..` traversal)
4. Commands run in system directory

**Mitigation**:

- Validate all substituted paths against whitelist (home directory, /tmp only)
- Preview all commands before template application
- Warn if any command targets system directories

**Mitigation**:

- **Path canonicalization**: Resolve all paths with `std::fs::canonicalize()` before use
- **Whitelist validation**: Only allow paths under:
    - User's home directory (`$HOME`)
    - `/tmp` (with warning)
    - Explicit user-created directories
- **Preview step**: Show all resolved commands before creating them
- **System directories**: Reject or warn `/etc`, `/sys`, `/proc`, `/boot`, `C:\Windows`

**Implementation**: Path validation function checks resolved paths don't escape user space

**Example validation**:

```rust
fn validate_template_path(path: &str, base: &Path) -> Result {
    let resolved = base.join(path).canonicalize()?;

    // Check it's still under base directory
    if !resolved.starts_with(base) {
        return Err("Path escapes base directory");
    }

    // Check against system directory blacklist, set from user settings
    let blocked = ["/etc", "/sys", "/proc", "/boot"];
    for dir in blocked {
        if resolved.starts_with(dir) {
            return Err("System directories not allowed");
        }
    }

    Ok(resolved)
}
```

**UI Flow**:

```
[Apply Template]
  ↓
[Show Preview: 4 commands will be created]
  - Create venv: /home/user/project/venv
  - Install deps: /home/user/project
  ⚠️ Warning: Command "Install" uses inherited directory
  ↓
[User clicks "Confirm"] → Commands created
```

---

## 4. Database Security

### Risk: Database file permissions

- **Issue**: `tgui.db` may be readable by other users
- **Impact**: Leak commands, env vars, directory structure
- **Mitigation**:
    - **File Permissions**: Set `0600` (user-only read/write) on Unix
    - **Location**: Store in user-specific config directory (not world-readable)
    - **Encryption**: Future versions may encrypt backups

### Database Location & Permissions

```rust
fn ensure_secure_db_path() -> PathBuf {
    let db_path = get_config_dir().join("tgui.db");

    // Create with secure permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o600)  // rw-------
            .open(&db_path)
            .expect("Failed to create database");
    }

    db_path
}
```

---

## 5. Process Isolation

### 5.1 Processes can affect each other

- **Issue**: Long-running process may consume all resources
- **Impact**: System slowdown, app unresponsive
- **Mitigation**:
    - **Resource Limits**: Set `RLIMIT_NPROC` soft limit (Unix) to prevent fork bombs
    - **Kill Switch**: "Stop All Processes" always works
    - **Timeout**: Warn if process runs >24h (possible zombie)
    - **CPU/Memory**: Monitor process RSS, show warning if >1GB per process

### 5.2 Command Whitelisting (Optional Feature)

```json
{
  "allowed_commands": [
    "npm",
    "python",
    "tail",
    "grep"
  ],
  // If empty, allow all
  "blocked_commands": [
    "rm",
    "dd",
    "mkfs"
  ]
  // Explicitly forbid dangerous commands
}
```

```rust
// In Rust backend
#[tauri::command]
fn spawn_command(cmd: String) -> Result<u32, String> {
    // Check against blacklist
    if config.blocked_commands.contains(&cmd) {
        return Err("Command blocked for security".into());
    }
    // ...
}
```

---

## 6. Export/Import Security

### Risk: Malicious template files

- **Issue**: JSON template could contain dangerous commands
- **Impact**: Command injection on import
- **Mitigation**:
    - **Schema Validation**: Validate against JSON Schema (ADR-006)
    - **Review Before Import**: Show preview dialog with all commands
    - **Warning**: If any command matches dangerous patterns, block import
    - **Sandbox**: Import into "Imported" category (not auto-executed)

---

## 7. Update Security (Future)

For v0.2.0+ when auto-update is added:

- **Signature Verification**: Sign binaries with GPG key
- **HTTPS Only**: Download updates only over TLS
- **Checksums**: Verify SHA256 before installing
- **User Consent**: Prompt before auto-updating

### Signature Verification

```tauri-conf
{
  "plugins": {
    "updater": {
      "pubkey": "YOUR_ED25519_PUBLIC_KEY",
      "endpoints": [
        "https://yourdomain.com/releases.json"
      ]
    }
  }
}
```

## 8. GitHub Security Features

### 8.1 Dependency Audit

**Run weekly**:

```bash
cargo audit                    # Check Rust deps for CVEs
pnpm audit                     # Check Node deps
```

**In CI**:

```yaml
- name: Security audit
  run: |
    cargo install cargo-audit
    cargo audit
    pnpm audit --audit-level high
```

### 8.2 CodeQL Analysis

Enable in GitHub: Settings → Security → Code scanning → Set up CodeQL

---

## 9. Logging Security

### 9.1 Sensitive Data in Logs

**Risk**: Command text, arguments, and environment variables may contain credentials

**Mitigation**:

- **Never log**:
    - Full command text (may contain passwords: `mysql -p MyPassword123`)
    - Command arguments
    - Environment variable values
    - Search terms (user might search for sensitive strings)

- **Do log**:
    - Command ID/name
    - Metadata: argument count, env var count, name length
    - Operation results: success/failure, affected rows
    - Timestamps and durations

**Example - Bad**:

```rust
info!("Executing command: {}", cmd.command);  // May leak password
debug!("Arguments: {:?}", cmd.arguments);   // May leak data
```

**Example - Good**:

```rust
info!(
    command_id = cmd.id,
    arg_count = cmd.arguments.len(),
    has_env_vars = cmd.env_vars.is_some(),
    "Command execution started"
);
```

### 9.2 Log File Security

**Automatic Cleanup**:

- Logs older than 30 days deleted on startup
- Daily log rotation (prevents single huge file)
- TODO: Make retention period configurable in settings

**File Permissions** (Unix):

```rust
#[cfg(unix)]
fn user_permission() {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&logs_dir)?.permissions();
    perms.set_mode(0o700); // Only owner can read/write/execute
    fs::set_permissions(&logs_dir, perms)?;
}
```

**Log Management**:

- Users can delete logs via UI (list, delete by date, delete all)
- `get_recent_logs(days, lines_per_day)`: Fetch last N days of logs
- Logs excluded from exports (don't leak in backups)

### 9.3 Execution History Security

**What's Stored**:

- Execution status, exit code, timestamps
- Which command/workflow executed
- Triggered by (manual/workflow/schedule)
- Optional context JSON (user-provided notes)

**What's NOT Stored**:

- stdout/stderr output (too large, may contain secrets)
- Full command text (already in commands table)
- Environment variables (already in commands table)

### 9.4 Database Security Updates

**Security Constraints**:

- Execution history cascade deletes when command/workflow deleted
- Triggers prevent modifying completed executions
- CHECK constraints enforce valid status transitions

**Cleanup**:

```rust
// Keep only last 100 executions per command
db.cleanup_command_executions(command_id, 100) ?;

// Delete executions older than 90 days
db.cleanup_old_executions(90) ?;
```

## 10. User Education

**Add to README.md**:

```markdown
## Security Notice

[YourAppName] executes commands you provide. Always review commands before running.

- Never run untrusted scripts (e.g., `curl http://example.com | bash`)
- The app respects your system permissions; don't run as root
- Logs are stored locally and not uploaded
```

**Add to app on first launch**:

```javascript
// In main.ts
if (!localStorage.getItem('hasSeenSecurityWarning')) {
    alert('⚠️ This app runs shell commands. Only execute code you trust.');
    localStorage.setItem('hasSeenSecurityWarning', 'true');
}
```

---

## Security Checklist for Code Review

Before merging any PR:

- [ ] New env var usage shows security warning
- [ ] Command strings use `std::process::Command` (not `sh -c`)
- [ ] Paths are validated and absolute
- [ ] File permissions set to 0600 for sensitive files
- [ ] No dangerous commands added without confirmation
- [ ] Error messages don't leak internal paths (unless user-provided)
- [ ] Input validation on all user inputs (command name, args, directory)
- [ ] Command/workflow logging uses no args/env vars/command text
- [ ] Execution history doesn't store sensitive output

---

## Incident Response

If security vulnerability discovered:

0. **Try to acknowledge within 24h** (even if just "we're investigating")
1. **Assess**: Reproduce and determine impact
2. **Patch**: Fix in private branch
3. **Disclose** Immediately if critical or after 30 days (if user agrees)
4. **Release**: Push security update (patch version bump: v1.0.0 → v1.0.1)
5. **Document**: Add to documentation

<!--TODO: add contact info/email-->

---

## Compliance Notes

- **GDPR**: TGUI stores data locally only, no cloud sync → no GDPR requirements
- **CIS Benchmarks**: Follows principle of least privilege (user-level execution)
- **No Telemetry**: App doesn't send usage data (only download count from source)

---
