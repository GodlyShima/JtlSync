use std::ptr::null;

use chrono::Utc;
use log::{info, error, warn};
use mysql::prelude::ToValue;
use tauri::Emitter;
use tokio::time::sleep;
use tokio::time::Duration;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::api::JtlApiClient;
use crate::commands::add_synced_order;
use crate::commands::should_abort;
use crate::database::{connect_to_joomla, get_new_orders, get_order_items, get_shipping_address};
use crate::models::LogEntry;
use crate::utils::emit_event;
use crate::models::{AppConfig, SyncStats, JtlCustomer, JtlOrder, JtlOrderItem, JtlCountry, JtlPaymentDetails, JtlShippingDetails};
use crate::utils::format_iso_date;
use crate::utils::get_country_code;
use crate::utils::{map_payment_method, create_address_object};

lazy_static! {
    static ref SYNC_STATS: Mutex<SyncStats> = Mutex::new(SyncStats {
        total_orders: 0,
        synced_orders: 0,
        skipped_orders: 0,
        error_orders: 0,
        last_sync_time: None,
        next_scheduled_run: None,
        aborted: false, // Initialisiere mit false
    });
}

pub fn update_sync_stats(stats: SyncStats) {
    let mut current = SYNC_STATS.lock().unwrap();
    *current = stats;
}

pub fn get_current_stats() -> SyncStats {
    SYNC_STATS.lock().unwrap().clone()
}

/// Hauptfunktion für die Synchronisierung
pub async fn perform_sync<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    config: &AppConfig
) -> Result<SyncStats, String> {
    info!("Starte Synchronisation Joomla -> JTL");

		let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Starting synchronization process...".to_string(),
        level: "info".to_string(),
        category: "sync".to_string(),
    });

		let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Connecting to Joomla database...".to_string(),
        level: "info".to_string(),
        category: "sync".to_string(),
    });

    // Verbindung zur Joomla-Datenbank herstellen
    let joomla_conn = connect_to_joomla(config)
        .map_err(|e| {
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: format!("Database connection failed: {}", e),
                level: "error".to_string(),
                category: "sync".to_string(),
            });
            format!("Database connection error: {}", e)
        })?;

    // JTL-API-Client initialisieren
    info!("Initialisiere JTL API Client...");
    let api_key = "4fef6933-ae20-4cbc-bd97-a5cd584f244e";
    let client = JtlApiClient::new(api_key);
    
    // Neue Bestellungen abrufen
    info!("Rufe neue Bestellungen aus Joomla ab...");

		let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: "Fetching new orders from last 24 hours...".to_string(),
        level: "info".to_string(),
        category: "sync".to_string(),
    });

    let orders = get_new_orders(&joomla_conn, config)?;

		 let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Found {} orders to process", orders.len()),
        level: "info".to_string(),
        category: "sync".to_string(),
    });
    
    let total_orders = orders.len();

		    // Initialize stats with correct total
    let mut stats = SyncStats {
        total_orders: total_orders as i32,
        synced_orders: 0,
        skipped_orders: 0,
        error_orders: 0,
        last_sync_time: Some(Utc::now()),
        next_scheduled_run: None,
        aborted: false, // Initialisiere mit false
    };
    
    
		update_sync_stats(stats.clone());
 		app_handle.emit("sync-stats-update", &stats)
        .map_err(|e| format!("Failed to emit event: {}", e))?;    

    info!("Starting sync with {} orders", total_orders);
    

    if orders.is_empty() {
        info!("Keine neuen Bestellungen in den letzten 24 Stunden");
   			return Ok(stats);
    }
    
    // Jede Bestellung verarbeiten
    for order in orders {

				if should_abort() {
            info!("Synchronisierung abgebrochen, stoppe nach aktueller Bestellung");
            
            let _ = app_handle.emit("log", LogEntry {
                timestamp: Utc::now(),
                message: "Synchronisierung auf Benutzeranfrage abgebrochen".to_string(),
                level: "warn".to_string(),
                category: "sync".to_string(),
            });
            
            // Setze das aborted-Flag in stats
            stats.aborted = true;
            
            update_sync_stats(stats.clone());
            app_handle.emit("sync-stats-update", &stats)
                .map_err(|e| format!("Failed to emit event: {}", e))?;
                
            break;
        }

        info!("Verarbeite Bestellung: ID={}, Kunde={} {}", 
              order.virtuemart_order_id, 
              order.first_name.as_deref().unwrap_or(""), 
              order.last_name.as_deref().unwrap_or(""));
        
				let _ = app_handle.emit("log", LogEntry {
            timestamp: Utc::now(),
            message: format!("Verarbeite Bestellung {}, Kunde: {} {}", 
                order.order_number,
                order.first_name.as_deref().unwrap_or(""),
                order.last_name.as_deref().unwrap_or("")
            ),
            level: "info".to_string(),
            category: "sync".to_string(),
        });

        match process_order(&client, &joomla_conn, &order, config).await {
            Ok(processed) => {
                if processed {
                    stats.synced_orders += 1;

										let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: format!("Successfully synchronized order {}", order.order_number),
                        level: "info".to_string(),
                        category: "sync".to_string(),
                    });

                    info!("Bestellung {} erfolgreich synchronisiert", order.order_number);
                } else {
                    stats.skipped_orders += 1;

										let _ = app_handle.emit("log", LogEntry {
                        timestamp: Utc::now(),
                        message: format!("Order {} already exists, skipped", order.order_number),
                        level: "warn".to_string(),
                        category: "sync".to_string(),
                    });

                    info!("Bestellung {} übersprungen (existiert bereits)", order.order_number);
                }
            },
            Err(e) => {
                stats.error_orders += 1;

								let _ = app_handle.emit("log", LogEntry {
                    timestamp: Utc::now(),
                    message: format!("Error processing order {}: {}", order.order_number, e),
                    level: "error".to_string(),
                    category: "sync".to_string(),
                });

                error!("Fehler bei Bestellung {}: {}", order.virtuemart_order_id, e);
            }
        }

        update_sync_stats(stats.clone());
				app_handle.emit("sync-stats-update", &stats)
							.map_err(|e| format!("Failed to emit event: {}", e))?;    

				 info!("Progress: {}/{} (synced: {}, skipped: {}, errors: {})", 
            stats.synced_orders + stats.skipped_orders + stats.error_orders,
            total_orders,
            stats.synced_orders,
            stats.skipped_orders,
            stats.error_orders
        );

				info!("Füge Bestellung {} zu SYNCED_ORDERS hinzu", order.order_number);
				add_synced_order(app_handle, order.clone());

        sleep(Duration::from_millis(150)).await;
    }
    
    // Zusammenfassung
    info!("Synchronisation abgeschlossen: {} übertragen, {} übersprungen, {} Fehler", 
          stats.synced_orders, stats.skipped_orders, stats.error_orders);
    
    update_sync_stats(stats.clone());
		app_handle.emit("sync-stats-update", &stats)
        .map_err(|e| format!("Failed to emit event: {}", e))?;    

		// Emit final sync complete event
    app_handle.emit("sync-process-complete", &stats)
        .map_err(|e| format!("Failed to emit process complete event: {}", e))?;
    
		let _ = app_handle.emit("log", LogEntry {
        timestamp: Utc::now(),
        message: format!("Sync completed: {} synced, {} skipped, {} errors", 
            stats.synced_orders, stats.skipped_orders, stats.error_orders),
        level: "info".to_string(),
        category: "sync".to_string(),
    });

    Ok(stats)
}

/// Einzelne Bestellung verarbeiten
async fn process_order(
    client: &JtlApiClient, 
    joomla_conn: &mysql::Pool, 
    order: &crate::models::VirtueMartOrder, 
    config: &AppConfig
) -> Result<bool, String> {
    // Kundennummer erstellen
    let customer_number = match order.virtuemart_order_userinfo_id {
        Some(id) => format!("VM{}", id),
        None => {
            warn!("Keine virtuemart_order_userinfo_id für Bestellung {}", order.virtuemart_order_id);
            format!("VM{}", order.virtuemart_order_id)
        }
    };
    
    info!("Kundennummer aus Joomla: {}", customer_number);
    
    // Lieferadresse abrufen
    let shipping_address = get_shipping_address(joomla_conn, config, order.virtuemart_order_id)?;
    
    // JTL-Zahlungsmethode mappen
    let jtl_payment_method_id = map_payment_method(order.virtuemart_paymentmethod_id);
    
    // Bestellnummer
    let order_number = if !order.order_number.is_empty() {
        order.order_number.clone()
    } else {
        format!("VM{}", order.virtuemart_order_id)
    };
    
    // Prüfen, ob der Kunde bereits existiert
    match client.get_customer_by_id(&customer_number).await {
        Ok(maybe_customer) => {
            let customer_id = match maybe_customer {
                Some(customer) => {
                    info!("Kunde {} existiert bereits mit ID: {}", customer_number, customer["Id"]);
                    customer["Id"].as_str().unwrap_or("0").to_string()
                },
                None => {
                    // Neuen Kunden erstellen
                    info!("Erstelle neuen Kunden {}", customer_number);
                    
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
                            info!("Kunde erstellt mit ID: {}", response["Id"]);
                            response["Id"].to_string()
                        },
                        Err(e) => return Err(format!("Fehler beim Erstellen des Kunden: {}", e))
                    }
                }
            };
            
            // Prüfen, ob die Bestellung bereits existiert
            match client.check_order_exists(&order_number, &customer_id).await {
                Ok(exists) => {
                    if exists {
                        warn!("Bestellung {} existiert bereits, überspringe", order_number);
                        return Ok(false);
                    }
                },
                Err(e) => return Err(format!("Fehler beim Prüfen der Bestellung: {}", e))
            }
            
            // Bestellpositionen abrufen
            let items = match get_order_items(joomla_conn, config, order.virtuemart_order_id) {
                Ok(i) => i,
                Err(e) => return Err(format!("Fehler beim Abrufen der Bestellpositionen: {}", e))
            };
            
            info!("Bestellpositionen gefunden: {}", items.len());
            
            // JTL-Bestellung erstellen
            info!("Erstelle Bestellung {} in JTL", order_number);
            
            let billing_address = create_address_object(order);
            let shipping_addr = match &shipping_address {
                Some(addr) => create_address_object(addr),
                None => billing_address.clone(),
            };

						info!("CustomerId: {}", customer_id.clone());
						info!("ExternalNumber: {}", order_number.clone());
						info!("Coutrny: {} ID: {}", get_country_code(order.virtuemart_country_id.clone().unwrap_or_default()).unwrap_or_default().to_string(), order.virtuemart_country_id.clone().unwrap_or_default());

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
                Comment: order.customer_note.clone().unwrap_or_default(),
                LanguageIso: "DE".to_string(),
            };
            
            // Bestellpositionen für JTL vorbereiten
						let mut all_items: Vec<JtlOrderItem> = items.iter().map(|item| {
								JtlOrderItem {
										Quantity: item.product_quantity,
										SalesPriceGross: Some(item.product_final_price),
										TaxRate: 19.0,
										Name: item.order_item_name.clone(),
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
												Name: format!("Gutschein: {}", coupon_code),
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
												Name: "Versand".to_string(),
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
                    info!("Bestellung {} erfolgreich in JTL erstellt mit ID: {}", order_number, order_id);
                    
                    // Falls bereits bezahlt
                    if let Some(status) = &order.order_status {
                        if status == "C" && jtl_payment_method_id != 4 {
                            info!("Bestellung {} ist bezahlt -> setze auf bezahlt", order_number);
                            match client.set_payment_paid(&order_id).await {
                                Ok(_) => {},
                                Err(e) => warn!("Fehler beim Setzen des Zahlungsstatus: {}", e)
                            }
                        }
                    }
                    
                    Ok(true)
                },
                Err(e) => Err(format!("Fehler beim Erstellen der Bestellung: {}", e))
            }
        },
        Err(e) => Err(format!("Fehler beim Abrufen des Kunden: {}", e))
    }
}