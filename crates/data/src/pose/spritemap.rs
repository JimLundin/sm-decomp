//! Samus's spritemaps: for each pose and animation frame, the spritemap of her top half and
//! of her bottom half, and the offsets that place them.
//!
//! The top half's spritemap index is a per-pose base plus the animation frame, the bottom
//! half's likewise from its own table; the index selects a spritemap pointer. The position
//! offsets come from three tables: landing, the crouching, standing, morphing and unmorphing
//! transitions, and being drained by a metroid.

use super::animation::AnimationFrame;
use super::{POSE_COUNT, PoseId};
use crate::rom::{AddressError, Lorom, RomAddress};
use crate::spritemap::Spritemap;

/// Bank `$92`, which holds Samus's spritemaps and their tables.
const BANK: u8 = 0x92;
/// The spritemap pointers by spritemap index.
const SPRITEMAP_POINTERS: RomAddress = RomAddress::new(BANK, 0x808D);
/// The spritemap pointers the half tables can name: the pointer table runs up to the first
/// spritemap.
pub const SPRITEMAP_COUNT: usize = 0x830;
/// The top half's first spritemap index by pose.
const TOP_HALVES: RomAddress = RomAddress::new(BANK, 0x9263);
/// The bottom half's first spritemap index by pose.
const BOTTOM_HALVES: RomAddress = RomAddress::new(BANK, 0x945D);
/// The landing poses' Y offsets, four frames a pose from pose 0xA4.
const LANDING_OFFSETS: RomAddress = RomAddress::new(0x90, 0x8D28);
/// The transition poses' Y offsets, two frames a pose from pose 0x35.
const TRANSITION_OFFSETS: RomAddress = RomAddress::new(0x90, 0x8D80);
/// The drained poses' Y offsets by animation frame.
const DRAINED_OFFSETS: RomAddress = RomAddress::new(0x90, 0x8DEF);
/// Landing offsets: poses 0xA4 to 0xA7, four frames each.
pub const LANDING_FRAMES: usize = 16;
/// Transition offsets: poses 0x35 to 0x40, two frames each.
pub const TRANSITION_FRAMES: usize = 24;
/// Drained offsets: one per frame.
pub const DRAINED_FRAMES: usize = 32;

/// A half of Samus's body, each drawn from its own spritemap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Half {
    /// The top half, drawn first.
    Top,
    /// The bottom half.
    Bottom,
}

/// An index into the spritemap pointer table: a half's first index for her pose plus her
/// animation frame. The original keeps it as a plain word, so it goes out unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpritemapIndex(pub u16);

/// The offsets that move a spritemap's origin up or down from her position for some poses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PositionOffsets {
    /// Landing, subtracted: `(pose - 0xA4) * 4 + frame`. The original reads a word and keeps
    /// only its low byte's effect on OAM, so the byte is kept.
    pub landing: [u8; LANDING_FRAMES],
    /// The transitions, added: `(pose - 0x35) * 2 + frame`.
    pub transition: [i8; TRANSITION_FRAMES],
    /// Drained by a metroid, added, by frame.
    pub drained: [i8; DRAINED_FRAMES],
}

impl Default for PositionOffsets {
    fn default() -> Self {
        Self {
            landing: [0; LANDING_FRAMES],
            transition: [0; TRANSITION_FRAMES],
            drained: [0; DRAINED_FRAMES],
        }
    }
}

/// Samus's spritemaps and the tables that select them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SamusSpritemaps {
    top: Vec<SpritemapIndex>,
    bottom: Vec<SpritemapIndex>,
    /// By spritemap index: which of `maps` it points at, `None` for a zero pointer.
    pointers: Vec<Option<usize>>,
    maps: Vec<Spritemap>,
    /// The position offsets.
    pub offsets: PositionOffsets,
}

/// Why Samus's spritemaps don't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpritemapError {
    /// A table or a spritemap isn't in the image.
    Address(AddressError),
}

impl From<AddressError> for SpritemapError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The `count` little-endian words at `address`.
fn words(rom: Lorom<'_>, address: RomAddress, count: usize) -> Result<Vec<u16>, AddressError> {
    let bytes = rom.read(address, count.saturating_mul(2))?;
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .collect())
}

/// The `N` bytes at `address`.
fn bytes<const N: usize>(rom: Lorom<'_>, address: RomAddress) -> Result<[u8; N], AddressError> {
    rom.read(address, N)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(address))
}

impl SamusSpritemaps {
    /// The tables and every spritemap they point at, from the ROM.
    ///
    /// # Errors
    ///
    /// [`SpritemapError`] if a table or a spritemap isn't in the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, SpritemapError> {
        let indices = |address| -> Result<Vec<SpritemapIndex>, AddressError> {
            Ok(words(rom, address, POSE_COUNT)?
                .into_iter()
                .map(SpritemapIndex)
                .collect())
        };
        let top = indices(TOP_HALVES)?;
        let bottom = indices(BOTTOM_HALVES)?;
        let mut targets: Vec<u16> = words(rom, SPRITEMAP_POINTERS, SPRITEMAP_COUNT)?;
        let raw = targets.clone();
        targets.retain(|&pointer| pointer != 0);
        targets.sort_unstable();
        targets.dedup();
        let maps = targets
            .iter()
            .map(|&pointer| Spritemap::load(rom, RomAddress::new(BANK, pointer)))
            .collect::<Result<Vec<_>, _>>()?;
        let pointers = raw
            .iter()
            .map(|&pointer| {
                (pointer != 0)
                    .then(|| targets.binary_search(&pointer).ok())
                    .flatten()
            })
            .collect();
        let offsets = PositionOffsets {
            landing: bytes(rom, LANDING_OFFSETS)?,
            transition: bytes(rom, TRANSITION_OFFSETS)?.map(|byte: u8| i8::from_le_bytes([byte])),
            drained: bytes(rom, DRAINED_OFFSETS)?.map(|byte: u8| i8::from_le_bytes([byte])),
        };
        Ok(Self {
            top,
            bottom,
            pointers,
            maps,
            offsets,
        })
    }

    /// The tables from parts, for tests: the halves' first indices by pose, the spritemap
    /// each index draws (`None` for a zero pointer) and the offsets.
    #[must_use]
    pub fn new(
        top: Vec<SpritemapIndex>,
        bottom: Vec<SpritemapIndex>,
        spritemaps: Vec<Option<Spritemap>>,
        offsets: PositionOffsets,
    ) -> Self {
        let mut maps = Vec::new();
        let pointers = spritemaps
            .into_iter()
            .map(|map| {
                map.map(|map| {
                    maps.push(map);
                    maps.len().saturating_sub(1)
                })
            })
            .collect();
        Self {
            top,
            bottom,
            pointers,
            maps,
            offsets,
        }
    }

    /// The spritemap index of `half` for `pose` at `frame`, or `None` past the pose table.
    #[must_use]
    pub fn index(&self, half: Half, pose: PoseId, frame: AnimationFrame) -> Option<SpritemapIndex> {
        let table = match half {
            Half::Top => &self.top,
            Half::Bottom => &self.bottom,
        };
        let first = table.get(usize::from(pose.0))?;
        Some(SpritemapIndex(first.0.wrapping_add(frame.0)))
    }

    /// The spritemap `index` draws, or `None` for a zero pointer or an index past the table.
    ///
    /// The original reads a zero pointer's count from low work RAM, and an
    /// index past the table from the spritemaps after it; here neither draws
    /// (docs/LEGACY.md).
    #[must_use]
    pub fn spritemap(&self, index: SpritemapIndex) -> Option<&Spritemap> {
        let map = self.pointers.get(usize::from(index.0)).copied().flatten()?;
        self.maps.get(map)
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Bank `$90` log, draw Samus `$90:85E2` (<https://patrickjohnston.org/bank/90>): the top
// half's spritemap index is `[$92:9263 + pose * 2]` plus the animation frame, the bottom
// half's `[$92:945D + pose * 2]` plus the frame. Bank `$81` log, add Samus spritemap to OAM
// `$81:89AE`: the index selects a pointer from `$92:808D`, a spritemap in bank `$92`.
// Calculate Samus spritemap position `$90:8C1F`'s tables: landing `$90:8D28`, the crouching,
// standing, morphing and unmorphing transitions `$90:8D80`, and drained by a metroid
// `$90:8DEF`.
// The spritemap pointers the half tables can name: the table runs up to `$92:90ED`, the
// animation frame. The original keeps it in `$0AC8` and `$0ACA`, so it goes out unchanged.
// The original reads a zero pointer's count from `$92:0000`, low work RAM, and an
