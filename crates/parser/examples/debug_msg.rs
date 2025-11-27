use ac_parser::PacketParser;
use std::fs::File;

fn main() {
    let pcap_path = "pkt_2025-11-18_1763490291_log.pcap";
    let file = File::open(pcap_path).expect("Failed to open test PCAP file");

    let mut parser = PacketParser::new();
    let (_, messages) = parser.parse_pcap(file).expect("Failed to parse PCAP file");

    // Find message id 9
    if let Some(msg) = messages.iter().find(|m| m.id == 9) {
        println!("Message ID 9 found!");
        println!("Type: {}", msg.message_type);
        println!("Data: {}", serde_json::to_string_pretty(&msg.data).unwrap());
    } else {
        println!("Message ID 9 NOT found");
        println!("Total messages: {}", messages.len());
        println!(
            "First few message IDs: {:?}",
            messages.iter().take(15).map(|m| m.id).collect::<Vec<_>>()
        );
    }
}
