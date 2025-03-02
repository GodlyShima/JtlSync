use std::collections::HashMap;
use log::{info};
use chrono::Utc;
use tauri::Emitter;

use crate::models::{VirtueMartOrder, JtlAddress};

// Standardwert für unbekannte Zahlungsmethoden
const DEFAULT_PAYMENT_METHOD_ID: i32 = 20;

// Mapping von VirtueMart-Zahlungsmethoden zu JTL-Zahlungsmethoden
lazy_static::lazy_static! {
    static ref PAYMENT_METHOD_MAPPING: HashMap<i32, i32> = {
        let mut map = HashMap::new();
        map.insert(2, 38);  // Joomla: Giropay -> JTL: Giropay
        map.insert(14, 4);  // Joomla: Klarna -> JTL: Kreditkarte
        map.insert(4, 2);   // Joomla: Vorauszahlung/Überweisung -> JTL: Überweisung
        map.insert(5, 4);   // Joomla: MasterCard/VISA -> JTL: Kreditkarte
        map.insert(6, 39);  // Joomla: Sofortüberweisung.de -> JTL: Sofortüberweisung
        map.insert(8, 27);  // Joomla: Barzahlung bei Abholung vor Ort -> JTL: Barzahlung
        map.insert(9, 9);   // Joomla: PayPal Express -> JTL: PayPal-Express
        map.insert(10, 34); // Joomla: Amazon Pay -> JTL: Amazon Pay Checkout
        map.insert(17, 10); // Joomla: PayPal Plus -> JTL: PayPal-Plus
        map
    };
}

pub fn emit_event<R: tauri::Runtime, T: serde::Serialize + Clone>(
    app_handle: &tauri::AppHandle<R>,
    event: &str, 
    payload: T
) -> Result<(), String> {
    app_handle
        .emit(event, payload)
        .map_err(|e| format!("Failed to emit event: {}", e))
}


/// Mappt die VirtueMart-Zahlungsmethode auf die JTL-Zahlungsmethode
pub fn map_payment_method(payment_method_id: Option<i32>) -> i32 {
    match payment_method_id {
        Some(id) => {
            match PAYMENT_METHOD_MAPPING.get(&id) {
                Some(&jtl_id) => jtl_id,
                None => {
                    info!("Unbekannte Zahlungsmethoden-ID: {}, verwende Standard: {}", 
                          id, DEFAULT_PAYMENT_METHOD_ID);
                    DEFAULT_PAYMENT_METHOD_ID
                }
            }
        },
        None => {
            info!("Keine Zahlungsmethoden-ID angegeben, verwende Standard: {}", 
                  DEFAULT_PAYMENT_METHOD_ID);
            DEFAULT_PAYMENT_METHOD_ID
        }
    }
}

/// Erzeugt ein JTL-Adressobjekt aus einem VirtueMart-Adressobjekt
pub fn create_address_object(address_data: &VirtueMartOrder) -> JtlAddress {
    JtlAddress {
        City: address_data.city.clone().unwrap_or_default(),
        CountryIso: address_data.country_2_code.clone().unwrap_or_else(|| "DE".to_string()),
        Company: String::new(), // VirtueMart-Modell hat kein Unternehmen in deinem Beispiel
        FormOfAddress: String::new(), // VirtueMart-Modell hat keine Anrede in deinem Beispiel
        Title: String::new(),
        FirstName: address_data.first_name.clone().unwrap_or_default(),
        LastName: address_data.last_name.clone().unwrap_or_default(),
        Street: format!("{}{}", 
            address_data.address_1.clone().unwrap_or_default(),
            address_data.address_2.clone().map_or("".to_string(), |a| format!(" {}", a))
        ),
        Address2: String::new(),
        PostalCode: address_data.zip.clone().unwrap_or_default(),
        State: String::new(),
        PhoneNumber: address_data.phone_1.clone().unwrap_or_default(), // Füge Phone-Feld hinzu, wenn verfügbar
        MobilePhoneNumber: String::new(),
        EmailAddress: address_data.email.clone().unwrap_or_default(),
        Fax: String::new(),
    }
}

/// Erzeugt eine Timestamp für Logs
pub fn get_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Hilfsfunktion zum Parsen von Float-Werten aus Strings
pub fn parse_float(value: Option<&str>) -> f64 {
    match value {
        Some(val) => val.parse::<f64>().unwrap_or(0.0),
        None => 0.0,
    }
}

pub fn format_iso_date(date_str: &str) -> String {
    // Try to parse the input date format
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        // Convert to DateTime<Utc> and format as ISO 8601
        return chrono::DateTime::<Utc>::from_utc(dt, Utc).to_rfc3339();
    }
    
    // If parsing fails, return a default date format
    Utc::now().to_rfc3339()
}