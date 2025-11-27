use crate::properties::LayeredSpellId;
use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MagicDispelEnchantment {
    #[serde(rename = "SpellId")]
    pub spell_id: LayeredSpellId,
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

impl MagicDispelEnchantment {
    pub fn read(
        reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        let spell_id = reader.read_u16()?;
        let layer = reader.read_u16()?;

        Ok(Self {
            spell_id: LayeredSpellId {
                id: spell_id,
                layer,
            },
            ordered_object_id,
            ordered_sequence,
            event_type: "Magic_DispelEnchantment".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MagicUpdateEnchantment {
    #[serde(rename = "Enchantment")]
    pub enchantment: Enchantment,
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

#[derive(Debug, Clone, Serialize)]
pub struct Enchantment {
    #[serde(rename = "Id")]
    pub id: LayeredSpellId,
    #[serde(rename = "HasEquipmentSet")]
    pub has_equipment_set: u32,
    #[serde(rename = "SpellCategory")]
    pub spell_category: String,
    #[serde(rename = "PowerLevel")]
    pub power_level: u32,
    #[serde(
        rename = "StartTime",
        serialize_with = "crate::serialization::serialize_f64"
    )]
    pub start_time: f64,
    #[serde(
        rename = "Duration",
        serialize_with = "crate::serialization::serialize_f64"
    )]
    pub duration: f64,
    #[serde(rename = "CasterId")]
    pub caster_id: u32,
    #[serde(
        rename = "DegradeModifier",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub degrade_modifier: f32,
    #[serde(
        rename = "DegradeLimit",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub degrade_limit: f32,
    #[serde(
        rename = "LastTimeDegraded",
        serialize_with = "crate::serialization::serialize_f64"
    )]
    pub last_time_degraded: f64,
    #[serde(rename = "StatMod")]
    pub stat_mod: StatMod,
    #[serde(rename = "EquipmentSet")]
    pub equipment_set: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatMod {
    #[serde(rename = "Type")]
    pub mod_type: String,
    #[serde(rename = "Key")]
    pub key: u32,
    #[serde(
        rename = "Value",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub value: f32,
}

impl Enchantment {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let spell_id = reader.read_u16()?;
        let layer = reader.read_u16()?;
        let has_equipment_set = reader.read_u16()? as u32;
        let spell_category_id = reader.read_u16()?;
        let power_level = reader.read_u32()?;
        let start_time = reader.read_f64()?;
        let duration = reader.read_f64()?;
        let caster_id = reader.read_u32()?;
        let degrade_modifier = reader.read_f32()?;
        let degrade_limit = reader.read_f32()?;
        let last_time_degraded = reader.read_f64()?;

        let stat_mod_type = reader.read_u32()?;
        let stat_mod_key = reader.read_u32()?;
        let stat_mod_value = reader.read_f32()?;

        let equipment_set_id = if has_equipment_set > 0 {
            reader.read_u32()?
        } else {
            0
        };

        Ok(Self {
            id: LayeredSpellId {
                id: spell_id,
                layer,
            },
            has_equipment_set,
            spell_category: spell_category_name(spell_category_id),
            power_level,
            start_time,
            duration,
            caster_id,
            degrade_modifier,
            degrade_limit,
            last_time_degraded,
            stat_mod: StatMod {
                mod_type: stat_mod_type_name(stat_mod_type),
                key: stat_mod_key,
                value: stat_mod_value,
            },
            equipment_set: equipment_set_name(equipment_set_id),
        })
    }
}

impl MagicUpdateEnchantment {
    pub fn read(
        reader: &mut BinaryReader,
        ordered_object_id: u32,
        ordered_sequence: u32,
    ) -> Result<Self> {
        let enchantment = Enchantment::read(reader)?;

        Ok(Self {
            enchantment,
            ordered_object_id,
            ordered_sequence,
            event_type: "Magic_UpdateEnchantment".to_string(),
            opcode: 0xF7B0,
            message_type: "Ordered_GameEvent".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

fn spell_category_name(id: u16) -> String {
    match id {
        1 => "StrengthRaising".to_string(),
        2 => "EnduranceRaising".to_string(),
        3 => "CoordinationRaising".to_string(),
        4 => "QuicknessRaising".to_string(),
        5 => "FocusRaising".to_string(),
        6 => "SelfRaising".to_string(),
        _ => format!("Category_{id}"),
    }
}

fn stat_mod_type_name(flags: u32) -> String {
    let mut parts = Vec::new();

    if flags & 0x0000001 != 0 {
        parts.push("Attribute");
    }
    if flags & 0x0000002 != 0 {
        parts.push("SecondAtt");
    }
    if flags & 0x0000004 != 0 {
        parts.push("Int");
    }
    if flags & 0x0000008 != 0 {
        parts.push("Float");
    }
    if flags & 0x0000010 != 0 {
        parts.push("Skill");
    }
    if flags & 0x0000020 != 0 {
        parts.push("BodyDamageValue");
    }
    if flags & 0x0000040 != 0 {
        parts.push("BodyDamageVariance");
    }
    if flags & 0x0000080 != 0 {
        parts.push("BodyArmorValue");
    }

    if flags & 0x0001000 != 0 {
        parts.push("SingleStat");
    }
    if flags & 0x0002000 != 0 {
        parts.push("MultipleStat");
    }
    if flags & 0x0004000 != 0 {
        parts.push("Multiplicative");
    }
    if flags & 0x0008000 != 0 {
        parts.push("Additive");
    }
    if flags & 0x0010000 != 0 {
        parts.push("AttackSkills");
    }
    if flags & 0x0020000 != 0 {
        parts.push("DefenseSkills");
    }
    if flags & 0x0100000 != 0 {
        parts.push("MultiplicativeDegrade");
    }
    if flags & 0x0200000 != 0 {
        parts.push("Additive_Degrade");
    }
    if flags & 0x0800000 != 0 {
        parts.push("Vitae");
    }
    if flags & 0x1000000 != 0 {
        parts.push("Cooldown");
    }
    if flags & 0x2000000 != 0 {
        parts.push("Beneficial");
    }

    if parts.is_empty() {
        format!("StatModType_{flags}")
    } else {
        parts.join(", ")
    }
}

fn equipment_set_name(id: u32) -> String {
    match id {
        0 => "None",
        1 => "Test",
        2 => "Test2",
        3 => "Unknown3",
        4 => "CarraidasBenediction",
        5 => "NobleRelic",
        6 => "AncientRelic",
        7 => "AlduressaRelic",
        8 => "Ninja",
        9 => "EmpyreanRings",
        10 => "ArmMindHeart",
        11 => "ArmorPerfectLight",
        12 => "ArmorPerfectLight2",
        13 => "Soldiers",
        14 => "Adepts",
        15 => "Archers",
        16 => "Defenders",
        17 => "Tinkers",
        18 => "Crafters",
        19 => "Hearty",
        20 => "Dexterous",
        21 => "Wise",
        22 => "Swift",
        23 => "Hardened",
        24 => "Reinforced",
        25 => "Interlocking",
        26 => "Flameproof",
        27 => "Acidproof",
        28 => "Coldproof",
        29 => "Lightningproof",
        30 => "SocietyArmor",
        31 => "ColosseumClothing",
        32 => "GraveyardClothing",
        33 => "OlthoiClothing",
        34 => "NoobieArmor",
        35 => "AetheriaDefense",
        36 => "AetheriaDestruction",
        37 => "AetheriaFury",
        38 => "AetheriaGrowth",
        39 => "AetheriaVigor",
        40 => "RareDamageResistance",
        41 => "RareDamageBoost",
        _ => return format!("Set_{id}"),
    }
    .to_string()
}
