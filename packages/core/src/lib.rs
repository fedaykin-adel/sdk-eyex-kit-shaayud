mod framekit;
pub mod structs;

pub use framekit::fingerprint::{ParsedDevice, extract_device_info};
use neo4rs::Graph;
use neo4rs::query;
use serde_json::json;
use std::sync::Arc;
pub use structs::eventos::EventoInput;

use crate::framekit::event::ParsedEvent;
use crate::framekit::event::extract_event_info;
use crate::framekit::geo::ParsedGeo;
use crate::framekit::geo::extract_geo_info;
use crate::framekit::identity::ParsedIdentity;
use crate::framekit::identity::extract_identity_info;
use crate::framekit::network::ParsedNetwork;
use crate::framekit::network::extract_network_info;
use crate::framekit::score::{RuleSet, eval_score};
use crate::framekit::session::ParsedSession;
use crate::framekit::session::extract_session_info;
// pub fn verify_user(data: UserData) -> VerificationResult {
//     let score = data.typing_speed.min(100.0);

//     let valid = score >= 50.0;
//     VerificationResult { valid, score }
// }

pub async fn handle_ingest(data: EventoInput, graph: &Arc<Graph>) -> Result<(), neo4rs::Error> {
    match serde_json::to_string_pretty(&data) {
        Ok(j) => tracing::debug!("📨 EventoInput:\n{}", j),
        Err(e) => tracing::warn!("⚠️  Falha ao serializar EventoInput: {e}"),
    }

    let dev: ParsedDevice = extract_device_info(&data.fingerprint);
    let device_id = dev.id;
    let os = dev.os.unwrap_or_default();
    let browser = dev.browser.unwrap_or_default();
    let device_type = dev.device_type.unwrap_or_else(|| "unknown".to_string());

    let idy: ParsedIdentity = extract_identity_info(&data.shaayud_id);
    let identity_id = idy.id;

    let sess: ParsedSession =
        extract_session_info(&data.header, &data.session_id, &device_id, data.timestamp);

    let net: ParsedNetwork = extract_network_info(&data.ip, &data.user_agent);

    let evt: ParsedEvent = extract_event_info(
        &data.event_id,
        &data.event_type,
        &data.method,
        &data.path,
        &identity_id,
        &sess.id,
        data.timestamp,
    );

    let geo: ParsedGeo = extract_geo_info(&data.geo, &data.header);

    let clicks_json: Option<String> = data
        .clicks
        .as_ref()
        .and_then(|v| serde_json::to_string(v).ok());

    let (viewport_w, viewport_h) = match &data.viewport {
        Some(v) => (Some(v.w), Some(v.h)),
        None => (None, None),
    };

    let (wheel_ticks, wheel_dy_sum) = match &data.wheel {
        Some(w) => (Some(w.ticks), Some(w.dy_sum)),
        None => (None, None),
    };
    let bag = json!({
    "event": {
        "type": evt.r#type,
        "front_path": data.front_path.clone().unwrap_or_default(),
        "user_agent": net.ua_raw.clone(),
        "ua_empty": net.ua_raw.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
    },
    "feat": {
        // placeholders: você pode preencher consultando o neo4j em outra query se quiser
        "identity_known_devices_recent": 1, // TODO: query real
        "is_new_device_for_identity": true, // TODO: compare deviceId com devices da identity
        "session_ip_country_changed": false, // TODO: se geo.country diferente do primeiro IP da sessão
        "mouse_points_len": data.points_deflate_b64.as_ref().map(|s| s.len()).unwrap_or(0)
    }
    });
    let rules: RuleSet = serde_json::from_str(include_str!("../rules.json")).unwrap();
    let score = eval_score(&bag, &rules);

    let params = json!({
        "identityId": identity_id,
        "deviceId": device_id,
        "os": os,
        "browser": browser,
        "deviceType": device_type,
        "ts_ms": evt.ts_ms,
        "sessionId": sess.id,

        "ip": net.ip,
        "uaRaw": net.ua_raw,
        "eventId": evt.id,
        "eventType": evt.r#type,

        "geoKey": if geo.key.is_empty() { None } else { Some(geo.key) },
        "geoCountry": geo.country,
        "geoRegion": geo.region,
        "geoCity": geo.city,
        "geoTimezone": geo.timezone,
        "geoLat": geo.latitude,
        "geoLng": geo.longitude,

        "front_url": data.front_url,
        "front_path": data.front_path,
        "front_referrer": data.front_referrer,

        "backend_path": data.backend_path,
        "backend_method": data.backend_method,
        "backend_host": data.backend_host,

        "mb_ts_start": data.ts_start,
        "mb_ts_end": data.ts_end,
        "mb_vw_w": viewport_w,
        "mb_vw_h": viewport_h,
        "mb_points_b64": data.points_deflate_b64,
        "mb_clicks_json": clicks_json,
        "mb_wheel_ticks": wheel_ticks,
        "mb_wheel_dy_sum": wheel_dy_sum,

        "score_total":score.total,
        "score_matched":serde_json::to_string(&score.matched).unwrap_or("[]".into())
    });
    tracing::debug!("🧪 Params p/ Neo4j: {params}");

    const UPSERT: &str = r#"
        WITH datetime({epochMillis: $ts_ms}) AS ts
MERGE (i:Identity {id: $identityId})
  ON CREATE SET i.createdAt = ts
WITH i, ts
MERGE (d:Device {id: $deviceId})
  ON CREATE SET d.first_seen = ts
SET d.os = $os, d.browser = $browser, d.device_type = $deviceType, d.last_seen = ts
MERGE (i)-[:USES]->(d)
WITH i, d, ts
MERGE (s:Session {id: $sessionId})
  ON CREATE SET s.startedAt = ts
SET s.ip = coalesce($ip, s.ip),
    s.ua = coalesce($uaRaw, s.ua)
MERGE (d)-[:OPENED]->(s)
WITH s, ts
MERGE (e:Event {id: $eventId})
SET e.type = $eventType,
    e.ts = ts,
    // FRONT
    e.front_url = coalesce($front_url, e.front_url),
    e.front_path = coalesce($front_path, e.front_path),
    e.front_referrer = coalesce($front_referrer, e.front_referrer),
    // BACKEND
    e.backend_path = coalesce($backend_path, e.backend_path),
    e.backend_method = coalesce($backend_method, e.backend_method),
    e.backend_host = coalesce($backend_host, e.backend_host)
MERGE (s)-[:EMITTED]->(e)
WITH s, e, $ip AS pip, $eventType AS et
FOREACH (_ IN CASE WHEN pip IS NULL OR pip = '' THEN [] ELSE [1] END |
  MERGE (ip:IP {addr: pip})
  MERGE (s)-[:FROM_IP]->(ip)
)

// ---------- GEO ----------
WITH s, e, $geoCountry AS gc, $geoRegion AS gr, $geoCity AS gci, $geoTz AS gtz
FOREACH (_ IN CASE WHEN (gc IS NULL OR gc = '') AND (gr IS NULL OR gr = '') AND (gci IS NULL OR gci = '') AND (gtz IS NULL OR gtz = '') THEN [] ELSE [1] END |
  MERGE (g:Geo {
    country: toUpper(coalesce(gc,'')),
    region:  coalesce(gr,''),
    city:    coalesce(gci,''),
    tz:      coalesce(gtz,'')
  })
  MERGE (s)-[:FROM_GEO]->(g)
  SET  s.country = coalesce(s.country, toUpper(gc)),
       s.region  = coalesce(s.region, gr),
       s.city    = coalesce(s.city, gci),
       s.tz      = coalesce(s.tz, gtz),
       e.geo_country = coalesce(e.geo_country, toUpper(gc)),
       e.geo_region  = coalesce(e.geo_region, gr),
       e.geo_city    = coalesce(e.geo_city, gci),
       e.geo_tz      = coalesce(e.geo_tz, gtz)
)

// ---------- MOUSE / SCORE ----------
SET e.mouse = coalesce(e.mouse, false) OR ($eventType = 'mouse_batch')
SET e.mouse_ts_start     = CASE WHEN $eventType = 'mouse_batch' AND $mb_ts_start     IS NOT NULL THEN $mb_ts_start     ELSE e.mouse_ts_start     END
SET e.mouse_ts_end       = CASE WHEN $eventType = 'mouse_batch' AND $mb_ts_end       IS NOT NULL THEN $mb_ts_end       ELSE e.mouse_ts_end       END
SET e.mouse_vw_w         = CASE WHEN $eventType = 'mouse_batch' AND $mb_vw_w         IS NOT NULL THEN $mb_vw_w         ELSE e.mouse_vw_w         END
SET e.mouse_vw_h         = CASE WHEN $eventType = 'mouse_batch' AND $mb_vw_h         IS NOT NULL THEN $mb_vw_h         ELSE e.mouse_vw_h         END
SET e.mouse_clicks       = CASE WHEN $eventType = 'mouse_batch' AND $mb_clicks_json  IS NOT NULL THEN $mb_clicks_json  ELSE e.mouse_clicks       END
SET e.mouse_wheel_ticks  = CASE WHEN $eventType = 'mouse_batch' AND $mb_wheel_ticks  IS NOT NULL THEN $mb_wheel_ticks  ELSE e.mouse_wheel_ticks  END
SET e.mouse_wheel_dy_sum = CASE WHEN $eventType = 'mouse_batch' AND $mb_wheel_dy_sum IS NOT NULL THEN $mb_wheel_dy_sum ELSE e.mouse_wheel_dy_sum END
SET e.mouse_points_b64   = CASE WHEN $eventType = 'mouse_batch' AND $mb_points_b64   IS NOT NULL THEN $mb_points_b64   ELSE e.mouse_points_b64   END

SET e.score = $score_total

WITH e
CREATE (sc:Score {
  id: randomUUID(),
  total: $score_total,
  matched: $score_matched,
  at: datetime()
})
MERGE (e)-[:SCORED]->(sc)
// RETURN 1
        "#;
    let mut tx = graph.start_txn().await.map_err(|e| {
        tracing::error!("🚫 start_tx falhou: {e}");
        e
    })?;
    let mut q = query(UPSERT);
    for (k, v) in params.as_object().unwrap() {
        match v {
            serde_json::Value::String(s) => {
                q = q.param(k, s.as_str());
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    q = q.param(k, i);
                } else if let Some(f) = n.as_f64() {
                    q = q.param(k, f);
                }
            }
            serde_json::Value::Bool(b) => {
                q = q.param(k, *b);
            }
            serde_json::Value::Null => {
                q = q.param::<Option<&str>>(k, None);
            }
            _ => {
                // For unsupported types, skip or handle as needed
                tracing::warn!("Unsupported param type for key: {}", k);
            }
        }
    }
    if let Err(e) = tx.run(q).await {
        tracing::error!("💥 Erro ao executar Cypher: {e}");
        return Err(e);
    }
    if let Err(e) = tx.commit().await {
        tracing::error!("💥 Erro ao dar commit na transação: {e}");
        return Err(e);
    }
    tracing::info!("✅ ingest concluído");

    Ok(())
}
