#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::Utc;
use tauri::Emitter;

// Import modules
use jtl_sync::{
    // Commands
    commands::{
        // Config commands
        load_config_command, save_config_command, add_shop_command, 
        update_shop_command, remove_shop_command, set_current_shop_command,
        
        // Sync commands
        start_sync_command, abort_sync_command, get_sync_stats, 
        get_synced_orders, start_multi_sync_command, set_sync_hours,
        schedule_sync, cancel_scheduled_sync, start_scheduled_sync,
        
        // System commands
        get_system_info,
    },
    
    // Notifications
    notifications::{setup_notification_handler, show_notification_command},
    
    // Models
    models::LogEntry,
    
    // Initialization
    init,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the application
    init()?;
    
    println!("JTL-VirtueMart Sync starting...");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Config commands
            load_config_command,
            save_config_command,
            add_shop_command,
            update_shop_command,
            remove_shop_command,
            set_current_shop_command,
            
            // Sync commands
            start_sync_command,
            start_multi_sync_command,
            get_sync_stats,
            set_sync_hours,
            schedule_sync,
            cancel_scheduled_sync,
            abort_sync_command,
            start_scheduled_sync,
            get_synced_orders,
            
            // System commands
            get_system_info,
            
            // Notification commands
            show_notification_command,
        ])
        .setup(|app| {
            // Set up the notification handler
            setup_notification_handler(app)?;
            
            // Get app handle for logging
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
    .run(tauri::generate_context!())?;
    
    Ok(())
}