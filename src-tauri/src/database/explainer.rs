use std::fs;
use std::path::{Path};

use super::{Database, DatabaseError, ExplainResult, Result};
use tracing::{debug};
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SegmentResult {
    /// The cleaned segment text (redirections/background stripped).
    pub raw: String,
    pub tldr_description: Option<String>,
    pub unknown_parts: Vec<String>,
    pub is_privileged: bool,
    pub is_destructive: bool,
    /// Operator that preceded this segment in the original string: &&, ||, |, ;
    pub connector: Option<String>,
    pub has_redirection: bool,
    pub is_background: bool,
}

#[derive(Debug)]
struct RawSegment {
    text: String,
    connector: Option<String>,
    has_redirection: bool,
    is_background: bool,
}

impl Database {
    pub fn ensure_tldr_populated(&self, pages_dir: &Path) -> Result<()> {
        let tldr_dir_name =
            pages_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or(DatabaseError::Internal(
                    "Could not locate tldr data".to_string(),
                ))?;

        let stored = self.get_setting("tldr_data_version")?;
        let current_version = &tldr_dir_name[11..];

        if stored == current_version {
            return Ok(());
        }

        tracing::info!(version = %tldr_dir_name, "Populating tldr commands");
        self.populate_tldr_from_folder(pages_dir.to_str().unwrap())?;
        self.set_setting("tldr_data_version", current_version)?;

        let stored = self.get_setting("tldr_data_version")?;
        debug!("TLDR data version: {}", stored);
        Ok(())
    }

    /// save tldr fetched entries to database
    pub fn populate_tldr_from_folder(&self, folder_path: &str) -> Result<()> {
        // Collect all file paths before acquiring the DB lock.
        let files = collect_tldr_files(Path::new(folder_path));

        let mut conn = self.conn()?;
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM tldr_commands", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO tldr_commands (page_name, command_name, description, platform)
             VALUES (?1, ?2, ?3, ?4)",
            )?;
            for (filepath, platform) in &files {
                let path = Path::new(filepath);
                let page_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let Ok(content) = fs::read_to_string(path) else {
                    continue;
                };

                for (cmd, desc) in parse_tldr_page(&content) {
                    let (_, effective_cmd) = strip_sudo(&cmd);
                    stmt.execute(params![page_name, effective_cmd, desc, platform])?;
                }

                if let Some(desc) = parse_tldr_page_description(&content) {
                    let cmd_name = page_name.replace('-', " ");
                    stmt.execute(params![page_name, cmd_name, desc, platform])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Fetch all tldr entries whose page_name is exactly `first_token` or starts with `first_token`
    fn get_tldr_candidates_for(&self, first_token: &str) -> Result<Vec<(String, String)>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT command_name, description,
                CASE WHEN page_name = ?1 THEN 0 ELSE 1 END AS rank
         FROM tldr_commands
         WHERE page_name = ?1
            OR page_name LIKE ?2
         ORDER BY rank, length(command_name) DESC",
        )?;
        let prefix_pattern = format!("{}-%", first_token);
        let mut rows = stmt.query(params![first_token, prefix_pattern])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((row.get::<_, String>(0)?, row.get::<_, String>(1)?));
        }
        Ok(out)
    }

    /// Fetch all tldr entries whose page_name matches prefix
    pub fn get_tldr_completions(&self, prefix: &str) -> Result<Vec<String>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT page_name FROM tldr_commands
         WHERE page_name = ?1
            OR page_name LIKE ?2
         ORDER BY page_name
         LIMIT 20",
        )?;
        let like_pattern = format!("{}%", prefix);
        let rows = stmt.query_map(params![prefix, like_pattern], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<String>>>()
            .map_err(Into::into)
    }

    /// Core resolution: match as many leading tokens as possible, returning
    /// the remainder as unknown_parts.
    ///
    /// One DB call fetches all candidates for the first token; length iteration
    /// and placeholder matching then happen entirely in memory.
    pub(crate) fn resolve_segment_text(&self, text: &str) -> (Option<String>, Vec<String>) {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        if tokens.is_empty() {
            return (None, Vec::new());
        }

        // Fetch pattern candidates once up front (needed for pass 2 at each length).
        let candidates = match self.get_tldr_candidates_for(tokens[0]) {
            Ok(c) => c,
            Err(_) => return (None, tokens.iter().map(|s| s.to_string()).collect()),
        };

        for len in (1..=tokens.len()).rev() {
            let candidate = tokens[..len].join(" ");
            let unknown: Vec<String> = tokens[len..].iter().map(|s| s.to_string()).collect();

            // Exact DB match first (catches literal examples + page fallback rows).
            if let Ok(Some(desc)) = self.get_tldr_exact(&candidate) {
                return (Some(desc), unknown);
            }

            // Pattern match at this same length before trying a shorter prefix.
            if let Some((_, desc)) = candidates
                .iter()
                .find(|(cmd, _)| matches_pattern(cmd, &candidate))
            {
                return (Some(desc.clone()), unknown);
            }
        }

        (None, tokens.iter().map(|s| s.to_string()).collect())
    }

    /// the rank hack is necessary because for some entries, tldr has duplicate values
    /// so page_name is a tiebreaker before insertion order
    fn get_tldr_exact(&self, command: &str) -> Result<Option<String>> {
        let conn = self.conn()?;
        let first_token = command.split_whitespace().next().unwrap_or(command);
        let mut stmt = conn.prepare(
            "SELECT description,
                CASE WHEN page_name = ?2 THEN 0 ELSE 1 END AS rank
         FROM tldr_commands
         WHERE command_name = ?1
         ORDER BY rank
         LIMIT 1",
        )?;
        let mut rows = stmt.query(params![command, first_token])?;
        Ok(rows.next()?.map(|r| r.get::<_, String>(0)).transpose()?)
    }

    /// Explain a raw shell command string in plain English.
    ///
    /// let r = db.explain_command("sudo apt update -y")?;
    /// r.summary = "[sudo] Refresh the local package index (`-y` unrecognized)"
    /// r.is_privileged = true; r.risk_level = 3
    ///
    /// let r = db.explain_command("cat /etc/passwd | grep root")?;
    /// // r.summary → "Print the contents of a file, piped to search for a pattern in input"
    ///
    /// let r = db.explain_command("rm -rf /tmp/build && echo done > out.txt")?;
    /// // r.summary        → " Remove files or directories, then print a message [output redirected]"
    /// // r.is_destructive → true
    /// // r.risk_level     → 7
    ///
    pub fn explain_command(&self, input: &str) -> Result<ExplainResult> {
        let raw_segments = parse_into_segments(input);
        let mut segment_results: Vec<SegmentResult> = Vec::new();

        for raw in raw_segments {
            let (is_privileged, effective_cmd) = strip_sudo(&raw.text);
            let (description, unknown_parts) = self.resolve_segment_text(&effective_cmd);
            let destructive = is_destructive(&effective_cmd);

            segment_results.push(SegmentResult {
                raw: raw.text,
                tldr_description: description,
                unknown_parts,
                is_privileged,
                is_destructive: destructive,
                connector: raw.connector,
                has_redirection: raw.has_redirection,
                is_background: raw.is_background,
            });
        }

        let is_privileged = segment_results.iter().any(|s| s.is_privileged);
        let is_destructive =
            input.contains(":(){:|:&};:") || segment_results.iter().any(|s| s.is_destructive);
        let summary = build_summary(&segment_results);

        Ok(ExplainResult {
            summary,
            is_privileged,
            is_destructive,
            segments: segment_results,
        })
    }
}

const DESTRUCTIVE_TOKEN_PATTERNS: &[&[&str]] = &[
    &["rm", "-rf"],
    &["rm", "-fr"],
    &["rm", "-r"],
    &["rm", "-f"],
    &["dd"],
    &["mkfs"],
    &["fdisk"],
    &["parted"],
    &["gdisk"],
    &["shred"],
    &["wipefs"],
    &["chmod", "777"],
    &["chmod", "-r", "777"],
    &["chmod", "-R", "777"],
    &["truncate", "-s", "0"],
];
const OPS: &[&str] = &["&&", "||", "|", ";"];


/// Pattern tokens that appear contiguously at the start; additional args after are fine
pub(crate) fn is_destructive(raw: &str) -> bool {
    // Fork bomb pattern doesn't tokenize cleanly.
    if raw.contains(":(){:|:&};:") {
        return true;
    }
    let tokens: Vec<&str> = raw.split_whitespace().collect();

    'outer: for pattern in DESTRUCTIVE_TOKEN_PATTERNS {
        if tokens.len() < pattern.len() {
            continue;
        }
        for (p, t) in pattern.iter().zip(tokens.iter()) {
            if *p != *t {
                continue 'outer;
            }
        }
        return true;
    }
    false
}

/// Split a shell command string into logical segments on &&, ||, |, ;
fn parse_into_segments(input: &str) -> Vec<RawSegment> {
    let mut result = Vec::new();
    let mut remaining = input;
    let mut pending_connector: Option<String> = None;

    loop {
        let (before, op, after) = find_next_operator(remaining);
        let seg = build_segment(before.trim(), pending_connector.take());
        if !seg.text.is_empty() {
            result.push(seg);
        }
        match op {
            Some(op_str) => {
                pending_connector = Some(op_str.to_string());
                remaining = after;
            }
            None => break,
        }
    }
    result
}

/// Extract the summary line (`> …`) from a tldr markdown page.
fn parse_tldr_page_description(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let t = line.trim();
        t.starts_with('>')
            .then(|| t.trim_start_matches('>').trim().to_owned())
            .filter(|s| !s.is_empty())
    })
}

/// Find the first shell operator in `s`. Longer operators win at the same position (so || beats |, && beats &).
fn find_next_operator<'a>(s: &'a str) -> (&'a str, Option<&'static str>, &'a str) {
    let mut earliest: Option<(usize, &'static str)> = None;

    for &op in OPS {
        if let Some(pos) = s.find(op) {
            let replace = match earliest {
                None => true,
                Some((ep, prev)) => pos < ep || (pos == ep && op.len() > prev.len()),
            };
            if replace {
                earliest = Some((pos, op));
            }
        }
    }

    match earliest {
        Some((pos, op)) => (&s[..pos], Some(op), &s[pos + op.len()..]),
        None => (s, None, ""),
    }
}

/// Strip surrounding quotes from a token so `"hello world"` and `'file.txt'`match tldr placeholders.
fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Normalize quotes, strip redirections and background markers from a segment's token list, and return a cleaned RawSegment.
fn build_segment(text: &str, connector: Option<String>) -> RawSegment {
    let mut has_redirection = false;
    let mut is_background = false;
    let mut clean: Vec<String> = Vec::new();
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut skip_next = false;

    for token in &tokens {
        if skip_next {
            skip_next = false;
            continue;
        }
        // Standalone redirection operator followed by a filename
        if matches!(*token, ">" | ">>" | "2>" | "&>" | "1>") {
            has_redirection = true;
            skip_next = true;
            continue;
        }
        // Attached redirections: >file, >>file, 2>&1, &>/dev/null …
        if token.starts_with(">>")
            || token.starts_with('>')
            || token.starts_with("2>")
            || token.starts_with("&>")
        {
            has_redirection = true;
            continue;
        }
        // Standalone background operator
        if *token == "&" {
            is_background = true;
            continue;
        }
        // Trailing & glued to a token (rare, but defensive)
        if token.ends_with('&') && token.len() > 1 {
            is_background = true;
            clean.push(strip_quotes(&token[..token.len() - 1]).to_string());
            continue;
        }
        clean.push(strip_quotes(token).to_string());
    }

    RawSegment {
        text: clean.join(" "),
        connector,
        has_redirection,
        is_background,
    }
}

/// Strip a leading `sudo` and all its flags (`sudo`, `sudo -E`, `sudo -u`, `sudo -E -u`) from a command string.
/// Returns `(is_privileged, remaining_command)`.
pub(crate) fn strip_sudo(text: &str) -> (bool, String) {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.first() != Some(&"sudo") {
        return (false, text.to_string());
    }

    let mut i = 1;
    while i < tokens.len() {
        let token = tokens[i];

        // Flags that consume the next token as an argument
        if matches!(
            token,
            "-u" | "--user"
                | "-g"
                | "--group"
                | "-C"
                | "--chdir"
                | "-D"
                | "--chroot"
                | "-p"
                | "--prompt"
                | "-r"
                | "--role"
                | "-t"
                | "--type"
                | "-T"
                | "--command-timeout"
        ) {
            i += 2; // skip flag + its argument
            continue;
        }

        // Any other flag token (e.g. -E, -i, -s, -n, -v, -k …)
        if token.starts_with('-') {
            i += 1;
            continue;
        }

        // First non-flag token is the actual command
        return (true, tokens[i..].join(" "));
    }

    // sudo with flags only, no command (e.g. `sudo -i`, `sudo -v`)
    (true, String::new())
}

/// Return true when `pattern` (a tldr command name, possibly with {{placeholders}})
/// positionally matches every token in `input`.
pub(crate) fn matches_pattern(pattern: &str, input: &str) -> bool {
    let pt: Vec<&str> = pattern.split_whitespace().collect();
    let it: Vec<&str> = input.split_whitespace().collect();
    if pt.len() != it.len() {
        return false;
    }
    pt.iter().zip(it.iter()).all(|(p, i)| {
        if p.starts_with("{{") && p.ends_with("}}") {
            let inner = &p[2..p.len() - 2];
            // {{[-a|--option]}} is constrained, not a free wildcard
            if inner.starts_with('[') && inner.ends_with(']') {
                let alts = &inner[1..inner.len() - 1];
                return alts.split('|').any(|alt| alt.trim() == *i);
            }
            return true;
        }
        p == i
    })
}

/// Build ExplainResult summary from segments
fn build_summary(segments: &[SegmentResult]) -> String {
    let mut parts: Vec<String> = Vec::new();

    for (i, seg) in segments.iter().enumerate() {
        let base = seg
            .tldr_description
            .as_deref()
            .unwrap_or("unrecognized command");

        let mut part = if seg.is_privileged {
            format!("[sudo] {}", base)
        } else {
            base.to_string()
        };

        // if !seg.unknown_parts.is_empty() {
        //     let list = seg
        //         .unknown_parts
        //         .iter()
        //         .map(|u| format!("`{}`", u))
        //         .collect::<Vec<_>>()
        //         .join(", ");
        //     part = format!("{} ({} unrecognized)", part, list);
        // }

        if seg.has_redirection {
            part = format!("{} [output redirected]", part);
        }
        if seg.is_background {
            part = format!("{} [in background]", part);
        }

        if i > 0 {
            if let Some(conn) = &seg.connector {
                let connector = match conn.as_str() {
                    "&&" => ", then ",
                    "||" => ", or if that fails, ",
                    "|" => ", piped to ",
                    ";" => ", then ",
                    _ => ", then ",
                };
                part = format!("{}{}", connector, part);
            }
        }

        parts.push(part);
    }

    let mut summary = parts.join("");
    let mut chars = summary.chars();
    if let Some(first) = chars.next() {
        summary = first.to_uppercase().to_string() + chars.as_str();
    }
    summary
}

/// Parse a page into (command_example, description) pairs.
fn parse_tldr_page(content: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current_desc = String::new();

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('-') {
            current_desc = t
                .trim_start_matches('-')
                .trim()
                .trim_end_matches(':')
                .to_owned();
        } else if t.starts_with('`') && t.ends_with('`') && t.len() > 1 {
            let cmd = t.trim_matches('`').trim().to_owned();
            if !cmd.is_empty() && !current_desc.is_empty() {
                results.push((cmd, current_desc.clone()));
                current_desc.clear();
            }
        }
    }
    results
}

/// Walk `dir` recursively; return `(file_path, platform)` pairs where
/// `platform` is the immediate parent folder name (e.g. "linux", "common").
fn collect_tldr_files(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let platform = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("common")
                .to_string();
            if let Some(p) = path.to_str() {
                out.push((p.to_owned(), platform));
            }
        } else if path.is_dir() {
            out.extend(collect_tldr_files(&path));
        }
    }
    out
}

#[test]
fn test_parse_into_segments_strips_redirections() {
    let segments = parse_into_segments("echo hello > file.txt 2>&1");
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "echo hello");
    assert!(segments[0].has_redirection);
}

#[test]
fn test_parse_into_segments_handles_mixed_operators() {
    let segs = parse_into_segments("a && b || c | d");
    assert_eq!(segs.len(), 4);
    assert_eq!(segs[0].connector, None);
    assert_eq!(segs[1].connector, Some("&&".to_string()));
    assert_eq!(segs[2].connector, Some("||".to_string()));
    assert_eq!(segs[3].connector, Some("|".to_string()));
}

#[test]
fn test_collect_tldr_files_finds_nested_files() {
    let tldr_dir = tempfile::tempdir().unwrap();
    let linux_dir = tldr_dir.path().join("linux");
    fs::create_dir(&linux_dir).unwrap();

    fs::write(linux_dir.join("ip.md"), "- Show network info:\n`ip addr`\n").unwrap();

    let files = collect_tldr_files(tldr_dir.path());
    assert_eq!(files.len(), 1);
    assert!(files[0].0.contains("ip.md"));
    assert_eq!(files[0].1, "linux");
}

#[test]
fn test_parse_tldr_page_extracts_examples() {
    let content = "- First example:\n`cmd one`\n\n- Second example:\n`cmd two {{arg}}`";
    let results = parse_tldr_page(content);

    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0],
        ("cmd one".to_string(), "First example".to_string())
    );
    assert_eq!(
        results[1],
        ("cmd two {{arg}}".to_string(), "Second example".to_string())
    );
}

#[test]
fn test_parse_tldr_page_ignores_lines_without_backticks() {
    let content = "Some header text\n- Valid example:\n`cmd arg`\nMore text";
    let results = parse_tldr_page(content);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "cmd arg");
}
