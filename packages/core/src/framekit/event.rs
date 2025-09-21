use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedEvent {
    pub id: String,
    pub r#type: String,
    pub ts_ms: i64,
}

pub fn extract_event_info(
    event_id: &Option<String>,
    event_type: &Option<String>,
    method: &str,
    path: &str,
    identity_id: &str,
    session_id: &str,
    timestamp: DateTime<Utc>,
) -> ParsedEvent {
    let id = event_id.clone().unwrap_or_else(|| {
        format!(
            "evt:{}:{}:{}",
            identity_id,
            session_id,
            timestamp.timestamp()
        )
    });
    let etype = event_type
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    ParsedEvent {
        id: id,
        r#type: etype,
        ts_ms: timestamp.timestamp_millis(),
    }
}
