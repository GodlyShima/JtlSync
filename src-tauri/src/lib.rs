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

// Public exports
pub use commands::*;
pub use config::{load_config, save_config};
pub use notifications::{setup_notification_handler, show_notification_command};
pub use sync::SyncEngine;

/// Initialize the application
pub fn init() -> error::Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    // Initialize other components as needed
    
    Ok(())
}