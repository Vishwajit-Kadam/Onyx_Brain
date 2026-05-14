//! Semantic memory — stores factual knowledge and conceptual understanding.
//!
//! Semantic memories represent "what the brain knows" as distilled facts rather
//! than temporal events. They support concept extraction from raw text, similarity
//! scoring between entries, and merge operations for deduplication.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs};
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

pub use crate::memory::MemoryItem;

/// A distilled factual entry in semantic memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEntry {
    pub entry_id: String,
    pub concept: String,
    pub definition: String,
    pub related_concepts: Vec<String>,
    pub source: String,
    pub confidence: f32,
    pub access_count: u64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl SemanticEntry {
    /// Create a new semantic entry.
    pub fn new(
        concept: impl Into<String>,
        definition: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            entry_id: format!("sem_{}", Uuid::new_v4()),
            concept: concept.into(),
            definition: definition.into(),
            related_concepts: Vec::new(),
            source: source.into(),
            confidence: 0.7,
            access_count: 0,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Link a related concept.
    pub fn add_related(&mut self, concept: impl Into<String>) {
        let c = concept.into();
        if !self.related_concepts.contains(&c) {
            self.related_concepts.push(c);
        }
    }

    /// Compute a keyword-overlap similarity score with a query string.
    /// Returns 0.0 to 1.0.
    pub fn similarity_to(&self, query: &str) -> f32 {
        let query_words = extract_keywords(query);
        if query_words.is_empty() {
            return 0.0;
        }
        let entry_words = self.keyword_set();
        let overlap = query_words.intersection(&entry_words).count();
        (overlap as f32 / query_words.len() as f32).clamp(0.0, 1.0)
    }

    /// Extract the set of normalized keywords from this entry.
    pub fn keyword_set(&self) -> BTreeSet<String> {
        let combined = format!(
            "{} {} {}",
            self.concept,
            self.definition,
            self.related_concepts.join(" ")
        );
        extract_keywords(&combined)
    }

    /// Validate entry integrity.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.concept.trim().is_empty() {
            issues.push("Concept is empty".into());
        }
        if self.definition.trim().is_empty() {
            issues.push("Definition is empty".into());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!("Confidence {} is out of range", self.confidence));
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Attempt to merge another entry into this one if they share the same concept.
    /// Returns true if merge happened.
    pub fn try_merge(&mut self, other: &SemanticEntry) -> bool {
        if normalize(&self.concept) != normalize(&other.concept) {
            return false;
        }
        // Merge definitions — append if the other has substantive new content
        if other.definition.len() > self.definition.len() {
            self.definition = other.definition.clone();
        }
        for rc in &other.related_concepts {
            self.add_related(rc.clone());
        }
        self.confidence = (self.confidence + other.confidence) / 2.0;
        self.access_count += other.access_count;
        true
    }

    /// Mark as accessed.
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Produce a short summary string.
    pub fn summarize(&self) -> String {
        format!(
            "{}: {} (confidence: {:.0}%, related: {})",
            self.concept,
            truncate(&self.definition, 80),
            self.confidence * 100.0,
            self.related_concepts.len()
        )
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn save_semantic_entry(store: &DiskStore, entry: &SemanticEntry) -> Result<()> {
    let dir = store.paths.data.join("semantic");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", entry.entry_id)), entry)
}

pub fn load_semantic_entries(store: &DiskStore) -> Result<Vec<SemanticEntry>> {
    let dir = store.paths.data.join("semantic");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for de in fs::read_dir(&dir)? {
        let path = de?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(entry) = load_json::<SemanticEntry>(&path) {
                entries.push(entry);
            }
        }
    }
    entries.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    Ok(entries)
}

/// Find entries most relevant to a query, by keyword similarity.
pub fn search_semantic(store: &DiskStore, query: &str, limit: usize) -> Result<Vec<SemanticEntry>> {
    let entries = load_semantic_entries(store)?;
    let mut scored: Vec<(f32, SemanticEntry)> = entries
        .into_iter()
        .map(|e| {
            let score = e.similarity_to(query);
            (score, e)
        })
        .filter(|(score, _)| *score > 0.0)
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    scored.truncate(limit);
    Ok(scored.into_iter().map(|(_, e)| e).collect())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn extract_keywords(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|w| w.len() > 2)
        .map(|w| w.to_lowercase())
        .collect()
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entry_validates() {
        let entry = SemanticEntry::new(
            "Sparse activation",
            "Only fire neurons relevant to the task",
            "architecture doc",
        );
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn empty_concept_fails_validation() {
        let entry = SemanticEntry::new("", "some definition", "src");
        assert!(entry.validate().is_err());
    }

    #[test]
    fn similarity_scoring_works() {
        let entry = SemanticEntry::new(
            "Sparse activation",
            "Only activate the neurons needed for the current task",
            "docs",
        );
        assert!(entry.similarity_to("sparse neuron activation task") > 0.3);
        assert!(entry.similarity_to("completely unrelated query about cooking") < 0.2);
    }

    #[test]
    fn merge_combines_related_concepts() {
        let mut a = SemanticEntry::new("Memory", "Disk-backed storage", "src");
        a.add_related("persistence");
        let mut b = SemanticEntry::new("memory", "Disk-backed storage system", "docs");
        b.add_related("retrieval");
        assert!(a.try_merge(&b));
        assert!(a.related_concepts.contains(&"retrieval".to_string()));
    }

    #[test]
    fn merge_rejects_different_concepts() {
        let mut a = SemanticEntry::new("Memory", "storage", "src");
        let b = SemanticEntry::new("Routing", "path selection", "src");
        assert!(!a.try_merge(&b));
    }

    #[test]
    fn touch_increments_access_count() {
        let mut entry = SemanticEntry::new("Test", "test def", "test");
        assert_eq!(entry.access_count, 0);
        entry.touch();
        entry.touch();
        assert_eq!(entry.access_count, 2);
    }
}
