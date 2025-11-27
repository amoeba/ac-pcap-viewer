use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ItemAppraise {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "OrderedSequence")]
    pub ordered_sequence: u32,
    #[serde(rename = "ActionType")]
    pub action_type: String,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl ItemAppraise {
    pub fn read(reader: &mut BinaryReader, sequence: u32) -> Result<Self> {
        let object_id = reader.read_u32()?;

        Ok(Self {
            object_id,
            ordered_sequence: sequence,
            action_type: "Item_Appraise".to_string(),
            opcode: 0xF7B1,
            message_type: "Ordered_GameAction".to_string(),
            message_direction: "ClientToServer".to_string(),
        })
    }
}
