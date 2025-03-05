use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Updated AppConfig to support multiple shops
#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub shops: Vec<ShopConfig>,
    pub current_shop_index: usize,
    pub logFile: String,
    pub jtlApiPath: String, // For Backwards-Kompatibilität
}

// New ShopConfig to store per-shop configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct ShopConfig {
    pub id: String,
    pub name: String,
    pub joomla: DatabaseConfig,
    pub jtl: DatabaseConfig,
    pub tables: TablesConfig,
}

// Database configuration remains the same
#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TablesConfig {
    pub orders: String,
    pub orderItems: String,
    pub customers: String,
}

// Statistik-Struktur für das Dashboard - updated with sync_hours
#[derive(Serialize, Deserialize, Clone)]
pub struct SyncStats {
    pub shop_id: String,  // Added shop_id to identify which shop these stats belong to
    pub total_orders: i32,
    pub synced_orders: i32,
    pub skipped_orders: i32,
    pub error_orders: i32,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub next_scheduled_run: Option<DateTime<Utc>>,
    pub aborted: bool,
    pub sync_hours: i32,  // Added field for configurable timeframe
}

impl Default for SyncStats {
    fn default() -> Self {
        SyncStats {
            shop_id: String::new(),
            total_orders: 0,
            synced_orders: 0,
            skipped_orders: 0,
            error_orders: 0,
            last_sync_time: None,
            next_scheduled_run: None,
            aborted: false,
            sync_hours: 24, // Default to 24 hours
        }
    }
}

// Log-Eintrags-Struktur für das Frontend
#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: String,
    pub category: String,
    pub shop_id: Option<String>, // Optional shop_id to identify which shop this log belongs to
}

// VirtueMart-Bestellstruktur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtueMartOrder {
    pub virtuemart_order_id: i32,
    pub order_number: String,
    pub created_on: String,
    pub order_total: f64,
    pub company: Option<String>,
    pub virtuemart_user_id: Option<i32>,
    pub order_status: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_1: Option<String>,
    pub phone_2: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
    pub virtuemart_paymentmethod_id: Option<i32>,
    pub virtuemart_shipmentmethod_id: Option<i32>,
    pub virtuemart_order_userinfo_id: Option<i32>,
    pub customer_note: Option<String>,
    pub order_shipment: Option<f64>,
    pub coupon_code: Option<String>,
    pub coupon_discount: Option<f64>,
    pub virtuemart_country_id: Option<i32>,
    pub shop_id: Option<String>, // Added shop_id to track which shop this order belongs to
}

// VirtueMart-Bestellpositionsstruktur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtueMartOrderItem {
    pub virtuemart_order_item_id: i32,
    pub virtuemart_order_id: i32,
    pub order_item_sku: Option<String>,
    pub order_item_name: String,
    pub product_quantity: i32,
    pub product_final_price: f64,
    pub product_tax: Option<f64>,
    pub product_priceWithoutTax: Option<f64>,
}

// Adressobjekt für JTL API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlAddress {
    pub City: String,
    pub CountryIso: String,
    pub Company: String,
    pub FormOfAddress: String,
    pub Title: String,
    pub FirstName: String,
    pub LastName: String,
    pub Street: String,
    pub Address2: String,
    pub PostalCode: String,
    pub State: String,
    pub PhoneNumber: String,
    pub MobilePhoneNumber: String,
    pub EmailAddress: String,
    pub Fax: String,
}

// JTL Bestellposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlOrderItem {
    pub Quantity: i32,
    pub SalesPriceGross: Option<f64>,
    pub TaxRate: f64,
    pub Name: String,
    pub SalesUnit: String,
    pub SalesPriceNet: Option<f64>,
    pub PurchasePriceNet: Option<f64>,
}

// JTL Bestellung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlOrder {
    pub CustomerId: i32,
    pub ExternalNumber: String,
    pub CompanyId: i32,
    pub DepartureCountry: JtlCountry,
    pub BillingAddress: JtlAddress,
    pub Shipmentaddress: JtlAddress,
    pub SalesOrderDate: String,
    pub SalesOrderPaymentDetails: JtlPaymentDetails,
    pub SalesOrderShippingDetail: JtlShippingDetails,
    pub Comment: String,
    pub LanguageIso: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlCountry {
    pub CountryISO: String,
    pub CurrencyIso: String,
    pub CurrencyFactor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlPaymentDetails {
    pub PaymentMethodId: i32,
    pub CurrencyIso: String,
    pub CurrencyFactor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlShippingDetails {
    pub ShippingMethodId: i32,
    pub ShippingDate: String,
}

// JTL Kunde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtlCustomer {
    pub CustomerGroupId: i32,
    pub BillingAddress: JtlAddress,
    pub InternalCompanyId: i32,
    pub LanguageIso: String,
    pub Shipmentaddress: JtlAddress,
    pub CustomerSince: String,
    pub Number: String,
}