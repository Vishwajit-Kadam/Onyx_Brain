use chrono::{DateTime, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn timestamp_slug() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}
