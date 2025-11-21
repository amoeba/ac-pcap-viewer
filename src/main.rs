use anyhow::{Context, Result};
use clap::Parser;
use pcap_parser::*;
use pcap_parser::traits::PcapReaderIterator;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

mod packet;
mod fragment;
mod message;
mod reader;
mod enums;
pub mod messages;
mod properties;
pub mod serialization;
mod cli;
mod tui;

use packet::{PacketHeader, PacketHeaderFlags};
use fragment::{FragmentHeader, Fragment};
use reader::BinaryReader;
use cli::{Cli, Commands, DirectionFilter, OutputFormat, SortField, FragmentSortField};

/// Direction of packet flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Send, // Client to Server
    Recv, // Server to Client
}

impl Serialize for Direction {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Direction::Send => serializer.serialize_str("Send"),
            Direction::Recv => serializer.serialize_str("Recv"),
        }
    }
}

/// Fragment info as stored in packets
#[derive(Debug, Clone, Serialize)]
pub struct FragmentInfo {
    #[serde(rename = "Data")]
    pub data: String, // Base64 encoded
    #[serde(rename = "Count")]
    pub count: u16,
    #[serde(rename = "Received")]
    pub received: usize,
    #[serde(rename = "Length")]
    pub length: usize,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
}

/// A parsed packet with all its data
#[derive(Debug, Clone, Serialize)]
pub struct ParsedPacket {
    #[serde(rename = "Header")]
    pub header: PacketHeader,
    #[serde(rename = "Direction")]
    pub direction: Direction,
    #[serde(rename = "Messages")]
    pub messages: Vec<serde_json::Value>,
    #[serde(rename = "Fragment")]
    pub fragment: Option<FragmentInfo>,
    #[serde(rename = "Id")]
    pub id: usize,
}

pub struct PacketParser {
    pending_fragments: HashMap<u32, Fragment>,
}

impl PacketParser {
    pub fn new() -> Self {
        Self {
            pending_fragments: HashMap::new(),
        }
    }

    pub fn parse_pcap_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(Vec<ParsedPacket>, Vec<messages::ParsedMessage>)> {
        let mut file = File::open(path.as_ref())
            .context("Failed to open pcap file")?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("Failed to read pcap file")?;

        let mut packets = Vec::new();
        let mut all_messages = Vec::new();
        let mut packet_count = 0;
        let mut udp_count = 0;
        let mut packet_id = 0;
        let mut message_id = 0;

        let mut reader = LegacyPcapReader::new(65536, buffer.as_slice())
            .context("Failed to create pcap reader")?;

        loop {
            match reader.next() {
                Ok((offset, block)) => {
                    match block {
                        PcapBlockOwned::Legacy(packet) => {
                            packet_count += 1;
                            let data = packet.data;

                            // Skip to UDP payload (Ethernet + IP + UDP headers = 42 bytes)
                            if data.len() > 42 {
                                udp_count += 1;
                                let udp_payload = &data[42..];

                                // Determine direction from port
                                let src_port = u16::from_be_bytes([data[34], data[35]]);
                                let direction = if src_port >= 9000 && src_port <= 9013 {
                                    Direction::Recv // From server
                                } else {
                                    Direction::Send // To server
                                };

                                match self.parse_packet(udp_payload, direction, &mut packet_id, &mut message_id) {
                                    Ok((mut parsed_packets, msgs)) => {
                                        packets.append(&mut parsed_packets);
                                        all_messages.extend(msgs);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse packet: {}", e);
                                    }
                                }
                            }
                        }
                        PcapBlockOwned::LegacyHeader(_) => {}
                        _ => {}
                    }
                    reader.consume(offset);
                }
                Err(PcapError::Eof) => break,
                Err(PcapError::Incomplete(_)) => {
                    reader.refill().ok();
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading packet: {:?}", e);
                    break;
                }
            }
        }

        eprintln!("Processed {} packets, {} UDP packets", packet_count, udp_count);

        Ok((packets, all_messages))
    }

    fn parse_packet(
        &mut self,
        data: &[u8],
        direction: Direction,
        packet_id: &mut usize,
        message_id: &mut usize,
    ) -> Result<(Vec<ParsedPacket>, Vec<messages::ParsedMessage>)> {
        let mut packets = Vec::new();
        let mut all_messages = Vec::new();
        let mut reader = BinaryReader::new(data);

        while reader.remaining() > 0 {
            let start_pos = reader.position() as usize;

            let header = match PacketHeader::parse(&mut reader) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Failed to parse packet header: {}", e);
                    break;
                }
            };

            let mut parsed_packet = ParsedPacket {
                header: header.clone(),
                direction,
                messages: Vec::new(),
                fragment: None,
                id: *packet_id,
            };
            *packet_id += 1;

            let packet_end = start_pos + PacketHeader::BASE_SIZE + header.size as usize;

            if header.flags.contains(PacketHeaderFlags::BLOB_FRAGMENTS) {
                while (reader.position() as usize) < packet_end && reader.remaining() > 0 {
                    match self.parse_fragment(&mut reader, direction, message_id) {
                        Ok((frag_info, msgs)) => {
                            parsed_packet.fragment = Some(frag_info);
                            for msg in msgs {
                                parsed_packet.messages.push(msg.data.clone());
                                all_messages.push(msg);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse fragment: {}", e);
                            break;
                        }
                    }
                }
            }

            let current_pos = reader.position() as usize;
            if current_pos < packet_end {
                reader.set_position(packet_end as u64);
            }

            packets.push(parsed_packet);
        }

        Ok((packets, all_messages))
    }

    fn parse_fragment(
        &mut self,
        reader: &mut BinaryReader,
        direction: Direction,
        message_id: &mut usize,
    ) -> Result<(FragmentInfo, Vec<messages::ParsedMessage>)> {
        let mut parsed_messages = Vec::new();

        let sequence = reader.read_u32()?;
        let id = reader.read_u32()?;
        let count = reader.read_u16()?;
        let size = reader.read_u16()?;
        let index = reader.read_u16()?;
        let _group = reader.read_u16()?;

        if size < 16 {
            anyhow::bail!("Invalid fragment size: {} (must be at least 16)", size);
        }

        let frag_length = size as usize - 16;

        if reader.remaining() < frag_length {
            anyhow::bail!(
                "Fragment data too short: need {} bytes, have {} bytes",
                frag_length,
                reader.remaining()
            );
        }

        let bytes = reader.read_bytes(frag_length)?;

        let fragment = self.pending_fragments
            .entry(sequence)
            .or_insert_with(|| Fragment::new(sequence, count));

        fragment.add_chunk(&bytes, index as usize);

        fragment.header = FragmentHeader {
            sequence,
            id,
            count,
            size,
            index,
            group: None,
        };

        let is_complete = fragment.is_complete();
        let frag_data = fragment.data[..fragment.length].to_vec();
        let _frag_count = fragment.count;
        let frag_received = fragment.received;
        let frag_length = fragment.length;

        let frag_info = FragmentInfo {
            data: BASE64.encode(&frag_data),
            count,
            received: frag_received,
            length: frag_length,
            sequence,
        };

        if is_complete {
            self.pending_fragments.remove(&sequence);

            let _dir_str = match direction {
                Direction::Send => "ClientToServer",
                Direction::Recv => "ServerToClient",
            };

            match messages::parse_message(&frag_data, *message_id) {
                Ok(mut parsed) => {
                    parsed.direction = match direction {
                        Direction::Send => "Send".to_string(),
                        Direction::Recv => "Recv".to_string(),
                    };
                    parsed_messages.push(parsed);
                    *message_id += 1;
                }
                Err(e) => {
                    let mut err_str = format!("{}", e);
                    let mut source = e.source();
                    while let Some(s) = source {
                        err_str.push_str(&format!(" -> {}", s));
                        source = s.source();
                    }
                    eprintln!("Failed to parse message: {}", err_str);
                }
            }
        }

        Ok((frag_info, parsed_messages))
    }
}

fn print_summary(packets: &[ParsedPacket], messages: &[messages::ParsedMessage]) {
    println!("=== PCAP Summary ===\n");

    println!("Packets: {}", packets.len());
    println!("Messages: {}", messages.len());

    // Direction breakdown
    let send_packets = packets.iter().filter(|p| matches!(p.direction, Direction::Send)).count();
    let recv_packets = packets.iter().filter(|p| matches!(p.direction, Direction::Recv)).count();
    println!("\nPackets by Direction:");
    println!("  Send (C→S): {}", send_packets);
    println!("  Recv (S→C): {}", recv_packets);

    let send_msgs = messages.iter().filter(|m| m.direction == "Send").count();
    let recv_msgs = messages.iter().filter(|m| m.direction == "Recv").count();
    println!("\nMessages by Direction:");
    println!("  Send (C→S): {}", send_msgs);
    println!("  Recv (S→C): {}", recv_msgs);

    // Message type breakdown
    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    for msg in messages {
        *type_counts.entry(&msg.message_type).or_insert(0) += 1;
    }

    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1));

    println!("\nMessage Types (top 20):");
    for (t, count) in sorted_types.iter().take(20) {
        println!("  {:40} {:>5}", t, count);
    }

    if sorted_types.len() > 20 {
        println!("  ... and {} more types", sorted_types.len() - 20);
    }
}

fn output_messages(
    messages: &[messages::ParsedMessage],
    filter_type: Option<&str>,
    direction: Option<DirectionFilter>,
    sort: SortField,
    reverse: bool,
    limit: Option<usize>,
    output: OutputFormat,
) {
    let mut filtered: Vec<&messages::ParsedMessage> = messages.iter()
        .filter(|m| {
            if let Some(ft) = filter_type {
                if !m.message_type.to_lowercase().contains(&ft.to_lowercase()) {
                    return false;
                }
            }
            if let Some(d) = direction {
                match d {
                    DirectionFilter::Send => if m.direction != "Send" { return false; }
                    DirectionFilter::Recv => if m.direction != "Recv" { return false; }
                }
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort {
            SortField::Id => a.id.cmp(&b.id),
            SortField::Type => a.message_type.cmp(&b.message_type),
            SortField::Direction => a.direction.cmp(&b.direction),
        };
        if reverse { cmp.reverse() } else { cmp }
    });

    if let Some(lim) = limit {
        filtered.truncate(lim);
    }

    match output {
        OutputFormat::Jsonl => {
            for msg in filtered {
                println!("{}", serde_json::to_string(&msg).unwrap());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
        }
        OutputFormat::Table => {
            println!("{:>6}  {:40}  {:>6}  {:>10}", "ID", "Type", "Dir", "OpCode");
            println!("{}", "-".repeat(70));
            for msg in filtered {
                println!("{:>6}  {:40}  {:>6}  {:>10}",
                    msg.id,
                    truncate(&msg.message_type, 40),
                    msg.direction,
                    msg.opcode
                );
            }
        }
    }
}

fn output_fragments(
    packets: &[ParsedPacket],
    direction: Option<DirectionFilter>,
    sort: FragmentSortField,
    reverse: bool,
    limit: Option<usize>,
    output: OutputFormat,
) {
    let mut filtered: Vec<&ParsedPacket> = packets.iter()
        .filter(|p| {
            if let Some(d) = direction {
                match d {
                    DirectionFilter::Send => if !matches!(p.direction, Direction::Send) { return false; }
                    DirectionFilter::Recv => if !matches!(p.direction, Direction::Recv) { return false; }
                }
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort {
            FragmentSortField::Id => a.id.cmp(&b.id),
            FragmentSortField::Sequence => a.header.sequence.cmp(&b.header.sequence),
            FragmentSortField::Direction => format!("{:?}", a.direction).cmp(&format!("{:?}", b.direction)),
        };
        if reverse { cmp.reverse() } else { cmp }
    });

    if let Some(lim) = limit {
        filtered.truncate(lim);
    }

    match output {
        OutputFormat::Jsonl => {
            for pkt in filtered {
                println!("{}", serde_json::to_string(&pkt).unwrap());
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
        }
        OutputFormat::Table => {
            println!("{:>6}  {:>10}  {:>6}  {:>12}  {:>6}", "ID", "Seq", "Dir", "Flags", "Size");
            println!("{}", "-".repeat(50));
            for pkt in filtered {
                println!("{:>6}  {:>10}  {:>6}  {:>12}  {:>6}",
                    pkt.id,
                    pkt.header.sequence,
                    format!("{:?}", pkt.direction),
                    format!("{:08X}", pkt.header.flags.bits()),
                    pkt.header.size
                );
            }
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut parser = PacketParser::new();

    eprintln!("Parsing PCAP file: {}", cli.file);

    let (packets, messages) = parser.parse_pcap_file(&cli.file)
        .context("Failed to parse pcap file")?;

    eprintln!("Found {} packets, {} messages", packets.len(), messages.len());

    match cli.command {
        Some(Commands::Messages { filter_type, direction, sort, reverse, limit, output }) => {
            output_messages(&messages, filter_type.as_deref(), direction, sort, reverse, limit, output);
        }
        Some(Commands::Fragments { direction, sort, reverse, limit, output }) => {
            output_fragments(&packets, direction, sort, reverse, limit, output);
        }
        Some(Commands::Summary) => {
            print_summary(&packets, &messages);
        }
        Some(Commands::Tui) => {
            tui::run_tui(messages, packets)?;
        }
        None => {
            // Default: output messages as JSONL (backwards compatible)
            for message in &messages {
                println!("{}", serde_json::to_string(&message)?);
            }
        }
    }

    Ok(())
}
