#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::Utc;
use tauri::{Emitter, Manager};
use std::error::Error;


use jtlsync_lib::{
    
    // Notifications
    notifications::{setup_notification_handler, show_notification_command},
    
    // Models
    models::LogEntry,
    
    // Initialization
    init,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the application
    init()?;
    
    println!("JTL-VirtueMart Sync starting...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            jtlsync_lib::commands::config::load_config_command,
            jtlsync_lib::commands::config::save_config_command,
            jtlsync_lib::commands::config::add_shop_command,
            jtlsync_lib::commands::config::update_shop_command,
            jtlsync_lib::commands::config::remove_shop_command,
            jtlsync_lib::commands::config::set_current_shop_command,

            jtlsync_lib::commands::sync::start_sync_command,
            jtlsync_lib::commands::sync::start_multi_sync_command,
            jtlsync_lib::commands::sync::get_sync_stats,
            jtlsync_lib::commands::sync::set_sync_hours,
            jtlsync_lib::commands::sync::schedule_sync,
            jtlsync_lib::commands::sync::cancel_scheduled_sync,
            jtlsync_lib::commands::sync::abort_sync_command,
            jtlsync_lib::commands::sync::start_scheduled_sync,
            jtlsync_lib::commands::sync::get_synced_orders,

            jtlsync_lib::commands::system::get_system_info,
            
        ])
        .setup(|app| {
            // Set up the notification handler
            setup_notification_handler(app)?;
            
            // Get app handle for logging
            let app_handle = app.app_handle();
            
            // Log application start
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: "Application started".to_string(),
                level: "info".to_string(),
                category: "system".to_string(),
                shop_id: None,
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())?;
    
    Ok(())
}