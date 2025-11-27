// Property definitions, readers, and name mappings for Asheron's Call protocol

/// AppraisalFlags - determines which property sets are included
pub mod appraisal_flags {
    pub const INT_PROPERTIES: u32 = 0x00000001;
    pub const BOOL_PROPERTIES: u32 = 0x00000002;
    pub const FLOAT_PROPERTIES: u32 = 0x00000004;
    pub const STRING_PROPERTIES: u32 = 0x00000008;
    pub const SPELL_BOOK: u32 = 0x00000010;
    pub const WEAPON_PROFILE: u32 = 0x00000020;
    pub const HOOK_PROFILE: u32 = 0x00000040;
    pub const ARMOR_PROFILE: u32 = 0x00000080;
    pub const CREATURE_PROFILE: u32 = 0x00000100;
    pub const ARMOR_ENCH_RATING: u32 = 0x00000200;
    pub const RESIST_ENCH_RATING: u32 = 0x00000400;
    pub const WEAPON_ENCH_RATING: u32 = 0x00000800;
    pub const DATA_ID_PROPERTIES: u32 = 0x00001000;
    pub const INT64_PROPERTIES: u32 = 0x00002000;
    pub const BASE_ARMOR: u32 = 0x00004000;
}

mod enum_names;
mod highlight_masks;
mod profiles;
mod property_names;
mod readers;

// Re-export commonly used items
pub use profiles::{ArmorProfile, CreatureProfile, HookProfile, LayeredSpellId, WeaponProfile};
pub use readers::{
    read_bool_properties, read_dataid_properties, read_float_properties, read_int64_properties,
    read_int_properties, read_spell_book, read_string_properties,
};

// Re-export all name mapping functions
pub use enum_names::*;
pub use highlight_masks::*;
pub use property_names::*;
