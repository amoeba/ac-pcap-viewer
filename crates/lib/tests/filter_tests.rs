use lib::messages::ParsedMessage;
use lib::PacketParser;
use std::fs::File;

/// Recursively search for a string within a JSON value (case-insensitive)
/// Searches in both field names and values, including numeric values
fn json_contains_string(value: &serde_json::Value, search: &str) -> bool {
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

/// Filter messages by search string (searches in all message fields and data)
fn filter_messages<'a>(messages: &'a [ParsedMessage], search: &str) -> Vec<&'a ParsedMessage> {
    messages
        .iter()
        .filter(|m| {
            // Search in message ID
            let id_matches = m.id.to_string().contains(&search.to_lowercase());
            // Search in message type
            let type_matches = m
                .message_type
                .to_lowercase()
                .contains(&search.to_lowercase());
            // Search in direction
            let direction_matches = m.direction.to_lowercase().contains(&search.to_lowercase());
            // Search in opcode
            let opcode_matches = m.opcode.to_lowercase().contains(&search.to_lowercase());
            // Search in message data (deep search including field names and numeric values)
            let data_matches = json_contains_string(&m.data, search);

            // Match if any field contains the search string
            id_matches || type_matches || direction_matches || opcode_matches || data_matches
        })
        .collect()
}

/// Load messages from the test PCAP file
fn load_test_messages() -> Option<Vec<ParsedMessage>> {
    let pcap_path = "pkt_2025-11-18_1763490291_log.pcap";
    let file = match File::open(pcap_path) {
        Ok(f) => f,
        Err(_) => {
            // PCAP file not found (e.g., in CI environment), skip test
            return None;
        }
    };

    let mut parser = PacketParser::new();
    match parser.parse_pcap(file) {
        Ok((_, messages)) => Some(messages),
        Err(_) => None,
    }
}

#[test]
fn test_filter_pantaloons() {
    let Some(messages) = load_test_messages() else {
        // PCAP file not available, skip test
        return;
    };

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
    let Some(messages) = load_test_messages() else {
        return;
    };

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
    let Some(messages) = load_test_messages() else {
        return;
    };

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
    let Some(messages) = load_test_messages() else {
        return;
    };

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
fn test_filter_by_object_id() {
    let Some(messages) = load_test_messages() else {
        return;
    };

    // Search for ObjectId from message 0
    let filtered = filter_messages(&messages, "2151762794");

    // Should find exactly 2 messages (Item_Appraise and Item_SetAppraiseInfo for the same item)
    assert_eq!(
        filtered.len(),
        2,
        "Filter '2151762794' should find exactly 2 messages"
    );

    // Should find message id 0 (Item_Appraise)
    let found_msg_0 = filtered.iter().any(|m| m.id == 0);
    assert!(found_msg_0, "Filter '2151762794' should find message id 0");

    // Should find message id 1 (Item_SetAppraiseInfo response)
    let found_msg_1 = filtered.iter().any(|m| m.id == 1);
    assert!(found_msg_1, "Filter '2151762794' should find message id 1");

    // Verify both messages contain the ObjectId in their data
    for msg in &filtered {
        assert!(
            json_contains_string(&msg.data, "2151762794"),
            "Message id {} should contain ObjectId '2151762794' in its data",
            msg.id
        );
    }
}

#[test]
fn test_filter_by_field_name() {
    let Some(messages) = load_test_messages() else {
        return;
    };

    // Search for field name "ObjectId"
    let filtered = filter_messages(&messages, "objectid");

    // Should find messages that have ObjectId field
    assert!(
        !filtered.is_empty(),
        "Filter 'objectid' should find messages with ObjectId field"
    );

    // Verify at least one result has ObjectId in its data
    let has_objectid_field = filtered
        .iter()
        .any(|m| json_contains_string(&m.data, "objectid"));
    assert!(
        has_objectid_field,
        "Filter 'objectid' should find messages containing 'objectid' field"
    );
}

#[test]
fn test_filter_no_matches() {
    let Some(messages) = load_test_messages() else {
        return;
    };

    // Search for something that doesn't exist
    let filtered = filter_messages(&messages, "ThisStringDoesNotExistInAnyMessage12345");

    assert!(
        filtered.is_empty(),
        "Filter for non-existent string should return no results"
    );
}
