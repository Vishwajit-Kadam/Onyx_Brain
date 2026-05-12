use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::utils::errors::OnyxError;

#[derive(Debug, Clone)]
pub struct TerminalTool {
    sandbox: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: Vec<String>,
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl TerminalTool {
    pub fn new(sandbox: impl AsRef<Path>) -> Result<Self> {
        let sandbox = sandbox.as_ref().canonicalize()?;
        Ok(Self { sandbox })
    }

    pub fn run(&self, args: &[&str], cwd: impl AsRef<Path>) -> Result<CommandResult> {
        if !is_allowed(args) {
            return Err(OnyxError::CommandNotAllowed.into());
        }
        let cwd = cwd.as_ref().canonicalize()?;
        if !cwd.starts_with(&self.sandbox) {
            return Err(OnyxError::SandboxEscape.into());
        }
        let Some((program, rest)) = args.split_first() else {
            return Err(OnyxError::CommandNotAllowed.into());
        };
        let output = Command::new(program)
            .args(rest)
            .current_dir(&cwd)
            .output()
            .with_context(|| format!("running {:?}", args))?;
        Ok(CommandResult {
            command: args.iter().map(|arg| (*arg).to_string()).collect(),
            status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn is_allowed(args: &[&str]) -> bool {
    matches!(
        args,
        ["cargo", "--version"] | ["rustc", "--version"] | ["cargo", "check"] | ["cargo", "test"]
    )
}
