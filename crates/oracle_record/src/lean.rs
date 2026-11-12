//! The record cache's lean tier: each record without its picture, its memories kept as the
//! bytes that changed since the record before.
//!
//! Lockstep, the domain harnesses and the free run read a record's registers and its five
//! memories (WRAM, VRAM, CGRAM, OAM, SRAM), never its picture, which is a third of a frame
//! that has one. From one frame to the next the memories change in a few hundred bytes, so
//! a row's records form a chain: a record holds its other parts in [`codec`]'s encoding,
//! then its memories as runs of changed bytes over the record before's. Every
//! [`KEY_INTERVAL`]th record holds them whole, so a reader can start part way.
//!
//! The hits stay out, as in the full tier: the cache keeps them in `hits.txt`.
//!
//! A record is the magic, the version, a kind byte (0 whole, 1 changes), the length and
//! bytes of the rest of the record in [`codec`]'s encoding without the picture and with
//! empty memories, the five memories' lengths in bytes, then either the memories' bytes or
//! the number of runs and each run's unchanged bytes before it, its length and its bytes.
//! VRAM and CGRAM are little-endian words, as [`codec`] writes them.

use crate::FrameRecord;
use crate::codec::{self, CodecError, PictureMode};

/// The bytes every lean record starts with.
pub const MAGIC: [u8; 4] = *b"SMFL";
/// The encoding's version, after the magic.
pub const VERSION: u8 = 1;
/// The chain's record at every multiple of this position holds its memories whole.
pub const KEY_INTERVAL: usize = 256;

/// Unchanged bytes fewer than a run's header between two changes stay inside one run.
const RUN_HEADER: usize = 8;

/// The memories of the chain's last record, which the next record's changes apply to.
#[derive(Clone, Debug, Default)]
pub struct Memories {
    /// WRAM, VRAM, CGRAM, OAM and SRAM, one after another.
    image: Vec<u8>,
    /// Their lengths in bytes; none before a chain's first record or after a failed one.
    lengths: Option<[usize; 5]>,
}

/// `record` as a lean record appended to `out`: its memories whole when `whole` asks or
/// they differ in size from `memories`, its changes over `memories` otherwise. `memories`
/// becomes the record's.
pub fn encode(record: &FrameRecord, memories: &mut Memories, whole: bool, out: &mut Vec<u8>) {
    let lengths = [
        record.wram.len(),
        record.vram.len().saturating_mul(2),
        record.cgram.len().saturating_mul(2),
        record.oam.len(),
        record.sram.len(),
    ];
    let mut image = Vec::with_capacity(lengths.iter().sum());
    image.extend_from_slice(&record.wram);
    image.extend(record.vram.iter().flat_map(|word| word.to_le_bytes()));
    image.extend(record.cgram.iter().flat_map(|colour| colour.to_le_bytes()));
    image.extend_from_slice(&record.oam);
    image.extend_from_slice(&record.sram);
    let changes = !whole && memories.lengths == Some(lengths);
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.push(u8::from(changes));
    let rest = FrameRecord {
        wram: Box::default(),
        vram: Box::default(),
        cgram: Box::default(),
        oam: Box::default(),
        sram: Box::default(),
        picture: None,
        hits: Vec::new(),
        ..record.clone()
    };
    let mut rest_bytes = Vec::new();
    codec::encode(&rest, PictureMode::Drop, &mut rest_bytes);
    push_len(out, rest_bytes.len());
    out.extend_from_slice(&rest_bytes);
    lengths.iter().for_each(|&length| push_len(out, length));
    if changes {
        let runs = runs(&memories.image, &image);
        push_len(out, runs.len());
        let mut end = 0;
        for run in runs {
            push_len(out, run.start.saturating_sub(end));
            push_len(out, run.len());
            out.extend_from_slice(image.get(run.clone()).unwrap_or_default());
            end = run.end;
        }
    } else {
        out.extend_from_slice(&image);
    }
    *memories = Memories {
        image,
        lengths: Some(lengths),
    };
}

/// A length as a u32; the machine's memories are far below 4 GiB.
fn push_len(out: &mut Vec<u8>, len: usize) {
    out.extend_from_slice(&u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes());
}

/// The ranges where `new` differs from `old`, of the same length, with short unchanged gaps
/// inside them.
fn runs(old: &[u8], new: &[u8]) -> Vec<core::ops::Range<usize>> {
    let mut runs: Vec<core::ops::Range<usize>> = Vec::new();
    let changed = old
        .iter()
        .zip(new)
        .enumerate()
        .filter_map(|(at, (was, is))| (was != is).then_some(at));
    for at in changed {
        match runs.last_mut() {
            Some(run) if at.saturating_sub(run.end) < RUN_HEADER => run.end = at.saturating_add(1),
            _ => runs.push(at..at.saturating_add(1)),
        }
    }
    runs
}

/// The record lean `bytes` encode, its changes applied to `memories`, which becomes the
/// record's. On an error `memories` is emptied, so the chain must restart at a whole record.
///
/// # Errors
///
/// A [`CodecError`] when the bytes are not one lean record, or [`CodecError::NoBase`] when
/// they hold changes and `memories` isn't the previous record's.
pub fn decode(bytes: &[u8], memories: &mut Memories) -> Result<FrameRecord, CodecError> {
    let decoded = decode_into(bytes, memories);
    if decoded.is_err() {
        *memories = Memories::default();
    }
    decoded
}

fn decode_into(bytes: &[u8], memories: &mut Memories) -> Result<FrameRecord, CodecError> {
    let mut r = Reader { rest: bytes };
    if r.take("magic", MAGIC.len())? != MAGIC {
        return Err(CodecError::BadMagic);
    }
    let [version] = r.array("version")?;
    if version != VERSION {
        return Err(CodecError::BadVersion(version));
    }
    let changes = match r.array("kind")? {
        [0] => false,
        [1] => true,
        [tag] => return Err(CodecError::BadTag { field: "kind", tag }),
    };
    let rest_len = r.len("rest")?;
    let mut record = codec::decode_with(r.take("rest", rest_len)?, PictureMode::Drop)?;
    let mut lengths = [0; 5];
    for length in &mut lengths {
        *length = r.len("lengths")?;
    }
    if changes {
        if memories.lengths != Some(lengths) {
            return Err(CodecError::NoBase);
        }
        let mut end = 0_usize;
        for _ in 0..r.len("runs")? {
            let start = end.saturating_add(r.len("run gap")?);
            let length = r.len("run length")?;
            end = start.saturating_add(length);
            let changed = r.take("run", length)?;
            memories
                .image
                .get_mut(start..end)
                .ok_or(CodecError::TooLong("run"))?
                .copy_from_slice(changed);
        }
    } else {
        let total = lengths
            .iter()
            .try_fold(0_usize, |total, &length| total.checked_add(length))
            .ok_or(CodecError::TooLong("memories"))?;
        memories.image.clear();
        memories.image.extend_from_slice(r.take("memories", total)?);
        memories.lengths = Some(lengths);
    }
    if !r.rest.is_empty() {
        return Err(CodecError::Trailing(r.rest.len()));
    }
    let mut image = memories.image.as_slice();
    let mut part = |length: usize, field: &'static str| {
        let (head, rest) = image
            .split_at_checked(length)
            .ok_or(CodecError::Truncated(field))?;
        image = rest;
        Ok::<_, CodecError>(head)
    };
    let [wram, vram, cgram, oam, sram] = lengths;
    record.wram = Box::from(part(wram, "wram")?);
    record.vram = words(part(vram, "vram")?, "vram")?;
    record.cgram = words(part(cgram, "cgram")?, "cgram")?;
    record.oam = Box::from(part(oam, "oam")?);
    record.sram = Box::from(part(sram, "sram")?);
    Ok(record)
}

/// Little-endian `bytes` as words.
fn words(bytes: &[u8], field: &'static str) -> Result<Box<[u16]>, CodecError> {
    let (pairs, odd) = bytes.as_chunks::<2>();
    if !odd.is_empty() {
        return Err(CodecError::Truncated(field));
    }
    Ok(pairs.iter().map(|&pair| u16::from_le_bytes(pair)).collect())
}

struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, field: &'static str, count: usize) -> Result<&'a [u8], CodecError> {
        let (head, rest) = self
            .rest
            .split_at_checked(count)
            .ok_or(CodecError::Truncated(field))?;
        self.rest = rest;
        Ok(head)
    }

    fn array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], CodecError> {
        let (head, rest) = self
            .rest
            .split_first_chunk::<N>()
            .ok_or(CodecError::Truncated(field))?;
        self.rest = rest;
        Ok(*head)
    }

    fn len(&mut self, field: &'static str) -> Result<usize, CodecError> {
        usize::try_from(u32::from_le_bytes(self.array(field)?))
            .map_err(|_| CodecError::TooLong(field))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CpuRegisters, Cut, Pad, Picture, PpuRegisters};

    /// Record `index`, whose WRAM byte $1234 and VRAM word $4000 hold `change`.
    fn record(index: u64, change: u8) -> FrameRecord {
        let mut wram = vec![0x55; 0x2_0000];
        if let Some(byte) = wram.get_mut(0x1234) {
            *byte = change;
        }
        let mut vram: Box<[u16]> = (0..0x8000u16).collect();
        if let Some(word) = vram.get_mut(0x4000) {
            *word = u16::from(change);
        }
        FrameRecord {
            index,
            video_frame: 100,
            pad: Pad(0x8010),
            cut: Cut::VblankWithoutNmi,
            joypad_reads: 1,
            wait_entry: None,
            overscan: false,
            cpu: CpuRegisters::default(),
            wram: wram.into_boxed_slice(),
            vram,
            cgram: vec![0x7FFF; 0x100].into_boxed_slice(),
            oam: vec![1; 0x220].into_boxed_slice(),
            sram: vec![2; 0x2000].into_boxed_slice(),
            ppu: PpuRegisters::default(),
            apu_log: Vec::new(),
            picture: Some(Picture::default()),
            spc: None,
            bank_instructions: None,
            writes: Vec::new(),
            hits: Vec::new(),
            legacy_ppu_digest: 7,
        }
    }

    fn same(a: &FrameRecord, b: &FrameRecord) -> bool {
        (a.index, a.cut, a.pad, a.cpu, a.ppu, a.legacy_ppu_digest)
            == (b.index, b.cut, b.pad, b.cpu, b.ppu, b.legacy_ppu_digest)
            && (&a.wram, &a.vram, &a.cgram, &a.oam, &a.sram)
                == (&b.wram, &b.vram, &b.cgram, &b.oam, &b.sram)
            && a.picture.is_none()
    }

    fn chain(records: &[FrameRecord]) -> Vec<Vec<u8>> {
        let mut memories = Memories::default();
        records
            .iter()
            .enumerate()
            .map(|(position, record)| {
                let mut bytes = Vec::new();
                encode(
                    record,
                    &mut memories,
                    position.is_multiple_of(KEY_INTERVAL),
                    &mut bytes,
                );
                bytes
            })
            .collect()
    }

    #[test]
    fn a_chain_decodes_to_its_records_without_their_pictures() {
        let records = [record(0, 0), record(1, 9), record(2, 9), record(3, 0xA0)];
        let encoded = chain(&records);
        let mut memories = Memories::default();
        for (bytes, record) in encoded.iter().zip(&records) {
            let decoded = decode(bytes, &mut memories);
            assert_eq!(decoded.map(|decoded| same(&decoded, record)), Ok(true));
        }
        let whole = encoded.first().map_or(0, Vec::len);
        assert!(
            encoded
                .iter()
                .skip(1)
                .all(|bytes| bytes.len() * 100 < whole)
        );
    }

    #[test]
    fn changes_without_the_record_before_are_refused() {
        let encoded = chain(&[record(0, 0), record(1, 9)]);
        let mut memories = Memories::default();
        let changes = encoded.get(1).map(Vec::as_slice).unwrap_or_default();
        assert_eq!(
            decode(changes, &mut memories).map(|_| ()),
            Err(CodecError::NoBase)
        );
    }

    #[test]
    fn short_and_long_bytes_are_refused() {
        let encoded = chain(&[record(0, 0)]);
        let mut bytes = encoded.into_iter().next().unwrap_or_default();
        let short = bytes
            .get(..bytes.len().saturating_sub(1))
            .unwrap_or_default();
        assert!(matches!(
            decode(short, &mut Memories::default()),
            Err(CodecError::Truncated(_))
        ));
        bytes.push(0);
        assert_eq!(
            decode(&bytes, &mut Memories::default()).map(|_| ()),
            Err(CodecError::Trailing(1))
        );
    }
}
