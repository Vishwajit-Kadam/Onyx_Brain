use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentReport {
    pub is_onedrive_path: bool,
    pub path_has_spaces: bool,
    pub long_path_warning: bool,
    pub potential_overhead_notes: Vec<String>,
}

pub fn environment_report(path: impl AsRef<Path>) -> EnvironmentReport {
    let display = path.as_ref().display().to_string();
    let lower = display.to_lowercase();
    let is_onedrive_path =
        lower.contains("onedrive") || lower.contains("dropbox") || lower.contains("google drive");
    let path_has_spaces = display.contains(' ');
    let long_path_warning = display.len() > 180;
    let mut notes = Vec::new();
    if is_onedrive_path {
        notes.push("Project is inside a synced cloud folder. File sync may increase runtime for file-heavy tasks.".to_string());
    }
    if path_has_spaces {
        notes.push("Path contains spaces; tool invocation remains safe but path handling may add overhead in some environments.".to_string());
    }
    if long_path_warning {
        notes.push("Path is long; Windows path handling may add filesystem overhead.".to_string());
    }
    if notes.is_empty() {
        notes.push("No obvious local path overhead detected.".to_string());
    }
    EnvironmentReport {
        is_onedrive_path,
        path_has_spaces,
        long_path_warning,
        potential_overhead_notes: notes,
    }
}
