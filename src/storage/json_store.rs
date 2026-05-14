//! JSON persistence helpers — atomic writes, backup-on-write, and corruption detection.
//!
//! All disk-backed state in Onyx Brain goes through these functions. They provide
//! crash-safe atomic writes (write to .tmp then rename) and optional backup-before-overwrite.

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

/// Save a value as pretty-printed JSON using an atomic write pattern.
///
/// Writes to a `.tmp` file first, then renames to the target path. This prevents
/// partial writes from corrupting state if the process crashes mid-write.
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating parent directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(value)?;

    // Atomic write: write to temp file, then rename
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &json)
        .with_context(|| format!("writing temp file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("renaming {} to {}", tmp_path.display(), path.display()))?;
    Ok(())
}

/// Save with a backup: if the target already exists, rename it to `.bak` first.
pub fn save_json_with_backup<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if path.exists() {
        let bak = path.with_extension("bak");
        let _ = fs::rename(path, &bak); // best-effort backup
    }
    save_json(path, value)
}

/// Load a JSON file and deserialize it.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// Load a JSON file if it exists; return None if it doesn't.
pub fn try_load_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if path.exists() {
        Ok(Some(load_json(path)?))
    } else {
        Ok(None)
    }
}

/// Attempt to load a JSON file; if it's corrupted, try loading from `.bak`.
pub fn load_json_with_recovery<T: DeserializeOwned>(path: &Path) -> Result<T> {
    match load_json(path) {
        Ok(value) => Ok(value),
        Err(primary_err) => {
            let bak = path.with_extension("bak");
            if bak.exists() {
                load_json(&bak).with_context(|| {
                    format!(
                        "primary file {} corrupted and backup also failed: {}",
                        path.display(),
                        primary_err
                    )
                })
            } else {
                Err(primary_err)
            }
        }
    }
}

/// Check if a JSON file is valid (parseable) without fully deserializing.
pub fn validate_json_file(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let bytes = fs::read(path)?;
    match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Count JSON files in a directory.
pub fn count_json_files(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    let count = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .count();
    Ok(count)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("onyx_json_test_{}", id));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = test_dir();
        let path = dir.join("test.json");
        let data = TestData {
            name: "hello".into(),
            value: 42,
        };
        save_json(&path, &data).unwrap();
        let loaded: TestData = load_json(&path).unwrap();
        assert_eq!(data, loaded);
    }

    #[test]
    fn try_load_returns_none_for_missing() {
        let path = test_dir().join("missing.json");
        let result: Option<TestData> = try_load_json(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn save_with_backup_creates_bak() {
        let dir = test_dir();
        let path = dir.join("backed.json");
        let v1 = TestData {
            name: "v1".into(),
            value: 1,
        };
        save_json(&path, &v1).unwrap();
        let v2 = TestData {
            name: "v2".into(),
            value: 2,
        };
        save_json_with_backup(&path, &v2).unwrap();
        assert!(path.with_extension("bak").exists());
        let loaded: TestData = load_json(&path).unwrap();
        assert_eq!(loaded.value, 2);
    }

    #[test]
    fn validate_json_detects_valid_file() {
        let dir = test_dir();
        let path = dir.join("valid.json");
        fs::write(&path, r#"{"name": "test"}"#).unwrap();
        assert!(validate_json_file(&path).unwrap());
    }

    #[test]
    fn validate_json_detects_corrupt_file() {
        let dir = test_dir();
        let path = dir.join("corrupt.json");
        fs::write(&path, "this is not json {{{").unwrap();
        assert!(!validate_json_file(&path).unwrap());
    }

    #[test]
    fn count_json_files_works() {
        let dir = test_dir();
        fs::write(dir.join("a.json"), "{}").unwrap();
        fs::write(dir.join("b.json"), "{}").unwrap();
        fs::write(dir.join("c.txt"), "not json").unwrap();
        assert_eq!(count_json_files(&dir).unwrap(), 2);
    }
}
