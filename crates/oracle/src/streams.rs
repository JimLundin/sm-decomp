//! Per-frame stream digests and their running fold.
//!
//! Two definitions sit side by side. `Legacy` is the existing lock's, where
//! the harness can compute it (the old engine's `result_digests` at git tag `archive/old-engine`, frozen with the
//! lock). `V1` is the oracle contract's: raw memories, the architectural PPU
//! registers, the APU port log and the PPU-domain picture.

use crate::record::{ApuAccess, ApuAccessKind, FrameRecord};
use core::hash::{Hash, Hasher};
use data::digest::{Digest, fold};

/// The machine streams, in the lock's order.
pub const NAMES: [&str; 8] = data::digest::MACHINE;

/// Frames between the combined digest's marks.
pub const MARK_FRAMES: u32 = 1000;

/// LEGACY(verifier): the WRAM the existing lock's `wram` stream zeroes: the
/// frozen `LOCK_WRAM_MASK` and snesrev's bug-fix counter `$C2-$C3`, copied
/// from the old engine's `lock_masks.rs` and `machine.rs` (git tag `archive/old-engine`)
/// so the harness can match the committed digests. The V1 `wram` stream masks nothing.
const LOCK_WRAM_MASK: [core::ops::Range<usize>; 32] = [
    0x0000..0x0051,
    0x1f5b..0x2000,
    0x00ad..0x00b1,
    0x05ed..0x05f5,
    0x05bc..0x05c5,
    0x060b..0x0611,
    0x0611..0x0617,
    0x077e..0x0783,
    0x078f..0x0791,
    0x07b7..0x07b9,
    0x0933..0x093d,
    0x0b24..0x0b28,
    0x0d82..0x0d8a,
    0x0d9c..0x0d9e,
    0x0dd2..0x0dd8,
    0x0d8a..0x0d90,
    0x0e20..0x0e46,
    0x0e54..0x0e56,
    0x0e02..0x0e04,
    0x0e4a..0x0e4c,
    0x0e56..0x0e5a,
    0x1784..0x178c,
    0x17a8..0x17ac,
    0x1834..0x183c,
    0x184a..0x185c,
    0x186e..0x1886,
    0x18a6..0x18a8,
    0x189a..0x18a6,
    0x1966..0x196c,
    0xef74..0xef78,
    0xf37a..0xf380,
    0x00c2..0x00c4,
];

/// Which definition of the streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Definition {
    /// The existing lock's (`oracle = "faithful"`).
    Legacy,
    /// The oracle contract v1's.
    V1,
}

/// One frame's digest of each stream; `None` where the definition cannot be
/// computed from this oracle's record.
pub type FrameDigests = [Option<u64>; 8];

fn hashed(value: &impl Hash) -> u64 {
    let mut digest = Digest::default();
    value.hash(&mut digest);
    digest.finish()
}

/// The `oam` stream; `None` for a record without the memories.
pub(crate) fn oam_digest(record: &FrameRecord) -> Option<u64> {
    let (low, high) = record.oam.split_at_checked(0x200)?;
    let words: Vec<u16> = low
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .collect();
    let mut digest = Digest::default();
    // The old PPU's `(oam: [u16; 256], highOam: [u8; 32])` as Rust hashes it.
    <[u16; 256]>::try_from(words.as_slice())
        .ok()?
        .hash(&mut digest);
    <[u8; 32]>::try_from(high).ok()?.hash(&mut digest);
    Some(digest.finish())
}

/// The v1 `apu` stream: the CPU side's port accesses as (kind, port, value).
fn apu_digest(log: &[ApuAccess]) -> u64 {
    let bytes: Vec<u8> = log
        .iter()
        .filter_map(|access| {
            let kind = match access.kind {
                ApuAccessKind::CpuWrite => 0,
                ApuAccessKind::CpuRead => 1,
                ApuAccessKind::DmaWrite => 2,
                ApuAccessKind::DmaRead => 3,
                ApuAccessKind::SpcWrite => return None,
            };
            Some([kind, access.port, access.value])
        })
        .flatten()
        .collect();
    Digest::of(&bytes)
}

/// The existing lock's streams for `record`.
pub fn legacy(record: &FrameRecord, scratch: &mut Vec<u8>) -> FrameDigests {
    scratch.clear();
    scratch.extend_from_slice(&record.wram);
    for range in LOCK_WRAM_MASK {
        scratch[range].fill(0);
    }
    let vram = <&[u16; 0x8000]>::try_from(&record.vram[..]).expect("VRAM");
    let cgram = <&[u16; 0x100]>::try_from(&record.cgram[..]).expect("CGRAM");
    [
        Some(Digest::of(scratch)),
        Some(Digest::of(&record.sram)),
        Some(hashed(vram)),
        Some(hashed(cgram)),
        oam_digest(record),
        Some(record.legacy_ppu_digest),
        None,
        None,
    ]
}

/// The oracle contract v1's streams for `record`.
pub fn v1(record: &FrameRecord) -> FrameDigests {
    let vram = <&[u16; 0x8000]>::try_from(&record.vram[..]).expect("VRAM");
    let cgram = <&[u16; 0x100]>::try_from(&record.cgram[..]).expect("CGRAM");
    let video = record.picture.as_ref().map(|picture| {
        let mut digest = Digest::default();
        picture.lines.hash(&mut digest);
        for row in picture.rows() {
            row.hash(&mut digest);
        }
        picture.brightness[..picture.lines].hash(&mut digest);
        picture.forced_blank[..picture.lines].hash(&mut digest);
        digest.finish()
    });
    [
        Some(Digest::of(&record.wram)),
        Some(Digest::of(&record.sram)),
        Some(hashed(vram)),
        Some(hashed(cgram)),
        oam_digest(record),
        Some(hashed(&record.ppu)),
        video,
        Some(apu_digest(&record.apu_log)),
    ]
}

/// The running digests of one run, as a lock entry commits them.
#[derive(Clone, Debug)]
pub struct Running {
    pub frames: u32,
    pub digests: [u64; 8],
    /// Streams some frame lacked; they have no running digest.
    pub missing: [bool; 8],
    pub combined: u64,
    pub marks: Vec<(u32, u64)>,
}

impl Default for Running {
    fn default() -> Self {
        let empty = Digest::default().finish();
        Self {
            frames: 0,
            digests: [empty; 8],
            missing: [false; 8],
            combined: empty,
            marks: Vec::new(),
        }
    }
}

impl Running {
    /// Fold one frame's digests in.
    pub fn fold(&mut self, frame: &FrameDigests) {
        for ((running, missing), digest) in
            self.digests.iter_mut().zip(&mut self.missing).zip(frame)
        {
            match digest {
                Some(digest) => *running = fold(*running, *digest),
                None => *missing = true,
            }
            // The combined digest folds every machine stream; a missing one
            // folds as the empty digest, so it cannot match a complete entry.
            self.combined = fold(self.combined, digest.unwrap_or_default());
        }
        self.frames += 1;
        if self.frames.is_multiple_of(MARK_FRAMES) {
            self.marks.push((self.frames, self.combined));
        }
    }

    /// Stream `index`'s running digest, if every frame had it.
    pub fn digest(&self, index: usize) -> Option<u64> {
        (!self.missing[index]).then_some(self.digests[index])
    }
}
