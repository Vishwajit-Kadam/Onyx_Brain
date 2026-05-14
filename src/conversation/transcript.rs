//! Conversation transcript rendering and export.
//!
//! Provides formatting for conversation histories, sanitizing markdown,
//! generating metadata reports, and exporting sessions to disk for external use.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    conversation::{latest_conversation_id, load_messages, ConversationMessage, ConversationRole},
    storage::{save_json, DiskStore},
};

/// Represents a loaded transcript ready for analysis or export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTranscript {
    pub session_id: String,
    pub messages: Vec<ConversationMessage>,
    pub summary: String,
    pub total_tokens_estimate: usize,
    pub turn_count: usize,
}

/// A structured report produced after exporting a transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptExportReport {
    pub session_id: String,
    pub export_path: String,
    pub files_written: Vec<String>,
    pub token_estimate: usize,
    pub exported_at: DateTime<Utc>,
}

/// Load a full transcript for a specific session.
pub fn load_transcript(store: &DiskStore, selector: &str) -> Result<ConversationTranscript> {
    let session_id = resolve_conversation_selector(store, selector)?;
    let messages = load_messages(store, &session_id)?;
    let turn_count = messages
        .iter()
        .filter(|m| m.role == ConversationRole::User)
        .count();
    let total_tokens_estimate = messages.iter().map(|m| estimate_tokens(&m.content)).sum();

    let summary = if messages.is_empty() {
        format!("Empty conversation {}", session_id)
    } else {
        format!(
            "Conversation {} ({} turns, ~{} tokens)",
            session_id, turn_count, total_tokens_estimate
        )
    };

    Ok(ConversationTranscript {
        session_id,
        messages,
        summary,
        total_tokens_estimate,
        turn_count,
    })
}

/// Export a transcript to the sandbox `exports` directory in Markdown and JSON formats.
pub fn export_transcript(store: &DiskStore, selector: &str) -> Result<TranscriptExportReport> {
    let transcript = load_transcript(store, selector)?;

    let export_dir = store
        .paths
        .sandbox
        .join("exports")
        .join("conversations")
        .join(&transcript.session_id);
    fs::create_dir_all(&export_dir)
        .with_context(|| format!("creating export directory {}", export_dir.display()))?;

    let transcript_md = render_transcript_markdown(&transcript);
    let transcript_path = export_dir.join("transcript.md");
    let summary_path = export_dir.join("summary.md");
    let metadata_path = export_dir.join("metadata.json");

    fs::write(&transcript_path, transcript_md)
        .with_context(|| format!("writing {}", transcript_path.display()))?;

    let summary_content = format!(
        "# Session Summary\n\n- **Session ID:** {}\n- **Turns:** {}\n- **Estimated Tokens:** {}\n- **Exported At:** {}\n\n{}",
        transcript.session_id,
        transcript.turn_count,
        transcript.total_tokens_estimate,
        Utc::now().to_rfc3339(),
        transcript.summary
    );
    fs::write(&summary_path, summary_content)?;

    let report = TranscriptExportReport {
        session_id: transcript.session_id.clone(),
        export_path: export_dir.display().to_string(),
        files_written: vec![
            transcript_path.display().to_string(),
            summary_path.display().to_string(),
            metadata_path.display().to_string(),
        ],
        token_estimate: transcript.total_tokens_estimate,
        exported_at: Utc::now(),
    };

    save_json(&metadata_path, &report)?;
    Ok(report)
}

/// Resolve `"latest"` to the actual active session ID, or return the ID verbatim.
pub fn resolve_conversation_selector(store: &DiskStore, selector: &str) -> Result<String> {
    if selector == "latest" {
        latest_conversation_id(store).context("no conversation sessions found")
    } else {
        Ok(selector.to_string())
    }
}

/// Search within a loaded transcript for a specific string query.
pub fn search_transcript(
    transcript: &ConversationTranscript,
    query: &str,
) -> Vec<ConversationMessage> {
    let lower_query = query.to_lowercase();
    transcript
        .messages
        .iter()
        .filter(|m| m.content.to_lowercase().contains(&lower_query))
        .cloned()
        .collect()
}

/// Format the transcript as a continuous Markdown document.
fn render_transcript_markdown(transcript: &ConversationTranscript) -> String {
    let mut out = format!(
        "# Conversation Transcript\n\n**Session:** {}\n**Tokens:** ~{}\n\n---\n\n",
        transcript.session_id, transcript.total_tokens_estimate
    );

    for message in &transcript.messages {
        let role_header = match message.role {
            ConversationRole::User => "👤 **User**",
            ConversationRole::Onyx => "🧠 **Onyx Brain**",
            ConversationRole::System => "⚙️ **System**",
            ConversationRole::Tool => "🛠️ **Tool**",
            ConversationRole::Reviewer => "🔎 **Reviewer**",
        };

        // Ensure content is well-spaced and sanitized
        let safe_content = message.content.trim();
        out.push_str(&format!(
            "### {}\n\n{}\n\n---\n\n",
            role_header, safe_content
        ));
    }

    out
}

/// Quick token estimation heuristic (4 characters ≈ 1 token).
fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::message;

    #[test]
    fn estimate_tokens_is_reasonable() {
        assert_eq!(estimate_tokens("hello"), 1);
        assert_eq!(estimate_tokens("this is a test of the token estimator"), 9);
    }

    #[test]
    fn render_transcript_markdown_formats_correctly() {
        let transcript = ConversationTranscript {
            session_id: "test_session".into(),
            messages: vec![
                message(ConversationRole::User, "Hello Onyx"),
                message(ConversationRole::Onyx, "Hello User!"),
            ],
            summary: "summary".into(),
            total_tokens_estimate: 5,
            turn_count: 1,
        };

        let md = render_transcript_markdown(&transcript);
        assert!(md.contains("# Conversation Transcript"));
        assert!(md.contains("👤 **User**"));
        assert!(md.contains("Hello Onyx"));
        assert!(md.contains("🧠 **Onyx Brain**"));
        assert!(md.contains("Hello User!"));
    }

    #[test]
    fn search_transcript_finds_matches_case_insensitive() {
        let transcript = ConversationTranscript {
            session_id: "test".into(),
            messages: vec![
                message(ConversationRole::User, "Do you know Rust?"),
                message(ConversationRole::Onyx, "Yes, I am written in Rust."),
                message(ConversationRole::System, "Rebooting..."),
            ],
            summary: "".into(),
            total_tokens_estimate: 10,
            turn_count: 1,
        };

        let hits = search_transcript(&transcript, "rust");
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().any(|m| m.role == ConversationRole::User));
        assert!(hits.iter().any(|m| m.role == ConversationRole::Onyx));

        let none = search_transcript(&transcript, "python");
        assert!(none.is_empty());
    }
}
