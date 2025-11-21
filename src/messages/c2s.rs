use serde::Serialize;
use crate::reader::BinaryReader;
use anyhow::Result;

// Game action types (for 0xF7B1 messages)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GameActionType {
    ItemAppraise = 0x00C8,
    InventoryPutItemInContainer = 0x0019,
    InventoryGetAndWieldItem = 0x001A,
    CharacterCharacterOptionsEvent = 0x01A1,
    Unknown = 0xFFFFFFFF,
}

impl GameActionType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x00C8 => GameActionType::ItemAppraise,
            0x0019 => GameActionType::InventoryPutItemInContainer,
            0x001A => GameActionType::InventoryGetAndWieldItem,
            0x01A1 => GameActionType::CharacterCharacterOptionsEvent,
            _ => GameActionType::Unknown,
        }
    }
}

pub fn parse_game_action(
    reader: &mut BinaryReader,
    sequence: u32,
    action_type: u32,
) -> Result<(String, serde_json::Value)> {
    let act_type = GameActionType::from_u32(action_type);

    match act_type {
        GameActionType::ItemAppraise => {
            let msg = ItemAppraise::read(reader, sequence)?;
            Ok(("Item_Appraise".to_string(), serde_json::to_value(&msg)?))
        }
        GameActionType::InventoryPutItemInContainer => {
            let msg = InventoryPutItemInContainer::read(reader, sequence)?;
            Ok(("Inventory_PutItemInContainer".to_string(), serde_json::to_value(&msg)?))
        }
        GameActionType::InventoryGetAndWieldItem => {
            let msg = InventoryGetAndWieldItem::read(reader, sequence)?;
            Ok(("Inventory_GetAndWieldItem".to_string(), serde_json::to_value(&msg)?))
        }
        GameActionType::CharacterCharacterOptionsEvent => {
            // PlayerModule is complex - output basic info for now
            let remaining = reader.remaining();
            let raw_data = if remaining > 0 {
                reader.read_bytes(remaining)?
            } else {
                vec![]
            };
            Ok((
                "Character_CharacterOptionsEvent".to_string(),
                serde_json::json!({
                    "Options": {
                        "RawData": hex::encode(&raw_data),
                    },
                    "OrderedSequence": sequence,
                    "ActionType": "Character_CharacterOptionsEvent",
                    "OpCode": 0xF7B1u32,
                    "MessageType": "Ordered_GameAction",
                    "MessageDirection": "ClientToServer",
                })
            ))
        }
        _ => {
            let remaining = reader.remaining();
            let raw_data = if remaining > 0 {
                reader.read_bytes(remaining)?
            } else {
                vec![]
            };
            Ok((
                format!("GameAction_{:04X}", action_type),
                serde_json::json!({
                    "OrderedSequence": sequence,
                    "ActionType": action_type,
                    "OpCode": 0xF7B1u32,
                    "MessageType": "Ordered_GameAction",
                    "MessageDirection": "ClientToServer",
                    "RawData": hex::encode(&raw_data),
                })
            ))
        }
    }
}

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
