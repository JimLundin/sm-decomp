use std::collections::BTreeMap;
use std::ops::Range;

use data::assets::audio::{Envelope, Instrument, SampleId, SampleSource, TimbreId};
use data::rom::{Lorom, RomAddress, sha256};

use super::ImportError;

#[derive(Clone, Copy)]
pub(super) struct Region<'rom> {
    pub(super) start: usize,
    bytes: &'rom [u8],
}

impl Region<'_> {
    pub(super) fn get<const N: usize>(self, at: usize) -> Result<[u8; N], ImportError> {
        self.slice(at..at.saturating_add(N))?
            .bytes
            .try_into()
            .map_err(|_| ImportError::Pointer)
    }

    pub(super) fn word(self, at: usize) -> Result<usize, ImportError> {
        Ok(usize::from(u16::from_le_bytes(self.get(at)?)))
    }

    pub(super) fn slice(self, range: Range<usize>) -> Result<Self, ImportError> {
        let start = range
            .start
            .checked_sub(self.start)
            .ok_or(ImportError::Pointer)?;
        let end = range
            .end
            .checked_sub(self.start)
            .ok_or(ImportError::Pointer)?;
        let bytes = self.bytes.get(start..end).ok_or(ImportError::Pointer)?;
        Ok(Self {
            start: range.start,
            bytes,
        })
    }

    pub(super) const fn end(self) -> usize {
        self.start.saturating_add(self.bytes.len())
    }
}

pub(super) struct Reader<'rom> {
    pub(super) region: Region<'rom>,
    pub(super) at: usize,
}

impl Reader<'_> {
    pub(super) fn take<const N: usize>(&mut self) -> Result<[u8; N], ImportError> {
        let value = self.region.get(self.at)?;
        self.at = self.at.saturating_add(N);
        Ok(value)
    }
}

/// Borrow the block ranges in ROM. No writable audio memory or driver image is built.
pub(super) struct Upload<'rom>(Vec<Region<'rom>>);

impl<'rom> Upload<'rom> {
    pub(super) fn read(rom: Lorom<'rom>, pointer: RomAddress) -> Result<Self, ImportError> {
        let address = rom.read(pointer, 3).map_err(ImportError::Address)?;
        let address = RomAddress::from_long(address.try_into().map_err(|_| ImportError::Upload)?);
        let bytes = rom.from(address).map_err(ImportError::Address)?;
        Self::parse(bytes)
    }

    pub(super) fn parse(mut bytes: &'rom [u8]) -> Result<Self, ImportError> {
        let mut blocks: Vec<Region<'_>> = Vec::new();
        loop {
            let ([lo, hi, dest_lo, dest_hi], tail) =
                bytes.split_first_chunk::<4>().ok_or(ImportError::Upload)?;
            let count = usize::from(u16::from_le_bytes([*lo, *hi]));
            if count == 0 {
                return Ok(Self(blocks));
            }
            let start = usize::from(u16::from_le_bytes([*dest_lo, *dest_hi]));
            let end = start.saturating_add(count);
            if end > 0x1_0000
                || blocks
                    .iter()
                    .any(|block| block.start < end && block.end() > start)
            {
                return Err(ImportError::Upload);
            }
            let (payload, rest) = tail.split_at_checked(count).ok_or(ImportError::Upload)?;
            blocks.push(Region {
                start,
                bytes: payload,
            });
            bytes = rest;
        }
    }

    pub(super) fn slice(&self, range: Range<usize>) -> Result<Region<'rom>, ImportError> {
        self.0
            .iter()
            .find_map(|block| block.slice(range.clone()).ok())
            .ok_or(ImportError::Pointer)
    }

    pub(super) fn overlaps(&self, range: &Range<usize>) -> bool {
        self.0
            .iter()
            .any(|block| block.start < range.end && range.start < block.end())
    }

    pub(super) fn block(&self, start: usize) -> Result<Region<'rom>, ImportError> {
        self.0
            .iter()
            .find(|block| block.start == start)
            .copied()
            .ok_or(ImportError::Pointer)
    }
}

pub(super) struct Resources<'rom, 'source> {
    common: &'source Upload<'rom>,
    area: &'source Upload<'rom>,
    timbres: BTreeMap<u8, TimbreId>,
    sample_ids: BTreeMap<u8, SampleId>,
    pub(super) instruments: Vec<Instrument>,
    pub(super) samples: Vec<SampleSource>,
}

const INSTRUMENTS: usize = 0x6c00;
const DIRECTORY: usize = 0x6d00;
const SAMPLE_DATA: usize = 0x6e00;
const SAMPLE_ENTRY: usize = 4;
const INSTRUMENT_ENTRY: usize = 6;
const BLOCK_BYTES: usize = 9;
const FRAMES_PER_BLOCK: u32 = 16;

impl<'rom, 'source> Resources<'rom, 'source> {
    pub(super) const fn new(common: &'source Upload<'rom>, area: &'source Upload<'rom>) -> Self {
        Self {
            common,
            area,
            timbres: BTreeMap::new(),
            sample_ids: BTreeMap::new(),
            instruments: Vec::new(),
            samples: Vec::new(),
        }
    }

    pub(super) fn instrument(&mut self, index: u8) -> Result<TimbreId, ImportError> {
        if let Some(&id) = self.timbres.get(&index) {
            return Ok(id);
        }
        let at = entry::<INSTRUMENT_ENTRY>(INSTRUMENTS, DIRECTORY, index)?;
        let [sample, attack_decay, sustain, gain, scale_hi, scale_lo] = self.read(at)?;
        if sample & 0x80 != 0 {
            return Err(ImportError::Resource);
        }
        let sample = self.sample(sample)?;
        let pitch_scale = u16::from_be_bytes([scale_hi, scale_lo]);
        let envelope = Envelope {
            attack_decay,
            sustain,
            gain,
        };
        let id =
            TimbreId(u16::try_from(self.instruments.len()).map_err(|_| ImportError::Resource)?);
        self.instruments.push(Instrument {
            sample,
            envelope,
            pitch_scale,
        });
        self.timbres.insert(index, id);
        Ok(id)
    }

    fn sample(&mut self, index: u8) -> Result<SampleId, ImportError> {
        if let Some(&id) = self.sample_ids.get(&index) {
            return Ok(id);
        }
        let at = entry::<SAMPLE_ENTRY>(DIRECTORY, SAMPLE_DATA, index)?;
        let [start_lo, start_hi, loop_lo, loop_hi] = self.read(at)?;
        let start = usize::from(u16::from_le_bytes([start_lo, start_hi]));
        let loop_at = usize::from(u16::from_le_bytes([loop_lo, loop_hi]));
        if start < SAMPLE_DATA {
            return Err(ImportError::Resource);
        }
        let sample = self.sample_bounds(start, loop_at)?;
        let id = SampleId(u16::try_from(self.samples.len()).map_err(|_| ImportError::Resource)?);
        self.samples.push(sample);
        self.sample_ids.insert(index, id);
        Ok(id)
    }

    fn sample_bounds(&self, start: usize, loop_at: usize) -> Result<SampleSource, ImportError> {
        let mut frames = 0_u32;
        let mut cursor = start;
        let mut loop_from = None;
        loop {
            let [header, ..] = self.read::<BLOCK_BYTES>(cursor)?;
            if cursor == loop_at {
                loop_from = Some(frames);
            }
            frames = frames.saturating_add(FRAMES_PER_BLOCK);
            if header & 1 != 0 {
                let loop_from = if header & 2 == 0 {
                    None
                } else {
                    Some(loop_from.ok_or(ImportError::Resource)?)
                };
                let bytes = self
                    .source(start..cursor.saturating_add(BLOCK_BYTES))?
                    .bytes;
                return Ok(SampleSource {
                    frames,
                    loop_from,
                    source_sha256: sha256(bytes),
                });
            }
            cursor = cursor.saturating_add(BLOCK_BYTES);
        }
    }

    fn read<const N: usize>(&self, at: usize) -> Result<[u8; N], ImportError> {
        self.source(at..at.saturating_add(N))?
            .bytes
            .try_into()
            .map_err(|_| ImportError::Resource)
    }

    fn source(&self, range: Range<usize>) -> Result<Region<'rom>, ImportError> {
        // A record must come wholly from one upload, with area data taking precedence.
        let at = range.start;
        if let Some(block) = self
            .area
            .0
            .iter()
            .find(|block| block.start <= at && at < block.end())
        {
            return block.slice(range).map_err(|_| ImportError::Resource);
        }
        if self
            .area
            .0
            .iter()
            .any(|block| block.start > at && block.start < range.end)
        {
            return Err(ImportError::Resource);
        }
        self.common.slice(range).map_err(|_| ImportError::Resource)
    }
}

fn entry<const N: usize>(base: usize, end: usize, index: u8) -> Result<usize, ImportError> {
    let at = base.saturating_add(usize::from(index).saturating_mul(N));
    (at.saturating_add(N) <= end)
        .then_some(at)
        .ok_or(ImportError::Resource)
}

#[cfg(test)]
mod tests;
