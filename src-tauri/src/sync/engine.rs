use chrono::Utc;
use log::{info, error, warn};
use std::sync::Arc;
use tokio::time::sleep;
use tokio::time::Duration as TokioDuration;
use tauri::{AppHandle, Runtime, Emitter, Manager};


use crate::api::jtl::JtlApiClient;
use crate::config::app::AppConfig;
use crate::config::shop::ShopConfig;
use crate::db::connection::ConnectionManager;
use crate::db::joomla::{get_orders_within_timeframe, get_order_items, get_shipping_address};
use crate::error::{Result, Error};
use crate::models::LogEntry;
use crate::sync::processor::process_order;
use crate::sync::stats::{SyncStats, update_sync_stats, get_shop_stats};
use crate::utils::abort::{should_abort, reset_abort_flag};

/// Main sync engine
pub struct SyncEngine {
    conn_manager: ConnectionManager,
    api_client: JtlApiClient,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(api_key: &str) -> Self {
        SyncEngine {
            conn_manager: ConnectionManager::new(),
            api_client: JtlApiClient::new(api_key),
        }
    }
    
    /// Synchronize multiple shops sequentially
    pub async fn sync_multiple_shops<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        config: &AppConfig,
        shop_ids: Vec<String>
    ) -> Result<()> {
        info!("Starting sequential synchronization for {} shops", shop_ids.len());

        let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: format!("Starting sequential synchronization for {} shops", shop_ids.len()),
            level: "info".to_string(),
            category: "sync".to_string(),
            shop_id: None,
        });

        // Reset abort flag before starting
        reset_abort_flag();

        // Sync each shop in sequence
        for shop_id in shop_ids {
            // Find the shop config
            let shop = match config.shops.iter().find(|s| s.id == shop_id) {
                Some(s) => s.clone(),
                None => {
                    let error_msg = format!("Shop with ID '{}' not found", shop_id);
                    let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: error_msg.clone(),
                        level: "error".to_string(),
                        category: "sync".to_string(),
                        shop_id: Some(shop_id.clone()),
                    });
                    continue; // Skip this shop and move to the next one
                }
            };
            
            // Get the sync hours for this shop (default to 24 if not set)
            let sync_hours = get_shop_stats(&shop_id).sync_hours;
            
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: format!("Starting synchronization for shop '{}' with {}h timeframe", shop.name, sync_hours),
                level: "info".to_string(),
                category: "sync".to_string(),
                shop_id: Some(shop_id.clone()),
            });
            
            // Perform sync for this shop
            match self.sync_shop(app_handle, &shop, sync_hours).await {
                Ok(stats) => {
                    update_sync_stats(stats.clone());
                    
                    // Send events for completion
                    let _ = app_handle.emit("sync-complete", stats.clone());
                    
                    let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: format!("Synchronization completed for shop '{}': {} synced, {} skipped, {} errors", 
                                      shop.name, stats.synced_orders, stats.skipped_orders, stats.error_orders),
                        level: "info".to_string(),
                        category: "sync".to_string(),
                        shop_id: Some(shop.id.clone()),
                    });
                },
                Err(e) => {
                    // Log error but continue with next shop
                    let _ = app_handle.emit("sync-error", (e.to_string(), shop.id.clone()));
                    let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: format!("Synchronization failed for shop '{}': {}", shop.name, e),
                        level: "error".to_string(),
                        category: "sync".to_string(),
                        shop_id: Some(shop.id.clone()),
                    });
                }
            }
            
            // Brief pause between shop syncs
            sleep(TokioDuration::from_millis(500)).await;
            
            // Check for abort between shop syncs
            if should_abort() {
                let _ = app_handle.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: "Multi-shop synchronization aborted by user".to_string(),
                    level: "warn".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
                
                return Ok(());
            }
        }
        
        // All shops synced
        let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: "Sequential synchronization of all selected shops completed".to_string(),
            level: "info".to_string(),
            category: "sync".to_string(),
            shop_id: None,
        });
        
        Ok(())
    }
    
    /// Synchronize a single shop
    pub async fn sync_shop<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        shop: &ShopConfig,
        hours: i32
    ) -> Result<SyncStats> {
        info!("Starting synchronization Joomla -> JTL for shop '{}' with {}h timeframe", shop.name, hours);

        let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: format!("Starting synchronization process for shop '{}' with {}h timeframe...", shop.name, hours),
            level: "info".to_string(),
            category: "sync".to_string(),
            shop_id: Some(shop.id.clone()),
        });

        // Get database connection
        let pool = self.conn_manager.get_joomla_pool(shop)?;

        // Get orders within timeframe
        let orders = get_orders_within_timeframe(&pool, shop, hours)?;
        
        let total_orders = orders.len();
        
        let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: format!("Found {} orders to process for shop '{}'", total_orders, shop.name),
            level: "info".to_string(),
            category: "sync".to_string(),
            shop_id: Some(shop.id.clone()),
        });

        // Initialize stats with correct total
        let mut stats = SyncStats {
            shop_id: shop.id.clone(),
            total_orders: total_orders as i32,
            synced_orders: 0,
            skipped_orders: 0,
            error_orders: 0,
            last_sync_time: Some(Utc::now()),
            next_scheduled_run: None,
            aborted: false,
            sync_hours: hours,
        };
        
        update_sync_stats(stats.clone());
        app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
            .map_err(|e| Error::System(format!("Failed to emit event: {}", e)))?;
            
        if orders.is_empty() {
            info!("No new orders in the past {} hours for shop '{}'", hours, shop.name);
            return Ok(stats);
        }
        
        // Process each order
        for order in orders {
            if should_abort() {
                info!("Synchronization aborted, stopping after current order for shop '{}'", shop.name);
                
                let _ = app_handle.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization for shop '{}' aborted on user request", shop.name),
                    level: "warn".to_string(),
                    category: "sync".to_string(),
                    shop_id: Some(shop.id.clone()),
                });
                
                // Set aborted flag in stats
                stats.aborted = true;
                
                update_sync_stats(stats.clone());
                app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
                    .map_err(|e| Error::System(format!("Failed to emit event: {}", e)))?;
                    
                break;
            }

            info!("Processing order: ID={}, Shop={}, Customer={} {}", 
                  order.virtuemart_order_id,
                  shop.name,
                  order.first_name.as_deref().unwrap_or(""), 
                  order.last_name.as_deref().unwrap_or(""));
            
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: format!("Processing order {} for shop '{}', customer: {} {}", 
                    order.order_number,
                    shop.name,
                    order.first_name.as_deref().unwrap_or(""),
                    order.last_name.as_deref().unwrap_or("")
                ),
                level: "info".to_string(),
                category: "sync".to_string(),
                shop_id: Some(shop.id.clone()),
            });

            match process_order(&self.api_client, &pool, &order, shop).await {
                Ok(processed) => {
                    if processed {
                        stats.synced_orders += 1;

                        let _ = app_handle.emit("log", LogEntry {
                            timestamp: Utc::now(),
                            message: format!("Successfully synchronized order {} for shop '{}'", order.order_number, shop.name),
                            level: "info".to_string(),
                            category: "sync".to_string(),
                            shop_id: Some(shop.id.clone()),
                        });

                        info!("Order {} successfully synchronized for shop '{}'", order.order_number, shop.name);
                    } else {
                        stats.skipped_orders += 1;

                        let _ = app_handle.emit("log", LogEntry {
                            timestamp: Utc::now(),
                            message: format!("Order {} for shop '{}' already exists, skipped", order.order_number, shop.name),
                            level: "warn".to_string(),
                            category: "sync".to_string(),
                            shop_id: Some(shop.id.clone()),
                        });

                        info!("Order {} skipped (already exists) for shop '{}'", order.order_number, shop.name);
                    }
                },
                Err(e) => {
                    stats.error_orders += 1;

                    let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: format!("Error processing order {} for shop '{}': {}", order.order_number, shop.name, e),
                        level: "error".to_string(),
                        category: "sync".to_string(),
                        shop_id: Some(shop.id.clone()),
                    });

                    error!("Error with order {} for shop '{}': {}", order.virtuemart_order_id, shop.name, e);
                }
            }

            update_sync_stats(stats.clone());
            app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
                .map_err(|e| Error::System(format!("Failed to emit event: {}", e)))?;

            // Track progress
            info!("Progress for shop '{}': {}/{} (synced: {}, skipped: {}, errors: {})", 
                shop.name,
                stats.synced_orders + stats.skipped_orders + stats.error_orders,
                total_orders,
                stats.synced_orders,
                stats.skipped_orders,
                stats.error_orders
            );

            // Add order to synced orders collection
            if let Some(window) = app_handle.get_webview_window("main") {
                window.emit("synced-order", (shop.id.clone(), order.clone()))
                    .map_err(|e| Error::System(format!("Failed to emit synced order: {}", e)))?;
            }

            // Brief pause between orders to prevent overwhelming the server
            sleep(TokioDuration::from_millis(150)).await;
        }
        
        // Summarize results
        info!("Synchronization completed for shop '{}': {} transferred, {} skipped, {} errors", 
            shop.name, stats.synced_orders, stats.skipped_orders, stats.error_orders);
        
        update_sync_stats(stats.clone());
        app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
            .map_err(|e| Error::System(format!("Failed to emit event: {}", e)))?;

        // Emit final sync complete event
        app_handle.emit("sync-process-complete", (shop.id.clone(), stats.clone()))
            .map_err(|e| Error::System(format!("Failed to emit process complete event: {}", e)))?;
        
        let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: format!("Sync completed for shop '{}': {} synced, {} skipped, {} errors", 
                shop.name, stats.synced_orders, stats.skipped_orders, stats.error_orders),
            level: "info".to_string(),
            category: "sync".to_string(),
            shop_id: Some(shop.id.clone()),
        });

        Ok(stats)
			}
		}