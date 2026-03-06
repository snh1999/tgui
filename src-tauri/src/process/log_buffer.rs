use crate::constants::MAX_LOG_LINES;
use crate::process::models::LogLineEvent;
use std::collections::VecDeque;
use std::sync::Arc;

/// Circular buffer (of fixed capacity) for storing log lines in memory,
/// capacity needs to be at least 1
/// when capacity exceeded, drops oldest lines
pub(crate) struct LogBuffer {
    buffer: VecDeque<Arc<LogLineEvent>>,
    capacity: usize,
    /// Total lines ever pushed (including dropped ones)
    total_pushed: usize,
}

impl LogBuffer {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity.min(MAX_LOG_LINES)),
            capacity: if capacity < 1 {
                MAX_LOG_LINES
            } else {
                capacity.min(MAX_LOG_LINES)
            },
            total_pushed: 0,
        }
    }

    pub(crate) fn push(&mut self, line: Arc<LogLineEvent>) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(line);
        self.total_pushed += 1;
    }

    pub(crate) fn push_many(&mut self, lines: Vec<Arc<LogLineEvent>>) {
        for line in lines {
            self.push(line);
        }
    }

    pub(crate) fn get_all(&self) -> Vec<Arc<LogLineEvent>> {
        self.buffer.iter().cloned().collect()
    }

    pub(crate) fn get_recent(&self, n: usize) -> Vec<Arc<LogLineEvent>> {
        self.buffer.iter().rev().take(n).cloned().collect()
    }

    /// Get lines with pagination support
    /// offset: skip N, from oldest
    /// limit: max lines to return
    pub(crate) fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Arc<LogLineEvent>> {
        self.buffer
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.buffer.len()
    }

    pub(crate) fn total_pushed(&self) -> usize {
        self.total_pushed
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    pub(crate) fn dropped_count(&self) -> usize {
        self.total_pushed.saturating_sub(self.capacity)
    }

    pub(crate) fn was_truncated(&self) -> bool {
        self.total_pushed > self.capacity
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
    }

    pub(crate) fn search(&self, pattern: &str, is_case_sensitive: bool) -> Vec<Arc<LogLineEvent>> {
        let pattern_lower = if is_case_sensitive {
            pattern
        } else {
            &pattern.to_lowercase()
        };

        self.buffer
            .iter()
            .filter(|line| {
                if is_case_sensitive {
                    line.content.contains(pattern_lower)
                } else {
                    line.content.to_lowercase().contains(pattern_lower)
                }
            })
            .cloned()
            .collect()
    }

    /// Resize buffer (drops oldest if shrinking)
    pub(crate) fn resize(&mut self, new_capacity: usize) {
        if new_capacity < 1 || new_capacity > MAX_LOG_LINES {
            return;
        }

        while self.buffer.len() > new_capacity {
            self.buffer.pop_front();
        }
        self.capacity = new_capacity;
        self.total_pushed = 0;
        self.buffer.shrink_to_fit();
    }
}
