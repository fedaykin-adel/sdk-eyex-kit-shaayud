use serde::Deserialize;
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ParsedDevice {
    pub id: String,                  // fingerprint.visitorId
    pub os: Option<String>,          // fingerprint.components.platform.value
    pub browser: Option<String>,     // fingerprint.components.userAgent.value
    pub device_type: Option<String>, // heurística baseada no user-agent
}
