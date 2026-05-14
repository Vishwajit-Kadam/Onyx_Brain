//! Storage compaction — re-serializes JSON files to remove formatting bloat,
//! reports space savings, and supports batch operations with dry-run mode.

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

use crate::storage::{load_json, save_json};

/// Result of a compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub path: String,
    pub original_size: u64,
    pub compacted_size: u64,
    pub saved_bytes: i64,
}

impl CompactionResult {
    pub fn savings_percent(&self) -> f32 {
        if self.original_size == 0 {
            return 0.0;
        }
        (self.saved_bytes as f32 / self.original_size as f32) * 100.0
    }
}

/// Compact a single JSON file by loading and re-saving it (removes stale formatting).
pub fn compact_json<T>(path: &Path) -> Result<CompactionResult>
where
    T: Serialize + DeserializeOwned,
{
    let original_size = fs::metadata(path)?.len();
    let value: T = load_json(path)?;
    save_json(path, &value)?;
    let compacted_size = fs::metadata(path)?.len();
    Ok(CompactionResult {
        path: path.display().to_string(),
        original_size,
        compacted_size,
        saved_bytes: original_size as i64 - compacted_size as i64,
    })
}

/// Compact all JSON files in a directory (generic — uses serde_json::Value).
pub fn compact_directory(dir: &Path) -> Result<Vec<CompactionResult>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match compact_json::<serde_json::Value>(&path) {
                Ok(result) => results.push(result),
                Err(_) => continue, // skip corrupt files
            }
        }
    }
    Ok(results)
}

/// Report total space savings from a batch of compaction results.
pub fn compaction_summary(results: &[CompactionResult]) -> String {
    let total_original: u64 = results.iter().map(|r| r.original_size).sum();
    let total_compacted: u64 = results.iter().map(|r| r.compacted_size).sum();
    let total_saved: i64 = results.iter().map(|r| r.saved_bytes).sum();
    format!(
        "Compacted {} files: {} → {} bytes ({} bytes saved, {:.1}% reduction)",
        results.len(),
        total_original,
        total_compacted,
        total_saved,
        if total_original > 0 {
            (total_saved as f64 / total_original as f64) * 100.0
        } else {
            0.0
        }
    )
}

/// Dry-run: report what compaction would save without actually writing.
pub fn estimate_compaction(dir: &Path) -> Result<Vec<CompactionResult>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let original_size = fs::metadata(&path)?.len();
            if let Ok(value) = load_json::<serde_json::Value>(&path) {
                let json = serde_json::to_string_pretty(&value)?;
                let compacted_size = json.len() as u64;
                results.push(CompactionResult {
                    path: path.display().to_string(),
                    original_size,
                    compacted_size,
                    saved_bytes: original_size as i64 - compacted_size as i64,
                });
            }
        }
    }
    Ok(results)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("onyx_compact_test_{}", id));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn compact_json_roundtrips() {
        let dir = test_dir();
        let path = dir.join("test.json");
        // Write with extra whitespace
        fs::write(&path, "  {  \"a\" :  1  ,  \"b\" :  2  }  ").unwrap();
        let result = compact_json::<serde_json::Value>(&path).unwrap();
        assert!(result.compacted_size > 0);
    }

    #[test]
    fn compact_directory_processes_all_files() {
        let dir = test_dir();
        fs::write(dir.join("a.json"), r#"{"x": 1}"#).unwrap();
        fs::write(dir.join("b.json"), r#"{"y": 2}"#).unwrap();
        fs::write(dir.join("c.txt"), "not json").unwrap();
        let results = compact_directory(&dir).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn savings_percent_correct() {
        let result = CompactionResult {
            path: "test".into(),
            original_size: 100,
            compacted_size: 80,
            saved_bytes: 20,
        };
        assert!((result.savings_percent() - 20.0).abs() < 0.01);
    }

    #[test]
    fn summary_is_readable() {
        let results = vec![CompactionResult {
            path: "test".into(),
            original_size: 100,
            compacted_size: 80,
            saved_bytes: 20,
        }];
        let summary = compaction_summary(&results);
        assert!(summary.contains("1 files"));
        assert!(summary.contains("20 bytes saved"));
    }
}
