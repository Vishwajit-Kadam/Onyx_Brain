pub mod actions;
pub mod errors;
pub mod events;
pub mod models;

use std::path::{Path, PathBuf};

use crate::{memory::MemoryType, Brain};

pub use errors::*;
pub use events::*;
pub use models::*;

#[derive(Debug, Clone)]
pub struct AppApi {
    root: PathBuf,
}

impl AppApi {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn brain(&self) -> Brain {
        Brain::new(&self.root)
    }
}

fn row(title: &str, memory_type: MemoryType, count: usize) -> MemorySummaryRow {
    MemorySummaryRow {
        title: format!("{title}: {count}"),
        memory_type: format!("{:?}", memory_type),
        importance: if count > 0 { 0.8 } else { 0.2 },
    }
}
