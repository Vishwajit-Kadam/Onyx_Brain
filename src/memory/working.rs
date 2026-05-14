//! Working memory — a capacity-limited, priority-ordered scratchpad for the current task.
//!
//! Working memory holds transient notes, context items, and intermediate results that
//! are relevant only during the active task. It enforces a maximum capacity and evicts
//! the lowest-priority items when full.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Default maximum number of items in working memory.
pub const DEFAULT_CAPACITY: usize = 16;

/// A single item held in working memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryItem {
    pub content: String,
    pub source: String,
    pub priority: f32,
    pub added_at: DateTime<Utc>,
}

impl WorkingMemoryItem {
    pub fn new(content: impl Into<String>, source: impl Into<String>, priority: f32) -> Self {
        Self {
            content: content.into(),
            source: source.into(),
            priority: priority.clamp(0.0, 1.0),
            added_at: Utc::now(),
        }
    }
}

/// Capacity-limited working memory for the active task context.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingMemory {
    pub items: Vec<WorkingMemoryItem>,
    pub capacity: usize,
    /// Legacy field preserved for backward compatibility.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            capacity: DEFAULT_CAPACITY,
            notes: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity: capacity.max(1),
            notes: Vec::new(),
        }
    }

    /// Add an item. If at capacity, evict the lowest-priority item first.
    pub fn push(&mut self, content: impl Into<String>, source: impl Into<String>, priority: f32) {
        if self.items.len() >= self.capacity {
            self.evict_lowest();
        }
        self.items
            .push(WorkingMemoryItem::new(content, source, priority));
    }

    /// Add a simple text note (convenience for the legacy `notes` interface).
    pub fn add_note(&mut self, note: impl Into<String>) {
        let note = note.into();
        self.notes.push(note.clone());
        self.push(note, "note", 0.5);
    }

    /// Remove and return the lowest-priority item.
    fn evict_lowest(&mut self) -> Option<WorkingMemoryItem> {
        if self.items.is_empty() {
            return None;
        }
        let min_idx = self
            .items
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.priority.total_cmp(&b.priority))
            .map(|(i, _)| i)
            .unwrap_or(0);
        Some(self.items.remove(min_idx))
    }

    /// How many items are currently held.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Is the working memory at capacity?
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    /// Clear all items — equivalent to "forgetting" the active context.
    pub fn clear(&mut self) {
        self.items.clear();
        self.notes.clear();
    }

    /// Return items sorted by priority (highest first).
    pub fn by_priority(&self) -> Vec<&WorkingMemoryItem> {
        let mut sorted: Vec<&WorkingMemoryItem> = self.items.iter().collect();
        sorted.sort_by(|a, b| b.priority.total_cmp(&a.priority));
        sorted
    }

    /// Produce a concise summary of current working memory contents.
    pub fn summarize(&self) -> String {
        if self.items.is_empty() {
            return "Working memory is empty.".to_string();
        }
        let top: Vec<String> = self
            .by_priority()
            .iter()
            .take(5)
            .map(|item| {
                let truncated: String = item.content.chars().take(60).collect();
                format!("  [{:.0}%] {}", item.priority * 100.0, truncated)
            })
            .collect();
        format!(
            "Working memory: {}/{} slots used.\nTop items:\n{}",
            self.items.len(),
            self.capacity,
            top.join("\n")
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_working_memory_is_empty() {
        let wm = WorkingMemory::new();
        assert!(wm.is_empty());
        assert!(!wm.is_full());
        assert_eq!(wm.capacity, DEFAULT_CAPACITY);
    }

    #[test]
    fn push_adds_items() {
        let mut wm = WorkingMemory::new();
        wm.push("goal context", "task", 0.8);
        wm.push("safety note", "system", 0.9);
        assert_eq!(wm.len(), 2);
    }

    #[test]
    fn eviction_removes_lowest_priority() {
        let mut wm = WorkingMemory::with_capacity(3);
        wm.push("low", "test", 0.1);
        wm.push("high", "test", 0.9);
        wm.push("mid", "test", 0.5);
        // Now at capacity (3). Adding one more should evict "low" (0.1).
        wm.push("new high", "test", 0.95);
        assert_eq!(wm.len(), 3);
        assert!(!wm.items.iter().any(|i| i.content == "low"));
    }

    #[test]
    fn by_priority_returns_sorted() {
        let mut wm = WorkingMemory::new();
        wm.push("A", "t", 0.3);
        wm.push("B", "t", 0.9);
        wm.push("C", "t", 0.6);
        let sorted = wm.by_priority();
        assert_eq!(sorted[0].content, "B");
        assert_eq!(sorted[2].content, "A");
    }

    #[test]
    fn clear_empties_everything() {
        let mut wm = WorkingMemory::new();
        wm.push("item", "src", 0.5);
        wm.add_note("note");
        wm.clear();
        assert!(wm.is_empty());
        assert!(wm.notes.is_empty());
    }

    #[test]
    fn add_note_uses_both_interfaces() {
        let mut wm = WorkingMemory::new();
        wm.add_note("remember this");
        assert_eq!(wm.notes.len(), 1);
        assert_eq!(wm.items.len(), 1);
    }

    #[test]
    fn summarize_shows_useful_info() {
        let mut wm = WorkingMemory::with_capacity(4);
        wm.push("goal: build calculator", "task", 0.9);
        wm.push("safety: sandbox active", "system", 0.8);
        let summary = wm.summarize();
        assert!(summary.contains("2/4 slots"));
        assert!(summary.contains("goal: build calculator"));
    }

    #[test]
    fn priority_is_clamped() {
        let item = WorkingMemoryItem::new("test", "src", 5.0);
        assert!((item.priority - 1.0).abs() < f32::EPSILON);
        let item2 = WorkingMemoryItem::new("test", "src", -2.0);
        assert!((item2.priority - 0.0).abs() < f32::EPSILON);
    }
}
