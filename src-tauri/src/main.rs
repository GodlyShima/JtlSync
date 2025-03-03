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

use chrono::Utc;
use env_logger::Env;

use models::LogEntry;
use commands::{
    cancel_scheduled_sync, get_sync_stats, get_synced_orders, get_system_info, load_config_command, save_config_command, schedule_sync, start_sync_command
};
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