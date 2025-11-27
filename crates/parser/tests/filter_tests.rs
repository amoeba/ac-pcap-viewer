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
fn filter_messages<'a>(messages: &'a [ParsedMessage], search: &str) -> Vec<&'a ParsedMessage> {
    messages
        .iter()
        .filter(|m| {
            // Search in message type
            let type_matches = m
                .message_type
                .to_lowercase()
                .contains(&search.to_lowercase());
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
    let (_, messages) = parser.parse_pcap(file).expect("Failed to parse PCAP file");

    messages
}

#[test]
fn test_filter_pantaloons() {
    let messages = load_test_messages();

    // Test case-insensitive search for "pantaloons"
    let filtered = filter_messages(&messages, "pantaloons");

    // Should find at least one message
    assert!(
        !filtered.is_empty(),
        "Filter 'pantaloons' should find at least one message"
    );

    // Verify that at least one filtered message contains the search term
    let has_pantaloons = filtered
        .iter()
        .any(|m| json_contains_string(&m.data, "pantaloons"));
    assert!(
        has_pantaloons,
        "Filter 'pantaloons' should find messages containing 'pantaloons'"
    );
}

#[test]
fn test_filter_pantaloons_case_insensitive() {
    let messages = load_test_messages();

    // Test with uppercase "Pantaloons"
    let filtered = filter_messages(&messages, "Pantaloons");

    // Should find at least one message
    assert!(
        !filtered.is_empty(),
        "Filter 'Pantaloons' should find at least one message"
    );

    // Verify that at least one filtered message contains the search term (case insensitive)
    let has_pantaloons = filtered
        .iter()
        .any(|m| json_contains_string(&m.data, "pantaloons"));
    assert!(
        has_pantaloons,
        "Filter 'Pantaloons' should find messages containing 'pantaloons'"
    );
}

#[test]
fn test_filter_haebrean_gauntlets() {
    let messages = load_test_messages();

    // Test search for "Haebrean Gauntlets"
    let filtered = filter_messages(&messages, "Haebrean Gauntlets");

    // Should find at least one message
    assert!(
        !filtered.is_empty(),
        "Filter 'Haebrean Gauntlets' should find at least one message"
    );

    // Verify that at least one filtered message contains the search term
    let has_gauntlets = filtered
        .iter()
        .any(|m| json_contains_string(&m.data, "Haebrean Gauntlets"));
    assert!(
        has_gauntlets,
        "Filter 'Haebrean Gauntlets' should find messages containing 'Haebrean Gauntlets'"
    );
}

#[test]
fn test_filter_partial_match() {
    let messages = load_test_messages();

    // Test partial match "Haebrean" should find messages containing "Haebrean Gauntlets"
    let filtered = filter_messages(&messages, "Haebrean");

    // Should find at least one message
    assert!(
        !filtered.is_empty(),
        "Filter 'Haebrean' should find at least one message"
    );

    // Verify that at least one filtered message contains "Haebrean Gauntlets"
    let has_gauntlets = filtered
        .iter()
        .any(|m| json_contains_string(&m.data, "Haebrean Gauntlets"));
    assert!(
        has_gauntlets,
        "Filter 'Haebrean' should find messages containing 'Haebrean Gauntlets'"
    );
}

#[test]
fn test_filter_no_matches() {
    let messages = load_test_messages();

    // Search for something that doesn't exist
    let filtered = filter_messages(&messages, "ThisStringDoesNotExistInAnyMessage12345");

    assert!(
        filtered.is_empty(),
        "Filter for non-existent string should return no results"
    );
}
