use napi::bindgen_prelude::*;
use napi_derive::napi;
use shaayud_core::structs::eventos::EventoInput;

// use reqwest::blocking::Client;
// use shaayud_ffi_macros::shaayud_export;

// #[shaayud_export]
// #[napi]
// pub fn verify_user_json(input: String) -> Result<String> {
//     let data: UserData = serde_json::from_str(&input)
//         .map_err(|e| Error::from_reason(format!("Invalid input: {}", e)))?;

//     let result: VerificationResult = verify_user(data);

//     let output = serde_json::to_string(&result)
//         .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))?;
//     Ok(output)
// }

// #[shaayud_export]
#[napi]
pub fn ingest(input: String) -> Result<String> {
    println!("📥 ingest() input bruto: {}", input);
    let mut v: serde_json::Value = serde_json::from_str(&input)
        .map_err(|e| Error::from_reason(format!("Invalid input: {e}")))?;

    fn ms_to_s(x: &mut serde_json::Value) {
        if let Some(n) = x.as_i64() {
            if n > i64::from(i32::MAX) {
                *x = serde_json::Value::from((n / 1000) as i32);
            }
        }
    }
    if let Some(obj) = v.as_object_mut() {
        if let Some(t) = obj.get_mut("timestamp") {
            ms_to_s(t);
        }
        if let Some(t) = obj.get_mut("ts_start") {
            ms_to_s(t);
        }
        if let Some(t) = obj.get_mut("ts_end") {
            ms_to_s(t);
        }
    }
    let data: EventoInput = serde_json::from_str(&input)
        .map_err(|e| Error::from_reason(format!("Invalid input: {}", e)))?;

    // Se EventoInput não tiver Debug, troque por:
    // println!("✅ EventoInput parseado: {}", serde_json::to_string(&data).unwrap());
    println!("✅ EventoInput parseado: {:?}", data);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| Error::from_reason(format!("Client build error: {e}")))?;

    let res = client
        .post("https://sdk-backend-flame.vercel.app/api/ingest")
        .header("content-type", "application/json")
        .json(&data)
        .send()
        .map_err(|e| Error::from_reason(format!("HTTP error: {e}")))?;

    // ✅ capture antes de consumir com `text()`
    let status = res.status();
    // (opcional) se quiser headers para log:
    // let headers = res.headers().clone();

    // `text()` move `res`, então faça por último:
    let body_text = res.text().unwrap_or_else(|_| "<sem corpo>".to_string());

    println!("🌐 Status: {}", status);
    // println!("🧾 Headers: {:?}", headers);
    println!("📩 Corpo da resposta: {}", body_text);

    if !status.is_success() {
        return Err(Error::from_reason(format!(
            "Failed with status: {status}, body: {body_text}"
        )));
    }

    Ok("ok".to_string())
}
