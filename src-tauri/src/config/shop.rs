use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::db::models::{DatabaseConfig, TablesConfig};
use crate::error::{Result, Error};

/// Shop configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct ShopConfig {
    pub id: String,
    pub name: String,
    pub joomla: DatabaseConfig,
    pub jtl: DatabaseConfig,
    pub tables: TablesConfig,
}

impl ShopConfig {
    /// Create a new shop configuration with default values
    pub fn new(name: &str) -> Self {
        ShopConfig {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
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
        }
    }
    
    /// Validate shop configuration
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::ValidationError("Shop ID cannot be empty".to_string()));
        }
        
        if self.name.is_empty() {
            return Err(Error::ValidationError("Shop name cannot be empty".to_string()));
        }
        
        // Validate database configurations
        if self.joomla.host.is_empty() {
            return Err(Error::ValidationError("Joomla database host cannot be empty".to_string()));
        }
        
        if self.joomla.user.is_empty() {
            return Err(Error::ValidationError("Joomla database user cannot be empty".to_string()));
        }
        
        if self.joomla.database.is_empty() {
            return Err(Error::ValidationError("Joomla database name cannot be empty".to_string()));
        }
        
        // Validate table names
        if self.tables.orders.is_empty() {
            return Err(Error::ValidationError("Orders table name cannot be empty".to_string()));
        }
        
        if self.tables.orderItems.is_empty() {
            return Err(Error::ValidationError("Order items table name cannot be empty".to_string()));
        }
        
        if self.tables.customers.is_empty() {
            return Err(Error::ValidationError("Customers table name cannot be empty".to_string()));
        }
        
        Ok(())
    }
}