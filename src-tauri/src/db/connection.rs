use mysql::{OptsBuilder, Pool, Error as MySqlError};
use std::sync::Arc;

use crate::config::shop::ShopConfig;
use crate::error::{Result, Error};

/// Connection pool manager for database connections
pub struct ConnectionManager {
    pools: std::collections::HashMap<String, Arc<Pool>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        ConnectionManager {
            pools: std::collections::HashMap::new(),
        }
    }
    
    /// Get a connection pool for a shop (create if it doesn't exist)
    pub fn get_joomla_pool(&mut self, shop: &ShopConfig) -> Result<Arc<Pool>> {
        // Check if we already have a pool for this shop
        if let Some(pool) = self.pools.get(&shop.id) {
            return Ok(pool.clone());
        }
        
        // Create a new pool
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(&shop.joomla.host))
            .user(Some(&shop.joomla.user))
            .pass(Some(&shop.joomla.password))
            .db_name(Some(&shop.joomla.database));
        
        let pool = Pool::new(opts)
            .map_err(|e| Error::Database(format!("Failed to create connection pool: {}", e)))?;
        
        // Store the pool
        let pool_arc = Arc::new(pool);
        self.pools.insert(shop.id.clone(), pool_arc.clone());
        
        Ok(pool_arc)
    }
    
    /// Test connection to verify credentials
    pub fn test_connection(&mut self, shop: &ShopConfig) -> Result<()> {
        let pool = self.get_joomla_pool(shop)?;
        let mut conn = pool.get_conn()
            .map_err(|e| Error::Database(format!("Connection test failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Clear connection pools
    pub fn clear_pools(&mut self) {
        self.pools.clear();
    }
}

/// Connect to Joomla database - legacy function for compatibility
pub fn connect_to_joomla(shop: &ShopConfig) -> std::result::Result<Pool, MySqlError> {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(&shop.joomla.host))
        .user(Some(&shop.joomla.user))
        .pass(Some(&shop.joomla.password))
        .db_name(Some(&shop.joomla.database));
    
    Pool::new(opts)
}