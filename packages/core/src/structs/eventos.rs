use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize, Serialize)]
pub struct Viewport {
    pub w: i32,
    pub h: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Click {
    pub x: i32,
    pub y: i32,
    pub t: i32,
    pub b: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Wheel {
    pub ticks: i32,
    pub dy_sum: f64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GeoPayload {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct EventoInput {
    pub shaayud_id: String,
    pub fingerprint: JsonValue,
    pub ip: String,
    pub user_agent: String,
    pub header: JsonValue,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub session_id: Option<String>,
    pub event_id: Option<String>,
    pub event_type: Option<String>,

    pub geo: Option<GeoPayload>,

    pub front_url: Option<String>,
    pub front_path: Option<String>,
    pub front_referrer: Option<String>,

    pub backend_path: Option<String>,
    pub backend_method: Option<String>,
    pub backend_host: Option<String>,

    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
    pub viewport: Option<Viewport>,
    pub points_deflate_b64: Option<String>,
    pub clicks: Option<Vec<Click>>,
    pub wheel: Option<Wheel>,
}
