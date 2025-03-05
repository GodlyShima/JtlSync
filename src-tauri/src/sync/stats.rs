use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::{Result, Error};

/// Sync statistics structure for dashboard
#[derive(Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub shop_id: String,
    pub total_orders: i32,
    pub synced_orders: i32,
    pub skipped_orders: i32,
    pub error_orders: i32,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub next_scheduled_run: Option<DateTime<Utc>>,
    pub aborted: bool,
    pub sync_hours: i32,
}

impl Default for SyncStats {
    fn default() -> Self {
        SyncStats {
            shop_id: String::new(),
            total_orders: 0,
            synced_orders: 0,
            skipped_orders: 0,
            error_orders: 0,
            last_sync_time: None,
            next_scheduled_run: None,
            aborted: false,
            sync_hours: 24, // Default to 24 hours
        }
    }
}

lazy_static! {
    // Map of shop_id -> SyncStats to track each shop's sync stats separately
    static ref SYNC_STATS: Mutex<HashMap<String, SyncStats>> = Mutex::new(HashMap::new());
    
    // Default stats for unknown shops
    static ref DEFAULT_STATS: SyncStats = SyncStats {
        shop_id: String::new(),
        total_orders: 0,
        synced_orders: 0,
        skipped_orders: 0,
        error_orders: 0,
        last_sync_time: None,
        next_scheduled_run: None,
        aborted: false,
        sync_hours: 24, // Default to 24 hours
    };
}

/// Update sync stats for a specific shop
pub fn update_sync_stats(stats: SyncStats) {
    let mut current_stats = SYNC_STATS.lock().unwrap();
    current_stats.insert(stats.shop_id.clone(), stats);
}

/// Get sync stats for a specific shop
pub fn get_shop_stats(shop_id: &str) -> SyncStats {
    let stats = SYNC_STATS.lock().unwrap();
    match stats.get(shop_id) {
        Some(shop_stats) => shop_stats.clone(),
        None => {
            // Return default stats with shop_id
            let mut default = DEFAULT_STATS.clone();
            default.shop_id = shop_id.to_string();
            default
        }
    }
}

/// Get stats for the "current" shop - used for backward compatibility
pub fn get_current_stats() -> SyncStats {
    let stats = SYNC_STATS.lock().unwrap();
    
    // If we have any stats, return the first one
    if let Some((_, first_stats)) = stats.iter().next() {
        return first_stats.clone();
    }
    
    // Otherwise return default stats
    DEFAULT_STATS.clone()
}

/// Update sync time range for a shop
pub fn update_shop_sync_hours(shop_id: &str, hours: i32) -> Result<()> {
    if hours <= 0 {
        return Err(Error::ValidationError("Sync timeframe must be greater than zero hours".to_string()));
    }
    
    let mut stats = SYNC_STATS.lock().unwrap();
    
    // If stats for this shop already exist, update them
    if let Some(shop_stats) = stats.get_mut(shop_id) {
        shop_stats.sync_hours = hours;
        return Ok(());
    }
    
    // Create new stats for this shop
    let mut new_stats = DEFAULT_STATS.clone();
    new_stats.shop_id = shop_id.to_string();
    new_stats.sync_hours = hours;
    stats.insert(shop_id.to_string(), new_stats);
    
    Ok(())
}

/// Reset stats for a specific shop
pub fn reset_shop_stats(shop_id: &str) {
    let mut stats = SYNC_STATS.lock().unwrap();
    
    if let Some(shop_stats) = stats.get_mut(shop_id) {
        shop_stats.total_orders = 0;
        shop_stats.synced_orders = 0;
        shop_stats.skipped_orders = 0;
        shop_stats.error_orders = 0;
        shop_stats.aborted = false;
    }
}

/// Reset stats for all shops
pub fn reset_all_stats() {
    let mut stats = SYNC_STATS.lock().unwrap();
    stats.clear();
}