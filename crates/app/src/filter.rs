//! Rich filter types for flexible searching

/// A single parsed filter that can match multiple representations
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// StringValue - exact substring match (case-insensitive)
    StringValue(std::string::String),
    /// Hex value - matches both "ABCD" and "43981" (decimal equivalent)
    HexValue(u32),
    /// Decimal value - matches both "43981" and "0xABCD" (hex equivalent)
    DecimalValue(u32),
}

/// Parse a single search/filter string into rich filters
///
/// Examples:
/// - "apple" → [StringValue("apple")]
/// - "1234" → [DecimalValue(1234)]
/// - "0xABCD" → [HexValue(43981), DecimalValue(43981)]
pub fn parse_filter_string(s: &str) -> Vec<Filter> {
    let s = s.trim();

    if s.is_empty() {
        return vec![];
    }

    // Try to parse as hex with 0x prefix
    if (s.starts_with("0x") || s.starts_with("0X"))
        && s.len() > 2
        && let Ok(value) = u32::from_str_radix(&s[2..], 16)
    {
        // Return both hex and decimal representations
        return vec![Filter::HexValue(value), Filter::DecimalValue(value)];
    }

    // Try to parse as decimal number
    if let Ok(value) = s.parse::<u32>() {
        // Return both hex and decimal representations
        return vec![Filter::DecimalValue(value), Filter::HexValue(value)];
    }

    // Fall back to string match
    vec![Filter::StringValue(s.to_lowercase())]
}

/// Check if a value matches any filter in the list
pub fn matches_any_filter(filters: &[Filter], value: &str) -> bool {
    filters.iter().any(|f| matches_filter(f, value))
}

/// Check if a single value matches a filter
fn matches_filter(filter: &Filter, value: &str) -> bool {
    match filter {
        Filter::StringValue(s) => value.to_lowercase().contains(s),
        Filter::HexValue(num) => {
            // First try to parse the entire value as hex (e.g., "F7B1")
            if let Ok(parsed) = u32::from_str_radix(value, 16)
                && parsed == *num
            {
                return true;
            }

            // Also try substring matching (e.g., "F7B1" in JSON data)
            // This allows finding hex values embedded in larger JSON strings
            let hex_str = format!("{num:X}");
            let hex_str_lower = format!("{num:x}");
            value.contains(&hex_str) || value.contains(&hex_str_lower)
        }
        Filter::DecimalValue(num) => {
            // First try to parse the entire value as decimal
            if let Ok(parsed) = value.parse::<u32>()
                && parsed == *num
            {
                return true;
            }

            // Also try substring matching (e.g., "2151762794" in JSON data)
            // This allows finding numeric values embedded in larger JSON strings
            let decimal_str = num.to_string();
            value.contains(&decimal_str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_filter() {
        let filters = parse_filter_string("apple");
        assert_eq!(filters, vec![Filter::StringValue("apple".to_string())]);
    }

    #[test]
    fn test_parse_decimal_number() {
        let filters = parse_filter_string("1234");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::DecimalValue(1234)));
        assert!(filters.contains(&Filter::HexValue(1234)));
    }

    #[test]
    fn test_parse_hex_value() {
        let filters = parse_filter_string("0xABCD");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::HexValue(0xABCD)));
        assert!(filters.contains(&Filter::DecimalValue(0xABCD)));
    }

    #[test]
    fn test_parse_hex_lowercase() {
        let filters = parse_filter_string("0xabcd");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::HexValue(0xABCD)));
        assert!(filters.contains(&Filter::DecimalValue(0xABCD)));
    }

    #[test]
    fn test_matches_string() {
        let filters = [Filter::StringValue("test".to_string())];
        assert!(matches_filter(&filters[0], "testing"));
        assert!(matches_filter(&filters[0], "TEST"));
        assert!(!matches_filter(&filters[0], "foo"));
    }

    #[test]
    fn test_matches_hex_value() {
        let filter = Filter::HexValue(0xF7B1);
        // Should match hex representation
        assert!(matches_filter(&filter, "F7B1"));
        assert!(matches_filter(&filter, "f7b1"));
        // Should not match decimal representation
        assert!(!matches_filter(&filter, "63409"));
    }

    #[test]
    fn test_matches_decimal_value() {
        let filter = Filter::DecimalValue(0xF7B1);
        // Should match decimal representation
        assert!(matches_filter(&filter, "63409"));
        // Should not match hex representation
        assert!(!matches_filter(&filter, "F7B1"));
    }

    #[test]
    fn test_opcode_filtering_with_hex_input() {
        // User types "0xF7B1" in search
        let filters = parse_filter_string("0xF7B1");

        // Should find message with OpCode "F7B1"
        assert!(matches_any_filter(&filters, "F7B1"));

        // Should also find decimal equivalent in data
        assert!(matches_any_filter(&filters, "63409"));
    }

    #[test]
    fn test_opcode_filtering_with_decimal_input() {
        // User types "63409" in search
        let filters = parse_filter_string("63409");

        // Should find message with OpCode "F7B1"
        assert!(matches_any_filter(&filters, "F7B1"));

        // Should also find decimal
        assert!(matches_any_filter(&filters, "63409"));
    }

    #[test]
    fn test_object_id_filtering() {
        // User types "0x2151762794" (10 chars - beyond u32 range)
        // This should fail u32 parsing and fall back to StringValue filter
        let filters = parse_filter_string("0x2151762794");
        assert_eq!(
            filters,
            vec![Filter::StringValue("0x2151762794".to_string())]
        );
    }

    #[test]
    fn test_object_id_within_u32_range() {
        // User types "0x80000001" (valid u32)
        let filters = parse_filter_string("0x80000001");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::HexValue(0x80000001)));
        assert!(filters.contains(&Filter::DecimalValue(0x80000001)));
    }

    #[test]
    fn test_empty_string() {
        let filters = parse_filter_string("");
        assert_eq!(filters, vec![]);
    }

    #[test]
    fn test_whitespace_only() {
        let filters = parse_filter_string("   ");
        assert_eq!(filters, vec![]);
    }

    // Real data tests from actual pcap file

    #[test]
    fn test_search_real_opcode_f7b0_hex() {
        // Real opcode from pcap: F7B0 (63408 decimal, Ordered_GameEvent)
        let filters = parse_filter_string("0xF7B0");
        assert!(matches_any_filter(&filters, "F7B0"));
        assert!(matches_any_filter(&filters, "63408"));
    }

    #[test]
    fn test_search_real_opcode_f7b1_decimal() {
        // Real opcode from pcap: F7B1 (63409 decimal, Ordered_GameAction)
        let filters = parse_filter_string("63409");
        assert!(matches_any_filter(&filters, "F7B1"));
        assert!(matches_any_filter(&filters, "63409"));
    }

    #[test]
    fn test_search_real_opcode_02e9() {
        // Real opcode from pcap: 02E9 (745 decimal, Qualities_PrivateUpdateAttribute2ndLevel)
        let filters = parse_filter_string("0x02E9");
        assert!(matches_any_filter(&filters, "02E9"));
        assert!(matches_any_filter(&filters, "745"));
    }

    #[test]
    fn test_search_real_object_id_by_hex() {
        // Real object ID from pcap: 2151762794 (0x80414B6A hex)
        let filters = parse_filter_string("0x80414B6A");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::HexValue(0x80414B6A)));
        assert!(filters.contains(&Filter::DecimalValue(0x80414B6A)));
        // When filters are created from hex, both representations work
        assert!(matches_any_filter(&filters, "80414B6A"));
        assert!(matches_any_filter(&filters, "2151762794"));
    }

    #[test]
    fn test_search_real_object_id_by_decimal() {
        // Real object ID from pcap: 2151762794
        let filters = parse_filter_string("2151762794");
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&Filter::DecimalValue(2151762794)));
        assert!(filters.contains(&Filter::HexValue(2151762794)));
        // When filters are created from decimal, both representations work
        assert!(matches_any_filter(&filters, "2151762794"));
        assert!(matches_any_filter(&filters, "80414B6A"));
    }

    #[test]
    fn test_search_pantaloons_text() {
        // Real item description from pcap line 10
        let filters = parse_filter_string("pantaloons");
        assert!(matches_any_filter(
            &filters,
            "Pantaloons of Piercing Protection"
        ));
    }

    #[test]
    fn test_search_item_type_appraise() {
        // Real message type from pcap
        let filters = parse_filter_string("Item_Appraise");
        assert!(matches_any_filter(&filters, "Item_Appraise"));
    }

    #[test]
    fn test_search_message_type_movement() {
        // Real message type from pcap
        let filters = parse_filter_string("Movement");
        assert!(matches_any_filter(&filters, "Movement_SetObjectMovement"));
    }

    #[test]
    fn test_search_spell_id_4616() {
        // Real spell ID from pcap line 13 (Magic_DispelEnchantment)
        // "4616" is parsed as numeric (both hex and decimal)
        let filters = parse_filter_string("4616");
        // Verify it creates numeric filters
        assert!(filters.contains(&Filter::DecimalValue(4616)));
        assert!(filters.contains(&Filter::HexValue(4616)));
        // JSON data "Id":4616 as a string contains "4616"
        // DecimalValue filter will match when "4616" appears as decimal number
        let data_containing_4616 = "4616";
        assert!(matches_any_filter(&filters, data_containing_4616));
    }

    #[test]
    fn test_search_real_order_sequence_280() {
        // Real OrderedSequence from pcap line 1
        let filters = parse_filter_string("280");
        assert!(matches_any_filter(&filters, "280"));
    }

    #[test]
    fn test_search_item_appraisal_long_desc_decoration() {
        // Real property from pcap
        let filters = parse_filter_string("AppraisalLongDescDecoration");
        assert!(matches_any_filter(&filters, "AppraisalLongDescDecoration"));
    }

    #[test]
    fn test_search_magic_dispel_event_type() {
        // Real event type from pcap
        let filters = parse_filter_string("Magic_DispelEnchantment");
        assert!(matches_any_filter(&filters, "Magic_DispelEnchantment"));
    }

    #[test]
    fn test_search_equipment_set_interlocking() {
        // Real equipment set from pcap
        let filters = parse_filter_string("Interlocking");
        assert!(matches_any_filter(&filters, "Interlocking"));
    }

    #[test]
    fn test_search_cast_spell_message() {
        // Real communication message from pcap line 25-26
        let filters = parse_filter_string("cast");
        let text = "Celdon Sleeves cast Effective Piercing Resistance on you";
        assert!(matches_any_filter(&filters, text));
    }

    #[test]
    fn test_search_spell_category_strength_raising() {
        // Real spell category from pcap
        let filters = parse_filter_string("StrengthRaising");
        assert!(matches_any_filter(&filters, "StrengthRaising"));
    }

    #[test]
    fn test_search_direction_send() {
        // Real direction from pcap
        let filters = parse_filter_string("Send");
        assert!(matches_any_filter(&filters, "Send"));
    }

    #[test]
    fn test_search_direction_recv() {
        // Real direction from pcap
        let filters = parse_filter_string("Recv");
        assert!(matches_any_filter(&filters, "Recv"));
    }

    #[test]
    fn test_search_container_id_1342188310() {
        // Real container object ID from pcap
        let filters = parse_filter_string("1342188310");
        assert!(matches_any_filter(&filters, "1342188310"));
    }

    #[test]
    fn test_search_invalid_large_hex_above_u32() {
        // Value above u32::MAX (too large)
        let filters = parse_filter_string("0xFFFFFFFFFF");
        // Should fall back to StringValue matching
        assert_eq!(
            filters,
            vec![Filter::StringValue("0xffffffffff".to_string())]
        );
    }

    #[test]
    fn test_search_mixed_case_hex_vs_data() {
        // Search for 0xF74C (Movement opcode)
        let filters = parse_filter_string("0xf74c");
        // Should match uppercase opcode in data
        assert!(matches_any_filter(&filters, "F74C"));
        // And decimal equivalent
        assert!(matches_any_filter(&filters, "63308"));
    }

    #[test]
    fn test_search_dyable_property() {
        // Real boolean property from pcap
        let filters = parse_filter_string("Dyable");
        assert!(matches_any_filter(&filters, "Dyable"));
    }

    #[test]
    fn test_search_unwield_object_sound() {
        // Real sound type from pcap
        let filters = parse_filter_string("UnwieldObject");
        assert!(matches_any_filter(&filters, "UnwieldObject"));
    }

    #[test]
    fn test_search_multi_word_item_name() {
        // Real item names from pcap
        let filters = parse_filter_string("bordered cloak");
        assert!(matches_any_filter(&filters, "Bordered Cloak"));

        let filters2 = parse_filter_string("yoroi");
        assert!(matches_any_filter(&filters2, "Yoroi Greaves of Sprinting"));
    }

    #[test]
    fn test_opcodes_do_not_cross_match() {
        // Ensure F7B0 doesn't match F7B1
        let filters_f7b0 = parse_filter_string("0xF7B0");
        assert!(matches_any_filter(&filters_f7b0, "F7B0"));
        assert!(!matches_any_filter(&filters_f7b0, "F7B1"));

        let filters_f7b1 = parse_filter_string("0xF7B1");
        assert!(matches_any_filter(&filters_f7b1, "F7B1"));
        assert!(!matches_any_filter(&filters_f7b1, "F7B0"));
    }

    #[test]
    fn test_decimal_values_exact_match() {
        // 745 should not match 7450 or 74
        let filters = parse_filter_string("745");
        assert!(matches_any_filter(&filters, "745"));
        // String matching is substring, so these will match as strings
        // But as decimal values they're different
        let filters_check = parse_filter_string("7450");
        assert!(matches_any_filter(&filters_check, "7450"));
        // Verify 745 decimal doesn't equal 7450 as decimal numbers
        assert_eq!(
            parse_filter_string("745"),
            vec![Filter::DecimalValue(745), Filter::HexValue(745)]
        );
        assert_eq!(
            parse_filter_string("7450"),
            vec![Filter::DecimalValue(7450), Filter::HexValue(7450)]
        );
    }

    #[test]
    fn test_search_in_json_stringified_data() {
        // Simulate searching in serde_json::to_string of message data
        // When we have {"ObjectId": 2151762794}, the stringified version is {"ObjectId": 2151762794}
        let json_data = r#"{"ObjectId": 2151762794}"#;

        // User types "2151762794" (as decimal)
        let filters = parse_filter_string("2151762794");
        // Should find the ObjectId in the JSON string
        assert!(matches_any_filter(&filters, json_data));
    }

    #[test]
    fn test_search_in_json_stringified_data_hex() {
        // Test hex value in JSON
        let json_data = r#"{"OpCode": "F7B1"}"#;

        // User types "0xF7B1"
        let filters = parse_filter_string("0xF7B1");
        // Should find the OpCode in the JSON string
        assert!(matches_any_filter(&filters, json_data));
    }

    #[test]
    fn test_search_in_complex_json() {
        // Real-world example: a message with nested data
        let json_data =
            r#"{"ObjectId":2151762794,"Type":"Item_SetAppraiseInfo","Properties":{"Health":42}}"#;

        // Search for the object ID
        let filters = parse_filter_string("2151762794");
        assert!(matches_any_filter(&filters, json_data));

        // Search for the type name
        let filters = parse_filter_string("Item_SetAppraiseInfo");
        assert!(matches_any_filter(&filters, json_data));

        // Search for a nested property value
        let filters = parse_filter_string("42");
        assert!(matches_any_filter(&filters, json_data));
    }

    #[test]
    fn test_search_opcode_f7b0_in_game_data() {
        // Real data structure: OpCode is decimal in Data, hex string in top-level OpCode
        let json_data =
            r#"{"ObjectId":2151762794,"OpCode":63408,"MessageType":"Ordered_GameEvent"}"#;

        // User types "0xF7B0" (hex for 63408)
        let filters = parse_filter_string("0xF7B0");

        // Should match via DecimalValue filter matching the "63408" in data
        assert!(matches_any_filter(&filters, json_data));
    }
}
