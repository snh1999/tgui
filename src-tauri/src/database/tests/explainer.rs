use crate::database;
use crate::database::tests::TestDb;
use crate::database::{ExplainResult, SegmentResult};
use std::fs;

#[test]
fn test_explain_simple_command() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("echo hello").unwrap();
    assert!(!result.is_privileged);
    assert!(!result.is_destructive);
    assert_eq!(result.segments.len(), 1);
}

#[test]
fn test_explain_sudo_command() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sudo apt update").unwrap();
    assert!(result.is_privileged);
    assert_eq!(result.segments[0].is_privileged, true);
}

#[test]
fn test_explain_sudo_with_flags() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sudo -E env").unwrap();
    assert!(result.is_privileged);

    let result = test_db.db.explain_command("sudo -u admin whoami").unwrap();
    assert!(result.is_privileged);
}

#[test]
fn test_explain_destructive_patterns() {
    let test_db = TestDb::setup_test_db();

    assert!(
        test_db
            .db
            .explain_command("rm -rf /")
            .unwrap()
            .is_destructive
    );
    assert!(
        test_db
            .db
            .explain_command("rm -fr dir")
            .unwrap()
            .is_destructive
    );
    assert!(
        test_db
            .db
            .explain_command("dd if=/dev/zero of=/dev/sda")
            .unwrap()
            .is_destructive
    );
    assert!(
        test_db
            .db
            .explain_command("chmod 777 /etc/passwd")
            .unwrap()
            .is_destructive
    );
    assert_eq!(
        test_db
            .db
            .explain_command(":(){:|:&};:")
            .unwrap()
            .is_destructive,
        true,
        "{:?}",
        test_db.db.explain_command(":(){:|:&};:").unwrap()
    );
}

#[test]
fn test_explain_non_destructive_commands() {
    let test_db = TestDb::setup_test_db();

    assert!(!test_db.db.explain_command("ls -la").unwrap().is_destructive);
    assert!(
        !test_db
            .db
            .explain_command("cat file.txt")
            .unwrap()
            .is_destructive
    );
    assert!(
        !test_db
            .db
            .explain_command("echo hello")
            .unwrap()
            .is_destructive
    );
}

#[test]
fn test_explain_piped_commands() {
    let test_db = TestDb::setup_test_db();

    let result = test_db
        .db
        .explain_command("cat file.txt | grep pattern")
        .unwrap();
    assert_eq!(result.segments.len(), 2);
    assert_eq!(result.segments[1].connector, Some("|".to_string()));
}

#[test]
fn test_explain_chained_commands() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("cmd1 && cmd2 || cmd3").unwrap();
    assert_eq!(result.segments.len(), 3);
    assert_eq!(result.segments[1].connector, Some("&&".to_string()));
    assert_eq!(result.segments[2].connector, Some("||".to_string()));
}

#[test]
fn test_explain_semicolon_separated() {
    let test_db = TestDb::setup_test_db();

    let result = test_db
        .db
        .explain_command("echo a; echo b; echo c")
        .unwrap();
    assert_eq!(result.segments.len(), 3);
    assert_eq!(result.segments[1].connector, Some(";".to_string()));
}

#[test]
fn test_explain_redirection_detection() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("echo hello > file.txt").unwrap();
    assert!(result.segments[0].has_redirection);

    let result = test_db.db.explain_command("cmd 2>&1").unwrap();
    assert!(result.segments[0].has_redirection);
}

#[test]
fn test_explain_append_redirection() {
    let test_db = TestDb::setup_test_db();

    let result = test_db
        .db
        .explain_command("echo hello >> file.txt")
        .unwrap();
    assert!(result.segments[0].has_redirection);
}

#[test]
fn test_explain_background_detection() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sleep 10 &").unwrap();
    assert!(result.segments[0].is_background);
}

#[test]
fn test_explain_operator_priority() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("a && b || c").unwrap();
    assert_eq!(result.segments.len(), 3);

    let result = test_db.db.explain_command("a & b").unwrap();
    assert_eq!(result.segments.len(), 1);
    assert!(result.segments[0].is_background);
}

#[test]
fn test_explain_empty_string() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("").unwrap();
    assert!(result.segments.is_empty());
    assert_eq!(result.summary, "");
}

#[test]
fn test_explain_whitespace_only() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("   \t\n  ").unwrap();
    assert!(result.segments.is_empty());
}

#[test]
fn test_explain_unknown_command_returns_all_unknown() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("xyzcmd -a -b").unwrap();
    assert_eq!(result.segments[0].unknown_parts, vec!["xyzcmd", "-a", "-b"]);
    assert!(result.segments[0].tldr_description.is_none());
}

#[test]
fn test_explain_summary_capitalization() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("echo test").unwrap();
    assert!(result.summary.is_empty() || result.summary.chars().next().unwrap().is_uppercase());
}

#[test]
fn test_explain_summary_includes_privileged_prefix() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sudo ls").unwrap();
    assert!(result.summary.contains("[sudo]"));
}

#[test]
fn test_explain_summary_includes_destructive_warning() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sudo rm -rf /").unwrap();

    // Debug: print segment details
    for (i, seg) in result.segments.iter().enumerate() {
        eprintln!(
            "Segment {}: raw='{}', is_destructive={}, is_privileged={}",
            i, seg.raw, seg.is_destructive, seg.is_privileged
        );
    }

    assert!(result.is_destructive, "summary: {}", result.summary);
}

#[test]
fn test_explain_summary_includes_background_marker() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sleep 10 &").unwrap();
    assert!(result.summary.contains("[in background]"));
}

#[test]
fn test_explain_summary_includes_redirection_marker() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("echo hi > file").unwrap();
    assert!(result.summary.contains("[output redirected]"));
}

#[test]
fn test_explain_connector_prose_and() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("a && b").unwrap();
    assert!(result.summary.contains(", then "));
}

#[test]
fn test_explain_connector_prose_or() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("a || b").unwrap();
    assert!(result.summary.contains(", or if that fails, "));
}

#[test]
fn test_explain_connector_prose_pipe() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("a | b").unwrap();
    assert!(result.summary.contains(", piped to "));
}

#[test]
fn test_explain_connector_prose_semicolon() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("a ; b").unwrap();
    assert!(result.summary.contains(", then "));
}

#[test]
fn test_strip_sudo_no_sudo() {
    let (is_priv, cmd) = database::explainer::strip_sudo("echo hello");
    assert!(!is_priv);
    assert_eq!(cmd, "echo hello");
}

#[test]
fn test_strip_sudo_plain_sudo() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo echo hello");
    assert!(is_priv);
    assert_eq!(cmd, "echo hello");
}

#[test]
fn test_strip_sudo_with_e_flag() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -E echo hello");
    assert!(is_priv);
    assert_eq!(cmd, "echo hello");
}

#[test]
fn test_strip_sudo_with_user_flag() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -u admin whoami");
    assert!(is_priv);
    assert_eq!(cmd, "whoami");
}

#[test]
fn test_strip_sudo_only_sudo() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo");
    assert!(is_priv);
    assert_eq!(cmd, "");
}

#[test]
fn test_matches_pattern_exact_match() {
    assert!(database::explainer::matches_pattern(
        "apt install",
        "apt install"
    ));
    assert!(!database::explainer::matches_pattern(
        "apt install",
        "apt remove"
    ));
}

#[test]
fn test_matches_pattern_with_placeholder() {
    assert!(database::explainer::matches_pattern(
        "apt install {{package}}",
        "apt install vim"
    ));
    assert!(database::explainer::matches_pattern(
        "echo {{text}}",
        "echo hello world"
    ));
    assert!(!database::explainer::matches_pattern(
        "apt install {{package}}",
        "apt remove vim"
    ));
}

#[test]
fn test_matches_pattern_token_count_mismatch() {
    assert!(!database::explainer::matches_pattern(
        "apt install {{package}}",
        "apt install"
    ));
    assert!(!database::explainer::matches_pattern(
        "apt install",
        "apt install vim"
    ));
}

#[test]
fn test_explain_sudo_alone_returns_empty() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.explain_command("sudo").unwrap();
    assert!(result.is_privileged);
    assert_eq!(result.segments[0].raw, "sudo");
    assert!(result.segments[0].tldr_description.is_none());
    assert!(result.segments[0].unknown_parts.is_empty());
}

#[test]
fn test_populate_tldr_from_folder_reads_markdown() {
    let test_db = TestDb::setup_test_db();

    let tldr_dir = tempfile::tempdir().unwrap();
    let common_dir = tldr_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();

    fs::write(
        common_dir.join("testcmd.md"),
        "- Do something:\n`testcmd {{arg}}`\n\n- Do another thing:\n`testcmd -v`\n",
    )
    .unwrap();

    test_db
        .db
        .populate_tldr_from_folder(tldr_dir.path().to_str().unwrap())
        .unwrap();

    let result = test_db.db.explain_command("testcmd {{arg}}").unwrap();
    assert_eq!(result.summary, "Do something".to_string());
}

#[test]
fn test_find_tldr_with_placeholders_finds_best_match() {
    let test_db = TestDb::setup_test_db();

    let tldr_dir = tempfile::tempdir().unwrap();
    let common_dir = tldr_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();

    fs::write(
        common_dir.join("git.md"),
        "- Clone a repository:\n`git clone {{repository}}`\n",
    )
    .unwrap();

    test_db
        .db
        .populate_tldr_from_folder(tldr_dir.path().to_str().unwrap())
        .unwrap();

    let desc = test_db
        .db
        .explain_command("git clone https://github.com/user/repo")
        .unwrap();
    assert_eq!(desc.summary, "Clone a repository".to_string());
}

#[test]
fn test_resolve_segment_text_right_drop_strategy() {
    let test_db = TestDb::setup_test_db();

    let tldr_dir = tempfile::tempdir().unwrap();
    let common_dir = tldr_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();

    fs::write(
        common_dir.join("tar.md"),
        "- Extract an archive:\n`tar -xvf {{archive}}`\n",
    )
    .unwrap();

    test_db
        .db
        .populate_tldr_from_folder(tldr_dir.path().to_str().unwrap())
        .unwrap();

    let (desc, _) = test_db.db.resolve_segment_text("tar -xvf file.tar extra");
    assert_eq!(desc, Some("Extract an archive".to_string()));
}

#[test]
fn test_resolve_segment_text_no_match_returns_all_unknown() {
    let test_db = TestDb::setup_test_db();

    let (desc, unknown) = test_db.db.resolve_segment_text("unknowncommand -a -b");
    assert!(desc.is_none());
    assert_eq!(unknown, vec!["unknowncommand", "-a", "-b"]);
}



#[test]
fn test_strip_sudo_combined_flags() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -E -u admin echo hello");
    assert!(is_priv);
    assert_eq!(cmd, "echo hello");

    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -u admin -E echo hello");
    assert!(is_priv);
    assert_eq!(cmd, "echo hello");

    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -i -u admin whoami");
    assert!(is_priv);
    assert_eq!(cmd, "whoami");
}

#[test]
fn test_strip_sudo_u_flag_without_command() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -u admin");
    assert!(is_priv);
    assert_eq!(cmd, "");
}

#[test]
fn test_strip_sudo_multiple_flags_no_command() {
    let (is_priv, cmd) = database::explainer::strip_sudo("sudo -E -H");
    assert!(is_priv);
    assert_eq!(cmd, "");
}

#[test]
fn test_explain_quoted_string_preserved() {
    let test_db = TestDb::setup_test_db();

    let tldr_dir = tempfile::tempdir().unwrap();
    let common_dir = tldr_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();

    fs::write(
        common_dir.join("echo.md"),
        "- Print text:\n`echo {{text}}`\n",
    )
    .unwrap();
    test_db
        .db
        .populate_tldr_from_folder(tldr_dir.path().to_str().unwrap())
        .unwrap();

    let result = test_db.db.explain_command("echo \"hello world\"").unwrap();
    assert_eq!(result.segments[0].unknown_parts, Vec::<String>::new());
    assert!(result.segments[0].tldr_description.is_some());
}

#[test]
fn test_explain_quoted_string_with_flags() {
    let test_db = TestDb::setup_test_db();

    let tldr_dir = tempfile::tempdir().unwrap();
    let common_dir = tldr_dir.path().join("common");
    fs::create_dir(&common_dir).unwrap();

    fs::write(
        common_dir.join("grep.md"),
        "- Search for pattern:\n`grep {{pattern}} {{file}}`\n",
    )
    .unwrap();
    test_db
        .db
        .populate_tldr_from_folder(tldr_dir.path().to_str().unwrap())
        .unwrap();

    let result = test_db
        .db
        .explain_command("grep \"search term\" file.txt")
        .unwrap();
    assert_eq!(result.segments[0].unknown_parts, Vec::<String>::new());
}

#[test]
fn test_is_destructive_dd_without_space() {
    let test_db = TestDb::setup_test_db();

    assert!(
        test_db
            .db
            .explain_command("dd if=/dev/zero of=/dev/sda")
            .unwrap()
            .is_destructive
    );
    assert!(
        test_db
            .db
            .explain_command("dd if=/dev/urandom of=/dev/sdb")
            .unwrap()
            .is_destructive
    );
}

// #[test]
// fn test_is_destructive_mkfs_variants() {
//     let test_db = TestDb::setup_test_db();
//
//     assert!(test_db.db.explain_command("mkfs.ext4 /dev/sda1").unwrap().is_destructive);
//     assert!(test_db.db.explain_command("mkfs.xfs /dev/sdb1").unwrap().is_destructive);
//     assert!(test_db.db.explain_command("mkfs -t ext4 /dev/sda1").unwrap().is_destructive);
// }

#[test]
fn test_matches_pattern_quoted_tokens() {
    assert!(database::explainer::matches_pattern(
        "echo {{text}}",
        "echo \"hello world\""
    ));
    assert!(database::explainer::matches_pattern(
        "echo {{text}}",
        "echo hello"
    ));
}
