//! Onyx error types — intentionally small.
//!
//! Central error enum for safety-critical failures. Kept minimal so that
//! error variants are easy to audit. New variants should only be added when
//! a genuinely new failure mode is discovered.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OnyxError {
    #[error("path escapes sandbox")]
    SandboxEscape,
    #[error("command is not in the safe allowlist")]
    CommandNotAllowed,
    #[error("missing item: {0}")]
    Missing(String),
}
