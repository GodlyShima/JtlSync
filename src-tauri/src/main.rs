#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

// Shared state to track JTL API process
struct JtlApiState {
    process_handle: Mutex<Option<std::process::Child>>,
}

// Configuration structure that matches the TypeScript config
#[derive(Serialize, Deserialize, Clone)]
struct DatabaseConfig {
    host: String,
    user: String,
    password: String,
    database: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TablesConfig {
    orders: String,
    order_items: String,
    customers: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    joomla: DatabaseConfig,
    jtl: DatabaseConfig,
    tables: TablesConfig,
    log_file: String,
    jtl_api_path: String,
}

// Stats structure for the dashboard
#[derive(Serialize, Deserialize, Clone)]
struct SyncStats {
    total_orders: i32,
    synced_orders: i32,
    skipped_orders: i32,
    error_orders: i32,
    last_sync_time: Option<DateTime<Utc>>,
    next_scheduled_run: Option<DateTime<Utc>>,
}

// Command to start JTL API
#[tauri::command]
async fn start_jtl_api(
    app_handle: AppHandle,
    state: tauri::State<'_, JtlApiState>,
    config: AppConfig,
) -> Result<String, String> {
    let api_path = config.jtl_api_path;
    
    // Check if already running
    let mut process_handle = state.process_handle.lock().unwrap();
    if process_handle.is_some() {
        return Err("JTL API is already running".to_string());
    }
    
    // Start the JTL API process
    match Command::new(&api_path)
        .args(["--w", "Standard", "--l", "127.0.0.1", "--port", "5883"])
        .spawn() {
            Ok(child) => {
                *process_handle = Some(child);
                
                // Emit event to frontend
                app_handle.emit_all("jtl-api-status", true).unwrap();
                
                Ok("JTL API started successfully".to_string())
            },
            Err(e) => {
                Err(format!("Failed to start JTL API: {}", e))
            }
        }
}

// Command to stop JTL API
#[tauri::command]
async fn stop_jtl_api(
    app_handle: AppHandle,
    state: tauri::State<'_, JtlApiState>,
) -> Result<String, String> {
    let mut process_handle = state.process_handle.lock().unwrap();
    
    match process_handle.take() {
        Some(mut child) => {
            match child.kill() {
                Ok(_) => {
                    // Emit event to frontend
                    app_handle.emit_all("jtl-api-status", false).unwrap();
                    Ok("JTL API stopped successfully".to_string())
                },
                Err(e) => {
                    *process_handle = Some(child); // Put it back if kill failed
                    Err(format!("Failed to stop JTL API: {}", e))
                }
            }
        },
        None => Err("JTL API is not running".to_string())
    }
}

// Command to check if JTL API is running
#[tauri::command]
fn is_jtl_api_running(state: tauri::State<JtlApiState>) -> bool {
    state.process_handle.lock().unwrap().is_some()
}

// Command to get system information
#[tauri::command]
fn get_system_info() -> serde_json::Value {
    serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "memory": "N/A", // Would need additional crate to get this
        "uptime": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
    })
}

// Command to save configuration
#[tauri::command]
async fn save_config(app_handle: AppHandle, config: AppConfig) -> Result<(), String> {
    let app_dir = app_handle.path_resolver()
        .app_config_dir()
        .ok_or_else(|| "Failed to get app config directory".to_string())?;
    
    // Create directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let config_path = app_dir.join("config.json");
    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

// Command to load configuration
#[tauri::command]
async fn load_config(app_handle: AppHandle) -> Result<AppConfig, String> {
    let app_dir = app_handle.path_resolver()
        .app_config_dir()
        .ok_or_else(|| "Failed to get app config directory".to_string())?;
    
    let config_path = app_dir.join("config.json");
    
    if !config_path.exists() {
        return Err("Config file does not exist".to_string());
    }
    
    let config_str = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let config: AppConfig = serde_json::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    Ok(config)
}

// The main function
fn main() {
    // Setup logger first
    env_logger::init();
    
    tauri::Builder::default()
        .manage(JtlApiState {
            process_handle: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            start_jtl_api,
            stop_jtl_api,
            is_jtl_api_running,
            get_system_info,
            save_config,
            load_config,
        ])
        .setup(|app| {
            // Any initialization code here
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}