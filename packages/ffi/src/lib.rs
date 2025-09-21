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
    let data: EventoInput = serde_json::from_str(&input)
        .map_err(|e| Error::from_reason(format!("Invalid input: {}", e)))?;

    // recive_data(data);
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://localhost:6666/ingest")
        .json(&data)
        .send()
        .map_err(|e| Error::from_reason(format!("HTTP error: {}", e)))?;

    if !res.status().is_success() {
        return Err(Error::from_reason(format!(
            "Failed with status: {}",
            res.status()
        )));
    }

    Ok("ok".to_string())
}
