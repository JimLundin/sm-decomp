//! Derived from snesrev-sm DecompressToMem, MIT, copyright 2023 snesrev / 2021 elzo_d.
//! Full upstream notices are retained in LICENSE.upstream.txt.
//! Bounded byte-addressed replacement for the upstream RAM decompressor.
//! Valid streams support every RAM command; malformed accesses return bounded errors.
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
pub struct WramView<'a> {
    bytes: &'a mut [u8],
    observer: Option<&'a mut dyn FnMut(usize, u8)>,
}
impl<'a> WramView<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self, DecompressError> {
        if bytes.len() != WRAM_SIZE {
            return Err(DecompressError::InvalidWramExtent);
        }
        Ok(Self { bytes, observer: None })
    }
    /// Reports every successful byte store, including repeated stores of the same value.
    pub fn with_observer(bytes: &'a mut [u8], observer: &'a mut dyn FnMut(usize, u8)) -> Result<Self, DecompressError> {
        let mut view = Self::new(bytes)?;
        view.observer = Some(observer);
        Ok(view)
    }
    pub fn read(&self, offset: usize) -> Result<u8, DecompressError> {
        self.bytes.get(offset).copied().ok_or(DecompressError::WramBounds(offset))
    }
    pub fn write(&mut self, offset: usize, value: u8) -> Result<(), DecompressError> {
        *self.bytes.get_mut(offset).ok_or(DecompressError::WramBounds(offset))? = value;
        if let Some(observer) = &mut self.observer { observer(offset, value); }
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
    let mut source_cases = BTreeSet::new();
    let mut next = |site: &'static str| -> Result<u8, DecompressError> {
        if reads >= 0x800000 {
            return Err(DecompressError::BudgetExceeded);
        }
        reads += 1;
        let byte = rom.read(cursor)?;
        let old = cursor;
        cursor = cursor.wrapping_add(1);
        if old & 0xffff == 0xffff {
            cursor = cursor.wrapping_add(0x8000);
            source_cases.insert("source-rollover");
            source_cases.insert(site);
        }
        Ok(byte)
    };
    let mut written = 0;
    let mut cases = BTreeSet::new();
    loop {
        let header = next("header-rollover")?;
        if header == 0xff {
            cases.insert("terminator");
            break;
        }
        let (command, length, extended) = if header & 0xe0 == 0xe0 {
            (
                (header.wrapping_mul(8)) & 0xe0,
                (((header & 3) as usize) << 8 | next("extended-header-rollover")? as usize) + 1,
                true,
            )
        } else {
            (header & 0xe0, (header & 31) as usize + 1, false)
        };
        let mode = match command {
            0 => "literal",
            0x20 => "byte-fill",
            0x40 => "word-fill",
            0x60 => "increment-fill",
            0x80 => "absolute-copy",
            0xa0 => "absolute-xor",
            0xc0 => "relative-copy",
            0xe0 => "relative-xor",
            _ => return Err(DecompressError::UnsupportedCommand(command)),
        };
        cases.insert(mode);
        cases.insert(match (command, extended) {
            (0, false) => "literal-short",
            (0, true) => "literal-extended",
            (0x20, false) => "byte-fill-short",
            (0x20, true) => "byte-fill-extended",
            (0x40, false) => "word-fill-short",
            (0x40, true) => "word-fill-extended",
            (0x60, false) => "increment-fill-short",
            (0x60, true) => "increment-fill-extended",
            (0x80, false) => "absolute-copy-short",
            (0x80, true) => "absolute-copy-extended",
            (0xa0, false) => "absolute-xor-short",
            (0xa0, true) => "absolute-xor-extended",
            (0xc0, false) => "relative-copy-short",
            (0xc0, true) => "relative-copy-extended",
            (0xe0, true) => "relative-xor-extended",
            _ => unreachable!(),
        });
        if command == 0xe0 && length == 768 {
            cases.insert("relative-xor-max");
        }
        if length == 1 {
            cases.insert(if extended { "extended-min" } else { "short-min" });
        }
        if length == if extended { 1024 } else { 32 } {
            cases.insert(if extended { "extended-max" } else { "short-max" });
        }
        if command & 0x80 != 0 {
            let relative_offset = if command >= 0xc0 {
                written as isize - isize::from(next("operand-rollover")?)
            } else {
                u16::from_le_bytes([next("operand-rollover")?, next("operand-rollover")?]) as isize
            };
            // Signed relative references may point into initialized WRAM before destination.
            let absolute = destination as isize + relative_offset;
            if absolute < 0 {
                return Err(DecompressError::WramBounds(usize::MAX));
            }
            let read_start = absolute as usize;
            if read_start < destination {
                cases.insert("preexisting-reference");
            }
            if read_start < destination + written && read_start + length > destination + written {
                cases.insert("overlap");
            }
            for i in 0..length {
                let value = wram.read(read_start + i)? ^ if command & 0x20 != 0 { 0xff } else { 0 };
                wram.write(destination + written, value)?;
                written += 1;
            }
            continue;
        }
        let first = if command != 0 { next("operand-rollover")? } else { 0 };
        let second = if command == 0x40 { next("operand-rollover")? } else { 0 };
        if command == 0x40 && length % 2 == 1 {
            cases.insert("word-odd");
        }
        for i in 0..length {
            let value = match command {
                0 => next("literal-data-rollover")?,
                0x20 => first,
                0x40 => {
                    if i % 2 == 0 {
                        first
                    } else {
                        second
                    }
                }
                0x60 => {
                    if usize::from(first) + i > 255 {
                        cases.insert("byte-wrap");
                    }
                    first.wrapping_add(i as u8)
                }
                _ => unreachable!(),
            };
            wram.write(destination + written, value)?;
            written += 1;
        }
    }
    cases.extend(source_cases);
    Ok(DecompressResult { written, final_source: cursor, cases })
}

pub mod ffi;
