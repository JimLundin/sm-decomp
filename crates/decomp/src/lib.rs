//! Derived from snesrev-sm DecompressToMem, MIT, copyright 2023 snesrev / 2021 elzo_d.
//! Full upstream notices are retained in LICENSE.upstream.txt.
//! Bounded byte-addressed replacement for the upstream RAM decompressor.
//! The current pilot supports literal streams and termination only.
use std::collections::BTreeSet;

pub const WRAM_SIZE: usize = 0x20000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompressError {
    InvalidRomExtent,
    InvalidWramExtent,
    InvalidSource(u32),
    WramBounds(usize),
    BudgetExceeded,
    UnsupportedCommand(u8),
}

pub struct RomView<'a>(&'a [u8]);
impl<'a> RomView<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecompressError> {
        if bytes.is_empty() || bytes.len() > 0x400000 {
            return Err(DecompressError::InvalidRomExtent);
        }
        Ok(Self(bytes))
    }
    pub fn read(&self, address: u32) -> Result<u8, DecompressError> {
        if address > 0xffffff || address & 0x8000 == 0 {
            return Err(DecompressError::InvalidSource(address));
        }
        let offset = (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff;
        self.0.get(offset as usize).copied().ok_or(DecompressError::InvalidSource(address))
    }
}

/// The canonical full WRAM slice preserves aliases as byte offsets into one allocation.
pub struct WramView<'a>(&'a mut [u8]);
impl<'a> WramView<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self, DecompressError> {
        if bytes.len() != WRAM_SIZE {
            return Err(DecompressError::InvalidWramExtent);
        }
        Ok(Self(bytes))
    }
    pub fn read(&self, offset: usize) -> Result<u8, DecompressError> {
        self.0.get(offset).copied().ok_or(DecompressError::WramBounds(offset))
    }
    pub fn write(&mut self, offset: usize, value: u8) -> Result<(), DecompressError> {
        *self.0.get_mut(offset).ok_or(DecompressError::WramBounds(offset))? = value;
        Ok(())
    }
    pub fn read_u16_le(&self, offset: usize) -> Result<u16, DecompressError> {
        Ok(u16::from_le_bytes([
            self.read(offset)?,
            self.read(offset.checked_add(1).ok_or(DecompressError::WramBounds(offset))?)?,
        ]))
    }
    pub fn write_u16_le(&mut self, offset: usize, value: u16) -> Result<(), DecompressError> {
        let next = offset.checked_add(1).ok_or(DecompressError::WramBounds(offset))?;
        self.read(next)?; // Validate both byte accesses before either store.
        let [lo, hi] = value.to_le_bytes();
        self.write(offset, lo)?;
        self.write(next, hi)
    }
}

#[derive(Debug)]
pub struct DecompressResult {
    pub written: usize,
    pub final_source: u32,
    pub cases: BTreeSet<&'static str>,
}

pub fn decompress_to_mem(
    rom: &RomView<'_>,
    wram: &mut WramView<'_>,
    source: u32,
    destination: usize,
) -> Result<DecompressResult, DecompressError> {
    if destination >= WRAM_SIZE {
        return Err(DecompressError::WramBounds(destination));
    }
    let mut cursor = source;
    let mut reads = 0;
    let mut next = || -> Result<u8, DecompressError> {
        if reads >= 0x800000 {
            return Err(DecompressError::BudgetExceeded);
        }
        reads += 1;
        let byte = rom.read(cursor)?;
        let old = cursor;
        cursor = cursor.wrapping_add(1);
        if old & 0xffff == 0xffff {
            cursor = cursor.wrapping_add(0x8000);
        }
        Ok(byte)
    };
    let mut written = 0;
    let mut cases = BTreeSet::new();
    loop {
        let header = next()?;
        if header == 0xff {
            cases.insert("terminator");
            break;
        }
        let (command, length, extended) = if header & 0xe0 == 0xe0 {
            ((header.wrapping_mul(8)) & 0xe0, (((header & 3) as usize) << 8 | next()? as usize) + 1, true)
        } else {
            (header & 0xe0, (header & 31) as usize + 1, false)
        };
        if command != 0 {
            return Err(DecompressError::UnsupportedCommand(command));
        }
        cases.insert(if extended { "literal-extended" } else { "literal-short" });
        for _ in 0..length {
            wram.write(destination + written, next()?)?;
            written += 1;
        }
    }
    Ok(DecompressResult { written, final_source: cursor, cases })
}
