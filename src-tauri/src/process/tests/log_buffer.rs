use crate::process::log_buffer::LogBuffer;
use crate::process::models::LogLineEvent;
use crate::process::tests::create_test_line;
use std::sync::Arc;

fn fill(buf: &mut LogBuffer, count: usize) {
    for i in 0..count {
        buf.push(create_test_line(&format!("line {i}"), false, 1));
    }
}

fn get_stdout(content: &str) -> Arc<LogLineEvent> {
    create_test_line(content, false, 1)
}

fn get_stderr(content: &str) -> Arc<LogLineEvent> {
    create_test_line(content, true, 1)
}

#[test]
fn new_starts_empty() {
    let buf = LogBuffer::new(100);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.total_pushed(), 0);
    assert!(!buf.was_truncated());
}

#[test]
fn new_sets_capacity() {
    let buf = LogBuffer::new(500);
    assert_eq!(buf.capacity(), 500);
}

#[test]
fn push_increases_len_total_pushed() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.push(get_stdout("c"));

    assert_eq!(buf.len(), 3);
    assert_eq!(buf.total_pushed(), 3);
    assert_eq!(buf.dropped_count(), 0);
    assert!(!buf.was_truncated());
}

#[test]
fn push_evicts_oldest_when_at_capacity() {
    let mut buf = LogBuffer::new(3);
    buf.push(get_stdout("first"));
    buf.push(get_stdout("second"));
    buf.push(get_stdout("third"));
    buf.push(get_stdout("fourth")); // should evict "first"

    assert_eq!(buf.len(), 3);
    let all = buf.get_all();
    assert_eq!(all[0].content, "second");
    assert_eq!(all[1].content, "third");
    assert_eq!(all[2].content, "fourth");
    assert_eq!(buf.total_pushed(), 4);
    assert!(buf.dropped_count() >= 1);
    assert!(buf.was_truncated());
}

#[test]
fn push_many_adds_all_lines() {
    let mut buf = LogBuffer::new(10);
    let lines = vec![get_stdout("a"), get_stdout("b"), get_stdout("c")];
    buf.push_many(lines);
    assert_eq!(buf.len(), 3);
}

#[test]
fn push_many_respects_capacity() {
    let mut buf = LogBuffer::new(2);
    let lines = vec![
        get_stdout("a"),
        get_stdout("b"),
        get_stdout("c"),
        get_stdout("d"),
    ];
    buf.push_many(lines);
    assert_eq!(buf.len(), 2);
    let all = buf.get_all();
    assert_eq!(all[0].content, "c");
    assert_eq!(all[1].content, "d");
}

#[test]
fn get_all_returns_chronological_order() {
    let mut buf = LogBuffer::new(10);
    assert!(buf.get_all().is_empty());

    buf.push(get_stdout("first"));
    buf.push(get_stdout("second"));
    buf.push(get_stdout("third"));

    assert!(!buf.get_all().is_empty());

    let all = buf.get_all();
    assert_eq!(all[0].content, "first");
    assert_eq!(all[1].content, "second");
    assert_eq!(all[2].content, "third");
}

#[test]
fn get_paginated_basic() {
    let mut buf = LogBuffer::new(10);
    fill(&mut buf, 5);

    let page = buf.get_paginated(1, 2);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].content, "line 1");
    assert_eq!(page[1].content, "line 2");
}

#[test]
fn get_paginated_offset_beyond_len_is_empty() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));

    let page = buf.get_paginated(5, 10);
    assert!(page.is_empty());
}

#[test]
fn get_paginated_limit_beyond_remaining_clips() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.push(get_stdout("c"));

    let page = buf.get_paginated(2, 100);
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].content, "c");
}

#[test]
fn get_paginated_zero_offset_zero_limit_is_empty() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));

    let page = buf.get_paginated(0, 0);
    assert!(page.is_empty());
}

#[test]
fn get_recent_returns_newest_first() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.push(get_stdout("c"));

    let recent = buf.get_recent(2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].content, "c"); // newest first
    assert_eq!(recent[1].content, "b");
}

#[test]
fn get_recent_n_larger_than_len_returns_all() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));

    let recent = buf.get_recent(100);
    assert_eq!(recent.len(), 2);
}

#[test]
fn clear_empties_buffer() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.clear();

    assert_eq!(buf.len(), 0);
    assert!(buf.get_all().is_empty());
}

#[test]
fn clear_resets_len_but_not_total_received() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.clear();

    assert_eq!(buf.total_pushed(), 2);
    assert_eq!(buf.len(), 0);
}

#[test]
fn push_after_clear_works_normally() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("old"));
    buf.clear();
    buf.push(get_stdout("new"));

    let all = buf.get_all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].content, "new");
}

#[test]
fn search_case_sensitive_finds_exact_match() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("ERROR: something bad"));
    buf.push(get_stdout("info: all fine"));
    buf.push(get_stdout("Error: partial match"));

    let results = buf.search("ERROR", true);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "ERROR: something bad");
}

#[test]
fn search_case_insensitive_finds_all_variants() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("ERROR: bad"));
    buf.push(get_stdout("error: also bad"));
    buf.push(get_stdout("info: fine"));

    let results = buf.search("error", false);
    assert_eq!(results.len(), 2);
}

#[test]
fn search_no_match_returns_empty() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("hello world"));
    let results = buf.search("xyz", false);
    assert!(results.is_empty());
}

#[test]
fn search_empty_buffer_returns_empty() {
    let buf = LogBuffer::new(10);
    let results = buf.search("anything", false);
    assert!(results.is_empty());
}

#[test]
fn search_includes_stderr_lines() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("normal line"));
    buf.push(get_stderr("ERROR from stderr"));

    let results = buf.search("ERROR", true);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_stderr);
}

#[test]
fn resize_growing_keeps_all_lines() {
    let mut buf = LogBuffer::new(3);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.push(get_stdout("c"));

    buf.resize(10);

    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 3);
}

#[test]
fn resize_shrinking_evicts_oldest() {
    let mut buf = LogBuffer::new(5);
    for c in ["a", "b", "c", "d", "e"] {
        buf.push(get_stdout(c));
    }

    buf.resize(3);

    assert_eq!(buf.capacity(), 3);
    assert_eq!(buf.len(), 3);
    let all = buf.get_all();
    assert_eq!(all[0].content, "c");
    assert_eq!(all[1].content, "d");
    assert_eq!(all[2].content, "e");
}

#[test]
fn resize_to_same_size_is_noop() {
    let mut buf = LogBuffer::new(5);
    buf.push(get_stdout("a"));
    buf.push(get_stdout("b"));
    buf.resize(5);

    assert_eq!(buf.len(), 2);
    assert_eq!(buf.capacity(), 5);
}

#[test]
fn push_preserves_stderr_flag() {
    let mut buf = LogBuffer::new(10);
    buf.push(get_stdout("out line"));
    buf.push(get_stderr("err line"));

    let all = buf.get_all();
    assert!(!all[0].is_stderr);
    assert!(all[1].is_stderr);
}

#[test]
fn push_preserves_execution_id() {
    let mut buf = LogBuffer::new(10);
    buf.push(create_test_line("from exec 1", false, 1));
    buf.push(create_test_line("from exec 2", false, 2));

    let all = buf.get_all();
    assert_eq!(all[0].execution_id, 1);
    assert_eq!(all[1].execution_id, 2);
}

#[test]
fn push_after_clear_works_correctly() {
    let mut buf = LogBuffer::new(5);
    fill(&mut buf, 5);
    buf.clear();
    buf.push(create_test_line("fresh start", false, 1));
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.get_all()[0].content, "fresh start");
}

#[test]
fn search_case_sensitive_finds_exact_match_only() {
    let mut buf = LogBuffer::new(10);
    buf.push(create_test_line("Error: something failed", false, 1));
    buf.push(create_test_line("error: lowercase", false, 1));
    buf.push(create_test_line("INFO: all good", false, 1));
    let results = buf.search("Error", true);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "Error: something failed");
}

#[test]
fn search_works_on_stderr_lines() {
    let mut buf = LogBuffer::new(10);
    buf.push(create_test_line("panic: out of memory", true, 1));
    let results = buf.search("panic", true);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_stderr);
}

#[test]
fn search_empty_pattern_matches_every_line() {
    let mut buf = LogBuffer::new(10);
    fill(&mut buf, 5);
    assert_eq!(buf.search("", false).len(), 5);
}

#[test]
fn resize_to_zero_does_nothing() {
    let mut buf = LogBuffer::new(5);
    fill(&mut buf, 5);
    buf.resize(0);
    assert_eq!(buf.len(), 5);
}
