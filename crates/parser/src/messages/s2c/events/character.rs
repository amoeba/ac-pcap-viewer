use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CharacterCharacterOptionsEvent {
    #[serde(rename = "OrderedObjectId")]
    pub ordered_object_id: u32,
    #[serde(rename = "OrderedSequence")]
    pub ordered_sequence: u32,
    #[serde(rename = "EventType")]
    pub event_type: String,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl CharacterCharacterOptionsEvent {
    pub fn read(
        _reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        // Complex options data - skip for now
        Ok(Self {
            ordered_object_id,
            ordered_sequence,
            event_type: "Character_CharacterOptionsEvent".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}
