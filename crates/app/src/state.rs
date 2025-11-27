use std::sync::{Arc, Mutex};

// Re-export shared enums from lib crate
// TODO: Re-enable these imports when needed for web UI
// pub use ac_pcap_lib::{Tab, ViewMode, SortField};

// Responsive breakpoints
pub const MOBILE_BREAKPOINT: f32 = 768.0;
pub const TABLET_BREAKPOINT: f32 = 1024.0;

// Mobile UI scaling factor
pub const MOBILE_SCALE: f32 = 1.5;

// Shared state for async loading
#[allow(dead_code)]
pub type SharedData = Arc<Mutex<Option<Vec<u8>>>>;
#[allow(dead_code)]
pub type SharedError = Arc<Mutex<Option<String>>>;

/// Recursively search for a string within a JSON value (case-insensitive)
/// Searches in both field names and values, including numeric values
pub fn json_contains_string(value: &serde_json::Value, search: &str) -> bool {
    let search_lower = search.to_lowercase();

    match value {
        serde_json::Value::String(s) => s.to_lowercase().contains(&search_lower),
        serde_json::Value::Number(n) => {
            // Check if the search string matches the numeric value as a string
            n.to_string().contains(&search_lower) ||
            // Also check if search is a number and matches exactly
            if let Ok(search_num) = search.parse::<i64>() {
                n.as_i64() == Some(search_num)
            } else if let Ok(search_num) = search.parse::<u64>() {
                n.as_u64() == Some(search_num)
            } else if let Ok(search_num) = search.parse::<f64>() {
                n.as_f64() == Some(search_num)
            } else {
                false
            }
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| json_contains_string(v, search)),
        serde_json::Value::Object(obj) => {
            // Search in both keys and values
            obj.keys().any(|k| k.to_lowercase().contains(&search_lower))
                || obj.values().any(|v| json_contains_string(v, search))
        }
        _ => false,
    }
}
