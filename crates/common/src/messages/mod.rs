use anyhow::Result;
use serde::Serialize;
use std::io::Cursor;

use acprotocol::readers::ACReader;
use acprotocol::unified::{Direction, MessageKind};

/// Parsed AC message with all fields decoded
#[derive(Debug, Clone, Serialize)]
pub struct ParsedMessage {
    #[serde(rename = "Id")]
    pub id: usize,
    #[serde(rename = "Type")]
    pub message_type: String,
    #[serde(rename = "Data")]
    pub data: serde_json::Value,
    #[serde(rename = "Direction")]
    pub direction: String,
    #[serde(rename = "OpCode")]
    pub opcode: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: f64, // Seconds since epoch (with microsecond precision)
    #[serde(skip)]
    pub raw_bytes: Vec<u8>,
}

/// Parse a message from raw bytes using acprotocol
pub fn parse_message(data: &[u8], id: usize) -> Result<ParsedMessage> {
    if data.len() < 4 {
        anyhow::bail!("Message data too short to contain opcode");
    }

    // Read the opcode
    let opcode = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    // Determine direction based on opcode
    let direction = determine_direction(opcode)?;
    let direction_str = match direction {
        Direction::ClientToServer => "Send",
        Direction::ServerToClient => "Recv",
    };

    // Parse the message using acprotocol
    let mut cursor = Cursor::new(data);
    let reader: &mut dyn ACReader = &mut cursor;

    let parsed_data = match MessageKind::read(reader, direction) {
        Ok(message) => {
            // Serialize the parsed message to JSON
            serde_json::to_value(&message)?
        }
        Err(e) => {
            // If parsing fails, return error info
            serde_json::json!({
                "error": format!("{}", e),
                "opcode": format!("0x{:04X}", opcode),
                "raw_data": hex::encode(&data[4..]),
            })
        }
    };

    // Get message type name
    let message_type = get_message_type_name(opcode, data);

    Ok(ParsedMessage {
        id,
        message_type,
        data: parsed_data,
        direction: direction_str.to_string(),
        opcode: format!("{:04X}", opcode),
        timestamp: 0.0,
        raw_bytes: data.to_vec(),
    })
}

/// Determine message direction based on opcode
fn determine_direction(opcode: u32) -> Result<Direction> {
    use acprotocol::enums::{C2SMessage, S2CMessage};

    if C2SMessage::try_from(opcode).is_ok() {
        Ok(Direction::ClientToServer)
    } else if S2CMessage::try_from(opcode).is_ok() {
        Ok(Direction::ServerToClient)
    } else {
        anyhow::bail!(
            "Unhandled opcode 0x{:04X}, couldn't determine C2S or S2C",
            opcode
        )
    }
}

/// Get human-readable message type name from parsed message data
fn get_message_type_name(opcode: u32, data: &[u8]) -> String {
    use acprotocol::enums::{C2SMessage, GameAction, GameEvent, S2CMessage};

    if data.len() < 4 {
        return "Unknown".to_string();
    }

    let payload = &data[4..];

    // Check C2S messages
    if let Ok(msg_type) = C2SMessage::try_from(opcode) {
        if msg_type == C2SMessage::OrderedGameAction && payload.len() >= 8 {
            let action_type_val =
                u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
            if let Ok(game_action) = GameAction::try_from(action_type_val) {
                // Serialize to get the serde-renamed version
                return serde_json::to_value(&game_action)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("{:?}", game_action));
            }
            return "OrderedGameAction".to_string();
        }
        // Serialize to get the serde-renamed version
        return serde_json::to_value(&msg_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", msg_type));
    }

    // Check S2C messages
    if let Ok(msg_type) = S2CMessage::try_from(opcode) {
        if msg_type == S2CMessage::OrderedGameEvent && payload.len() >= 12 {
            let event_type_val =
                u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
            if let Ok(game_event) = GameEvent::try_from(event_type_val) {
                // Serialize to get the serde-renamed version
                return serde_json::to_value(&game_event)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("{:?}", game_event));
            }
            return "OrderedGameEvent".to_string();
        }
        // Serialize to get the serde-renamed version
        return serde_json::to_value(&msg_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", msg_type));
    }

    "Unknown".to_string()
}
