pub mod connection;
pub mod models;
pub mod joomla;

// Re-export commonly used types and functions
pub use connection::ConnectionManager;
pub use models::{VirtueMartOrder, VirtueMartOrderItem, JtlOrder, JtlCustomer};