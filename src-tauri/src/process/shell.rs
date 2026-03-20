// TODO: allow shell configuring TGUI-33

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub name: String,
    pub path: String,
}

pub struct Shell;

impl Shell {
    #[cfg(not(target_os = "windows"))]
    const COMMON_SHELLS: &[&str] = &["sh", "bash", "zsh", "fish", "nu", "dash", "ksh", "tcsh"];

    #[cfg(target_os = "windows")]
    const COMMON_SHELLS: &[&str] = &["cmd", "powershell", "pwsh"];

    /// Detect all available shells on the system
    pub fn detect_available_shells() -> Vec<ShellInfo> {
        debug!("Detecting available shells on the system");
        let mut shells = Vec::new();

        for shell_name in Self::COMMON_SHELLS {
            if let Some(shell_info) = Self::check_shell(shell_name) {
                debug!("Found shell: {} at {}", shell_info.name, shell_info.path);
                shells.push(shell_info);
            }
        }

        // Also check for shells in common paths that might not be in PATH
        Self::check_common_paths(&mut shells);

        debug!("Detected {} available shells", shells.len());
        shells
    }

    /// Check if a specific shell is available
    fn check_shell(name: &str) -> Option<ShellInfo> {
        let path = Self::find_in_path(name)?;
        Some(ShellInfo {
            name: name.to_string(),
            path: path.to_string_lossy().into_owned(),
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn find_in_path(command: &str) -> Option<std::path::PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {

            let candidate = dir.join(command);

            if candidate.is_file() {
                use std::os::unix::fs::PermissionsExt;
                if std::fs::metadata(&candidate)
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
                {
                    return Some(candidate);
                }
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    fn find_in_path(command: &str) -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        let pathext = std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string());
        let extensions: Vec<&str> = pathext.split(';').collect();

        for dir in std::env::split_paths(&path_var) {
            for ext in &extensions {
                let candidate = dir.join(format!("{}{}", command, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn check_common_paths(shells: &mut Vec<ShellInfo>) {
        use std::collections::HashSet;
        use std::os::unix::fs::PermissionsExt;

        let common_paths = [
            "/bin",
            "/usr/bin",
            "/usr/local/bin",
            "/opt/homebrew/bin",
            "/usr/pkg/bin",  // NetBSD pkgsrc
            "/opt/local/bin", // MacPorts
        ];

        // Build a set of already-found shell names for O(1) dedup
        let mut seen: HashSet<String> = shells.iter().map(|s| s.name.clone()).collect();

        for dir in &common_paths {
            for shell_name in Self::COMMON_SHELLS {
                if seen.contains(*shell_name) {
                    continue;
                }

                let path = format!("{}/{}", dir, shell_name);
                if !Path::new(&path).exists() { continue; }

                let Ok(metadata) = std::fs::metadata(&path) else {
                    continue;
                };

                // Must be a file or symlink, and must be executable by someone
                let is_file_or_symlink = metadata.is_file();
                let is_executable = metadata.permissions().mode() & 0o111 != 0;

                if is_file_or_symlink && is_executable {
                    debug!("Found shell via common path: {} at {}", shell_name, path);
                    seen.insert(shell_name.to_string());
                    shells.push(ShellInfo {
                        name: shell_name.to_string(),
                        path,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn check_common_paths(shells: &mut Vec<ShellInfo>) {
        use std::collections::HashSet;

        let common_paths = [
            r"C:\Windows\System32",
            r"C:\Windows\System32\WindowsPowerShell\v1.0",
            r"C:\Program Files\PowerShell\7",
            r"C:\Program Files\PowerShell",
            r"C:\Program Files\Git\bin", // Git for Windows ships sh/bash
        ];

        // Build a set of already-found shell names for O(1) dedup
        let mut seen: HashSet<String> = shells.iter().map(|s| s.name.clone()).collect();

        for dir in &common_paths {
            for shell_name in Self::COMMON_SHELLS {
                if seen.contains(*shell_name) {
                    continue;
                }

                // Bind the formatted string first to avoid a dangling reference
                let formatted_exe;
                let exe_name: &str = match *shell_name {
                    "cmd"        => "cmd.exe",
                    "powershell" => "powershell.exe",
                    "pwsh"       => "pwsh.exe",
                    other => {
                        formatted_exe = format!("{}.exe", other);
                        &formatted_exe
                    }
                };

                let path = format!("{}\\{}", dir, exe_name);
                let p = Path::new(&path);

                if !p.exists() {
                    continue;
                }

                let Ok(metadata) = std::fs::metadata(&path) else {
                    continue;
                };

                // On Windows, existence + is_file is sufficient — no permission bits
                if metadata.is_file() {
                    debug!("Found shell via common path: {} at {}", shell_name, path);
                    seen.insert(shell_name.to_string());
                    shells.push(ShellInfo {
                        name: shell_name.to_string(),
                        path,
                    });
                }
            }
        }
    }

    /// Get the default shell for the current platform
    pub fn get_system_default_shell() -> String {
        #[cfg(target_os = "windows")]
        {
            // Check if PowerShell 7+ is available, fallback to Windows PowerShell, then cmd
            if Self::check_shell("pwsh").is_some() {
                return "pwsh".to_string();
            }
            if Self::check_shell("powershell").is_some() {
                return "powershell".to_string();
            }
            return "cmd".to_string();
        }

        #[cfg(not(target_os = "windows"))]
        {

            if let Ok(shell) = std::env::var("SHELL") {
                if let Some(name) = Path::new(&shell).file_name().and_then(|n| n.to_str()) {
                    if Self::check_shell(name).is_some() {
                        return name.to_string();
                    }
                }
            }

            let shell_output = Command::new("sh")
                .arg("-c")
                .arg("ps -p $$ -o 'comm='")
                .output();

            if let Ok(output) = shell_output {
                if output.status.success() {
                    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !name.is_empty() && Self::check_shell(&name).is_some() {
                        return name;
                    }
                }
            }


            for shell in Self::COMMON_SHELLS {
                if Self::check_shell(shell).is_some() {
                    return shell.to_string();
                }
            }
            "sh".to_string()
        }
    }
}

static SHELL_CACHE: LazyLock<Mutex<Option<(Vec<ShellInfo>, SystemTime)>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn get_shells_cached() -> Vec<ShellInfo> {
    let mut cache = SHELL_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let now = SystemTime::now();

    if let Some((ref shells, timestamp)) = *cache {
        if now.duration_since(timestamp).unwrap_or_default() < Duration::from_secs(3600) {
            return shells.clone(); // Return cached
        }
    }

    let shells = Shell::detect_available_shells();
    *cache = Some((shells.clone(), now));
    shells
}

pub fn is_valid_shell(shell: &str) -> bool {
    // let name = Path::new(shell)
    //     .file_name()
    //     .and_then(|n| n.to_str())
    //     .unwrap_or(shell);

    get_shells_cached()
        .iter()
        .any(|s| s.name == shell || s.path == shell)
}

pub fn get_allowed_shells() -> Vec<String> {
    get_shells_cached().into_iter().map(|s| s.name).collect()
}

/// Result of `build_exec`.
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub executable: String,
    pub args: Vec<String>,
}

/// Returns (executable, args_prefix) for the chosen shell.
///
/// When `shell` is None the command is executed directly (Command -> Group -> Application -> direct)
/// When set, the full command string is passed to the shell as a single argument
///
/// # Argument quoting
/// For direct exec, arguments come from the already-parsed `Vec<String>` stored in the database, no re-quoting is needed.
/// For shell exec we join them into a single string — the shell handles tokenisation itself,
/// which supports pipelines, redirections, and `&&`/`||` operators the user might have typed.
///
/// # Returns
/// `(executable_name, prefix_args)` — caller appends the actual command/args after.
pub fn build_exec(command: &str, arguments: &[String], shell: Option<&str>) -> BuildResult {
    match shell {
        None => BuildResult {
            executable: command.to_string(),
            args: arguments.to_vec(),
        },

        #[cfg(target_os = "windows")]
        Some("cmd") => BuildResult {
            executable: "cmd.exe".to_string(),
            args: vec!["/C".to_string(), join_command(command, arguments)],
        },

        Some(sh @ ("powershell" | "pwsh")) => {
            #[cfg(target_os = "windows")]
            let exe = if *sh == "pwsh" {
                "pwsh.exe"
            } else {
                "powershell.exe"
            };
            #[cfg(not(target_os = "windows"))]
            let exe = sh; // pwsh is cross-platform

            BuildResult {
                executable: exe.to_string(),
                args: vec![
                    "-NonInteractive".to_string(),
                    "-Command".to_string(),
                    join_command(command, arguments),
                ],
            }
        }

        Some(sh) => BuildResult {
            executable: sh.to_string(),
            args: vec!["-c".to_string(), join_command(command, arguments)],
        },
    }
}

/// Joins command + arguments into a single shell string.
/// Arguments that contain whitespace are double-quoted.
fn join_command(command: &str, arguments: &[String]) -> String {
    let mut parts = Vec::with_capacity(arguments.len() + 1);
    parts.push(shell_quote(command));
    for arg in arguments {
        parts.push(shell_quote(arg));
    }
    parts.join(" ")
}

///  If the token already has quotes or contains special chars, pass it through.
/// For simple whitespace cases we add double quotes.
fn shell_quote(s: &str) -> String {
    if s.contains(' ') && !s.starts_with('"') && !s.starts_with('\'') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}
