use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub id: Option<u64>,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub result: Value,
    #[serde(default)]
    pub error: Option<Value>,
}

/// Parse foreground and background normalized float colors from `session.inspect` JSON response into hex strings (`#rrggbb`).
/// Falls back to `("#000000", "#ffffff")` if keys are missing or malformed.
pub fn parse_session_colors(res: &serde_json::Value) -> (String, String) {
    let parse_color = |key: &str| -> Option<String> {
        let arr = res.get(key)?.as_array()?;
        if arr.len() >= 3 {
            let r = (arr[0].as_f64()? * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (arr[1].as_f64()? * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (arr[2].as_f64()? * 255.0).round().clamp(0.0, 255.0) as u8;
            Some(format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            None
        }
    };

    let fg = parse_color("foreground").unwrap_or_else(|| "#000000".to_string());
    let bg = parse_color("background").unwrap_or_else(|| "#ffffff".to_string());

    (fg, bg)
}
