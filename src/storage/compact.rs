use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

use crate::storage::{load_json, save_json};

pub fn compact_json<T>(path: &Path) -> Result<()>
where
    T: Serialize + DeserializeOwned,
{
    let value: T = load_json(path)?;
    save_json(path, &value)
}
