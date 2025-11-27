use ac_parser::PacketParser;
use std::fs::File;

fn json_contains_string(value: &serde_json::Value, search: &str) -> bool {
    let search_lower = search.to_lowercase();
    match value {
        serde_json::Value::String(s) => s.to_lowercase().contains(&search_lower),
        serde_json::Value::Array(arr) => arr.iter().any(|v| json_contains_string(v, search)),
        serde_json::Value::Object(obj) => obj.values().any(|v| json_contains_string(v, search)),
        _ => false,
    }
}

fn main() {
    let pcap_path = "pkt_2025-11-18_1763490291_log.pcap";
    let file = File::open(pcap_path).expect("Failed to open test PCAP file");

    let mut parser = PacketParser::new();
    let (_, messages) = parser.parse_pcap(file).expect("Failed to parse PCAP file");

    println!("Total messages: {}", messages.len());

    // Find all messages containing "pantaloons"
    let pantaloons_msgs: Vec<_> = messages
        .iter()
        .filter(|m| json_contains_string(&m.data, "pantaloons"))
        .collect();

    println!(
        "\nMessages containing 'pantaloons': {}",
        pantaloons_msgs.len()
    );
    for msg in pantaloons_msgs.iter().take(3) {
        println!("\n--- Message ID {} ---", msg.id);
        println!("Type: {}", msg.message_type);
        println!(
            "Data snippet: {}",
            serde_json::to_string(&msg.data)
                .unwrap_or_default()
                .chars()
                .take(200)
                .collect::<String>()
        );
    }
}
