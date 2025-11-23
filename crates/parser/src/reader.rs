use anyhow::Result;
use std::io::{Cursor, Read};

/// A binary reader that provides little-endian reading of AC protocol data
#[allow(dead_code)]
pub struct BinaryReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

#[allow(dead_code)]
impl<'a> BinaryReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    pub fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        let len = self.cursor.get_ref().len();
        len.saturating_sub(pos)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.cursor.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u32()? != 0)
    }

    pub fn read_bool_byte(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Read a packed string (16-bit length prefix followed by bytes, aligned to 4 bytes)
    pub fn read_string16l(&mut self) -> Result<String> {
        let len = self.read_u16()? as usize;
        if len == 0 {
            // Align to 4 bytes
            self.read_u16()?;
            return Ok(String::new());
        }
        let bytes = self.read_bytes(len)?;
        // Align to 4 bytes
        let total = 2 + len;
        let padding = (4 - (total % 4)) % 4;
        if padding > 0 {
            self.read_bytes(padding)?;
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Read a compressed uint (1-4 bytes depending on value)
    pub fn read_compressed_uint(&mut self) -> Result<u32> {
        let first = self.read_u8()? as u32;
        if first & 0x80 == 0 {
            // 1 byte
            Ok(first)
        } else if first & 0x40 == 0 {
            // 2 bytes
            let second = self.read_u8()? as u32;
            Ok(((first & 0x7F) << 8) | second)
        } else {
            // 4 bytes
            let b2 = self.read_u8()? as u32;
            let b3 = self.read_u8()? as u32;
            let b4 = self.read_u8()? as u32;
            Ok(((first & 0x3F) << 24) | (b2 << 16) | (b3 << 8) | b4)
        }
    }

    /// Read a string with 16-bit length prefix where -1 indicates 32-bit length
    pub fn read_string16l_ex(&mut self) -> Result<String> {
        let start = self.position();
        let len = self.read_i16()?;

        let actual_len = if len == -1 {
            self.read_i32()? as usize
        } else if len < 0 {
            // Invalid negative length (not -1), probably misaligned
            anyhow::bail!("Invalid string length: {}", len);
        } else {
            len as usize
        };

        // Sanity check - strings shouldn't be huge
        if actual_len > 10000 {
            anyhow::bail!(
                "String length too large: {} (likely misaligned)",
                actual_len
            );
        }

        if actual_len == 0 {
            // Align to 4 bytes from start
            let total = (self.position() - start) as usize;
            let padding = (4 - (total % 4)) % 4;
            if padding > 0 {
                self.read_bytes(padding)?;
            }
            return Ok(String::new());
        }

        let bytes = self.read_bytes(actual_len)?;

        // Align to 4 bytes from start
        let total = (self.position() - start) as usize;
        let padding = (4 - (total % 4)) % 4;
        if padding > 0 {
            self.read_bytes(padding)?;
        }

        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.cursor.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    /// Read a PackedDWORD - variable length format used in ObjDesc
    /// If high bit of first u16 is set, it's a 4-byte value
    pub fn read_packed_dword(&mut self) -> Result<u32> {
        let tmp = self.read_u16()? as i32;
        if (tmp & 0x8000) != 0 {
            // 4-byte format: high bit indicates extended
            let high = ((tmp << 16) & 0x7FFFFFFF) as u32;
            let low = self.read_u16()? as u32;
            Ok(high | low)
        } else {
            Ok(tmp as u32)
        }
    }

    /// Align cursor position to 4-byte boundary
    pub fn align4(&mut self) -> Result<()> {
        let pos = self.position() as usize;
        let padding = (4 - (pos % 4)) % 4;
        if padding > 0 {
            self.read_bytes(padding)?;
        }
        Ok(())
    }
}
