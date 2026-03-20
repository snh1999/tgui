// TODO: allow shell configuring TGUI-33

use std::collections::HashSet;
use std::path::Path;
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
pub const ALLOWED_SHELLS: &[&str] = &["sh", "bash", "zsh", "fish", "nu"];

#[cfg(target_os = "windows")]
const ALLOWED_SHELL_PATHS: &[&str] = &[
    "C:\\Windows\\System32\\cmd.exe",
    "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
];

/// Returns `true` if the shell name is in `ALLOWED_SHELLS` for this platform.
pub fn is_valid_shell(shell: &str) -> bool {
    ALLOWED_SHELLS.contains(&shell)
}

pub fn get_allowed_shells() -> Vec<&'static str> {
    ALLOWED_SHELLS.to_vec()
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
