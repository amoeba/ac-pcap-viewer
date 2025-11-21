use anyhow::{Context, Result};
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
mod messages;
mod properties;

use packet::{PacketHeader, PacketHeaderFlags};
use fragment::{FragmentHeader, Fragment};
use reader::BinaryReader;

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
                                // AC server port is typically 9000-9013
                                // In the pcap, source port tells us direction
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
                    // Need more data - refill the buffer and continue
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

            // Calculate where this packet's data ends
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

            // Ensure we're at the right position for the next packet
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

        // Read fragment header
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

        // Get or create fragment for reassembly
        let fragment = self.pending_fragments
            .entry(sequence)
            .or_insert_with(|| Fragment::new(sequence, count));

        fragment.add_chunk(&bytes, index as usize);

        // Update fragment header info
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
        let frag_count = fragment.count;
        let frag_received = fragment.received;
        let frag_length = fragment.length;

        // Build fragment info for output
        let frag_info = FragmentInfo {
            data: BASE64.encode(&frag_data),
            count,
            received: frag_received,
            length: frag_length,
            sequence,
        };

        if is_complete {
            self.pending_fragments.remove(&sequence);

            // Parse the complete message
            let dir_str = match direction {
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
                    // Print full error chain
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

fn main() -> Result<()> {
    let mut parser = PacketParser::new();

    let pcap_file = "pkt_2025-11-18_1763490291_log.pcap";

    eprintln!("Parsing PCAP file: {}", pcap_file);

    let (packets, messages) = parser.parse_pcap_file(pcap_file)
        .context("Failed to parse pcap file")?;

    eprintln!("Found {} packets", packets.len());
    eprintln!("Found {} messages", messages.len());

    // Output packets as JSONL to fragments.json equivalent
    for packet in &packets {
        // This would be fragments.json output
        // println!("{}", serde_json::to_string(&packet)?);
    }

    // Output messages as JSONL
    for message in &messages {
        println!("{}", serde_json::to_string(&message)?);
    }

    Ok(())
}
