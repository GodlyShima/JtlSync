#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Modules
pub mod api;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod notifications;
pub mod sync;
pub mod utils;

// Explicitly export specific commands
pub use commands::{
    // Config commands
    config::{
        load_config_command,
        save_config_command,
        add_shop_command,
        update_shop_command,
        remove_shop_command,
        set_current_shop_command,
    },
    
    // Sync commands
    sync::{
        start_sync_command,
        abort_sync_command,
        get_sync_stats,
        get_synced_orders,
        start_multi_sync_command,
        set_sync_hours,
        schedule_sync,
        cancel_scheduled_sync,
        start_scheduled_sync,
    },
    
    // System commands
    system::get_system_info,
};

// Export notification commands
pub use notifications::{
    setup_notification_handler,
    show_notification_command,
};

// Reuse other existing exports
pub use config::{load_config, save_config};
pub use notifications::show_notification;
pub use sync::SyncEngine;

/// Initialize the application
pub fn init() -> error::Result<()> {
    // Make sure we only initialize the logger once
    static LOGGER_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    
    if !LOGGER_INITIALIZED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        // Set up logging
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_secs()
            .init();
        
        log::info!("Logger initialized");
    }
    
    // Initialize other components as needed
    Ok(())
}