use serde::{Serialize, Deserialize};

use crate::config::shop::ShopConfig;
use crate::error::{Result, Error};
use crate::db::models::{DatabaseConfig, TablesConfig};

/// Application configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub shops: Vec<ShopConfig>,
    pub current_shop_index: usize,
    pub logFile: String,
    pub jtlApiPath: String, // For backward compatibility
}

impl AppConfig {
    /// Create a new default configuration
    pub fn default() -> Self {
        // Create a default shop
        let default_shop = ShopConfig {
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
        
        AppConfig {
            shops: vec![default_shop],
            current_shop_index: 0,
            logFile: "sync_log.txt".to_string(),
            jtlApiPath: "C:\\Program Files (x86)\\JTL-Software\\JTL.Wawi.Rest.exe".to_string(),
        }
    }
    
    /// Get the current shop configuration
    pub fn get_current_shop(&self) -> ShopConfig {
        if self.shops.is_empty() {
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
        
        let index = if self.current_shop_index < self.shops.len() {
            self.current_shop_index
        } else {
            0
        };
        
        self.shops[index].clone()
    }
    
    /// Get API key from configuration
    pub fn get_api_key(&self) -> String {
        // This would ideally come from secure storage or environment variables
        // For now, return a hardcoded key for compatibility
        "4fef6933-ae20-4cbc-bd97-a5cd584f244e".to_string()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.shops.is_empty() {
            return Err(Error::ValidationError("No shops configured".to_string()));
        }
        
        if self.current_shop_index >= self.shops.len() {
            return Err(Error::ValidationError("Invalid current shop index".to_string()));
        }
        
        // Validate each shop
        for shop in &self.shops {
            shop.validate()?;
        }
        
        Ok(())
    }
}