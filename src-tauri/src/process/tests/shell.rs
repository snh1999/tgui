use crate::process::shell::{build_exec, get_allowed_shells, is_valid_shell};

#[test]
fn build_exec_no_shell_returns_executable_and_args_unchanged() {
    let result = build_exec("npm", &["run".into(), "build".into()], None);
    assert_eq!(result.executable, "npm");
    assert_eq!(result.args, vec!["run", "build"]);
}

#[test]
fn build_exec_no_shell_preserves_args_with_spaces_as_separate_elements() {
    // Direct exec: args with spaces are passed as-is, the OS handles quoting
    let result = build_exec("echo", &["hello world".into()], None);
    assert_eq!(result.executable, "echo");
    assert_eq!(result.args, vec!["hello world"]);
}

#[test]
fn build_exec_no_shell_with_empty_args_returns_no_args() {
    let result = build_exec("ls", &[], None);
    assert_eq!(result.executable, "ls");
    assert!(result.args.is_empty());
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_bash_wraps_in_c_flag() {
    let result = build_exec("echo", &["hello".into()], Some("bash"));
    assert_eq!(result.executable, "bash");
    assert_eq!(result.args[0], "-c");
    assert!(result.args[1].contains("echo"));
    assert!(result.args[1].contains("hello"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_sh_wraps_in_c_flag() {
    let result = build_exec("cat", &["file.txt".into()], Some("sh"));
    assert_eq!(result.executable, "sh");
    assert_eq!(result.args[0], "-c");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_zsh_wraps_in_c_flag() {
    let result = build_exec("echo", &[], Some("zsh"));
    assert_eq!(result.executable, "zsh");
    assert_eq!(result.args[0], "-c");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_shell_joins_args_into_single_c_string() {
    // When using a shell, command + args are joined into one string passed to -c
    let result = build_exec("echo", &["a".into(), "b".into(), "c".into()], Some("bash"));
    // args[0] is "-c", args[1] is the joined command string
    assert_eq!(result.args.len(), 2);
    let cmd_str = &result.args[1];
    assert!(cmd_str.contains("echo"));
    assert!(cmd_str.contains('a'));
    assert!(cmd_str.contains('b'));
    assert!(cmd_str.contains('c'));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_shell_quotes_args_with_spaces() {
    let result = build_exec("echo", &["hello world".into()], Some("bash"));
    let cmd_str = &result.args[1];
    assert!(cmd_str.contains("\"hello world\""));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_shell_already_quoted_args_with_spaces() {
    let result = build_exec("echo", &["\"hello world\"".into()], Some("bash"));
    let cmd_str = &result.args[1];
    assert!(cmd_str.contains("\"hello world\""));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn build_exec_shell_already_quoted_args_no_spaces() {
    let result = build_exec("echo", &["\"hello\"".into()], Some("bash"));
    let cmd_str = &result.args[1];
    assert!(cmd_str.contains("\"hello\""));
}

#[cfg(target_os = "windows")]
#[test]
fn build_exec_cmd_uses_slash_c() {
    let result = build_exec("echo", &["hello".into()], Some("cmd"));
    assert_eq!(result.executable, "cmd.exe");
    assert_eq!(result.args[0], "/C");
}

#[cfg(target_os = "windows")]
#[test]
fn build_exec_powershell_adds_non_interactive_flag() {
    let result = build_exec("Get-Process", &[], Some("powershell"));
    assert!(result.args.contains(&"-NonInteractive".to_string()));
    assert!(result.args.contains(&"-Command".to_string()));
}

#[cfg(target_os = "windows")]
#[test]
fn build_exec_pwsh_uses_pwsh_executable() {
    let result = build_exec("Get-Date", &[], Some("pwsh"));
    assert_eq!(result.executable, "pwsh.exe");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn is_valid_shell_accepts_all_unix_allowed_shells() {
    for shell in &["sh", "bash", "zsh", "fish", "dash", "ksh", "nu"] {
        assert!(
            is_valid_shell(shell),
            "Expected '{shell}' to be valid on Unix"
        );
    }
}

#[cfg(target_os = "windows")]
#[test]
fn is_valid_shell_accepts_all_windows_allowed_shells() {
    for shell in &["cmd", "powershell", "pwsh"] {
        assert!(
            is_valid_shell(shell),
            "Expected '{shell}' to be valid on Windows"
        );
    }
}

#[test]
fn is_valid_shell_rejects_empty_string() {
    assert!(!is_valid_shell(""));
}

#[test]
fn is_valid_shell_rejects_path_traversal() {
    assert!(!is_valid_shell("../../../bin/sh"));
    assert!(!is_valid_shell("/bin/bash"));
}

#[test]
fn is_valid_shell_rejects_arbitrary_executables() {
    assert!(!is_valid_shell("rm"));
    assert!(!is_valid_shell("python"));
    assert!(!is_valid_shell("node"));
}

#[test]
fn is_valid_shell_is_case_sensitive() {
    assert!(!is_valid_shell("Bash"));
    assert!(!is_valid_shell("BASH"));
}

#[test]
fn allowed_shells_returns_non_empty_list() {
    assert!(!get_allowed_shells().is_empty());
}

#[test]
fn allowed_shells_every_entry_passes_is_valid_shell() {
    for shell in get_allowed_shells() {
        assert!(
            is_valid_shell(shell),
            "allowed_shells() returned '{shell}' but is_valid_shell() rejected it"
        );
    }
}

#[test]
fn allowed_shells_contains_no_duplicates() {
    let shells = get_allowed_shells();
    let mut seen = std::collections::HashSet::new();
    for shell in &shells {
        assert!(
            seen.insert(*shell),
            "Duplicate shell in allowed_shells(): '{shell}'"
        );
    }
}
