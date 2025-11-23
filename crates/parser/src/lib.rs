//! Asheron's Call PCAP Parser Library
//!
//! This library provides functionality to parse PCAP files containing
//! Asheron's Call network traffic.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::*;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;

pub mod enums;
pub mod fragment;
pub mod message;
pub mod messages;
pub mod packet;
pub mod properties;
pub mod reader;
pub mod serialization;

use fragment::{Fragment, FragmentHeader};
use packet::{PacketHeader, PacketHeaderFlags};
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

/// Main parser for PCAP files
pub struct PacketParser {
    pending_fragments: HashMap<u32, Fragment>,
}

impl PacketParser {
    pub fn new() -> Self {
        Self {
            pending_fragments: HashMap::new(),
        }
    }

    /// Parse a PCAP file from a reader
    pub fn parse_pcap<R: Read>(
        &mut self,
        mut reader: R,
    ) -> Result<(Vec<ParsedPacket>, Vec<messages::ParsedMessage>)> {
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .context("Failed to read pcap data")?;

        self.parse_pcap_bytes(&buffer)
    }

    /// Parse PCAP data from bytes
    pub fn parse_pcap_bytes(
        &mut self,
        buffer: &[u8],
    ) -> Result<(Vec<ParsedPacket>, Vec<messages::ParsedMessage>)> {
        let mut packets = Vec::new();
        let mut all_messages = Vec::new();
        let mut packet_id = 0;
        let mut message_id = 0;

        let mut reader =
            LegacyPcapReader::new(65536, buffer).context("Failed to create pcap reader")?;

        loop {
            match reader.next() {
                Ok((offset, block)) => {
                    match block {
                        PcapBlockOwned::Legacy(packet) => {
                            let data = packet.data;

                            // Skip to UDP payload (Ethernet + IP + UDP headers = 42 bytes)
                            if data.len() > 42 {
                                let udp_payload = &data[42..];

                                // Determine direction from port
                                let src_port = u16::from_be_bytes([data[34], data[35]]);
                                let direction = if (9000..=9013).contains(&src_port) {
                                    Direction::Recv // From server
                                } else {
                                    Direction::Send // To server
                                };

                                match self.parse_packet(
                                    udp_payload,
                                    direction,
                                    &mut packet_id,
                                    &mut message_id,
                                ) {
                                    Ok((mut parsed_packets, msgs)) => {
                                        packets.append(&mut parsed_packets);
                                        all_messages.extend(msgs);
                                    }
                                    Err(_e) => {
                                        // Skip failed packets silently
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
                Err(_e) => {
                    break;
                }
            }
        }

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

            let header = PacketHeader::parse(&mut reader)?;

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
                        Err(_e) => {
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
            anyhow::bail!("Invalid fragment size: {}", size);
        }

        let frag_length = size as usize - 16;

        if reader.remaining() < frag_length {
            anyhow::bail!("Fragment data too short");
        }

        let bytes = reader.read_bytes(frag_length)?;

        let fragment = self
            .pending_fragments
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

            match messages::parse_message(&frag_data, *message_id) {
                Ok(mut parsed) => {
                    parsed.direction = match direction {
                        Direction::Send => "Send".to_string(),
                        Direction::Recv => "Recv".to_string(),
                    };
                    parsed_messages.push(parsed);
                    *message_id += 1;
                }
                Err(_e) => {
                    // Skip failed messages
                }
            }
        }

        Ok((frag_info, parsed_messages))
    }
}

impl Default for PacketParser {
    fn default() -> Self {
        Self::new()
    }
}
