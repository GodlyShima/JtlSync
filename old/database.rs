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

/// Neue Bestellungen innerhalb eines konfigurierbaren Zeitraums abrufen
pub fn get_orders_within_timeframe(pool: &Pool, shop: &ShopConfig, hours: i32) -> Result<Vec<VirtueMartOrder>, String> {
    let now = Utc::now();
    let past_time = now - Duration::hours(hours as i64);
    let formatted_time = past_time.format("%Y-%m-%d %H:%M:%S").to_string();
    
    info!("Searching orders since: {} ({}h timeframe) for Shop '{}'", formatted_time, hours, shop.name);
    
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
        .map_err(|e| format!("Error connecting to database for shop '{}': {}", shop.name, e))?;
    
    let results = conn.exec_map(query, (formatted_time,), |row: Row| {
        // Converting MySQL Row to VirtueMartOrder
        let order_id: i32 = row.get("virtuemart_order_id").unwrap_or(0);
        let order_number: String = row.get("order_number").unwrap_or_else(|| format!("VM{}", order_id));
        
        // Get formatted date as string
        let created_on: String = row.get("created_on_str").unwrap_or_else(|| {
            // Fallback: If formatted date not available, try raw date
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
    }).map_err(|e| format!("Error fetching orders for shop '{}': {}", shop.name, e))?;
    
    info!("Found {} orders for shop '{}'", results.len(), shop.name);
    Ok(results)
}

/// Bestellpositionen für eine Bestellung abrufen
pub fn get_order_items(pool: &Pool, shop: &ShopConfig, order_id: i32) -> Result<Vec<VirtueMartOrderItem>, String> {
    info!("Fetching order items for order {} in Shop '{}'", order_id, shop.name);
    
    let query = format!(
        "SELECT * FROM {} WHERE virtuemart_order_id = ?",
        shop.tables.orderItems
    );
    
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Error connecting to database for shop '{}': {}", shop.name, e))?;
    
    let results = conn.exec_map(query, (order_id,), |row: Row| {
        VirtueMartOrderItem {
            virtuemart_order_item_id: row.get("virtuemart_order_item_id").unwrap_or(0),
            virtuemart_order_id: row.get("virtuemart_order_id").unwrap_or(0),
            order_item_sku: row.get("order_item_sku"),
            order_item_name: row.get("order_item_name").unwrap_or_else(|| "Unknown Product".to_string()),
            product_quantity: row.get("product_quantity").unwrap_or(1),
            product_final_price: row.get("product_final_price").unwrap_or(0.0),
            product_tax: row.get("product_tax"),
            product_priceWithoutTax: row.get("product_priceWithoutTax"),
        }
    }).map_err(|e| format!("Error fetching order items for shop '{}': {}", shop.name, e))?;
    
    info!("Found {} order items for shop '{}'", results.len(), shop.name);
    Ok(results)
}

/// Lieferadresse (ST) abrufen, falls vorhanden
pub fn get_shipping_address(pool: &Pool, shop: &ShopConfig, order_id: i32) -> Result<Option<VirtueMartOrder>, String> {
    info!("Checking shipping address for order {} in Shop '{}'", order_id, shop.name);
    
    let query = format!(
        "SELECT * FROM {} WHERE virtuemart_order_id = ? AND address_type = 'ST'",
        shop.tables.customers
    );
    
    let mut conn = pool.get_conn()
        .map_err(|e| format!("Error connecting to database for shop '{}': {}", shop.name, e))?;
    
    let results: Vec<VirtueMartOrder> = conn.exec_map(query, (order_id,), |row: Row| {
        VirtueMartOrder {
            virtuemart_order_id: row.get("virtuemart_order_id").unwrap_or(0),
            order_number: "".to_string(), // Not needed for shipping address
            created_on: "".to_string(),   // Not needed for shipping address
            order_total: 0.0,             // Not needed for shipping address
            virtuemart_user_id: None,     // Not needed for shipping address
            order_status: None,           // Not needed for shipping address
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            phone_1: row.get("phone_1"),
            phone_2: row.get("phone_2"),
            address_1: row.get("address_1"),
            address_2: row.get("address_2"),
            zip: row.get("zip"),
            city: row.get("city"),
            virtuemart_country_id: row.get("virtuemart_country_id").unwrap_or(Some(81)),
            email: row.get("email"),
            virtuemart_paymentmethod_id: None, // Not needed for shipping address
            virtuemart_shipmentmethod_id: None, // Not needed for shipping address
            virtuemart_order_userinfo_id: row.get("virtuemart_order_userinfo_id"),
            customer_note: None, // Not needed for shipping address
            order_shipment: None,
            coupon_code: None,
            coupon_discount: None,
            company: row.get("company"),
            shop_id: Some(shop.id.clone()),
        }
    }).map_err(|e| format!("Error fetching shipping address for shop '{}': {}", shop.name, e))?;
    
    if results.is_empty() {
        info!("No separate shipping address (ST) found for order {} in shop '{}'", order_id, shop.name);
        Ok(None)
    } else {
        info!("Separate shipping address (ST) found for order {} in shop '{}'", order_id, shop.name);
        Ok(Some(results[0].clone()))
    }
}