use ac_parser::messages::ParsedMessage;
use ac_parser::PacketParser;
use std::fs::File;

/// Recursively search for a string within a JSON value (case-insensitive)
fn json_contains_string(value: &serde_json::Value, search: &str) -> bool {
    let search_lower = search.to_lowercase();
    match value {
        serde_json::Value::String(s) => s.to_lowercase().contains(&search_lower),
        serde_json::Value::Array(arr) => arr.iter().any(|v| json_contains_string(v, search)),
        serde_json::Value::Object(obj) => obj.values().any(|v| json_contains_string(v, search)),
        _ => false,
    }
}

/// Filter messages by search string (searches both message type and data)
fn filter_messages<'a>(
    messages: &'a [ParsedMessage],
    search: &str,
) -> Vec<&'a ParsedMessage> {
    messages
        .iter()
        .filter(|m| {
            // Search in message type
            let type_matches = m.message_type.to_lowercase().contains(&search.to_lowercase());
            // Search in message data (deep search)
            let data_matches = json_contains_string(&m.data, search);
            // Match if either type or data contains the search string
            type_matches || data_matches
        })
        .collect()
}

/// Load messages from the test PCAP file
fn load_test_messages() -> Vec<ParsedMessage> {
    let pcap_path = "pkt_2025-11-18_1763490291_log.pcap";
    let file = File::open(pcap_path).expect("Failed to open test PCAP file");

    let mut parser = PacketParser::new();
    let (_, messages) = parser.parse_pcap(file)
        .expect("Failed to parse PCAP file");

    messages
}

#[test]
fn test_filter_pantaloons() {
    let messages = load_test_messages();

    // Test case-insensitive search for "pantaloons"
    let filtered = filter_messages(&messages, "pantaloons");

    // Should find at least one message
    assert!(!filtered.is_empty(), "Filter 'pantaloons' should find at least one message");

    // Should find message id 9
    let found_msg_9 = filtered.iter().any(|m| m.id == 9);
    assert!(found_msg_9, "Filter 'pantaloons' should find message id 9");

    // Verify message 9 contains the search term in its data
    let msg_9 = messages.iter().find(|m| m.id == 9).expect("Message id 9 should exist");
    assert!(
        json_contains_string(&msg_9.data, "pantaloons"),
        "Message id 9 should contain 'pantaloons' in its data"
    );
}

#[test]
fn test_filter_pantaloons_case_insensitive() {
    let messages = load_test_messages();

    // Test with uppercase "Pantaloons"
    let filtered = filter_messages(&messages, "Pantaloons");

    // Should find message id 9
    let found_msg_9 = filtered.iter().any(|m| m.id == 9);
    assert!(found_msg_9, "Filter 'Pantaloons' (uppercase) should find message id 9");
}

#[test]
fn test_filter_haebrean_gauntlets() {
    let messages = load_test_messages();

    // Test search for "Haebrean Gauntlets"
    let filtered = filter_messages(&messages, "Haebrean Gauntlets");

    // Should find at least one message
    assert!(!filtered.is_empty(), "Filter 'Haebrean Gauntlets' should find at least one message");

    // Should find message id 2030
    let found_msg_2030 = filtered.iter().any(|m| m.id == 2030);
    assert!(found_msg_2030, "Filter 'Haebrean Gauntlets' should find message id 2030");

    // Verify message 2030 contains the search term in its data
    let msg_2030 = messages.iter().find(|m| m.id == 2030).expect("Message id 2030 should exist");
    assert!(
        json_contains_string(&msg_2030.data, "Haebrean Gauntlets"),
        "Message id 2030 should contain 'Haebrean Gauntlets' in its data"
    );
}

#[test]
fn test_filter_partial_match() {
    let messages = load_test_messages();

    // Test partial match "Haebrean" should also find message 2030
    let filtered = filter_messages(&messages, "Haebrean");

    let found_msg_2030 = filtered.iter().any(|m| m.id == 2030);
    assert!(found_msg_2030, "Filter 'Haebrean' should find message id 2030");
}

#[test]
fn test_filter_no_matches() {
    let messages = load_test_messages();

    // Search for something that doesn't exist
    let filtered = filter_messages(&messages, "ThisStringDoesNotExistInAnyMessage12345");

    assert!(filtered.is_empty(), "Filter for non-existent string should return no results");
}
