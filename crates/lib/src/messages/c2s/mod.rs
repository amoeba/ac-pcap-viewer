use crate::protocol::BinaryReader;
use anyhow::Result;

// Sub-modules
pub mod actions;

// Re-export action types
pub use actions::*;

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
            let msg = actions::item::ItemAppraise::read(reader, sequence)?;
            Ok(("Item_Appraise".to_string(), serde_json::to_value(&msg)?))
        }
        GameActionType::InventoryPutItemInContainer => {
            let msg = actions::inventory::InventoryPutItemInContainer::read(reader, sequence)?;
            Ok((
                "Inventory_PutItemInContainer".to_string(),
                serde_json::to_value(&msg)?,
            ))
        }
        GameActionType::InventoryGetAndWieldItem => {
            let msg = actions::inventory::InventoryGetAndWieldItem::read(reader, sequence)?;
            Ok((
                "Inventory_GetAndWieldItem".to_string(),
                serde_json::to_value(&msg)?,
            ))
        }
        GameActionType::CharacterCharacterOptionsEvent => {
            let msg = actions::character::CharacterCharacterOptionsEvent::read(reader, sequence)?;
            Ok((
                "Character_CharacterOptionsEvent".to_string(),
                serde_json::to_value(&msg)?,
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
                format!("GameAction_{action_type:04X}"),
                serde_json::json!({
                    "OrderedSequence": sequence,
                    "ActionType": action_type,
                    "OpCode": 0xF7B1u32,
                    "MessageType": "Ordered_GameAction",
                    "MessageDirection": "ClientToServer",
                    "RawData": hex::encode(&raw_data),
                }),
            ))
        }
    }
}
