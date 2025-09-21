use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ParsedDevice {
    pub id: String,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub device_type: Option<String>,
}

pub fn extract_device_info(fingerprint: &Value) -> ParsedDevice {
    let id = fingerprint
        .get("visitorId")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let components = fingerprint.get("components").unwrap_or(&Value::Null);

    let os = components
        .get("platform")
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .map(String::from);

    let browser = components
        .get("userAgent")
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .map(String::from);

    let device_type = browser
        .as_ref()
        .map(|ua| {
            if ua.contains("Mobile") {
                "mobile"
            } else if ua.contains("Tablet") {
                "tablet"
            } else {
                "desktop"
            }
        })
        .map(String::from);

    ParsedDevice {
        id,
        os,
        browser,
        device_type,
    }
}
