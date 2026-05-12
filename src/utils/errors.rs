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
