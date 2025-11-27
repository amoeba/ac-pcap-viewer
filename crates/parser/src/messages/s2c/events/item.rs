use crate::properties::{
    self, appraisal_flags, ArmorProfile, CreatureProfile, HookProfile, WeaponProfile,
};
use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ItemWearItem {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "Slot")]
    pub slot: String,
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

impl ItemWearItem {
    pub fn read(
        reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        let object_id = reader.read_u32()?;
        let slot_raw = reader.read_u32()?;
        let slot = crate::properties::equip_mask_name(slot_raw);

        Ok(Self {
            object_id,
            slot,
            ordered_object_id,
            ordered_sequence,
            event_type: "Item_WearItem".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemSetAppraiseInfo {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "IntProperties")]
    pub int_properties: HashMap<String, i32>,
    #[serde(rename = "Int64Properties")]
    pub int64_properties: HashMap<String, i64>,
    #[serde(rename = "BoolProperties")]
    pub bool_properties: HashMap<String, bool>,
    #[serde(rename = "FloatProperties")]
    pub float_properties: HashMap<String, f64>,
    #[serde(rename = "StringProperties")]
    pub string_properties: HashMap<String, String>,
    #[serde(rename = "DataIdProperties")]
    pub dataid_properties: HashMap<String, u32>,
    #[serde(rename = "SpellBook")]
    pub spell_book: Vec<properties::LayeredSpellId>,
    #[serde(rename = "ArmorProfile")]
    pub armor_profile: Option<ArmorProfile>,
    #[serde(rename = "CreatureProfile")]
    pub creature_profile: Option<CreatureProfile>,
    #[serde(rename = "WeaponProfile")]
    pub weapon_profile: Option<WeaponProfile>,
    #[serde(rename = "HookProfile")]
    pub hook_profile: Option<HookProfile>,
    #[serde(rename = "ArmorHighlight")]
    pub armor_highlight: serde_json::Value,
    #[serde(rename = "ArmorColor")]
    pub armor_color: serde_json::Value,
    #[serde(rename = "WeaponHighlight")]
    pub weapon_highlight: serde_json::Value,
    #[serde(rename = "WeaponColor")]
    pub weapon_color: serde_json::Value,
    #[serde(rename = "ResistHighlight")]
    pub resist_highlight: serde_json::Value,
    #[serde(rename = "ResistColor")]
    pub resist_color: serde_json::Value,
    #[serde(rename = "BaseArmorHead")]
    pub base_armor_head: u32,
    #[serde(rename = "BaseArmorChest")]
    pub base_armor_chest: u32,
    #[serde(rename = "BaseArmorGroin")]
    pub base_armor_groin: u32,
    #[serde(rename = "BaseArmorBicep")]
    pub base_armor_bicep: u32,
    #[serde(rename = "BaseArmorWrist")]
    pub base_armor_wrist: u32,
    #[serde(rename = "BaseArmorHand")]
    pub base_armor_hand: u32,
    #[serde(rename = "BaseArmorThigh")]
    pub base_armor_thigh: u32,
    #[serde(rename = "BaseArmorShin")]
    pub base_armor_shin: u32,
    #[serde(rename = "BaseArmorFoot")]
    pub base_armor_foot: u32,
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

impl ItemSetAppraiseInfo {
    pub fn read(
        reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        let object_id = reader.read_u32()?;
        let flags = reader.read_u32()?;
        let success = reader.read_bool()?;

        let int_properties = if flags & appraisal_flags::INT_PROPERTIES != 0 {
            properties::read_int_properties(reader)?
        } else {
            HashMap::new()
        };

        let int64_properties = if flags & appraisal_flags::INT64_PROPERTIES != 0 {
            properties::read_int64_properties(reader)?
        } else {
            HashMap::new()
        };

        let bool_properties = if flags & appraisal_flags::BOOL_PROPERTIES != 0 {
            properties::read_bool_properties(reader)?
        } else {
            HashMap::new()
        };

        let float_properties = if flags & appraisal_flags::FLOAT_PROPERTIES != 0 {
            properties::read_float_properties(reader)?
        } else {
            HashMap::new()
        };

        let string_properties = if flags & appraisal_flags::STRING_PROPERTIES != 0 {
            properties::read_string_properties(reader)?
        } else {
            HashMap::new()
        };

        let dataid_properties = if flags & appraisal_flags::DATA_ID_PROPERTIES != 0 {
            properties::read_dataid_properties(reader)?
        } else {
            HashMap::new()
        };

        let spell_book = if flags & appraisal_flags::SPELL_BOOK != 0 {
            properties::read_spell_book(reader)?
        } else {
            Vec::new()
        };

        let armor_profile = if flags & appraisal_flags::ARMOR_PROFILE != 0 {
            Some(ArmorProfile::read(reader)?)
        } else {
            None
        };

        let creature_profile = if flags & appraisal_flags::CREATURE_PROFILE != 0 {
            Some(CreatureProfile::read(reader)?)
        } else {
            None
        };

        let weapon_profile = if flags & appraisal_flags::WEAPON_PROFILE != 0 {
            Some(WeaponProfile::read(reader)?)
        } else {
            None
        };

        let hook_profile = if flags & appraisal_flags::HOOK_PROFILE != 0 {
            Some(HookProfile::read(reader)?)
        } else {
            None
        };

        fn highlight_to_json(value: u16, f: fn(u16) -> String) -> serde_json::Value {
            if value == 0 {
                serde_json::Value::Number(0.into())
            } else {
                serde_json::Value::String(f(value))
            }
        }

        let (armor_highlight, armor_color) = if flags & appraisal_flags::ARMOR_ENCH_RATING != 0 {
            let ah = reader.read_u16()?;
            let ac = reader.read_u16()?;
            (
                highlight_to_json(ah, properties::armor_highlight_mask_name),
                highlight_to_json(ac, properties::armor_highlight_mask_name),
            )
        } else {
            (
                serde_json::Value::Number(0.into()),
                serde_json::Value::Number(0.into()),
            )
        };

        let (weapon_highlight, weapon_color) = if flags & appraisal_flags::WEAPON_ENCH_RATING != 0 {
            let wh = reader.read_u16()?;
            let wc = reader.read_u16()?;
            (
                highlight_to_json(wh, properties::weapon_highlight_mask_name),
                highlight_to_json(wc, properties::weapon_highlight_mask_name),
            )
        } else {
            (
                serde_json::Value::Number(0.into()),
                serde_json::Value::Number(0.into()),
            )
        };

        let (resist_highlight, resist_color) = if flags & appraisal_flags::RESIST_ENCH_RATING != 0 {
            let rh = reader.read_u16()?;
            let rc = reader.read_u16()?;
            (
                highlight_to_json(rh, properties::resist_highlight_mask_name),
                highlight_to_json(rc, properties::resist_highlight_mask_name),
            )
        } else {
            (
                serde_json::Value::Number(0.into()),
                serde_json::Value::Number(0.into()),
            )
        };

        let (
            base_armor_head,
            base_armor_chest,
            base_armor_groin,
            base_armor_bicep,
            base_armor_wrist,
            base_armor_hand,
            base_armor_thigh,
            base_armor_shin,
            base_armor_foot,
        ) = if flags & appraisal_flags::BASE_ARMOR != 0 {
            (
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
            )
        } else {
            (0, 0, 0, 0, 0, 0, 0, 0, 0)
        };

        Ok(Self {
            object_id,
            flags,
            success,
            int_properties,
            int64_properties,
            bool_properties,
            float_properties,
            string_properties,
            dataid_properties,
            spell_book,
            armor_profile,
            creature_profile,
            weapon_profile,
            hook_profile,
            armor_highlight,
            armor_color,
            weapon_highlight,
            weapon_color,
            resist_highlight,
            resist_color,
            base_armor_head,
            base_armor_chest,
            base_armor_groin,
            base_armor_bicep,
            base_armor_wrist,
            base_armor_hand,
            base_armor_thigh,
            base_armor_shin,
            base_armor_foot,
            ordered_object_id,
            ordered_sequence,
            event_type: "Item_SetAppraiseInfo".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemServerSaysContainId {
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "ContainerId")]
    pub container_id: u32,
    #[serde(rename = "SlotIndex")]
    pub slot_index: u32,
    #[serde(rename = "ContainerType")]
    pub container_type: String,
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

impl ItemServerSaysContainId {
    pub fn read(
        reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        let object_id = reader.read_u32()?;
        let container_id = reader.read_u32()?;
        let slot_index = reader.read_u32()?;
        let container_type_raw = reader.read_u32()?;

        let container_type = match container_type_raw {
            0 => "None",
            1 => "Container",
            2 => "Foci",
            _ => {
                return Ok(Self {
                    object_id,
                    container_id,
                    slot_index,
                    container_type: format!("ContainerType_{container_type_raw}"),
                    ordered_object_id,
                    ordered_sequence,
                    event_type: "Item_ServerSaysContainId".to_string(),
                    opcode: 0xF7B0,
                    message_type: "Ordered_GameEvent".to_string(),
                    message_direction: "ServerToClient".to_string(),
                })
            }
        }
        .to_string();

        Ok(Self {
            object_id,
            container_id,
            slot_index,
            container_type,
            ordered_object_id,
            ordered_sequence,
            event_type: "Item_ServerSaysContainId".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}
