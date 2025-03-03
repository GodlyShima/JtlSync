pub mod models;
pub mod config;
pub mod database;
pub mod api;
pub mod sync;
pub mod utils;
pub mod commands;

use chrono::Utc;
use commands::get_synced_orders;
use env_logger::Env;
use tauri::Emitter;
use tauri::Manager;

use models::LogEntry;
use commands::{
    get_system_info,
    save_config_command,
    load_config_command,
    start_sync_command,
    get_sync_stats,
    schedule_sync,
    cancel_scheduled_sync,
};



// Einfacher Greeting-Befehl für Tests
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Logger für Konsolenausgabe einrichten
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            get_synced_orders,
            save_config_command,
            load_config_command,
            start_sync_command,
            get_sync_stats,
            schedule_sync,
            cancel_scheduled_sync,
        ])
        .setup(|app| {
            // System initialisieren
            let app_handle = app.handle();
            
            // Anwendungsstart loggen
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: "Application started".to_string(),
                level: "info".to_string(),
                category: "system".to_string(),
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}