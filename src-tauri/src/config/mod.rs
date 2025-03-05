pub mod app;
pub mod shop;

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, Error};
use crate::config::app::AppConfig;
use crate::config::shop::ShopConfig;

/// Determine configuration path
pub fn get_config_path() -> PathBuf {
    let mut app_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
    app_dir.push("config");
    app_dir.push("config.json");
    app_dir
}

/// Save configuration
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
        }
    }
    
    let config_str = serde_json::to_string_pretty(config)
        .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(&config_path, config_str)
        .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
}

/// Load configuration
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        // If config doesn't exist, create default
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
    
    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
    
    Ok(config)
}

/// Add a new shop to the configuration
pub fn add_shop(config: &mut AppConfig, shop: ShopConfig) -> Result<()> {
    // Check for duplicate IDs
    if config.shops.iter().any(|s| s.id == shop.id) {
        return Err(Error::ValidationError(format!("A shop with ID '{}' already exists", shop.id)));
    }
    
    config.shops.push(shop);
    save_config(config)?;
    
    Ok(())
}

/// Update an existing shop
pub fn update_shop(config: &mut AppConfig, shop: ShopConfig) -> Result<()> {
    let shop_index = config.shops.iter().position(|s| s.id == shop.id)
        .ok_or_else(|| Error::NotFound(format!("No shop found with ID '{}'", shop.id)))?;
    
    config.shops[shop_index] = shop;
    save_config(config)?;
    
    Ok(())
}

/// Remove a shop from the configuration
pub fn remove_shop(config: &mut AppConfig, shop_id: &str) -> Result<()> {
    let initial_len = config.shops.len();
    
    // Don't allow removing the last shop
    if initial_len <= 1 {
        return Err(Error::ValidationError("Cannot remove the last shop".to_string()));
    }
    
    config.shops.retain(|s| s.id != shop_id);
    
    // If we actually removed something
    if config.shops.len() < initial_len {
        // Make sure current_shop_index is valid
        if config.current_shop_index >= config.shops.len() {
            config.current_shop_index = 0;
        }
        
        save_config(config)?;
        Ok(())
    } else {
        Err(Error::NotFound(format!("No shop found with ID '{}'", shop_id)))
    }
}

/// Set the current active shop
pub fn set_current_shop(config: &mut AppConfig, shop_id: &str) -> Result<()> {
    let shop_index = config.shops.iter().position(|s| s.id == shop_id)
        .ok_or_else(|| Error::NotFound(format!("No shop found with ID '{}'", shop_id)))?;
    
    config.current_shop_index = shop_index;
    save_config(config)?;
    
    Ok(())
}