use chrono::{Utc, Duration};
use log::{info, error, warn};
use mysql::prelude::ToValue;
use tauri::Emitter;
use tokio::time::sleep;
use tokio::time::Duration as TokioDuration;
use std::sync::Mutex;
use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::api::JtlApiClient;
use crate::commands::add_synced_order;
use crate::commands::should_abort;
use crate::database::{connect_to_joomla, get_orders_within_timeframe, get_order_items, get_shipping_address};
use crate::models::LogEntry;
use crate::utils::emit_event;
use crate::models::{AppConfig, ShopConfig, SyncStats, VirtueMartOrder, JtlCustomer, JtlOrder, JtlOrderItem, JtlCountry, JtlPaymentDetails, JtlShippingDetails};
use crate::utils::{format_iso_date, get_country_code, map_payment_method, create_address_object};

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

// Update sync stats for a specific shop
pub fn update_sync_stats(stats: SyncStats) {
    let mut current_stats = SYNC_STATS.lock().unwrap();
    current_stats.insert(stats.shop_id.clone(), stats);
}

// Get sync stats for a specific shop
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

// Get stats for the "current" shop - used for backward compatibility
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
pub fn update_shop_sync_hours(shop_id: &str, hours: i32) -> Result<(), String> {
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

/// Synchronize multiple shops sequentially
pub async fn sync_multiple_shops<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    config: &AppConfig,
    shop_ids: Vec<String>
) -> Result<(), String> {
    info!("Starting sequential synchronization for {} shops", shop_ids.len());

    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting sequential synchronization for {} shops", shop_ids.len()),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: None,
    });

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
        match perform_sync(app_handle, config, &shop, sync_hours).await {
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
                    shop_id: Some(shop.id),
                });
            },
            Err(e) => {
                // Log error but continue with next shop
                let _ = app_handle.emit("sync-error", (e.clone(), shop.id.clone()));
                let _ = app_handle.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Synchronization failed for shop '{}': {}", shop.name, e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                    shop_id: Some(shop.id),
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
            
            return Ok(()); // Exit early but return Ok since this is an intentional abort
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

/// Hauptfunktion für die Synchronisierung eines bestimmten Shops
pub async fn perform_sync<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    config: &AppConfig,
    shop: &ShopConfig,
    hours: i32
) -> Result<SyncStats, String> {
    info!("Starting synchronization Joomla -> JTL for shop '{}' with {}h timeframe", shop.name, hours);

    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Starting synchronization process for shop '{}' with {}h timeframe...", shop.name, hours),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });

    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Connecting to Joomla database for shop '{}'...", shop.name),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });

    // Verbindung zur Joomla-Datenbank herstellen
    let joomla_conn = connect_to_joomla(shop)
        .map_err(|e| {
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: format!("Database connection failed for shop '{}': {}", shop.name, e),
                level: "error".to_string(),
                category: "sync".to_string(),
                shop_id: Some(shop.id.clone()),
            });
            format!("Database connection error: {}", e)
        })?;

    // JTL-API-Client initialisieren
    info!("Initializing JTL API Client for shop '{}'...", shop.name);
    let api_key = "4fef6933-ae20-4cbc-bd97-a5cd584f244e";
    let client = JtlApiClient::new(api_key);
    
    // Neue Bestellungen im konfigurierten Zeitraum abrufen
    info!("Fetching orders from past {} hours for shop '{}'...", hours, shop.name);

    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Fetching orders from past {} hours for shop '{}'...", hours, shop.name),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });

    let orders = get_orders_within_timeframe(&joomla_conn, shop, hours)?;

    let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Found {} orders to process for shop '{}'", orders.len(), shop.name),
        level: "info".to_string(),
        category: "sync".to_string(),
        shop_id: Some(shop.id.clone()),
    });
    
    let total_orders = orders.len();

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
        .map_err(|e| format!("Failed to emit event: {}", e))?;    

    info!("Starting sync for shop '{}' with {} orders", shop.name, total_orders);
    
    if orders.is_empty() {
        info!("No new orders in the past {} hours for shop '{}'", hours, shop.name);
        return Ok(stats);
    }
    
    // Jede Bestellung verarbeiten
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
            
            // Setze das aborted-Flag in stats
            stats.aborted = true;
            
            update_sync_stats(stats.clone());
            app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
                .map_err(|e| format!("Failed to emit event: {}", e))?;
                
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

        match process_order(&client, &joomla_conn, &order, shop).await {
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
            .map_err(|e| format!("Failed to emit event: {}", e))?;    

        info!("Progress for shop '{}': {}/{} (synced: {}, skipped: {}, errors: {})", 
            shop.name,
            stats.synced_orders + stats.skipped_orders + stats.error_orders,
            total_orders,
            stats.synced_orders,
            stats.skipped_orders,
            stats.error_orders
        );

        info!("Adding order {} to SYNCED_ORDERS for shop '{}'", order.order_number, shop.name);
        add_synced_order(app_handle, &shop.id, order.clone());

        sleep(TokioDuration::from_millis(150)).await;
    }
    
    // Zusammenfassung
    info!("Synchronization completed for shop '{}': {} transferred, {} skipped, {} errors", 
          shop.name, stats.synced_orders, stats.skipped_orders, stats.error_orders);
    
    update_sync_stats(stats.clone());
    app_handle.emit("sync-stats-update", (shop.id.clone(), stats.clone()))
        .map_err(|e| format!("Failed to emit event: {}", e))?;    

    // Emit final sync complete event
    app_handle.emit("sync-process-complete", (shop.id.clone(), stats.clone()))
        .map_err(|e| format!("Failed to emit process complete event: {}", e))?;
    
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

/// Einzelne Bestellung verarbeiten
async fn process_order(
    client: &JtlApiClient, 
    joomla_conn: &mysql::Pool, 
    order: &crate::models::VirtueMartOrder, 
    shop: &ShopConfig
) -> Result<bool, String> {
    // Kundennummer erstellen mit Shop-ID-Prefix für Eindeutigkeit zwischen Shops
    let customer_number = format!("VM{}", order.virtuemart_order_userinfo_id.clone().unwrap_or_default().to_string());
    
    info!("Kundennummer aus Joomla für Shop '{}': {}", shop.name, customer_number);
    
    // Lieferadresse abrufen
    let shipping_address = get_shipping_address(joomla_conn, shop, order.virtuemart_order_id)?;
    
    // JTL-Zahlungsmethode mappen
    let jtl_payment_method_id = map_payment_method(order.virtuemart_paymentmethod_id);
    
    // Bestellnummer mit Shop-ID-Prefix für Eindeutigkeit zwischen Shops
    let order_number = format!("VM{}", order.virtuemart_order_id);

    
    // Prüfen, ob der Kunde bereits existiert
    match client.get_customer_by_id(&customer_number).await {
        Ok(maybe_customer) => {
            let customer_id = match maybe_customer {
                Some(customer) => {
                    info!("Kunde {} existiert bereits mit ID: {} (Shop: '{}')", 
                          customer_number, customer["Id"], shop.name);
                    customer["Id"].as_str().unwrap_or("0").to_string()
                },
                None => {
                    // Neuen Kunden erstellen
                    info!("Erstelle neuen Kunden {} für Shop '{}'", customer_number, shop.name);
                    
                    let billing_address = create_address_object(order);
                    let shipping_addr = match &shipping_address {
                        Some(addr) => create_address_object(addr),
                        None => billing_address.clone(),
                    };
                    
                    let customer_data = JtlCustomer {
                        CustomerGroupId: 1,
                        BillingAddress: billing_address,
                        InternalCompanyId: 1,
                        LanguageIso: "DE".to_string(),
                        Shipmentaddress: shipping_addr,
                        CustomerSince: format_iso_date(&order.created_on),
                        Number: customer_number.clone(),
                    };
                    
                    match client.create_customer(&customer_data).await {
                        Ok(response) => {
                            info!("Kunde erstellt mit ID: {} für Shop '{}'", response["Id"], shop.name);
                            response["Id"].to_string()
                        },
                        Err(e) => return Err(format!("Fehler beim Erstellen des Kunden für Shop '{}': {}", shop.name, e))
                    }
                }
            };
            
            // Prüfen, ob die Bestellung bereits existiert
            match client.check_order_exists(&order_number, &customer_id).await {
                Ok(exists) => {
                    if exists {
                        warn!("Bestellung {} existiert bereits für Shop '{}', überspringe", 
                              order_number, shop.name);
                        return Ok(false);
                    }
                },
                Err(e) => return Err(format!("Fehler beim Prüfen der Bestellung für Shop '{}': {}", shop.name, e))
            }
            
            // Bestellpositionen abrufen
            let items = match get_order_items(joomla_conn, shop, order.virtuemart_order_id) {
                Ok(i) => i,
                Err(e) => return Err(format!("Fehler beim Abrufen der Bestellpositionen für Shop '{}': {}", shop.name, e))
            };
            
            info!("Bestellpositionen gefunden: {} für Shop '{}'", items.len(), shop.name);
            
            // JTL-Bestellung erstellen
            info!("Erstelle Bestellung {} in JTL für Shop '{}'", order_number, shop.name);
            
            let billing_address = create_address_object(order);
            let shipping_addr = match &shipping_address {
                Some(addr) => create_address_object(addr),
                None => billing_address.clone(),
            };

            info!("CustomerId: {} für Shop '{}'", customer_id.clone(), shop.name);
            info!("ExternalNumber: {} für Shop '{}'", order_number.clone(), shop.name);
            info!("Country: {} ID: {} für Shop '{}'", 
                  get_country_code(order.virtuemart_country_id.clone().unwrap_or_default()).unwrap_or_default().to_string(), 
                  order.virtuemart_country_id.clone().unwrap_or_default(),
                  shop.name);

            let jtl_order = JtlOrder {
                CustomerId: customer_id.parse::<i32>().unwrap_or_default(),
                ExternalNumber: order_number.clone(),
                CompanyId: 1,
                DepartureCountry: JtlCountry {
                    CountryISO: "DE".to_string(),
                    CurrencyIso: "EUR".to_string(),
                    CurrencyFactor: 1.0,
                },
                BillingAddress: billing_address,
                Shipmentaddress: shipping_addr,
                SalesOrderDate: format_iso_date(&order.created_on),
                SalesOrderPaymentDetails: JtlPaymentDetails {
                    PaymentMethodId: jtl_payment_method_id,
                    CurrencyIso: "EUR".to_string(),
                    CurrencyFactor: 1.0,
                },
                SalesOrderShippingDetail: JtlShippingDetails {
                    ShippingMethodId: 7, // Standard-Versandmethode
                    ShippingDate: format_iso_date(&order.created_on),
                },
                Comment: format!("Shop: {} - {}", shop.name, order.customer_note.clone().unwrap_or_default()),
                LanguageIso: "DE".to_string(),
            };
            
            // Bestellpositionen für JTL vorbereiten
            let mut all_items: Vec<JtlOrderItem> = items.iter().map(|item| {
                JtlOrderItem {
                    Quantity: item.product_quantity,
                    SalesPriceGross: Some(item.product_final_price),
                    TaxRate: 19.0,
                    Name: format!("[{}] {}", shop.name, item.order_item_name.clone()),
                    SalesUnit: "stk".to_string(),
                    SalesPriceNet: Some(item.product_priceWithoutTax.unwrap_or(item.product_final_price / 1.19)),
                    PurchasePriceNet: None,
                }
            }).collect();

            // add coupon
            if let Some(coupon_code) = &order.coupon_code {
                if !coupon_code.is_empty() {
                    let discount = order.coupon_discount.unwrap_or_default();
                    all_items.push(JtlOrderItem {
                        Quantity: 1,
                        SalesPriceGross: Some(discount),
                        TaxRate: 0.0,
                        Name: format!("[{}] Gutschein: {}", shop.name, coupon_code),
                        SalesUnit: "stk".to_string(),
                        PurchasePriceNet: None,
                        SalesPriceNet: Some(discount),
                    });
                }
            }

            // Versandposition hinzufügen     
            if let Some(shipping_cost) = order.order_shipment {
                if shipping_cost > 0.0 {
                    all_items.push(JtlOrderItem {
                        Quantity: 1,
                        SalesPriceGross: Some(shipping_cost),
                        TaxRate: 19.0,
                        Name: format!("[{}] Versand", shop.name),
                        SalesUnit: "stk".to_string(),
                        SalesPriceNet: Some(shipping_cost / 1.19),
                        PurchasePriceNet: None,
                    });
                }
            }
            
            // Bestellung in JTL erstellen
            match client.create_order(&jtl_order, &all_items).await {
                Ok(response) => {
                    let order_id = response["Id"].to_string();
                    info!("Bestellung {} erfolgreich in JTL erstellt mit ID: {} für Shop '{}'", 
                          order_number, order_id, shop.name);
                    
                    // Falls bereits bezahlt
                    if let Some(status) = &order.order_status {
                        if status == "C" && jtl_payment_method_id != 4 {
                            info!("Bestellung {} ist bezahlt -> setze auf bezahlt für Shop '{}'", 
                                  order_number, shop.name);
                            match client.set_payment_paid(&order_id).await {
                                Ok(_) => {},
                                Err(e) => warn!("Fehler beim Setzen des Zahlungsstatus für Shop '{}': {}", shop.name, e)
                            }
                        }
                    }


										match client.set_order_hold(&order_id).await {
											Ok(_) => {},
                      Err(e) => warn!("Fehler beim Setzen der Bestellung auf In Prüfung '{}': {}", shop.name, e)
										}
                    
                    Ok(true)
                },
                Err(e) => Err(format!("Fehler beim Erstellen der Bestellung für Shop '{}': {}", shop.name, e))
            }
        },
        Err(e) => Err(format!("Fehler beim Abrufen des Kunden für Shop '{}': {}", shop.name, e))
    }
}