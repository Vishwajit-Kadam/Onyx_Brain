use anyhow::{Context, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::utils::errors::OnyxError;

#[derive(Debug, Clone)]
pub struct FilesystemTool {
    sandbox: PathBuf,
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
}
