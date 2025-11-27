pub mod c2s;
pub mod s2c;

use crate::enums::{C2SMessageType, S2CMessageType};
use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

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

/// Parse a message from raw bytes
pub fn parse_message(data: &[u8], id: usize) -> Result<ParsedMessage> {
    let mut reader = BinaryReader::new(data);

    // Read the opcode
    let opcode = reader.read_u32()?;

    // Determine direction based on opcode
    let s2c_type = S2CMessageType::from_u32(opcode);
    let c2s_type = C2SMessageType::from_u32(opcode);

    let mut parsed = if s2c_type != S2CMessageType::Unknown {
        parse_s2c_message(&mut reader, opcode, s2c_type, id)?
    } else if c2s_type != C2SMessageType::Unknown {
        parse_c2s_message(&mut reader, opcode, c2s_type, id)?
    } else {
        // Unknown message type
        ParsedMessage {
            id,
            message_type: "Unknown".to_string(),
            data: serde_json::json!({
                "OpCode": opcode,
                "RawData": hex::encode(&data[4..]),
            }),
            direction: "Unknown".to_string(),
            opcode: format!("{opcode:04X}"),
            timestamp: 0.0,
            raw_bytes: Vec::new(),
        }
    };

    // Store raw bytes
    parsed.raw_bytes = data.to_vec();
    Ok(parsed)
}

fn parse_s2c_message(
    reader: &mut BinaryReader,
    opcode: u32,
    msg_type: S2CMessageType,
    id: usize,
) -> Result<ParsedMessage> {
    use s2c::*;

    let (type_name, data) = match msg_type {
        S2CMessageType::OrderedGameEvent => {
            // Read ordered game event header
            let object_id = reader.read_u32()?;
            let sequence = reader.read_u32()?;
            let event_type = reader.read_u32()?;

            parse_game_event(reader, object_id, sequence, event_type)?
        }
        S2CMessageType::QualitiesPrivateUpdateInt => {
            let msg = QualitiesPrivateUpdateInt::read(reader)?;
            (
                "Qualities_PrivateUpdateInt".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::QualitiesPrivateUpdateAttribute2ndLevel => {
            let msg = QualitiesPrivateUpdateAttribute2ndLevel::read(reader)?;
            (
                "Qualities_PrivateUpdateAttribute2ndLevel".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::QualitiesUpdateInt => {
            let msg = QualitiesUpdateInt::read(reader)?;
            (
                "Qualities_UpdateInt".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::QualitiesUpdateInstanceId => {
            let msg = QualitiesUpdateInstanceId::read(reader)?;
            (
                "Qualities_UpdateInstanceId".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::MovementSetObjectMovement => {
            let msg = MovementSetObjectMovement::read(reader)?;
            (
                "Movement_SetObjectMovement".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::InventoryPickupEvent => {
            let msg = InventoryPickupEvent::read(reader)?;
            (
                "Inventory_PickupEvent".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::EffectsSoundEvent => {
            let msg = EffectsSoundEvent::read(reader)?;
            (
                "Effects_SoundEvent".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::EffectsPlayScriptType => {
            let msg = EffectsPlayScriptType::read(reader)?;
            (
                "Effects_PlayScriptType".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::CommunicationTextboxString => {
            let msg = CommunicationTextboxString::read(reader)?;
            (
                "Communication_TextboxString".to_string(),
                serde_json::to_value(&msg)?,
            )
        }
        S2CMessageType::ItemObjDescEvent => {
            let msg = ItemObjDescEvent::read(reader)?;
            ("Item_ObjDescEvent".to_string(), serde_json::to_value(&msg)?)
        }
        _ => {
            let remaining = reader.remaining();
            let raw_data = if remaining > 0 {
                reader.read_bytes(remaining)?
            } else {
                vec![]
            };
            (
                format!("{msg_type:?}"),
                serde_json::json!({
                    "OpCode": opcode,
                    "MessageType": format!("{:?}", msg_type),
                    "MessageDirection": "ServerToClient",
                    "RawData": hex::encode(&raw_data),
                }),
            )
        }
    };

    Ok(ParsedMessage {
        id,
        message_type: type_name,
        data,
        direction: "Recv".to_string(),
        opcode: format!("{opcode:04X}"),
        timestamp: 0.0,
        raw_bytes: Vec::new(),
    })
}

fn parse_c2s_message(
    reader: &mut BinaryReader,
    opcode: u32,
    msg_type: C2SMessageType,
    id: usize,
) -> Result<ParsedMessage> {
    use c2s::*;

    let (type_name, data) = match msg_type {
        C2SMessageType::OrderedGameAction => {
            // Read ordered game action header
            let sequence = reader.read_u32()?;
            let action_type = reader.read_u32()?;

            parse_game_action(reader, sequence, action_type)?
        }
        _ => {
            let remaining = reader.remaining();
            let raw_data = if remaining > 0 {
                reader.read_bytes(remaining)?
            } else {
                vec![]
            };
            (
                format!("{msg_type:?}"),
                serde_json::json!({
                    "OpCode": opcode,
                    "MessageType": format!("{:?}", msg_type),
                    "MessageDirection": "ClientToServer",
                    "RawData": hex::encode(&raw_data),
                }),
            )
        }
    };

    Ok(ParsedMessage {
        id,
        message_type: type_name,
        data,
        direction: "Send".to_string(),
        opcode: format!("{opcode:04X}"),
        timestamp: 0.0,
        raw_bytes: Vec::new(),
    })
}
