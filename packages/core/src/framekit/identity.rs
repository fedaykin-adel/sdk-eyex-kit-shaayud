use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIdentity {
    pub id: String,
    pub user_id: Option<String>,
}
pub fn extract_identity_info(shaayud_id: &str) -> ParsedIdentity {
    ParsedIdentity {
        id: shaayud_id.to_string(),
        user_id: None,
    }
}
