// Modules
pub mod config;
pub mod sync;
pub mod system;

// Re-export all commands for easy registration
pub use config::{
    load_config_command, 
    save_config_command, 
    add_shop_command, 
    update_shop_command, 
    remove_shop_command, 
    set_current_shop_command
};
pub use sync::{
    start_sync_command, 
    abort_sync_command, 
    get_sync_stats, 
    start_multi_sync_command, 
    set_sync_hours, 
    schedule_sync, 
    cancel_scheduled_sync, 
    start_scheduled_sync, 
    get_synced_orders
};
pub use system::get_system_info;

// Optional: Make commands publicly accessible
pub mod prelude {
    pub use super::config::*;
    pub use super::sync::*;
    pub use super::system::*;
}