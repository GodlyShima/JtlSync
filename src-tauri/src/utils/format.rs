use chrono::Utc;

/// Format a date string into ISO 8601 format
pub fn format_iso_date(date_str: &str) -> String {
    // Try to parse the input date format
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        // Convert to DateTime<Utc> and format as ISO 8601
        return chrono::DateTime::<Utc>::from_utc(dt, Utc).to_rfc3339();
    }
    
    // If parsing fails, return the current time in ISO format
    Utc::now().to_rfc3339()
}

/// Generate a timestamp for logs
pub fn get_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Parse a float from an optional string
pub fn parse_float(value: Option<&str>) -> f64 {
    match value {
        Some(val) => val.parse::<f64>().unwrap_or(0.0),
        None => 0.0,
    }
}