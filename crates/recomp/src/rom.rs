use anyhow::{bail, Result};

/// Super Metroid is LoROM, 3 MiB. Code lives in banks $80-$B4 (FastROM mirror).
pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let mut data = std::fs::read(path)?;
        if data.len() % 1024 == 512 {
            data.drain(..512); // copier header
        }
        if data.len() != 3 * 1024 * 1024 {
            bail!(
                "{}: expected a 3 MiB Super Metroid ROM, got {} bytes",
                path.display(),
                data.len()
            );
        }
        Ok(Self { data })
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// LoROM: banks $00-$7D / $80-$FF, ROM visible at $8000-$FFFF.
    pub fn offset(&self, addr: u32) -> Option<usize> {
        let bank = (addr >> 16) & 0x7f;
        let lo = addr & 0xffff;
        if lo < 0x8000 || bank >= 0x7e {
            return None;
        }
        let off = (bank as usize) << 15 | (lo as usize & 0x7fff);
        (off < self.data.len()).then_some(off)
    }

    pub fn byte(&self, addr: u32) -> Option<u8> {
        self.offset(addr).map(|o| self.data[o])
    }
}
