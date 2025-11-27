use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

/// ArmorProfile for protection values
#[derive(Debug, Clone, Serialize)]
pub struct ArmorProfile {
    #[serde(
        rename = "ProtSlashing",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_slashing: f32,
    #[serde(
        rename = "ProtPiercing",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_piercing: f32,
    #[serde(
        rename = "ProtBludgeoning",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_bludgeoning: f32,
    #[serde(
        rename = "ProtCold",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_cold: f32,
    #[serde(
        rename = "ProtFire",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_fire: f32,
    #[serde(
        rename = "ProtAcid",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_acid: f32,
    #[serde(
        rename = "ProtNether",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_nether: f32,
    #[serde(
        rename = "ProtLightning",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub prot_lightning: f32,
}

impl ArmorProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            prot_slashing: reader.read_f32()?,
            prot_piercing: reader.read_f32()?,
            prot_bludgeoning: reader.read_f32()?,
            prot_cold: reader.read_f32()?,
            prot_fire: reader.read_f32()?,
            prot_acid: reader.read_f32()?,
            prot_nether: reader.read_f32()?,
            prot_lightning: reader.read_f32()?,
        })
    }
}

/// WeaponProfile for weapon appraisal data (28 bytes)
#[derive(Debug, Clone, Serialize)]
pub struct WeaponProfile {
    #[serde(rename = "DamageType")]
    pub damage_type: u32,
    #[serde(rename = "WeaponTime")]
    pub weapon_time: u32,
    #[serde(rename = "WeaponSkill")]
    pub weapon_skill: u32,
    #[serde(rename = "WeaponDamage")]
    pub weapon_damage: u32,
    #[serde(
        rename = "DamageVariance",
        serialize_with = "crate::serialization::serialize_f64"
    )]
    pub damage_variance: f64,
    #[serde(
        rename = "DamageMod",
        serialize_with = "crate::serialization::serialize_f64"
    )]
    pub damage_mod: f64,
}

impl WeaponProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            damage_type: reader.read_u32()?,
            weapon_time: reader.read_u32()?,
            weapon_skill: reader.read_u32()?,
            weapon_damage: reader.read_u32()?,
            damage_variance: reader.read_f64()?,
            damage_mod: reader.read_f64()?,
        })
    }
}

/// HookProfile for housing hook data
#[derive(Debug, Clone, Serialize)]
pub struct HookProfile {
    #[serde(rename = "Flags")]
    pub flags: u32,
}

impl HookProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            flags: reader.read_u32()?,
        })
    }
}

/// CreatureProfile for creature/monster stats
#[derive(Debug, Clone, Serialize)]
pub struct CreatureProfile {
    #[serde(rename = "Health")]
    pub health: u32,
    #[serde(rename = "Stamina")]
    pub stamina: u32,
    #[serde(rename = "Mana")]
    pub mana: u32,
    #[serde(rename = "Strength")]
    pub strength: u32,
    #[serde(rename = "Endurance")]
    pub endurance: u32,
    #[serde(rename = "Coordination")]
    pub coordination: u32,
    #[serde(rename = "Quickness")]
    pub quickness: u32,
    #[serde(rename = "Focus")]
    pub focus: u32,
    #[serde(rename = "Self")]
    pub self_: u32,
}

impl CreatureProfile {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        Ok(Self {
            health: reader.read_u32()?,
            stamina: reader.read_u32()?,
            mana: reader.read_u32()?,
            strength: reader.read_u32()?,
            endurance: reader.read_u32()?,
            coordination: reader.read_u32()?,
            quickness: reader.read_u32()?,
            focus: reader.read_u32()?,
            self_: reader.read_u32()?,
        })
    }
}

/// LayeredSpellId for spell book entries
#[derive(Debug, Clone, Serialize)]
pub struct LayeredSpellId {
    #[serde(rename = "Id")]
    pub id: u16,
    #[serde(rename = "Layer")]
    pub layer: u16,
}
