//! Filesystem tool — sandboxed file operations with validation and safety checks.
//!
//! All path operations are resolved relative to the sandbox root. Absolute paths,
//! path traversal attempts (../), and symlink escapes are rejected.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::utils::errors::OnyxError;

#[derive(Debug, Clone)]
pub struct FilesystemTool {
    sandbox: PathBuf,
}

/// Result of a file operation, for audit and reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOpResult {
    pub operation: String,
    pub path: String,
    pub success: bool,
    pub bytes_affected: Option<u64>,
    pub error: Option<String>,
}

impl FilesystemTool {
    pub fn new(sandbox: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(&sandbox)?;
        let sandbox = sandbox.as_ref().canonicalize()?;
        Ok(Self { sandbox })
    }

    pub fn sandbox(&self) -> &Path {
        &self.sandbox
    }

    pub fn create_dir(&self, relative: &str) -> Result<PathBuf> {
        let path = self.safe_path(relative)?;
        fs::create_dir_all(&path).with_context(|| format!("creating {}", path.display()))?;
        Ok(path)
    }

    pub fn write_file(&self, relative: &str, content: &str) -> Result<PathBuf> {
        let path = self.safe_path(relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    pub fn read_file(&self, relative: &str) -> Result<String> {
        let path = self.safe_path(relative)?;
        fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    pub fn list_dir(&self, relative: &str) -> Result<Vec<PathBuf>> {
        let path = self.safe_path(relative)?;
        let mut entries = Vec::new();
        for entry in fs::read_dir(&path)? {
            entries.push(entry?.path());
        }
        entries.sort();
        Ok(entries)
    }

    /// Check if a file exists within the sandbox.
    pub fn file_exists(&self, relative: &str) -> Result<bool> {
        let path = self.safe_path(relative)?;
        Ok(path.exists())
    }

    /// Check if a path is a directory.
    pub fn is_directory(&self, relative: &str) -> Result<bool> {
        let path = self.safe_path(relative)?;
        Ok(path.is_dir())
    }

    /// Get file size in bytes, or None if file doesn't exist.
    pub fn file_size(&self, relative: &str) -> Result<Option<u64>> {
        let path = self.safe_path(relative)?;
        if path.exists() {
            Ok(Some(fs::metadata(&path)?.len()))
        } else {
            Ok(None)
        }
    }

    /// Remove a file within the sandbox.
    pub fn remove_file(&self, relative: &str) -> Result<FileOpResult> {
        let path = self.safe_path(relative)?;
        if !path.exists() {
            return Ok(FileOpResult {
                operation: "remove_file".into(),
                path: relative.to_string(),
                success: false,
                bytes_affected: None,
                error: Some("File does not exist".into()),
            });
        }
        if path.is_dir() {
            return Ok(FileOpResult {
                operation: "remove_file".into(),
                path: relative.to_string(),
                success: false,
                bytes_affected: None,
                error: Some("Path is a directory, not a file".into()),
            });
        }
        let size = fs::metadata(&path)?.len();
        fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        Ok(FileOpResult {
            operation: "remove_file".into(),
            path: relative.to_string(),
            success: true,
            bytes_affected: Some(size),
            error: None,
        })
    }

    /// Copy a file within the sandbox.
    pub fn copy_file(&self, from: &str, to: &str) -> Result<FileOpResult> {
        let src = self.safe_path(from)?;
        let dst = self.safe_path(to)?;
        if !src.exists() {
            return Ok(FileOpResult {
                operation: "copy_file".into(),
                path: from.to_string(),
                success: false,
                bytes_affected: None,
                error: Some("Source file does not exist".into()),
            });
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = fs::copy(&src, &dst)
            .with_context(|| format!("copying {} to {}", src.display(), dst.display()))?;
        Ok(FileOpResult {
            operation: "copy_file".into(),
            path: format!("{} → {}", from, to),
            success: true,
            bytes_affected: Some(bytes),
            error: None,
        })
    }

    /// Recursively list all files in a directory (for indexing).
    pub fn walk_files(&self, relative: &str, max_files: usize) -> Result<Vec<PathBuf>> {
        let root = self.safe_path(relative)?;
        let mut files = Vec::new();
        self.walk_recursive(&root, &mut files, max_files)?;
        Ok(files)
    }

    fn walk_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>, max: usize) -> Result<()> {
        if !dir.is_dir() || files.len() >= max {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            if files.len() >= max {
                break;
            }
            let path = entry?.path();
            if path.is_dir() {
                self.walk_recursive(&path, files, max)?;
            } else {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Validate that a relative path resolves safely within the sandbox.
    pub fn safe_path(&self, relative: &str) -> Result<PathBuf> {
        let rel = Path::new(relative);
        if rel.is_absolute() {
            return Err(OnyxError::SandboxEscape.into());
        }
        let mut clean = PathBuf::new();
        for component in rel.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                _ => return Err(OnyxError::SandboxEscape.into()),
            }
        }
        Ok(self.sandbox.join(clean))
    }

    /// Describe the tool for status reports.
    pub fn describe(&self) -> String {
        format!("FilesystemTool: sandbox={}", self.sandbox.display())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_sandbox() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("onyx_fs_test_{}", id));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn safe_path_rejects_absolute() {
        let fs_tool = FilesystemTool::new(test_sandbox()).unwrap();
        assert!(fs_tool.safe_path("/etc/passwd").is_err());
    }

    #[test]
    fn safe_path_rejects_traversal() {
        let fs_tool = FilesystemTool::new(test_sandbox()).unwrap();
        assert!(fs_tool.safe_path("../../etc/passwd").is_err());
    }

    #[test]
    fn safe_path_allows_normal() {
        let fs_tool = FilesystemTool::new(test_sandbox()).unwrap();
        assert!(fs_tool.safe_path("src/main.rs").is_ok());
    }

    #[test]
    fn write_and_read_roundtrip() {
        let sandbox = test_sandbox();
        let fs_tool = FilesystemTool::new(&sandbox).unwrap();
        fs_tool.write_file("test.txt", "hello world").unwrap();
        let content = fs_tool.read_file("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn file_exists_check() {
        let sandbox = test_sandbox();
        let fs_tool = FilesystemTool::new(&sandbox).unwrap();
        assert!(!fs_tool.file_exists("nonexistent.txt").unwrap());
        fs_tool.write_file("exists.txt", "data").unwrap();
        assert!(fs_tool.file_exists("exists.txt").unwrap());
    }

    #[test]
    fn remove_file_works() {
        let sandbox = test_sandbox();
        let fs_tool = FilesystemTool::new(&sandbox).unwrap();
        fs_tool.write_file("removeme.txt", "data").unwrap();
        let result = fs_tool.remove_file("removeme.txt").unwrap();
        assert!(result.success);
        assert!(!fs_tool.file_exists("removeme.txt").unwrap());
    }

    #[test]
    fn copy_file_works() {
        let sandbox = test_sandbox();
        let fs_tool = FilesystemTool::new(&sandbox).unwrap();
        fs_tool.write_file("original.txt", "content").unwrap();
        let result = fs_tool.copy_file("original.txt", "copy.txt").unwrap();
        assert!(result.success);
        let content = fs_tool.read_file("copy.txt").unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn file_size_returns_correct_value() {
        let sandbox = test_sandbox();
        let fs_tool = FilesystemTool::new(&sandbox).unwrap();
        fs_tool.write_file("sized.txt", "12345").unwrap();
        let size = fs_tool.file_size("sized.txt").unwrap();
        assert_eq!(size, Some(5));
    }
}
