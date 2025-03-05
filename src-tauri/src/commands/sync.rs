use chrono::Utc;
use log::info;
use tauri::{AppHandle, Runtime};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::models::LogEntry;
use crate::config::load_config;
use crate::sync::{SyncEngine, SyncStats, get_shop_stats, update_shop_sync_hours, get_current_stats};
use crate::db::models::VirtueMartOrder;
use crate::error::{Result, Error};
use crate::utils::abort::{reset_abort_flag, set_abort_flag, should_abort};

// Store synced orders in memory
lazy_static! {
    static ref SYNCED_ORDERS: Mutex<HashMap<String, Vec<VirtueMartOrder>>> = Mutex::new(HashMap::new());
}

/// Command to abort the current synchronization
#[tauri::command]
pub async fn abort_sync_command<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    info!("Aborting synchronization...");
    
    // Set abort flag
    set_abort_flag();
    
    // Log the abort
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Synchronization aborted by user".to_string(),
        level: "warn".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    Ok(())
}

/// Start scheduled synchronization
#[tauri::command]
pub async fn start_scheduled_sync<R: Runtime>(
    app_handle: AppHandle<R>,
    shop_ids: Vec<String>,
    job_id: String
) -> Result<(), String> {
    // Load config
    let config = load_config()?;
    
    if shop_ids.is_empty() {
        return Err("No shops selected for synchronization".to_string());
    }
    
    // Log start of scheduled sync
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting scheduled synchronization for {} shops, job {}", shop_ids.len(), job_id),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    // Reset abort flag before starting
    reset_abort_flag();
    
    // Start background task for synchronization
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let shop_ids_clone = shop_ids.clone();
    
    tauri::async_runtime::spawn(async move {
        // Create sync engine
        let api_key = "4fef6933-ae20-4cbc-bd97-a5cd584f244e"; // Should come from config
        let mut engine = SyncEngine::new(api_key);
        
        match engine.sync_multiple_shops(&app_handle_clone, &config_clone, shop_ids_clone).await {
            Ok(_) => {
                // Send events
                let _ = app_handle_clone.emit("multi-sync-complete", job_id.clone());
                let _ = app_handle_clone.emit("scheduled-sync-completed", (job_id.clone(), shop_ids));
                
                // Log success
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Scheduled synchronization completed for job {}", job_id),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            },
            Err(e) => {
                // Log error
                let error_message = e.to_string();
                let _ = app_handle_clone.emit("sync-error", error_message.clone());
                let _ = app_handle_clone.emit("scheduled-sync-error", (job_id.clone(), error_message.clone()));
                
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Scheduled synchronization failed for job {}: {}", job_id, error_message),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            }
        }
    });
    
    Ok(())
}

/// Store synced orders for a specific shop
pub fn store_synced_orders(shop_id: &str, orders: Vec<VirtueMartOrder>) {
    let mut stored_orders = SYNCED_ORDERS.lock().unwrap();
    
    // Add shop_id to each order
    let orders_with_shop_id = orders.into_iter()
        .map(|mut order| {
            order.shop_id = Some(shop_id.to_string());
            order
        })
        .collect();
    
    stored_orders.insert(shop_id.to_string(), orders_with_shop_id);
}

/// Add a synced order
pub fn add_synced_order<R: Runtime>(app_handle: &AppHandle<R>, shop_id: &str, order: VirtueMartOrder) {
    let mut stored_orders = SYNCED_ORDERS.lock().unwrap();
    
    // Ensure there's an entry for this shop
    if !stored_orders.contains_key(shop_id) {
        stored_orders.insert(shop_id.to_string(), Vec::new());
    }
    
    // Add shop_id to the order
    let mut order_with_shop = order.clone();
    order_with_shop.shop_id = Some(shop_id.to_string());
    
    // Add the order to the shop's list
    if let Some(orders) = stored_orders.get_mut(shop_id) {
        orders.push(order_with_shop.clone());
        
        // Add debug log
        info!("Order added to SYNCED_ORDERS for shop {}. Current count: {}", shop_id, orders.len());
        
        // Send data to frontend
        app_handle.emit("synced-orders", (shop_id.to_string(), orders.clone()))
            .map_err(|e| format!("Failed to emit synced orders: {}", e)).ok();
    }
}

#[tauri::command]
pub async fn get_synced_orders<R: Runtime>(
    app_handle: AppHandle<R>,
    shop_id: Option<String>
) -> Result<Vec<VirtueMartOrder>, String> {
    info!("Getting synced orders for shop: {:?}", shop_id);
    
    let stored_orders = SYNCED_ORDERS.lock().unwrap();
    
    // If shop_id is provided, return orders for that shop only
    if let Some(id) = shop_id {
        let orders = stored_orders.get(&id).cloned().unwrap_or_default();
        
        // Emit the orders to the frontend
        app_handle.emit("synced-orders", (id.clone(), orders.clone()))
            .map_err(|e| format!("Failed to emit synced orders: {}", e))?;
        
        Ok(orders)
    } else {
        // If no shop_id, return all orders from all shops
        let all_orders: Vec<VirtueMartOrder> = stored_orders.values()
            .flat_map(|orders| orders.clone())
            .collect();
        
        // Emit all orders to the frontend
        app_handle.emit("synced-orders-all", all_orders.clone())
            .map_err(|e| format!("Failed to emit all synced orders: {}", e))?;
        
        Ok(all_orders)
    }
}

/// Start manual synchronization of multiple shops
#[tauri::command]
pub async fn start_multi_sync_command<R: Runtime>(
    app_handle: AppHandle<R>, 
    shop_ids: Vec<String>
) -> Result<(), String> {
    if shop_ids.is_empty() {
        return Err("No shops selected for synchronization".to_string());
    }
    
    // Load the configuration
    let config = load_config()?;
    
    // Log start of synchronization
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting manual synchronization for {} shops...", shop_ids.len()),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    // Reset abort flag
    reset_abort_flag();
    
    // Create background task
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let shop_ids_clone = shop_ids.clone();
    
    // Start background task
    tauri::async_runtime::spawn(async move {
        // Create sync engine
        let api_key = "4fef6933-ae20-4cbc-bd97-a5cd584f244e"; // Should come from config
        let mut engine = SyncEngine::new(api_key);
        
        match engine.sync_multiple_shops(&app_handle_clone, &config_clone, shop_ids_clone).await {
            Ok(_) => {
                // Send success event
                let _ = app_handle_clone.emit("multi-sync-complete", ());
                
                // Log success
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: "Multi-shop synchronization completed successfully".to_string(),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            },
            Err(e) => {
                // Send error event
                let error_message = e.to_string();
                let _ = app_handle_clone.emit("sync-error", error_message.clone());
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Multi-shop synchronization failed: {}", error_message),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            }
        }
    });
    
    Ok(())
}

/// Start manual synchronization of a single shop
#[tauri::command]
pub async fn start_sync_command<R: Runtime>(
    app_handle: AppHandle<R>, 
    shop_id: Option<String>,
    hours: Option<i32>
) -> Result<(), String> {
    // Load the configuration
    let config = load_config()?;
    
    // Determine which shop to sync
    let shop = if let Some(id) = shop_id.clone() {
        // Find the specific shop
        config.shops.iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Shop with ID '{}' not found", id))?
            .clone()
    } else {
        // Use the current shop
        config.get_current_shop()
    };
    
    // Get the sync hours (default to 24 if not provided)
    let sync_hours = hours.unwrap_or_else(|| get_shop_stats(&shop.id).sync_hours);
    
    // If hours was provided, update the shop's sync_hours
    if let Some(h) = hours {
        update_shop_sync_hours(&shop.id, h)?;
    }
    
    // Log start of synchronization
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting manual synchronization for shop '{}' with {}h timeframe...", shop.name, sync_hours),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });
    
    // Reset abort flag
    reset_abort_flag();
    
    // Create background task
    let app_handle_clone = app_handle.clone();
    let shop_clone = shop.clone();
    
    // Start background task
    tauri::async_runtime::spawn(async move {
        // Create sync engine
        let api_key = "4fef6933-ae20-4cbc-bd97-a5cd584f244e"; // Should come from config
        let mut engine = SyncEngine::new(api_key);
        
        match engine.sync_shop(&app_handle_clone, &shop_clone, sync_hours).await {
            Ok(stats) => {
                // Send success event
                let _ = app_handle_clone.emit("sync-complete", stats.clone());
                
                // Log success
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization completed for shop '{}': {} synced, {} skipped, {} errors", 
                                   shop_clone.name, stats.synced_orders, stats.skipped_orders, stats.error_orders),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                    shop_id: Some(shop_clone.id),
                });
            },
            Err(e) => {
                // Send error event
                let error_message = e.to_string();
                let _ = app_handle_clone.emit("sync-error", (error_message.clone(), shop_clone.id.clone()));
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization failed for shop '{}': {}", shop_clone.name, error_message),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: Some(shop_clone.id),
                });
            }
        }
    });
    
    // Return immediately (actual stats will be updated via events)
    Ok(())
}

/// Set synchronization timeframe for a shop
#[tauri::command]
pub async fn set_sync_hours<R: Runtime>(
    app_handle: AppHandle<R>,
    shop_id: String,
    hours: i32
) -> Result<SyncStats, String> {
    // Validate the hours parameter
    if hours <= 0 {
        return Err("Sync timeframe must be greater than zero hours".to_string());
    }
    
    // Update the shop's sync hours
    update_shop_sync_hours(&shop_id, hours)?;
    
    // Get updated stats
    let stats = get_shop_stats(&shop_id);
    
    // Log the change
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Sync timeframe for shop '{}' updated to {} hours", shop_id, hours),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop_id.clone()),
    });
    
    Ok(stats)
}

/// Get current synchronization statistics
#[tauri::command]
pub async fn get_sync_stats(shop_id: Option<String>) -> Result<SyncStats, String> {
    if let Some(id) = shop_id {
        Ok(get_shop_stats(&id))
    } else {
        Ok(get_current_stats())
    }
}

/// Schedule synchronization
#[tauri::command]
pub async fn schedule_sync(shop_ids: Vec<String>, cron_expression: String) -> Result<(), String> {
    // In a real implementation, set up a cron job or timer
    // For now, just log it
    info!("Scheduled sync for {} shops with cron: {}", shop_ids.len(), cron_expression);
    Ok(())
}

/// Cancel scheduled synchronization jobs
#[tauri::command]
pub async fn cancel_scheduled_sync(shop_id: Option<String>) -> Result<(), String> {
    // In a real implementation, cancel scheduled jobs
    if let Some(id) = shop_id {
        info!("Canceled scheduled sync jobs for shop {}", id);
    } else {
        info!("Canceled all scheduled sync jobs");
    }
    Ok(())
}