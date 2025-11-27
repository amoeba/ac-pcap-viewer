use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

// ============================================================================
// Character_CharacterOptionsEvent (0x01A1)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CharacterCharacterOptionsEvent {
    #[serde(rename = "Options")]
    pub options: PlayerModule,
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

impl CharacterCharacterOptionsEvent {
    pub fn read(reader: &mut BinaryReader, sequence: u32) -> Result<Self> {
        let options = PlayerModule::read(reader)?;
        Ok(Self {
            options,
            ordered_sequence: sequence,
            action_type: "Character_CharacterOptionsEvent".to_string(),
            opcode: 0xF7B1,
            message_type: "Ordered_GameAction".to_string(),
            message_direction: "ClientToServer".to_string(),
        })
    }
}

// ============================================================================
// PlayerModule - Character options structure
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PlayerModule {
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Options")]
    pub options: String,
    #[serde(rename = "Shortcuts")]
    pub shortcuts: Vec<ShortCutData>,
    #[serde(rename = "Tab1Spells")]
    pub tab1_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab2Spells")]
    pub tab2_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab3Spells")]
    pub tab3_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab4Spells")]
    pub tab4_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab5Spells")]
    pub tab5_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab6Spells")]
    pub tab6_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab7Spells")]
    pub tab7_spells: Vec<LayeredSpellId>,
    #[serde(rename = "Tab8Spells")]
    pub tab8_spells: Vec<LayeredSpellId>,
    #[serde(rename = "FillComps")]
    pub fill_comps: std::collections::HashMap<String, u32>,
    #[serde(rename = "SpellBookFilters")]
    pub spell_book_filters: u32,
    #[serde(rename = "OptionFlags")]
    pub option_flags: u32,
    #[serde(rename = "Unknown100_1")]
    pub unknown100_1: u32,
    #[serde(rename = "OptionStrings")]
    pub option_strings: std::collections::HashMap<String, String>,
    #[serde(rename = "GameplayOptions")]
    pub gameplay_options: Option<GameplayOptions>,
}

impl PlayerModule {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let flags = reader.read_u32()?;
        let options_raw = reader.read_u32()?;
        let options = character_options1_to_string(options_raw);

        // Flag 0x01: Shortcuts
        let shortcuts = if flags & 0x01 != 0 {
            read_packable_list_shortcut(reader)?
        } else {
            Vec::new()
        };

        // Tab spells are always present (not conditional on flags)
        let tab1_spells = read_packable_list_layered_spell(reader)?;
        let tab2_spells = read_packable_list_layered_spell(reader)?;
        let tab3_spells = read_packable_list_layered_spell(reader)?;
        let tab4_spells = read_packable_list_layered_spell(reader)?;
        let tab5_spells = read_packable_list_layered_spell(reader)?;
        let tab6_spells = read_packable_list_layered_spell(reader)?;
        let tab7_spells = read_packable_list_layered_spell(reader)?;
        let tab8_spells = read_packable_list_layered_spell(reader)?;

        // Flag 0x08: FillComps
        let fill_comps = if flags & 0x08 != 0 {
            read_packable_hash_table_uint(reader)?
        } else {
            std::collections::HashMap::new()
        };

        // Flag 0x20: SpellBookFilters
        let spell_book_filters = if flags & 0x20 != 0 {
            reader.read_u32()?
        } else {
            0
        };

        // Flag 0x40: OptionFlags
        let option_flags = if flags & 0x40 != 0 {
            reader.read_u32()?
        } else {
            0
        };

        // Flag 0x100: Unknown100_1 + OptionStrings
        let (unknown100_1, option_strings) = if flags & 0x100 != 0 {
            let unk = reader.read_u32()?;
            let strings = read_packable_hash_table_string(reader)?;
            (unk, strings)
        } else {
            (0, std::collections::HashMap::new())
        };

        // Flag 0x200: GameplayOptions
        let gameplay_options = if flags & 0x200 != 0 {
            Some(GameplayOptions::read(reader)?)
        } else {
            None
        };

        // Skip any remaining bytes (for flags we don't handle)
        let remaining = reader.remaining();
        if remaining > 0 {
            reader.read_bytes(remaining)?;
        }

        Ok(Self {
            flags,
            options,
            shortcuts,
            tab1_spells,
            tab2_spells,
            tab3_spells,
            tab4_spells,
            tab5_spells,
            tab6_spells,
            tab7_spells,
            tab8_spells,
            fill_comps,
            spell_book_filters,
            option_flags,
            unknown100_1,
            option_strings,
            gameplay_options,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortCutData {
    #[serde(rename = "Index")]
    pub index: u32,
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "SpellId")]
    pub spell_id: LayeredSpellId,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayeredSpellId {
    #[serde(rename = "Id")]
    pub id: u32,
    #[serde(rename = "Layer")]
    pub layer: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameplayOptions {
    #[serde(rename = "Size")]
    pub size: u32,
    #[serde(rename = "Unknown200_2")]
    pub unknown200_2: u8,
    #[serde(rename = "OptionPropertyCount")]
    pub option_property_count: u8,
    #[serde(rename = "OptionProperties")]
    pub option_properties: Vec<OptionProperty>,
}

impl GameplayOptions {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let size = reader.read_u32()?;
        let unknown200_2 = reader.read_u8()?;
        let option_property_count = reader.read_u8()?;

        let mut option_properties = Vec::new();
        for _ in 0..option_property_count {
            option_properties.push(OptionProperty::read(reader)?);
        }

        // Align to DWORD (4-byte boundary)
        let pos = reader.position();
        let align = (4 - (pos % 4)) % 4;
        if align > 0 {
            reader.read_bytes(align as usize)?;
        }

        Ok(Self {
            size,
            unknown200_2,
            option_property_count,
            option_properties,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OptionProperty {
    #[serde(rename = "Type")]
    pub prop_type: u32,
    #[serde(rename = "Unknown_a")]
    pub unknown_a: u32,
    #[serde(rename = "WindowOptions")]
    pub window_options: Vec<WindowOption>,
    #[serde(rename = "Unknown_k")]
    pub unknown_k: u32,
    #[serde(
        rename = "ActiveOpacity",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub active_opacity: f32,
    #[serde(rename = "Unknown_l")]
    pub unknown_l: u32,
    #[serde(
        rename = "InactiveOpacity",
        serialize_with = "crate::serialization::serialize_f32"
    )]
    pub inactive_opacity: f32,
}

impl OptionProperty {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let prop_type = reader.read_u32()?;

        let mut unknown_a = 0u32;
        let mut window_options = Vec::new();
        let mut unknown_k = 0u32;
        let mut active_opacity = 0.0f32;
        let mut unknown_l = 0u32;
        let mut inactive_opacity = 0.0f32;

        match prop_type {
            0x1000008c => {
                unknown_a = reader.read_u32()?;
                window_options = read_packable_list_window_option(reader)?;
            }
            0x10000081 => {
                unknown_k = reader.read_u32()?;
                active_opacity = reader.read_f32()?;
            }
            0x10000080 => {
                unknown_l = reader.read_u32()?;
                inactive_opacity = reader.read_f32()?;
            }
            _ => {}
        }

        Ok(Self {
            prop_type,
            unknown_a,
            window_options,
            unknown_k,
            active_opacity,
            unknown_l,
            inactive_opacity,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowOption {
    #[serde(rename = "Type_a")]
    pub type_a: u32,
    #[serde(rename = "Unknown_b")]
    pub unknown_b: u8,
    #[serde(rename = "PropertyCount")]
    pub property_count: u8,
    #[serde(rename = "Properties")]
    pub properties: Vec<WindowProperty>,
}

impl WindowOption {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let type_a = reader.read_u32()?;

        let (unknown_b, property_count, properties) = if type_a == 0x1000008b {
            let unk = reader.read_u8()?;
            let count = reader.read_u8()?;
            let mut props = Vec::new();
            for _ in 0..count {
                props.push(WindowProperty::read(reader)?);
            }
            (unk, count, props)
        } else {
            (0, 0, Vec::new())
        };

        Ok(Self {
            type_a,
            unknown_b,
            property_count,
            properties,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowProperty {
    #[serde(rename = "Key_a")]
    pub key_a: u32,
    #[serde(rename = "Unknown_c")]
    pub unknown_c: u32,
    #[serde(rename = "TitleSource")]
    pub title_source: u8,
    #[serde(rename = "StringId")]
    pub string_id: u32,
    #[serde(rename = "FileId")]
    pub file_id: u32,
    #[serde(rename = "Value_a")]
    pub value_a: Option<String>,
    #[serde(rename = "Unknown_1b")]
    pub unknown_1b: u32,
    #[serde(rename = "Unknown_1c")]
    pub unknown_1c: u16,
    #[serde(rename = "Unknown_d")]
    pub unknown_d: u32,
    #[serde(rename = "Value_d")]
    pub value_d: u8,
    #[serde(rename = "Unknown_e")]
    pub unknown_e: u32,
    #[serde(rename = "Value_e")]
    pub value_e: u32,
    #[serde(rename = "Unknown_f")]
    pub unknown_f: u32,
    #[serde(rename = "Value_f")]
    pub value_f: u32,
    #[serde(rename = "Unknown_h")]
    pub unknown_h: u32,
    #[serde(rename = "Value_h")]
    pub value_h: u32,
    #[serde(rename = "Unknown_i")]
    pub unknown_i: u32,
    #[serde(rename = "Value_i")]
    pub value_i: u32,
    #[serde(rename = "Unknown_j")]
    pub unknown_j: u32,
    #[serde(rename = "Value_j")]
    pub value_j: u64,
}

impl WindowProperty {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let key_a = reader.read_u32()?;

        let mut unknown_c = 0u32;
        let mut title_source = 0u8;
        let mut string_id = 0u32;
        let mut file_id = 0u32;
        let mut value_a = None;
        let mut unknown_1b = 0u32;
        let mut unknown_1c = 0u16;
        let mut unknown_d = 0u32;
        let mut value_d = 0u8;
        let mut unknown_e = 0u32;
        let mut value_e = 0u32;
        let mut unknown_f = 0u32;
        let mut value_f = 0u32;
        let mut unknown_h = 0u32;
        let mut value_h = 0u32;
        let mut unknown_i = 0u32;
        let mut value_i = 0u32;
        let mut unknown_j = 0u32;
        let mut value_j = 0u64;

        match key_a {
            0x1000008d => {
                unknown_c = reader.read_u32()?;
                title_source = reader.read_u8()?;
                if title_source == 0 {
                    string_id = reader.read_u32()?;
                    file_id = reader.read_u32()?;
                } else if title_source == 1 {
                    value_a = Some(reader.read_string16l()?);
                }
                unknown_1b = reader.read_u32()?;
                unknown_1c = reader.read_u16()?;
            }
            0x1000008a => {
                unknown_d = reader.read_u32()?;
                value_d = reader.read_u8()?;
            }
            0x10000089 => {
                unknown_e = reader.read_u32()?;
                value_e = reader.read_u32()?;
            }
            0x10000088 => {
                unknown_f = reader.read_u32()?;
                value_f = reader.read_u32()?;
            }
            0x10000087 => {
                unknown_h = reader.read_u32()?;
                value_h = reader.read_u32()?;
            }
            0x10000086 => {
                unknown_i = reader.read_u32()?;
                value_i = reader.read_u32()?;
            }
            0x1000007f => {
                unknown_j = reader.read_u32()?;
                value_j = reader.read_u64()?;
            }
            _ => {}
        }

        Ok(Self {
            key_a,
            unknown_c,
            title_source,
            string_id,
            file_id,
            value_a,
            unknown_1b,
            unknown_1c,
            unknown_d,
            value_d,
            unknown_e,
            value_e,
            unknown_f,
            value_f,
            unknown_h,
            value_h,
            unknown_i,
            value_i,
            unknown_j,
            value_j,
        })
    }
}

// ============================================================================
// Helper functions for reading packable lists/tables
// ============================================================================

fn read_packable_list_shortcut(reader: &mut BinaryReader) -> Result<Vec<ShortCutData>> {
    let count = reader.read_u32()?;
    let mut result = Vec::new();
    for _ in 0..count {
        let index = reader.read_u32()?;
        let object_id = reader.read_u32()?;
        let id = reader.read_u16()? as u32;
        let layer = reader.read_u16()?;
        result.push(ShortCutData {
            index,
            object_id,
            spell_id: LayeredSpellId { id, layer },
        });
    }
    Ok(result)
}

fn read_packable_list_layered_spell(reader: &mut BinaryReader) -> Result<Vec<LayeredSpellId>> {
    let count = reader.read_u32()?;
    let mut result = Vec::new();
    for _ in 0..count {
        let id = reader.read_u16()? as u32;
        let layer = reader.read_u16()?;
        result.push(LayeredSpellId { id, layer });
    }
    Ok(result)
}

fn read_packable_hash_table_uint(
    reader: &mut BinaryReader,
) -> Result<std::collections::HashMap<String, u32>> {
    let count = reader.read_u16()?;
    let _max_size = reader.read_u16()?;
    let mut result = std::collections::HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_u32()?;
        result.insert(key.to_string(), value);
    }
    Ok(result)
}

fn read_packable_hash_table_string(
    reader: &mut BinaryReader,
) -> Result<std::collections::HashMap<String, String>> {
    let count = reader.read_u16()?;
    let _max_size = reader.read_u16()?;
    let mut result = std::collections::HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_string16l()?;
        result.insert(key.to_string(), value);
    }
    Ok(result)
}

fn read_packable_list_window_option(reader: &mut BinaryReader) -> Result<Vec<WindowOption>> {
    let count = reader.read_u32()?;
    let mut result = Vec::new();
    for _ in 0..count {
        result.push(WindowOption::read(reader)?);
    }
    Ok(result)
}

fn character_options1_to_string(flags: u32) -> String {
    let mut options = Vec::new();
    if flags & 0x00000002 != 0 {
        options.push("AutoRepeatAttack");
    }
    if flags & 0x00000004 != 0 {
        options.push("IgnoreAllegianceRequests");
    }
    if flags & 0x00000008 != 0 {
        options.push("IgnoreFellowshipRequests");
    }
    if flags & 0x00000040 != 0 {
        options.push("AllowGive");
    }
    if flags & 0x00000080 != 0 {
        options.push("ViewCombatTarget");
    }
    if flags & 0x00000100 != 0 {
        options.push("ShowTooltips");
    }
    if flags & 0x00000200 != 0 {
        options.push("UseDeception");
    }
    if flags & 0x00000400 != 0 {
        options.push("ToggleRun");
    }
    if flags & 0x00000800 != 0 {
        options.push("StayInChatMode");
    }
    if flags & 0x00001000 != 0 {
        options.push("AdvancedCombatUI");
    }
    if flags & 0x00002000 != 0 {
        options.push("AutoTarget");
    }
    if flags & 0x00008000 != 0 {
        options.push("VividTargetingIndicator");
    }
    if flags & 0x00010000 != 0 {
        options.push("DisableMostWeatherEffects");
    }
    if flags & 0x00020000 != 0 {
        options.push("IgnoreTradeRequests");
    }
    if flags & 0x00040000 != 0 {
        options.push("FellowshipShareXP");
    }
    if flags & 0x00080000 != 0 {
        options.push("AcceptLootPermits");
    }
    if flags & 0x00100000 != 0 {
        options.push("FellowshipShareLoot");
    }
    if flags & 0x00200000 != 0 {
        options.push("SideBySideVitals");
    }
    if flags & 0x00400000 != 0 {
        options.push("CoordinatesOnRadar");
    }
    if flags & 0x00800000 != 0 {
        options.push("SpellDuration");
    }
    if flags & 0x02000000 != 0 {
        options.push("DisableHouseRestrictionEffects");
    }
    if flags & 0x04000000 != 0 {
        options.push("DragItemOnPlayerOpensSecureTrade");
    }
    if flags & 0x08000000 != 0 {
        options.push("DisplayAllegianceLogonNotifications");
    }
    if flags & 0x10000000 != 0 {
        options.push("UseChargeAttack");
    }
    if flags & 0x20000000 != 0 {
        options.push("AutoAcceptFellowRequest");
    }
    if flags & 0x40000000 != 0 {
        options.push("HearAllegianceChat");
    }
    if flags & 0x80000000 != 0 {
        options.push("UseCraftSuccessDialog");
    }
    options.join(", ")
}
