use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::structs::eventos::GeoPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedGeo {
    pub key: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub fn extract_geo_info(geo: &Option<GeoPayload>, headers: &JsonValue) -> ParsedGeo {
    let (country, region, city, timezone, latitude, longitude) = if let Some(g) = geo {
        (
            g.country.clone(),
            g.region.clone(),
            g.city.clone(),
            g.timezone.clone(),
            g.latitude,
            g.longitude,
        )
    } else {
        let get = |k: &str| {
            headers
                .get(k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };
        let country = get("x-geo-country");
        let region = get("x-geo-region");
        let city = get("x-geo-city");
        let timezone = get("x-geo-timezone");
        (country, region, city, timezone, None, None)
    };
    let key = format!(
        "{}|{}|{}|{}",
        country.clone().unwrap_or_default(),
        region.clone().unwrap_or_default(),
        city.clone().unwrap_or_default(),
        timezone.clone().unwrap_or_default()
    );
    ParsedGeo {
        key,
        country,
        region,
        city,
        timezone,
        latitude,
        longitude,
    }
}
