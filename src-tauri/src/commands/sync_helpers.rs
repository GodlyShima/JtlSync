// Helper functions for sync commands adapted for Tauri 2.0

use crate::db::models::VirtueMartOrder;
use crate::models::LogEntry;
use chrono::Utc;
use log::info;
use tauri::AppHandle;

/// Emit a log entry via AppHandle
pub fn emit_log(
    app_handle: &AppHandle,
    message: String,
    level: &str,
    category: &str,
    shop_id: Option<String>,
) {
    let _ = app_handle.emit_all(
        "log",
        LogEntry {
            timestamp: Utc::now(),
            message,
            level: level.to_string(),
            category: category.to_string(),
            shop_id,
        },
    );
}

/// Emit an event with payload to all windows
pub fn emit_event<T: serde::Serialize + Clone>(
    app_handle: &AppHandle,
    event: &str,
    payload: T,
) -> Result<(), String> {
    app_handle
        .emit_all(event, payload)
        .map_err(|e| format!("Failed to emit event: {}", e))
}

/// Emit a synced order
pub fn emit_synced_order(
    app_handle: &AppHandle,
    shop_id: &str,
    orders: &[VirtueMartOrder],
) -> Result<(), String> {
    emit_event(app_handle, "synced-orders", (shop_id.to_string(), orders.clone()))
}