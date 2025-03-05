use log::info;
use mysql::prelude::ToValue;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde_json::Value;
use std::time::Duration;

use crate::models::{JtlCustomer, JtlOrder, JtlOrderItem};

pub struct JtlApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl JtlApiClient {
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
    
    // Hilfsfunktion zum Erstellen der HTTP-Header
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&format!("Wawi {}", self.api_key)).unwrap());
        headers.insert("X-AppId", HeaderValue::from_static("syncWithJoomla/v2"));
        headers.insert("X-AppVersion", HeaderValue::from_static("2.0.0"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers
    }
    
    // Kunden anhand seiner ID abrufen
    pub async fn get_customer_by_id(&self, customer_id: &str) -> Result<Option<Value>, String> {
        let url = format!("{}/customers?searchKeyWord={}", self.base_url, customer_id);
        
        match self.client.get(&url)
            .headers(self.create_headers())
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => {
                                // Prüfen, ob Kunden gefunden wurden
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
                            },
                            Err(e) => Err(format!("Fehler beim Parsen der Antwort: {}", e))
                        }
                    } else {
                        Err(format!("Fehler beim Abrufen des Kunden: HTTP Status {}", status))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
    
    // Prüfen, ob eine Bestellung bereits existiert
    pub async fn check_order_exists(&self, order_number: &str, customer_id: &str) -> Result<bool, String> {
        let url = format!("{}/salesOrders?externalOrderNumber={}&customerId={}", 
                        self.base_url, order_number, customer_id);
        
        match self.client.get(&url)
            .headers(self.create_headers())
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => {
                                if let Some(total_items) = data["TotalItems"].as_i64() {
                                    Ok(total_items > 0)
                                } else {
                                    Ok(false)
                                }
                            },
                            Err(e) => Err(format!("Fehler beim Parsen der Antwort: {}", e))
                        }
                    } else {
                        Err(format!("Fehler beim Prüfen der Bestellung: HTTP Status {}", status))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
    
    // Neuen Kunden erstellen
    pub async fn create_customer(&self, customer: &JtlCustomer) -> Result<Value, String> {
        let url = format!("{}/customers", self.base_url);
        
        let customer_json = serde_json::to_string(&customer)
            .map_err(|e| format!("Fehler beim Serialisieren der Kundendaten: {}", e))?;
        
        match self.client.post(&url)
            .headers(self.create_headers())
            .body(customer_json)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => Ok(data),
                            Err(e) => Err(format!("Fehler beim Parsen der Antwort: {}", e))
                        }
                    } else {
                        // Fehlerdetails aus der Antwort extrahieren
                        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
                        Err(format!("Fehler beim Erstellen des Kunden: HTTP Status {} - {}", 
                                    status, error_text))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
    
    // Neue Bestellung erstellen
    pub async fn create_order(&self, order: &JtlOrder, items: &[JtlOrderItem]) -> Result<Value, String> {
        // Zuerst die Bestellung erstellen
        let url = format!("{}/salesOrders", self.base_url);
        
        let order_json = serde_json::to_string(&order)
            .map_err(|e| format!("Fehler beim Serialisieren der Bestelldaten: {}", e))?;
        
        match self.client.post(&url)
            .headers(self.create_headers())
            .body(order_json)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<Value>().await {
                            Ok(data) => {

															 info!("Bestellung: {}", data["Id"]);

															let order_id = match data["Id"].as_i64() {
																	Some(id) => id as i32,
																	None => return Err("Ungültige Bestellungs-ID".to_string())
															};
                        

                              match self.add_order_items(&order_id, items).await {
                                Ok(_) => Ok(data),
                                Err(e) => Err(e)
                            	}
                            },
                            Err(e) => Err(format!("Fehler beim Parsen der Antwort: {}", e))
                        }
                    } else {
                        // Fehlerdetails aus der Antwort extrahieren
                        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
                        Err(format!("Fehler beim Erstellen der Bestellung: HTTP Status {} - {}", 
                                    status, error_text))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
    
    // Bestellpositionen zu einer Bestellung hinzufügen
    async fn add_order_items(&self, order_id: &i32, items: &[JtlOrderItem]) -> Result<(), String> {
        let url = format!("{}/salesOrders/{}/lineitems", self.base_url, order_id);
        
        let items_json = serde_json::to_string(&items)
            .map_err(|e| format!("Fehler beim Serialisieren der Bestellpositionen: {}", e))?;
        
        match self.client.post(&url)
            .headers(self.create_headers())
            .body(items_json)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        info!("{} Bestellpositionen erfolgreich hinzugefügt", items.len());
                        Ok(())
                    } else {
                        // Fehlerdetails aus der Antwort extrahieren
                        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
                        Err(format!("Fehler beim Hinzufügen der Bestellpositionen: HTTP Status {} - {}", 
                                    status, error_text))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
    
    // Bestellung als bezahlt markieren
    pub async fn set_payment_paid(&self, order_id: &String) -> Result<(), String> {
        let url = format!("{}/salesOrders/{}/workflowEvents", self.base_url, order_id);
        
        // Bestellstatus "Bezahlt" (ID 15)
        let payload = r#"{"Id": 15}"#;
        
        match self.client.post(&url)
            .headers(self.create_headers())
            .body(payload)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        info!("Bestellung {} erfolgreich auf bezahlt gesetzt", order_id);
                        Ok(())
                    } else {
                        // Fehlerdetails aus der Antwort extrahieren
                        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
                        Err(format!("Fehler beim Setzen des Bestellstatus: HTTP Status {} - {}", 
                                    status, error_text))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }

		pub async fn set_order_hold(&self, order_id: &String) -> Result<(), String> {
        let url = format!("{}/salesOrders/{}/workflowEvents", self.base_url, order_id);
        
        // Bestellstatus "Bezahlt" (ID 15)
        let payload = r#"{"Id": 16}"#;
        
        match self.client.post(&url)
            .headers(self.create_headers())
            .body(payload)
            .send()
            .await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        info!("Bestellung {} erfolgreich auf bezahlt gesetzt", order_id);
                        Ok(())
                    } else {
                        // Fehlerdetails aus der Antwort extrahieren
                        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
                        Err(format!("Fehler beim Setzen des Bestellstatus: HTTP Status {} - {}", 
                                    status, error_text))
                    }
                },
                Err(e) => Err(format!("Fehler bei der Anfrage: {}", e))
            }
    }
}