use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InventoryPutItemInContainer {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "ContainerId")]
    pub container_id: u32,
    #[serde(rename = "SlotIndex")]
    pub slot_index: u32,
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

impl InventoryPutItemInContainer {
    pub fn read(reader: &mut BinaryReader, sequence: u32) -> Result<Self> {
        let object_id = reader.read_u32()?;
        let container_id = reader.read_u32()?;
        let slot_index = reader.read_u32()?;

        Ok(Self {
            object_id,
            container_id,
            slot_index,
            ordered_sequence: sequence,
            action_type: "Inventory_PutItemInContainer".to_string(),
            opcode: 0xF7B1,
            message_type: "Ordered_GameAction".to_string(),
            message_direction: "ClientToServer".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryGetAndWieldItem {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "Slot")]
    pub slot: String,
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

impl InventoryGetAndWieldItem {
    pub fn read(reader: &mut BinaryReader, sequence: u32) -> Result<Self> {
        let object_id = reader.read_u32()?;
        let slot_raw = reader.read_u32()?;
        let slot = crate::properties::equip_mask_name(slot_raw);

        Ok(Self {
            object_id,
            slot,
            ordered_sequence: sequence,
            action_type: "Inventory_GetAndWieldItem".to_string(),
            opcode: 0xF7B1,
            message_type: "Ordered_GameAction".to_string(),
            message_direction: "ClientToServer".to_string(),
        })
    }
}
