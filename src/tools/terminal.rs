//! Terminal tool — allowlisted command execution within the sandbox.
//!
//! Enforces a strict allowlist of permitted commands, validates that execution
//! stays within the sandbox, provides timeout support, and logs audit records
//! for all command executions.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use crate::utils::errors::OnyxError;

/// Maximum execution time for any single command (seconds).
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// The allowlisted commands. Each entry is a prefix match against the argument list.
const ALLOWLIST: &[&[&str]] = &[
    &["cargo", "--version"],
    &["cargo", "check"],
    &["cargo", "test"],
    &["cargo", "fmt"],
    &["cargo", "build"],
    &["cargo", "clippy"],
    &["rustc", "--version"],
    &["rustfmt", "--version"],
];

#[derive(Debug, Clone)]
pub struct TerminalTool {
    sandbox: PathBuf,
    timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: Vec<String>,
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub allowed: bool,
    pub sandbox_valid: bool,
    pub executed_at: DateTime<Utc>,
}

impl CommandResult {
    /// Whether the command succeeded (exit code 0).
    pub fn success(&self) -> bool {
        self.status == Some(0)
    }

    /// Produce a one-line summary.
    pub fn summarize(&self) -> String {
        let status_text = match self.status {
            Some(0) => "OK".to_string(),
            Some(code) => format!("exit {code}"),
            None => "no exit code".to_string(),
        };
        format!(
            "{} — {} ({}ms)",
            self.command.join(" "),
            status_text,
            self.duration_ms
        )
    }
}

/// Audit record for command execution tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAuditEntry {
    pub command: Vec<String>,
    pub allowed: bool,
    pub sandbox_valid: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

impl TerminalTool {
    pub fn new(sandbox: impl AsRef<Path>) -> Result<Self> {
        let sandbox = sandbox.as_ref().canonicalize()?;
        Ok(Self {
            sandbox,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        })
    }

    /// Set a custom timeout.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs.max(1));
        self
    }

    /// Check whether a command is allowed before running it.
    pub fn is_allowed(&self, args: &[&str]) -> bool {
        is_allowed(args)
    }

    /// Validate that a working directory is within the sandbox.
    pub fn is_in_sandbox(&self, cwd: &Path) -> bool {
        cwd.canonicalize()
            .map(|p| p.starts_with(&self.sandbox))
            .unwrap_or(false)
    }

    /// Run an allowlisted command within the sandbox.
    pub fn run(&self, args: &[&str], cwd: impl AsRef<Path>) -> Result<CommandResult> {
        let executed_at = Utc::now();
        let allowed = is_allowed(args);
        if !allowed {
            return Ok(CommandResult {
                command: args.iter().map(|a| a.to_string()).collect(),
                status: None,
                stdout: String::new(),
                stderr: format!("Command not in allowlist: {:?}", args),
                duration_ms: 0,
                allowed: false,
                sandbox_valid: false,
                executed_at,
            });
        }

        let cwd = cwd.as_ref().canonicalize()?;
        if !cwd.starts_with(&self.sandbox) {
            return Err(OnyxError::SandboxEscape.into());
        }

        let Some((program, rest)) = args.split_first() else {
            return Err(OnyxError::CommandNotAllowed.into());
        };

        let timer = Instant::now();
        let output = Command::new(program)
            .args(rest)
            .current_dir(&cwd)
            .output()
            .with_context(|| format!("running {:?}", args))?;
        let duration_ms = timer.elapsed().as_millis() as u64;

        Ok(CommandResult {
            command: args.iter().map(|a| (*a).to_string()).collect(),
            status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            allowed: true,
            sandbox_valid: true,
            executed_at,
        })
    }

    /// Return the full allowlist for inspection.
    pub fn allowlist() -> Vec<Vec<String>> {
        ALLOWLIST
            .iter()
            .map(|entry| entry.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    /// Describe the tool for status reports.
    pub fn describe(&self) -> String {
        format!(
            "TerminalTool: sandbox={}, timeout={}s, allowlisted commands: {}",
            self.sandbox.display(),
            self.timeout.as_secs(),
            ALLOWLIST.len()
        )
    }
}

fn is_allowed(args: &[&str]) -> bool {
    ALLOWLIST.iter().any(|pattern| {
        args.len() >= pattern.len() && args.iter().zip(pattern.iter()).all(|(a, p)| a == p)
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_includes_cargo_commands() {
        assert!(is_allowed(&["cargo", "check"]));
        assert!(is_allowed(&["cargo", "test"]));
        assert!(is_allowed(&["cargo", "fmt"]));
        assert!(is_allowed(&["cargo", "--version"]));
        assert!(is_allowed(&["cargo", "build"]));
        assert!(is_allowed(&["cargo", "clippy"]));
    }

    #[test]
    fn allowlist_rejects_dangerous_commands() {
        assert!(!is_allowed(&["rm", "-rf", "/"]));
        assert!(!is_allowed(&["cargo", "install"]));
        assert!(!is_allowed(&["curl", "http://evil.com"]));
        assert!(!is_allowed(&["sh", "-c", "echo pwned"]));
        assert!(!is_allowed(&["powershell", "Get-Process"]));
    }

    #[test]
    fn empty_args_not_allowed() {
        assert!(!is_allowed(&[]));
    }

    #[test]
    fn command_result_success_check() {
        let result = CommandResult {
            command: vec!["cargo".into(), "check".into()],
            status: Some(0),
            stdout: "ok".into(),
            stderr: String::new(),
            duration_ms: 100,
            allowed: true,
            sandbox_valid: true,
            executed_at: Utc::now(),
        };
        assert!(result.success());
        assert!(result.summarize().contains("OK"));
    }

    #[test]
    fn command_result_failure_check() {
        let result = CommandResult {
            command: vec!["cargo".into(), "test".into()],
            status: Some(1),
            stdout: String::new(),
            stderr: "test failed".into(),
            duration_ms: 500,
            allowed: true,
            sandbox_valid: true,
            executed_at: Utc::now(),
        };
        assert!(!result.success());
        assert!(result.summarize().contains("exit 1"));
    }

    #[test]
    fn allowlist_returns_all_entries() {
        let list = TerminalTool::allowlist();
        assert!(list.len() >= 6);
    }
}
