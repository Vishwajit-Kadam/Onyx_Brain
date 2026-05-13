use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsistencyReport {
    pub score: f32,
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsistencyRepairResult {
    pub issues_found: usize,
    pub issues_repaired: usize,
    pub remaining_issues: usize,
    pub score_after_repair: f32,
}

pub fn check_consistency(files: &[String]) -> ConsistencyReport {
    let mut issues = Vec::new();
    let presentation = files
        .iter()
        .find(|path| path.ends_with("presentation.md") || path.ends_with("slide_deck.md"));
    let notes = files.iter().find(|path| path.ends_with("speaker_notes.md"));
    if let (Some(deck), Some(notes)) = (presentation, notes) {
        let deck_count = count_occurrences(deck, "## Slide");
        let notes_count = count_occurrences(notes, "## Slide");
        if deck_count > 0 && notes_count > 0 && deck_count != notes_count {
            issues.push(ConsistencyIssue {
                message: "slide count does not match speaker notes".to_string(),
                severity: "warning".to_string(),
            });
        }
    }
    for path in files {
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.contains("C:\\Users\\") || content.contains("/Users/") {
            issues.push(ConsistencyIssue {
                message: format!("personal path reference in {}", file_name(path)),
                severity: "critical".to_string(),
            });
        }
    }
    ConsistencyReport {
        score: (1.0 - issues.len() as f32 * 0.15).clamp(0.0, 1.0),
        issues,
    }
}

pub fn repair_consistency(files: &[String]) -> ConsistencyRepairResult {
    let before = check_consistency(files);
    let mut repaired = 0;
    let presentation = files
        .iter()
        .find(|path| path.ends_with("presentation.md") || path.ends_with("slide_deck.md"));
    let notes = files.iter().find(|path| path.ends_with("speaker_notes.md"));
    if let (Some(deck), Some(notes)) = (presentation, notes) {
        let deck_count = count_occurrences(deck, "## Slide");
        let notes_count = count_occurrences(notes, "## Slide");
        if deck_count > 0 && notes_count < deck_count {
            let mut content = fs::read_to_string(notes).unwrap_or_default();
            for slide in notes_count + 1..=deck_count {
                content.push_str(&format!(
                    "\n## Slide {slide}\nAdd speaker notes aligned with slide {slide}.\n"
                ));
            }
            if fs::write(notes, content).is_ok() {
                repaired += 1;
            }
        }
    }
    let after = check_consistency(files);
    ConsistencyRepairResult {
        issues_found: before.issues.len(),
        issues_repaired: repaired,
        remaining_issues: after.issues.len(),
        score_after_repair: after.score,
    }
}

fn count_occurrences(path: &str, needle: &str) -> usize {
    fs::read_to_string(path)
        .unwrap_or_default()
        .matches(needle)
        .count()
}

fn file_name(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}
