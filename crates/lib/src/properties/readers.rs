use crate::protocol::BinaryReader;
use anyhow::Result;
use std::collections::HashMap;

use super::profiles::LayeredSpellId;
use super::property_names::*;

/// Read a PackableHashTable<u32, i32> (property int)
pub fn read_int_properties(reader: &mut BinaryReader) -> Result<HashMap<String, i32>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i32()?;
        map.insert(property_int_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, i64> (property int64)
pub fn read_int64_properties(reader: &mut BinaryReader) -> Result<HashMap<String, i64>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i64()?;
        map.insert(property_int64_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, bool> (property bool) - bools stored as i32
pub fn read_bool_properties(reader: &mut BinaryReader) -> Result<HashMap<String, bool>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_i32()? != 0;
        map.insert(property_bool_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, f64> (property float) - f64 in appraisal
pub fn read_float_properties(reader: &mut BinaryReader) -> Result<HashMap<String, f64>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_f64()?;
        map.insert(property_float_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, string> (property string)
pub fn read_string_properties(reader: &mut BinaryReader) -> Result<HashMap<String, String>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_string16l()?;
        map.insert(property_string_name(key), value);
    }
    Ok(map)
}

/// Read a PackableHashTable<u32, u32> (property DataId)
pub fn read_dataid_properties(reader: &mut BinaryReader) -> Result<HashMap<String, u32>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut map = HashMap::new();
    for _ in 0..count {
        let key = reader.read_u32()?;
        let value = reader.read_u32()?;
        map.insert(property_dataid_name(key), value);
    }
    Ok(map)
}

/// Read spell book - PackableHashTable<u32, float> where key is LayeredSpellId packed into u32
/// LayeredSpellId has: u16 Id (spell id) + u16 Layer (buff layer)
/// The float value is castable_by_huntable flag - always 0 in practice
pub fn read_spell_book(reader: &mut BinaryReader) -> Result<Vec<LayeredSpellId>> {
    let count = reader.read_u16()? as usize;
    let _max_size = reader.read_u16()?;
    let mut spells = Vec::new();

    for _ in 0..count {
        let key = reader.read_u32()?;
        let _castable_by_huntable = reader.read_f32()?;

        // Unpack LayeredSpellId from u32 key
        let id = (key & 0xFFFF) as u16;
        let layer = ((key >> 16) & 0xFFFF) as u16;

        spells.push(LayeredSpellId { id, layer });
    }

    Ok(spells)
}
