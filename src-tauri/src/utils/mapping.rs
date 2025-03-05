use std::collections::HashMap;
use lazy_static::lazy_static;
use log::info;

use crate::db::models::{VirtueMartOrder, JtlAddress};

// Default value for unknown payment methods
const DEFAULT_PAYMENT_METHOD_ID: i32 = 20;

// Mapping from VirtueMart payment methods to JTL payment methods
lazy_static! {
    static ref PAYMENT_METHOD_MAPPING: HashMap<i32, i32> = {
        let mut map = HashMap::new();
        map.insert(2, 38);  // Joomla: Giropay -> JTL: Giropay
        map.insert(14, 4);  // Joomla: Klarna -> JTL: Credit Card
        map.insert(4, 2);   // Joomla: Prepayment/Bank Transfer -> JTL: Bank Transfer
        map.insert(5, 4);   // Joomla: MasterCard/VISA -> JTL: Credit Card
        map.insert(6, 39);  // Joomla: Sofortüberweisung.de -> JTL: Sofortüberweisung
        map.insert(8, 27);  // Joomla: Cash on Pickup -> JTL: Cash
        map.insert(9, 9);   // Joomla: PayPal Express -> JTL: PayPal-Express
        map.insert(10, 34); // Joomla: Amazon Pay -> JTL: Amazon Pay Checkout
        map.insert(17, 10); // Joomla: PayPal Plus -> JTL: PayPal-Plus
        map
    };
}

// Country code mapping
lazy_static! {
    static ref COUNTRY_MAP: HashMap<i32, &'static str> = {
        let mut map = HashMap::new();
        map.insert(81, "DE"); // Germany
        map.insert(14, "AT"); // Austria
        map.insert(204, "CH"); // Switzerland
        map.insert(21, "BE"); // Belgium
        map.insert(150, "NL"); // Netherlands
        map.insert(105, "IT"); // Italy
        map.insert(73, "FR"); // France
        map.insert(195, "ES"); // Spain
        map.insert(222, "GB"); // United Kingdom
        // Add more countries as needed or implement a full map
        map
    };
}

/// Get country code from country ID
pub fn get_country_code(id: i32) -> Option<&'static str> {
    COUNTRY_MAP.get(&id).copied()
}

/// Map VirtueMart payment method to JTL payment method
pub fn map_payment_method(payment_method_id: Option<i32>) -> i32 {
    match payment_method_id {
        Some(id) => {
            match PAYMENT_METHOD_MAPPING.get(&id) {
                Some(&jtl_id) => jtl_id,
                None => {
                    info!("Unknown payment method ID: {}, using default: {}", 
                          id, DEFAULT_PAYMENT_METHOD_ID);
                    DEFAULT_PAYMENT_METHOD_ID
                }
            }
        },
        None => {
            info!("No payment method ID provided, using default: {}", 
                  DEFAULT_PAYMENT_METHOD_ID);
            DEFAULT_PAYMENT_METHOD_ID
        }
    }
}

/// Create a JTL address object from a VirtueMart address
pub fn create_address_object(address_data: &VirtueMartOrder) -> JtlAddress {
    JtlAddress {
        City: address_data.city.clone().unwrap_or_default(),
        CountryIso: get_country_code(address_data.virtuemart_country_id.unwrap_or_default()).unwrap_or("DE").to_string(),
        Company: address_data.company.clone().unwrap_or_default(),
        FormOfAddress: String::new(),
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
        PhoneNumber: address_data.phone_1.clone().unwrap_or_default(),
        MobilePhoneNumber: address_data.phone_2.clone().unwrap_or_default(),
        EmailAddress: address_data.email.clone().unwrap_or_default(),
        Fax: String::new(),
    }
}