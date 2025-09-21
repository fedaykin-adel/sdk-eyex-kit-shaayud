use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParsedSession {
    pub id: String,
    pub started_at_iso: String,
}

pub fn extract_session_info(
    header: &JsonValue,
    payload_sesssion_id: &Option<String>,
    device_id: &str,
    timestamp: DateTime<Utc>,
) -> ParsedSession {
    let hearder_session = header
        .get("sessionId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let id = payload_sesssion_id
        .as_ref()
        .cloned()
        .or(hearder_session)
        .unwrap_or_else(|| format!("sess:{}:{}", device_id, timestamp.timestamp()));

    ParsedSession {
        id: id,
        started_at_iso: timestamp.to_rfc3339(),
    }
}
