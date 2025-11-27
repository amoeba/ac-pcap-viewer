use crate::properties;
use crate::protocol::BinaryReader;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QualitiesPrivateUpdateInt {
    #[serde(rename = "Sequence")]
    pub sequence: u8,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl QualitiesPrivateUpdateInt {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let sequence = reader.read_u8()?;
        let key_raw = reader.read_u32()?;
        let value = reader.read_i32()?;

        Ok(Self {
            sequence,
            key: properties::property_int_name(key_raw),
            value,
            opcode: 0x02CD,
            message_type: "Qualities_PrivateUpdateInt".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QualitiesPrivateUpdateAttribute2ndLevel {
    #[serde(rename = "Sequence")]
    pub sequence: u8,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: u32,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl QualitiesPrivateUpdateAttribute2ndLevel {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let sequence = reader.read_u8()?;
        let key_raw = reader.read_u32()?;
        let value = reader.read_u32()?;

        Ok(Self {
            sequence,
            key: vital_name(key_raw),
            value,
            opcode: 0x02E9,
            message_type: "Qualities_PrivateUpdateAttribute2ndLevel".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QualitiesUpdateInt {
    #[serde(rename = "Sequence")]
    pub sequence: u8,
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: i32,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl QualitiesUpdateInt {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let sequence = reader.read_u8()?;
        let object_id = reader.read_u32()?;
        let key_raw = reader.read_u32()?;
        let value = reader.read_i32()?;

        Ok(Self {
            sequence,
            object_id,
            key: properties::property_int_name(key_raw),
            value,
            opcode: 0x02CE,
            message_type: "Qualities_UpdateInt".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QualitiesUpdateInstanceId {
    #[serde(rename = "Sequence")]
    pub sequence: u8,
    #[serde(rename = "ObjectId")]
    pub object_id: u32,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: u32,
    #[serde(rename = "OpCode")]
    pub opcode: u32,
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MessageDirection")]
    pub message_direction: String,
}

impl QualitiesUpdateInstanceId {
    pub fn read(reader: &mut BinaryReader) -> Result<Self> {
        let sequence = reader.read_u8()?;
        let object_id = reader.read_u32()?;
        let key_raw = reader.read_u32()?;
        let value = reader.read_u32()?;

        Ok(Self {
            sequence,
            object_id,
            key: property_instance_id_name(key_raw),
            value,
            opcode: 0x02DA,
            message_type: "Qualities_UpdateInstanceId".to_string(),
            message_direction: "ServerToClient".to_string(),
        })
    }
}

// Helper functions

fn vital_name(key: u32) -> String {
    match key {
        2 => "Health".to_string(),
        4 => "Stamina".to_string(),
        6 => "Mana".to_string(),
        _ => format!("Vital_{key}"),
    }
}

fn property_instance_id_name(key: u32) -> String {
    match key {
        1 => "Owner".to_string(),
        2 => "Container".to_string(),
        3 => "Wielder".to_string(),
        _ => format!("PropertyInstanceId_{key}"),
    }
}
