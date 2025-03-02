use std::fs;
use std::path::PathBuf;
use crate::models::AppConfig;

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
        return Err("Config file does not exist".to_string());
    }
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    Ok(config)
}

/// Default-Konfiguration erstellen (für Neuinstallationen)
pub fn create_default_config() -> AppConfig {
    AppConfig {
        joomla: crate::models::DatabaseConfig {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
            database: "joomla".to_string(),
        },
        jtl: crate::models::DatabaseConfig {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
            database: "jtl".to_string(),
        },
        tables: crate::models::TablesConfig {
            orders: "jos_virtuemart_orders".to_string(),
            orderItems: "jos_virtuemart_order_items".to_string(),
            customers: "jos_virtuemart_order_userinfos".to_string(),
        },
        logFile: "sync_log.txt".to_string(),
        jtlApiPath: "C:\\Program Files (x86)\\JTL-Software\\JTL.Wawi.Rest.exe".to_string(),
    }
}