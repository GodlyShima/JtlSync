use serde::Serialize;
use tauri::{AppHandle, Runtime, Manager, Window};

/// Helper function to emit events to windows
pub fn emit_to_window<R: Runtime, T: Serialize + Clone>(
    window: &Window<R>, 
    event: &str, 
    payload: T
) -> Result<(), String> {
    window
        .emit(event, payload)
        .map_err(|e| format!("Failed to emit event to window: {}", e))
}

/// Helper function to emit events to all windows via app handle
pub fn emit_to_all<R: Runtime, T: Serialize + Clone>(
    app_handle: &AppHandle<R>,
    event: &str, 
    payload: T
) -> Result<(), String> {
    app_handle
        .emit_all(event, payload)
        .map_err(|e| format!("Failed to emit event: {}", e))
}