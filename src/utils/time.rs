//! Time utilities — intentionally small.
//!
//! Wraps `chrono::Utc::now()` and provides a timestamp slug for file naming.
//! Kept separate so callers don't need to import chrono directly.
use chrono::{DateTime, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn timestamp_slug() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}
