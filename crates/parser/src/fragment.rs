use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u16)]
pub enum FragmentGroup {
    Event = 5,
    Private = 9,
    Object = 10,
}

impl FragmentGroup {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            5 => Some(FragmentGroup::Event),
            9 => Some(FragmentGroup::Private),
            10 => Some(FragmentGroup::Object),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct FragmentHeader {
    pub sequence: u32,
    pub id: u32,
    pub count: u16,
    pub size: u16,
    pub index: u16,
    pub group: Option<FragmentGroup>,
}

impl FragmentHeader {
    pub const SIZE: usize = 16;
    pub const CHUNK_SIZE: usize = 448;

    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            bail!("Not enough data for fragment header");
        }

        let sequence = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let count = u16::from_le_bytes([data[8], data[9]]);
        let size = u16::from_le_bytes([data[10], data[11]]);
        let index = u16::from_le_bytes([data[12], data[13]]);
        let group_raw = u16::from_le_bytes([data[14], data[15]]);
        let group = FragmentGroup::from_u16(group_raw);

        Ok(Self {
            sequence,
            id,
            count,
            size,
            index,
            group,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Fragment {
    pub header: FragmentHeader,
    pub sequence: u32,
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>,
    pub count: usize,
    pub received: usize,
    pub length: usize,
    #[serde(skip)]
    chunks: Vec<bool>,
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

impl Fragment {
    pub fn new(sequence: u32, count: u16) -> Self {
        let count_usize = count as usize;
        Self {
            header: FragmentHeader {
                sequence,
                id: 0,
                count,
                size: 0,
                index: 0,
                group: None,
            },
            sequence,
            data: vec![0; count_usize * FragmentHeader::CHUNK_SIZE],
            count: count_usize,
            received: 0,
            length: 0,
            chunks: vec![false; count_usize],
        }
    }

    pub fn add_chunk(&mut self, data: &[u8], index: usize) {
        if index < self.chunks.len() && !self.chunks[index] {
            let start = index * FragmentHeader::CHUNK_SIZE;
            let end = start + data.len();
            if end <= self.data.len() {
                self.data[start..end].copy_from_slice(data);
                self.received += 1;
                self.length += data.len();
                self.chunks[index] = true;
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.count == self.received
    }
}
