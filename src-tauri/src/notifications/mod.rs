mod platform;

use log::{info, error};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::error::{Result, Error};

pub use platform::show_notification;

#[derive(Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
}

/// Tauri command to show a notification
#[tauri::command]
pub fn show_notification_command(notification: NotificationPayload) -> Result<(), String> {
    info!("Notification command received: {} - {}", notification.title, notification.body);
    show_notification(&notification.title, &notification.body)
        .map_err(|e| e.to_string())
}

/// Setup notification handler for the app
pub fn setup_notification_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Get the app handle and use that instead of the mutable app reference
    let app_handle = app.handle();
    
    // Now use app_handle for anything needed
    if let Some(window) = app_handle.get_webview_window("main") {
        info!("Notification system initialized for window: {}", window.label());
    } else {
        info!("Notification system initialized (no main window found)");
    }
    
    Ok(())
}