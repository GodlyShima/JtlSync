use chrono::Utc;
use tauri::{AppHandle, Runtime, Emitter};

use crate::config::{load_config, save_config, add_shop, update_shop, remove_shop, set_current_shop};
use crate::config::app::AppConfig;
use crate::config::shop::ShopConfig;
use crate::models::LogEntry;
use crate::error::{Result, Error};
use tauri::ipc::InvokeError;
use anyhow::Context;

/// Save configuration
#[tauri::command]
pub fn save_config_command<R: Runtime>(app_handle: AppHandle<R>, config: AppConfig) -> Result<()> {
    save_config(&config)?;
    
    // Send log event
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Configuration saved successfully".to_string(),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: None,
    });
    
    Ok(())
}

/// Load configuration
#[tauri::command]
pub fn load_config_command<R: Runtime>(_app_handle: AppHandle<R>) -> Result<AppConfig> {
    let config = load_config()?;
    
    // Optional: Send log event
    // let _ = app_handle.emit("log", LogEntry { ... });
    
    Ok(config)
}

/// Add shop
#[tauri::command]
pub fn add_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop: ShopConfig) -> Result<AppConfig> {
    let mut config = load_config()?;
    
    add_shop(&mut config, shop.clone())?;
    
    // Send log event
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("New shop '{}' added successfully", shop.name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop.id),
    });
    
    Ok(config)
}

/// Update shop
#[tauri::command]
pub fn update_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop: ShopConfig) -> Result<AppConfig> {
    let mut config = load_config()?;
    
    update_shop(&mut config, shop.clone())?;
    
    // Send log event
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Shop '{}' updated successfully", shop.name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop.id),
    });
    
    Ok(config)
}

/// Remove shop
#[tauri::command]
pub fn remove_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop_id: String) -> Result<AppConfig> {
    let mut config = load_config()?;
    
    // Find shop name for logging before removing
    let shop_name = config.shops.iter()
        .find(|s| s.id == shop_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    remove_shop(&mut config, &shop_id)?;
    
    // Send log event
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Shop '{}' removed successfully", shop_name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: None,
    });
    
    Ok(config)
}

/// Set current shop
#[tauri::command]
pub fn set_current_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop_id: String) -> Result<AppConfig> {
    let mut config = load_config()?;
    
    set_current_shop(&mut config, &shop_id)?;
    
    // Find the shop name
    let shop_name = config.shops.iter()
        .find(|s| s.id == shop_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Send log event
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Active shop changed to '{}'", shop_name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop_id),
    });
    
    Ok(config)
}