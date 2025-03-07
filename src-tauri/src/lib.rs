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


// Export notification command
pub use notifications::show_notification_command;

// Other exports
pub use config::{load_config, save_config};
pub use notifications::setup_notification_handler;
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