use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>,
}

impl Message {
    pub fn parse(data: &[u8]) -> Result<Self> {
        Ok(Self {
            data: data.to_vec(),
        })
    }
}

mod hex_bytes {
    use serde::Serializer;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        serializer.serialize_str(&hex_string)
    }
}
