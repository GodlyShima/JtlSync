use mysql::{OptsBuilder, Pool, Error as MySqlError, params, prelude::Queryable, Row, Value};
use chrono::{Utc, Duration};
use log::{info, error};

use crate::models::{ShopConfig, VirtueMartOrder, VirtueMartOrderItem};

/// Verbindung zur Joomla-Datenbank herstellen
pub fn connect_to_joomla(shop: &ShopConfig) -> Result<Pool, MySqlError> {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(&shop.joomla.host))
        .user(Some(&shop.joomla.user))
        .pass(Some(&shop.joomla.password))
        .db_name(Some(&shop.joomla.database));
    
    Pool::new(opts)
}

/// Konvertiere MySQL-Datumswert in String
fn mysql_date_to_string(value: Value) -> String {
    match value {
        Value::Date(year, month, day, hour, min, sec, _) => {
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, min, sec)
        },
        _ => Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// Neue Bestellungen aus Joomla abrufen (letzte 24 Stunden)
pub fn get_new_orders(pool: &Pool, shop: &ShopConfig) -> Result<Vec<VirtueMartOrder>, String> {
    let now = Utc::now();
    let yesterday = now - Duration::days(1);
    let formatted_time = yesterday.format("%Y-%m-%d %H:%M:%S").to_string();
    
    info!("Suche Bestellungen seit: {} für Shop '{}'", formatted_time, shop.name);
    
    let query = format!(
        "SELECT o.*, c.*, 
         DATE_FORMAT(o.created_on, '%Y-%m-%d %H:%M:%S') as created_on_str 
         FROM {} o
         JOIN {} c ON o.virtuemart_order_id = c.virtuemart_order_id
         WHERE o.created_on >= ? AND c.address_type = 'BT'
         ORDER BY o.created_on DESC",
        shop.tables.orders, shop.tables.customers
    );
    
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Fehler beim Verbinden zur Datenbank für Shop '{}': {}", shop.name, e))?;
    
    let results = conn.exec_map(query, (formatted_time,), |row: Row| {
        // Hier müsstest du die Umwandlung von MySQL-Row zu VirtueMartOrder implementieren
        // Dies ist eine vereinfachte Implementierung
        let order_id: i32 = row.get("virtuemart_order_id").unwrap_or(0);
        let order_number: String = row.get("order_number").unwrap_or_else(|| format!("VM{}", order_id));
        
        // Holen des formatierten Datums als String
        let created_on: String = row.get("created_on_str").unwrap_or_else(|| {
            // Fallback: Wenn das formatierte Datum nicht verfügbar ist, versuche das Rohdatum
            let raw_date: Value = row.get("created_on").unwrap_or(Value::NULL);
            mysql_date_to_string(raw_date)
        });

        let phone_1: Option<String> = match row.get_opt::<String, _>("phone_1") {
            Some(Ok(value)) => Some(value),
            _ => None // Field doesn't exist or is NULL or has wrong type
        };
        
        let phone_2: Option<String> = match row.get_opt::<String, _>("phone_2") {
            Some(Ok(value)) => Some(value),
            _ => None // Field doesn't exist or is NULL or has wrong type
        };
                        
        VirtueMartOrder {
            virtuemart_order_id: order_id,
            order_number,
            created_on,
            order_total: row.get("order_total").unwrap_or(0.0),
            virtuemart_user_id: row.get("virtuemart_user_id"),
            order_status: row.get("order_status"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            phone_1,
            phone_2,
            address_1: row.get("address_1"),
            address_2: row.get("address_2"),
            zip: row.get("zip"),
            city: row.get("city"),
            virtuemart_country_id: row.get("virtuemart_country_id").unwrap_or(Some(81)),
            email: row.get("email"),
            virtuemart_paymentmethod_id: row.get("virtuemart_paymentmethod_id"),
            virtuemart_shipmentmethod_id: row.get("virtuemart_shipmentmethod_id"),
            virtuemart_order_userinfo_id: row.get("virtuemart_order_userinfo_id"),
            customer_note: row.get("customer_note").unwrap_or(Some(String::new())), 
            order_shipment: row.get("order_shipment"),
            coupon_code: row.get("coupon_code").unwrap_or(Some(String::new())),
            coupon_discount: row.get("coupon_discount").unwrap_or(Some(0.0)),
            company: row.get("company").unwrap_or(Some(String::new())),
            shop_id: Some(shop.id.clone()),
        }
    }).map_err(|e| format!("Fehler beim Abrufen der Bestellungen für Shop '{}': {}", shop.name, e))?;
    
    info!("Gefundene Bestellungen für Shop '{}': {}", shop.name, results.len());
    Ok(results)
}

/// Bestellpositionen für eine Bestellung abrufen
pub fn get_order_items(pool: &Pool, shop: &ShopConfig, order_id: i32) -> Result<Vec<VirtueMartOrderItem>, String> {
    info!("Hole Bestellpositionen für Bestellung {} in Shop '{}'", order_id, shop.name);
    
    let query = format!(
        "SELECT * FROM {} WHERE virtuemart_order_id = ?",
        shop.tables.orderItems
    );
    
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Fehler beim Verbinden zur Datenbank für Shop '{}': {}", shop.name, e))?;
    
    let results = conn.exec_map(query, (order_id,), |row: Row| {
        VirtueMartOrderItem {
            virtuemart_order_item_id: row.get("virtuemart_order_item_id").unwrap_or(0),
            virtuemart_order_id: row.get("virtuemart_order_id").unwrap_or(0),
            order_item_sku: row.get("order_item_sku"),
            order_item_name: row.get("order_item_name").unwrap_or_else(|| "Unbekanntes Produkt".to_string()),
            product_quantity: row.get("product_quantity").unwrap_or(1),
            product_final_price: row.get("product_final_price").unwrap_or(0.0),
            product_tax: row.get("product_tax"),
            product_priceWithoutTax: row.get("product_priceWithoutTax"),
        }
    }).map_err(|e| format!("Fehler beim Abrufen der Bestellpositionen für Shop '{}': {}", shop.name, e))?;
    
    info!("Bestellpositionen gefunden für Shop '{}': {}", shop.name, results.len());
    Ok(results)
}

/// Lieferadresse (ST) abrufen, falls vorhanden
pub fn get_shipping_address(pool: &Pool, shop: &ShopConfig, order_id: i32) -> Result<Option<VirtueMartOrder>, String> {
    info!("Prüfe Lieferadresse für Bestellung {} in Shop '{}'", order_id, shop.name);
    
    let query = format!(
        "SELECT * FROM {} WHERE virtuemart_order_id = ? AND address_type = 'ST'",
        shop.tables.customers
    );
    
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Fehler beim Verbinden zur Datenbank für Shop '{}': {}", shop.name, e))?;
    
    let results: Vec<VirtueMartOrder> = conn.exec_map(query, (order_id,), |row: Row| {
        VirtueMartOrder {
            virtuemart_order_id: row.get("virtuemart_order_id").unwrap_or(0),
            order_number: "".to_string(), // Wird für die Lieferadresse nicht benötigt
            created_on: "".to_string(),   // Wird für die Lieferadresse nicht benötigt
            order_total: 0.0,            // Wird für die Lieferadresse nicht benötigt
            virtuemart_user_id: None,     // Wird für die Lieferadresse nicht benötigt
            order_status: None,           // Wird für die Lieferadresse nicht benötigt
            first_name: row.get("first_name").unwrap_or(Some(String::new())),
            last_name: row.get("last_name").unwrap_or(Some(String::new())),
            phone_1: row.get("phone_1").unwrap_or(Some(String::new())),
            phone_2: row.get("phone_2").unwrap_or(Some(String::new())),
            address_1: row.get("address_1").unwrap_or(Some(String::new())),
            address_2: row.get("address_2").unwrap_or(Some(String::new())),
            zip: row.get("zip"),
            city: row.get("city"),
            virtuemart_country_id: row.get("virtuemart_country_id").unwrap_or(Some(81)),
            email: row.get("email").unwrap_or(Some(String::new())),
            virtuemart_paymentmethod_id: None, // Wird für die Lieferadresse nicht benötigt
            virtuemart_shipmentmethod_id: None, // Wird für die Lieferadresse nicht benötigt
            virtuemart_order_userinfo_id: row.get("virtuemart_order_userinfo_id"),
            customer_note: row.get("customer_note").unwrap_or(Some(String::new())), // Wird für die Lieferadresse nicht benötigt
            order_shipment: None,
            coupon_code: row.get("coupon_code").unwrap_or(Some(String::new())),
            coupon_discount: row.get("coupon_discount").unwrap_or(Some(0.0)),
            company: row.get("company").unwrap_or(Some(String::new())),
            shop_id: Some(shop.id.clone()),
        }
    }).map_err(|e| format!("Fehler beim Abrufen der Lieferadresse für Shop '{}': {}", shop.name, e))?;
    
    if results.is_empty() {
        info!("Keine separate Lieferadresse (ST) für Bestellung {} in Shop '{}' gefunden", order_id, shop.name);
        Ok(None)
    } else {
        info!("Separate Lieferadresse (ST) für Bestellung {} in Shop '{}' gefunden", order_id, shop.name);
        Ok(Some(results[0].clone()))
    }
}