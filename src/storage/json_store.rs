use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating parent directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json).with_context(|| format!("writing {}", path.display()))
}

pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

pub fn try_load_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if path.exists() {
        Ok(Some(load_json(path)?))
    } else {
        Ok(None)
    }
}
