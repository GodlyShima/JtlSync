use std::fs;
use std::path::PathBuf;
use crate::models::{AppConfig, ShopConfig, DatabaseConfig, TablesConfig};

/// Konfigurationspfad bestimmen
pub fn get_config_path() -> PathBuf {
    let mut app_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
    app_dir.push("config");
    app_dir.push("config.json");
    app_dir
}

/// Konfiguration speichern
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    
    // Verzeichnis erstellen, falls es nicht existiert
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }
    
    let config_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

/// Konfiguration laden
pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        // If config doesn't exist, create default
        let default_config = create_default_config();
        save_config(&default_config)?;
        return Ok(default_config);
    }
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    Ok(config)
}

/// Default-Konfiguration erstellen (für Neuinstallationen)
pub fn create_default_config() -> AppConfig {
    // Create a default shop
    let default_shop = ShopConfig {
        id: "shop1".to_string(),
        name: "Default Shop".to_string(),
        joomla: DatabaseConfig {
            host: "w01539f0.kasserver.com".to_string(),
            user: "d0243b57".to_string(),
            password: "mallorca".to_string(),
            database: "d0243b57".to_string(),
        },
        jtl: DatabaseConfig {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
            database: "jtl".to_string(),
        },
        tables: TablesConfig {
            orders: "y13ci_virtuemart_orders".to_string(),
            orderItems: "y13ci_virtuemart_order_items".to_string(),
            customers: "y13ci_virtuemart_order_userinfos".to_string(),
        },
    };
    
    AppConfig {
        shops: vec![default_shop],
        current_shop_index: 0,
        logFile: "sync_log.txt".to_string(),
        jtlApiPath: "C:\\Program Files (x86)\\JTL-Software\\JTL.Wawi.Rest.exe".to_string(),
    }
}

/// Get the current shop configuration
pub fn get_current_shop(config: &AppConfig) -> ShopConfig {
    if config.shops.is_empty() {
        // Return a default shop if none exists
        return ShopConfig {
            id: "shop1".to_string(),
            name: "Default Shop".to_string(),
            joomla: DatabaseConfig {
                host: "localhost".to_string(),
                user: "root".to_string(),
                password: "".to_string(),
                database: "joomla".to_string(),
            },
            jtl: DatabaseConfig {
                host: "localhost".to_string(),
                user: "root".to_string(),
                password: "".to_string(),
                database: "jtl".to_string(),
            },
            tables: TablesConfig {
                orders: "jos_virtuemart_orders".to_string(),
                orderItems: "jos_virtuemart_order_items".to_string(),
                customers: "jos_virtuemart_order_userinfos".to_string(),
            },
        };
    }
    
    let index = if config.current_shop_index < config.shops.len() {
        config.current_shop_index
    } else {
        0
    };
    
    config.shops[index].clone()
}

/// Add a new shop to the configuration
pub fn add_shop(config: &mut AppConfig, shop: ShopConfig) -> Result<(), String> {
    // Check for duplicate IDs
    if config.shops.iter().any(|s| s.id == shop.id) {
        return Err(format!("A shop with ID '{}' already exists", shop.id));
    }
    
    config.shops.push(shop);
    save_config(config)?;
    
    Ok(())
}

/// Update an existing shop
pub fn update_shop(config: &mut AppConfig, shop: ShopConfig) -> Result<(), String> {
    let shop_index = config.shops.iter().position(|s| s.id == shop.id)
        .ok_or_else(|| format!("No shop found with ID '{}'", shop.id))?;
    
    config.shops[shop_index] = shop;
    save_config(config)?;
    
    Ok(())
}

/// Remove a shop from the configuration
pub fn remove_shop(config: &mut AppConfig, shop_id: &str) -> Result<(), String> {
    let initial_len = config.shops.len();
    
    // Don't allow removing the last shop
    if initial_len <= 1 {
        return Err("Cannot remove the last shop".to_string());
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
        Err(format!("No shop found with ID '{}'", shop_id))
    }
}

/// Set the current active shop
pub fn set_current_shop(config: &mut AppConfig, shop_id: &str) -> Result<(), String> {
    let shop_index = config.shops.iter().position(|s| s.id == shop_id)
        .ok_or_else(|| format!("No shop found with ID '{}'", shop_id))?;
    
    config.current_shop_index = shop_index;
    save_config(config)?;
    
    Ok(())
}