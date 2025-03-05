use tauri::{AppHandle, Runtime, Emitter};
use chrono::Utc;
use std::time::SystemTime;
use log::info;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;

use crate::models::{AppConfig, LogEntry, SyncStats, VirtueMartOrder, ShopConfig};
use crate::config::{load_config, save_config, add_shop, update_shop, remove_shop, set_current_shop, get_current_shop};
use crate::sync::{get_current_stats, perform_sync, update_sync_stats, get_shop_stats, update_shop_sync_hours, sync_multiple_shops};

lazy_static! {
    static ref ABORT_FLAG: AtomicBool = AtomicBool::new(false);
    static ref SYNCED_ORDERS: Mutex<HashMap<String, Vec<VirtueMartOrder>>> = Mutex::new(HashMap::new());
}

/// Prüft, ob die Synchronisierung abgebrochen werden soll
pub fn should_abort() -> bool {
    ABORT_FLAG.load(Ordering::SeqCst)
}

/// Setzt das Abort-Flag zurück
pub fn reset_abort_flag() {
    ABORT_FLAG.store(false, Ordering::SeqCst);
}

/// Befehl zum Abbrechen der laufenden Synchronisierung
#[tauri::command]
pub async fn abort_sync_command<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    info!("Synchronisierung wird abgebrochen...");
    
    // Abort-Flag setzen
    ABORT_FLAG.store(true, Ordering::SeqCst);
    
    // Abbruch protokollieren
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Synchronization aborted by user".to_string(),
        level: "warn".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    Ok(())
}

/// Geplante Synchronisierung starten
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
    
    // Log über Start der geplanten Synchronisierung
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting scheduled synchronization for {} shops, job {}", shop_ids.len(), job_id),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    // Abort-Flag zurücksetzen vor dem Start
    reset_abort_flag();
    
    // Hintergrundaufgabe für die Synchronisierung starten
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let shop_ids_clone = shop_ids.clone();
    
    tauri::async_runtime::spawn(async move {
        match sync_multiple_shops(&app_handle_clone, &config_clone, shop_ids_clone).await {
            Ok(_) => {
                // Events senden
                let _ = app_handle_clone.emit("multi-sync-complete", job_id.clone());
                let _ = app_handle_clone.emit("scheduled-sync-completed", (job_id.clone(), shop_ids));
                
                // Erfolg protokollieren
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Scheduled synchronization completed for job {}", job_id),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            },
            Err(e) => {
                // Fehler protokollieren
                let _ = app_handle_clone.emit("sync-error", e.clone());
                let _ = app_handle_clone.emit("scheduled-sync-error", (job_id.clone(), e.clone()));
                
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Scheduled synchronization failed for job {}: {}", job_id, e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            }
        }
    });
    
    Ok(())
}

/// Speichert synchronisierte Bestellungen für einen bestimmten Shop
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

/// Fügt eine synchronisierte Bestellung hinzu
pub fn add_synced_order<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>, shop_id: &str, order: VirtueMartOrder) {
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
        
        // Debug-Log hinzufügen
        info!("Bestellung zu SYNCED_ORDERS für Shop {} hinzugefügt. Aktuelle Anzahl: {}", shop_id, orders.len());
        
        // Daten an das Frontend senden
        app_handle.emit("synced-orders", (shop_id.to_string(), orders.clone()))
            .map_err(|e| format!("Failed to emit synced orders: {}", e)).ok();
    }
}

#[tauri::command]
pub async fn get_synced_orders<R: Runtime>(
    app_handle: AppHandle<R>,
    shop_id: Option<String>
) -> Result<Vec<VirtueMartOrder>, String> {
    info!("getting synced orders for shop: {:?}", shop_id);
    
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

/// System-Informationen abrufen
#[tauri::command]
pub fn get_system_info() -> serde_json::Value {
    serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "memory": "N/A", // Würde zusätzliches Crate benötigen
        "uptime": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
    })
}

/// Konfiguration speichern
#[tauri::command]
pub async fn save_config_command<R: Runtime>(app_handle: AppHandle<R>, config: AppConfig) -> Result<(), String> {
    match save_config(&config) {
        Ok(_) => {
            // Log-Ereignis senden
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: "Configuration saved successfully".to_string(),
                level: "info".to_string(),
                category: "system".to_string(),
                shop_id: None,
            });
            
            Ok(())
        },
        Err(e) => Err(e)
    }
}

/// Konfiguration laden
#[tauri::command]
pub async fn load_config_command<R: Runtime>(app_handle: AppHandle<R>) -> Result<AppConfig, String> {
    match load_config() {
        Ok(config) => {
            // Log-Ereignis senden
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: "Configuration loaded successfully".to_string(),
                level: "info".to_string(),
                category: "system".to_string(),
                shop_id: None,
            });
            
            Ok(config)
        },
        Err(e) => Err(e)
    }
}

/// Shop hinzufügen
#[tauri::command]
pub async fn add_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop: ShopConfig) -> Result<AppConfig, String> {
    let mut config = load_config()?;
    
    add_shop(&mut config, shop.clone())?;
    
    // Log-Ereignis senden
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("New shop '{}' added successfully", shop.name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop.id),
    });
    
    Ok(config)
}

/// Shop aktualisieren
#[tauri::command]
pub async fn update_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop: ShopConfig) -> Result<AppConfig, String> {
    let mut config = load_config()?;
    
    update_shop(&mut config, shop.clone())?;
    
    // Log-Ereignis senden
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Shop '{}' updated successfully", shop.name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop.id),
    });
    
    Ok(config)
}

/// Shop löschen
#[tauri::command]
pub async fn remove_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop_id: String) -> Result<AppConfig, String> {
    let mut config = load_config()?;
    
    // Find shop name for logging before removing
    let shop_name = config.shops.iter()
        .find(|s| s.id == shop_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    remove_shop(&mut config, &shop_id)?;
    
    // Log-Ereignis senden
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Shop '{}' removed successfully", shop_name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: None,
    });
    
    Ok(config)
}

/// Aktuellen Shop setzen
#[tauri::command]
pub async fn set_current_shop_command<R: Runtime>(app_handle: AppHandle<R>, shop_id: String) -> Result<AppConfig, String> {

    let mut config = load_config()?;
    
    set_current_shop(&mut config, &shop_id)?;
    
    // Find the shop name
    let shop_name = config.shops.iter()
        .find(|s| s.id == shop_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Log-Ereignis senden
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Active shop changed to '{}'", shop_name),
        level: "info".to_string(),
        category: "system".to_string(),
        shop_id: Some(shop_id),
    });
    
    Ok(config)
}

/// Manuelle Synchronisierung von mehreren Shops starten
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
    
    // Log über Start der Synchronisierung
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting manual synchronization for {} shops...", shop_ids.len()),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });
    
    // Reset abort flag
    reset_abort_flag();
    
    // Hintergrundaufgabe erstellen, um die Synchronisierung durchzuführen
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let shop_ids_clone = shop_ids.clone();
    
    // Hintergrundaufgabe starten, damit wir die UI nicht blockieren
    tauri::async_runtime::spawn(async move {
        match sync_multiple_shops(&app_handle_clone, &config_clone, shop_ids_clone).await {
            Ok(_) => {
                // Erfolgs-Event senden
                let _ = app_handle_clone.emit("multi-sync-complete", ());
                
                // Erfolg loggen
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Multi-shop synchronization completed successfully"),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            },
            Err(e) => {
                // Fehler-Event senden
                let _ = app_handle_clone.emit("sync-error", e.clone());
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Multi-shop synchronization failed: {}", e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: None,
                });
            }
        }
    });
    
    Ok(())
}

/// Manuelle Synchronisierung eines Shops starten
#[tauri::command]
pub async fn start_sync_command<R: Runtime>(
    app_handle: AppHandle<R>, 
    shop_id: Option<String>,
    hours: Option<i32>
) -> Result<(), String> {
    // Load the configuration
    let config = load_config()?;
    
    // Determine which shop to sync
    let shop = if let Some(id) = shop_id {
        // Find the specific shop
        config.shops.iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Shop with ID '{}' not found", id))?
            .clone()
    } else {
        // Use the current shop
        get_current_shop(&config)
    };
    
    // Get the sync hours (default to 24 if not provided)
    let sync_hours = hours.unwrap_or_else(|| get_shop_stats(&shop.id).sync_hours);
    
    // If hours was provided, update the shop's sync_hours
    if let Some(h) = hours {
        update_shop_sync_hours(&shop.id, h)?;
    }
    
    // Log über Start der Synchronisierung
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting manual synchronization for shop '{}' with {}h timeframe...", shop.name, sync_hours),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });
    
    // Reset abort flag
    reset_abort_flag();
    
    // Hintergrundaufgabe erstellen, um die Synchronisierung durchzuführen
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let shop_clone = shop.clone();
    
    // Hintergrundaufgabe starten, damit wir die UI nicht blockieren
    tauri::async_runtime::spawn(async move {
        match perform_sync(&app_handle_clone, &config_clone, &shop_clone, sync_hours).await {
            Ok(stats) => {
                update_sync_stats(stats.clone());
                
                // Erfolgs-Event mit einer Kopie der Statistiken senden
                let _ = app_handle_clone.emit("sync-complete", stats.clone());
                
                // Erfolg loggen
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
                // Fehler-Event senden
                let _ = app_handle_clone.emit("sync-error", (e.clone(), shop_clone.id.clone()));
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization failed for shop '{}': {}", shop_clone.name, e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: Some(shop_clone.id),
                });
            }
        }
    });
    
    // Sofort initiale Statistiken zurückgeben (tatsächliche Statistiken werden über Events aktualisiert)
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

/// Aktuelle Synchronisierungsstatistiken abrufen
#[tauri::command]
pub async fn get_sync_stats(shop_id: Option<String>) -> Result<SyncStats, String> {
    if let Some(id) = shop_id {
        Ok(get_shop_stats(&id))
    } else {
        Ok(get_current_stats())
    }
}

/// Synchronisierung planen
#[tauri::command]
pub async fn schedule_sync(shop_ids: Vec<String>, cron_expression: String) -> Result<(), String> {
    // In einer echten Implementierung würde ein Cron-Job oder Timer eingerichtet
    // Für jetzt loggen wir es einfach
    info!("Scheduled sync for {} shops with cron: {}", shop_ids.len(), cron_expression);
    Ok(())
}

/// Geplante Synchronisierungsjobs abbrechen
#[tauri::command]
pub async fn cancel_scheduled_sync(shop_id: Option<String>) -> Result<(), String> {
    // In einer echten Implementierung würden geplante Jobs abgebrochen
    if let Some(id) = shop_id {
        info!("Canceled scheduled sync jobs for shop {}", id);
    } else {
        info!("Canceled all scheduled sync jobs");
    }
    Ok(())
}