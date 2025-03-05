use std::time::SystemTime;

/// Get system information
#[tauri::command]
pub fn get_system_info() -> serde_json::Value {
    serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "memory": "N/A", // Would require additional crate
        "uptime": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
    })
}