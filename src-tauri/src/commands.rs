use tauri::{AppHandle, Runtime, Emitter};
use chrono::Utc;
use std::time::SystemTime;
use log::info;
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::models::{AppConfig, LogEntry, SyncStats, VirtueMartOrder};
use crate::config::{load_config, save_config};
use crate::sync::{get_current_stats, perform_sync, update_sync_stats};

lazy_static! {
    static ref SYNCED_ORDERS: Mutex<Vec<VirtueMartOrder>> = Mutex::new(Vec::new());
}

/// Speichert synchronisierte Bestellungen
pub fn store_synced_orders(orders: Vec<VirtueMartOrder>) {
    let mut stored_orders = SYNCED_ORDERS.lock().unwrap();
    *stored_orders = orders;
}

/// Fügt eine synchronisierte Bestellung hinzu
pub fn add_synced_order<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>, order: VirtueMartOrder) {
    let mut stored_orders = SYNCED_ORDERS.lock().unwrap();
    stored_orders.push(order);

    // Debug-Log hinzufügen
    info!("Bestellung zu SYNCED_ORDERS hinzugefügt. Aktuelle Anzahl: {}", stored_orders.len());

    // Daten an das Frontend senden
    app_handle.emit("synced-orders", stored_orders.clone())
        .map_err(|e| format!("Failed to emit synced orders: {}", e));
}


#[tauri::command]
pub async fn get_synced_orders<R: Runtime>(
    app_handle: AppHandle<R>
) -> Result<Vec<VirtueMartOrder>, String> {
    info!("Requesting synced orders");
    
    let stored_orders = SYNCED_ORDERS.lock().unwrap().clone();
    
    // Emit the orders to the frontend
    app_handle.emit("synced-orders", stored_orders.clone())
        .map_err(|e| format!("Failed to emit synced orders: {}", e))?;
    
    Ok(stored_orders)
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
            });
            
            Ok(config)
        },
        Err(e) => Err(e)
    }
}

/// Manuelle Synchronisierung starten
#[tauri::command]
pub async fn start_sync_command<R: Runtime>(app_handle: AppHandle<R>, config: AppConfig) -> Result<(), String> {
    // Log über Start der Synchronisierung
    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Starting manual synchronization...".to_string(),
        level: "info".to_string(),
        category: "sync".to_string(),
    });
    
    // Hintergrundaufgabe erstellen, um die Synchronisierung durchzuführen
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    
    // Hintergrundaufgabe starten, damit wir die UI nicht blockieren
    tauri::async_runtime::spawn(async move {
        match perform_sync(&app_handle_clone, &config_clone).await {
            Ok(stats) => {
                update_sync_stats(stats.clone());

                
                // Erfolgs-Event mit einer Kopie der Statistiken senden
                let _ = app_handle_clone.emit("sync-complete", stats.clone());
                
                // Erfolg loggen
               let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization completed: {} synced, {} skipped, {} errors", 
                                   stats.synced_orders, stats.skipped_orders, stats.error_orders),
                    level: "info".to_string(),
                    category: "sync".to_string(),
                }); 
            },
            Err(e) => {
                // Fehler-Event senden
                let _ = app_handle_clone.emit("sync-error", e.clone());
                let _ = app_handle_clone.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization failed: {}", e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                });
            }
        }
    });
    
    // Sofort initiale Statistiken zurückgeben (tatsächliche Statistiken werden über Events aktualisiert)
    Ok(())
}

/// Aktuelle Synchronisierungsstatistiken abrufen
#[tauri::command]
pub async fn get_sync_stats() -> Result<SyncStats, String> {
    Ok(get_current_stats())
}

/// Synchronisierung planen
#[tauri::command]
pub async fn schedule_sync(cron_expression: String) -> Result<(), String> {
    // In einer echten Implementierung würde ein Cron-Job oder Timer eingerichtet
    // Für jetzt loggen wir es einfach
    info!("Scheduled sync with cron: {}", cron_expression);
    Ok(())
}

/// Geplante Synchronisierungsjobs abbrechen
#[tauri::command]
pub async fn cancel_scheduled_sync() -> Result<(), String> {
    // In einer echten Implementierung würden geplante Jobs abgebrochen
    info!("Canceled all scheduled sync jobs");
    Ok(())
}