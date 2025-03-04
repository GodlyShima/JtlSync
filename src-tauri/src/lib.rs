#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod models;
mod config;
mod database;
mod api;
mod sync;
mod utils;
mod commands;
mod notifications;

use chrono::Utc;
use env_logger::Env;

use models::LogEntry;
use commands::{
    abort_sync_command, cancel_scheduled_sync, get_sync_stats, get_synced_orders, 
    get_system_info, load_config_command, save_config_command, schedule_sync, 
    start_sync_command, start_scheduled_sync, add_shop_command, update_shop_command,
    remove_shop_command, set_current_shop_command
};
use notifications::{setup_notification_handler, show_notification_command};
use tauri::Emitter;

fn main() {
    // Logger für Konsolenausgabe einrichten
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            get_synced_orders,
            save_config_command,
            load_config_command,
            start_sync_command,
            get_sync_stats,
            schedule_sync,
            cancel_scheduled_sync,
            abort_sync_command,
            start_scheduled_sync,
            show_notification_command,
            // New multi-shop commands
            add_shop_command,
            update_shop_command,
            remove_shop_command,
            set_current_shop_command,
        ])
        .setup(|app| {
            // First, set up the notification handler with the app
            // This consumes the mutable borrow of app
            setup_notification_handler(app)?;
            
            // Now that we're done with the mutable borrow, we can create an immutable borrow
            let app_handle = app.handle();
            
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}