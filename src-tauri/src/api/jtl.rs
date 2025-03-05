use log::info;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde_json::Value;
use std::time::Duration;

use crate::error::{Result, Error};
use crate::db::models::{JtlCustomer, JtlOrder, JtlOrderItem};

/// JTL API client for interacting with the JTL-Wawi API
pub struct JtlApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl JtlApiClient {
    /// Create a new JTL API client with the given API key
    pub fn new(api_key: &str) -> Self {
        let base_url = "http://127.0.0.1:5883/api/eazybusiness/v1".to_string();
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        JtlApiClient {
            client,
            base_url,
            api_key: api_key.to_string(),
        }
    }
    
    /// Create HTTP headers for API requests
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&format!("Wawi {}", self.api_key)).unwrap());
        headers.insert("X-AppId", HeaderValue::from_static("syncWithJoomla/v2"));
        headers.insert("X-AppVersion", HeaderValue::from_static("2.0.0"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers
    }
    
    /// Get a customer by their ID
    pub async fn get_customer_by_id(&self, customer_id: &str) -> Result<Option<Value>> {
        let url = format!("{}/customers?searchKeyWord={}", self.base_url, customer_id);
        
        let response = self.client.get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            let data = response.json::<Value>().await
                .map_err(|e| Error::Api(format!("Response parsing error: {}", e)))?;
                
            // Check if customers were found
            if let Some(total_items) = data["TotalItems"].as_i64() {
                if total_items > 0 {
                    if let Some(items) = data["Items"].as_array() {
                        if !items.is_empty() {
                            return Ok(Some(items[0].clone()));
                        }
                    }
                }
            }
            Ok(None)
        } else {
            Err(Error::Api(format!("HTTP error: {}", status)))
        }
    }
    
    /// Check if an order already exists
    pub async fn check_order_exists(&self, order_number: &str, customer_id: &str) -> Result<bool> {
        let url = format!("{}/salesOrders?externalOrderNumber={}&customerId={}", 
                         self.base_url, order_number, customer_id);
        
        let response = self.client.get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            let data = response.json::<Value>().await
                .map_err(|e| Error::Api(format!("Response parsing error: {}", e)))?;
                
            if let Some(total_items) = data["TotalItems"].as_i64() {
                Ok(total_items > 0)
            } else {
                Ok(false)
            }
        } else {
            Err(Error::Api(format!("HTTP error: {}", status)))
        }
    }
    
    /// Create a new customer
    pub async fn create_customer(&self, customer: &JtlCustomer) -> Result<Value> {
        let url = format!("{}/customers", self.base_url);
        
        let customer_json = serde_json::to_string(customer)
            .map_err(|e| Error::Api(format!("Serialization error: {}", e)))?;
        
        let response = self.client.post(&url)
            .headers(self.create_headers())
            .body(customer_json)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            let data = response.json::<Value>().await
                .map_err(|e| Error::Api(format!("Response parsing error: {}", e)))?;
                
            Ok(data)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(format!("HTTP error {}: {}", status, error_text)))
        }
    }
    
    /// Create a new order with items
    pub async fn create_order(&self, order: &JtlOrder, items: &[JtlOrderItem]) -> Result<Value> {
        // First create the order
        let url = format!("{}/salesOrders", self.base_url);
        
        let order_json = serde_json::to_string(order)
            .map_err(|e| Error::Api(format!("Serialization error: {}", e)))?;
        
        let response = self.client.post(&url)
            .headers(self.create_headers())
            .body(order_json)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            let data = response.json::<Value>().await
                .map_err(|e| Error::Api(format!("Response parsing error: {}", e)))?;

            info!("Order: {}", data["Id"]);

            let order_id = match data["Id"].as_i64() {
                Some(id) => id as i32,
                None => return Err(Error::Api("Invalid order ID".to_string()))
            };
            
            // Add order items
            self.add_order_items(&order_id, items).await?;
            
            Ok(data)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(format!("HTTP error {}: {}", status, error_text)))
        }
    }
    
    /// Add items to an order
    async fn add_order_items(&self, order_id: &i32, items: &[JtlOrderItem]) -> Result<()> {
        let url = format!("{}/salesOrders/{}/lineitems", self.base_url, order_id);
        
        let items_json = serde_json::to_string(items)
            .map_err(|e| Error::Api(format!("Serialization error: {}", e)))?;
        
        let response = self.client.post(&url)
            .headers(self.create_headers())
            .body(items_json)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            info!("{} order items successfully added", items.len());
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(format!("HTTP error {}: {}", status, error_text)))
        }
    }
    
    /// Mark an order as paid
    pub async fn set_payment_paid(&self, order_id: &String) -> Result<()> {
        let url = format!("{}/salesOrders/{}/workflowEvents", self.base_url, order_id);
        
        // Order status "Paid" (ID 15)
        let payload = r#"{"Id": 15}"#;
        
        let response = self.client.post(&url)
            .headers(self.create_headers())
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            info!("Order {} successfully marked as paid", order_id);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(format!("HTTP error {}: {}", status, error_text)))
        }
    }

    /// Set order on hold
    pub async fn set_order_hold(&self, order_id: &String) -> Result<()> {
        let url = format!("{}/salesOrders/{}/workflowEvents", self.base_url, order_id);
        
        // Order status "On Hold" (ID 16)
        let payload = r#"{"Id": 16}"#;
        
        let response = self.client.post(&url)
            .headers(self.create_headers())
            .body(payload)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Request error: {}", e)))?;
            
        let status = response.status();
        if status.is_success() {
            info!("Order {} successfully put on hold", order_id);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(format!("HTTP error {}: {}", status, error_text)))
        }
    }
}