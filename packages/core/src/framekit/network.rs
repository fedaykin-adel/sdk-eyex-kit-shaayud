use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedNetwork {
    pub ip: Option<String>,
    pub ua_raw: Option<String>,
}
pub fn extract_network_info(ip: &str, user_agent: &str) -> ParsedNetwork {
    ParsedNetwork {
        ip: if ip.is_empty() {
            None
        } else {
            Some(ip.to_string())
        },
        ua_raw: if user_agent.is_empty() {
            None
        } else {
            Some(user_agent.to_string())
        },
    }
}
