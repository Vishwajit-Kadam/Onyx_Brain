use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppApiError {
    #[error("{message}")]
    Friendly {
        message: String,
        details: Option<String>,
        doctor_recommended: bool,
    },
}

impl AppApiError {
    pub fn friendly(message: impl Into<String>) -> Self {
        Self::Friendly {
            message: message.into(),
            details: None,
            doctor_recommended: false,
        }
    }

    pub fn with_details(
        message: impl Into<String>,
        details: impl Into<String>,
        doctor_recommended: bool,
    ) -> Self {
        Self::Friendly {
            message: message.into(),
            details: Some(details.into()),
            doctor_recommended,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Friendly { message, .. } => message,
        }
    }

    pub fn details(&self) -> Option<&str> {
        match self {
            Self::Friendly { details, .. } => details.as_deref(),
        }
    }

    pub fn doctor_recommended(&self) -> bool {
        match self {
            Self::Friendly {
                doctor_recommended, ..
            } => *doctor_recommended,
        }
    }
}

impl From<anyhow::Error> for AppApiError {
    fn from(error: anyhow::Error) -> Self {
        let details = error.to_string();
        let lower = details.to_ascii_lowercase();
        let doctor_recommended = lower.contains("json")
            || lower.contains("index")
            || lower.contains("state")
            || lower.contains("conversation")
            || lower.contains("artifact");
        let message = if doctor_recommended {
            "Local state could not be loaded cleanly. Run Doctor to inspect and repair indexes."
        } else if lower.contains("no artifacts") {
            "No generated artifacts are available yet."
        } else if lower.contains("no artifact packs") {
            "No artifact packs are available yet."
        } else {
            "Onyx Brain could not complete that action safely."
        };
        Self::with_details(message, details, doctor_recommended)
    }
}

pub type AppApiResult<T> = Result<T, AppApiError>;
