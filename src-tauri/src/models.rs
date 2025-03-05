use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export database models
pub use crate::db::models::{
    DatabaseConfig, TablesConfig,
    VirtueMartOrder, VirtueMartOrderItem,
    JtlOrder, JtlOrderItem, JtlAddress, JtlCustomer,
    JtlCountry, JtlPaymentDetails, JtlShippingDetails
};

// Re-export sync stats
pub use crate::sync::stats::SyncStats;

// Log Entry structure for the frontend
#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: String,
    pub category: String,
    pub shop_id: Option<String>, // Optional shop_id to identify which shop this log belongs to
}