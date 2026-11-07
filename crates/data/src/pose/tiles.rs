//! Samus's tile uploads: for each pose and animation frame, the graphics her top and bottom
//! halves upload to VRAM, and those graphics.
//!
//! The pose's first animation definition is found by pose, four bytes a frame. Bytes 0 and 1
//! pick the top half's tiles definition from a list of lists; bytes 2 and 3 pick the bottom
//! half's, none when byte 2 is 0xFF. A tiles definition is a long source address, part 1's
//! size and part 2's size; part 2 follows part 1 in the source.

use canonical::Canonical;

use super::animation::{AnimationDelays, AnimationFrame, reached};
use super::{POSE_COUNT, PoseId, Poses};
use crate::rom::{AddressError, Lorom, RomAddress};

/// Bank `$92`, which holds the tables and the definitions.
const BANK: u8 = 0x92;
/// Each pose's first animation definition.
const ANIMATION_DEFINITIONS: RomAddress = RomAddress::new(BANK, 0xD94E);
/// The top half's tiles definition lists.
const TOP_LISTS: RomAddress = RomAddress::new(BANK, 0xD91E);
/// The bottom half's tiles definition lists.
const BOTTOM_LISTS: RomAddress = RomAddress::new(BANK, 0xD938);
/// An animation definition's bytes: the top half's list and entry, then the bottom half's.
const FRAME_BYTES: usize = 4;
/// A tiles definition's bytes: the long source, then the two parts' sizes.
const DEFINITION_BYTES: usize = 7;
/// The bottom half's list byte for a frame without bottom tiles; vanilla has none.
const NO_BOTTOM: u8 = 0xFF;
/// The banks Samus's graphics are in.
pub const GRAPHICS_BANKS: std::ops::RangeInclusive<u8> = 0x9B..=0x9F;

/// A tiles definition, by its place among the ones the animation definitions name, in ROM
/// order.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileDefinitionId(pub u16);

/// Where in ROM an upload's graphics start: a reference [`Graphics`] resolves.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GraphicsRef {
    /// The bank.
    pub bank: u8,
    /// The offset in it.
    pub offset: u16,
}

impl From<RomAddress> for GraphicsRef {
    fn from(address: RomAddress) -> Self {
        Self {
            bank: address.bank,
            offset: address.offset,
        }
    }
}

/// One half's tile upload: two DMAs from consecutive graphics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileDefinition {
    /// Part 1's graphics.
    pub source: GraphicsRef,
    /// Part 1's size in bytes; 0 is the DMA's 10000h.
    pub first: u16,
    /// Part 2's size in bytes; 0 transfers nothing.
    pub second: u16,
}

impl TileDefinition {
    /// The definition from its seven bytes.
    #[must_use]
    pub const fn decode(bytes: [u8; DEFINITION_BYTES]) -> Self {
        let [
            low,
            high,
            bank,
            first_low,
            first_high,
            second_low,
            second_high,
        ] = bytes;
        Self {
            source: GraphicsRef {
                bank,
                offset: u16::from_le_bytes([low, high]),
            },
            first: u16::from_le_bytes([first_low, first_high]),
            second: u16::from_le_bytes([second_low, second_high]),
        }
    }

    /// Part 2's graphics: on from part 1's by its size, within the bank as the DMA's
    /// address register wraps.
    #[must_use]
    pub const fn second_source(&self) -> GraphicsRef {
        GraphicsRef {
            offset: self.source.offset.wrapping_add(self.first),
            ..self.source
        }
    }
}

/// The tiles definitions one animation frame uploads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FrameTiles {
    /// The top half's.
    pub top: TileDefinitionId,
    /// The bottom half's, if the frame has one.
    pub bottom: Option<TileDefinitionId>,
}

/// Samus's tiles definitions and which each pose's frames upload.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SamusTiles {
    /// By pose, by animation frame.
    frames: Vec<Vec<FrameTiles>>,
    /// By id: where the original keeps the definition, the word it stores in work RAM.
    pointers: Vec<u16>,
    /// By id.
    definitions: Vec<TileDefinition>,
}

/// The little-endian word at `address`.
fn word(rom: Lorom<'_>, address: RomAddress) -> Result<u16, AddressError> {
    let bytes = rom.read(address, 2)?;
    let pair = bytes
        .first_chunk::<2>()
        .ok_or(AddressError::PastEnd(address))?;
    Ok(u16::from_le_bytes(*pair))
}

/// `address` moved on by `count` bytes, in the bank.
fn at(address: RomAddress, count: usize) -> Result<RomAddress, AddressError> {
    address.advance(count).ok_or(AddressError::PastEnd(address))
}

/// A frame's top and bottom tiles definition pointers, as `$92:8000` computes them.
fn frame_pointers(
    rom: Lorom<'_>,
    first: u16,
    frame: usize,
) -> Result<(u16, Option<u16>), AddressError> {
    let address = at(
        RomAddress::new(BANK, first),
        frame.wrapping_mul(FRAME_BYTES),
    )?;
    let [top_list, top_entry, bottom_list, bottom_entry] = rom
        .read(address, FRAME_BYTES)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(address))?;
    let pointer = |lists: RomAddress, list: u8, entry: u8| -> Result<u16, AddressError> {
        let list = word(rom, at(lists, usize::from(list).wrapping_mul(2))?)?;
        let offset = u16::from(entry).wrapping_mul(7);
        Ok(list.wrapping_add(offset))
    };
    let top = pointer(TOP_LISTS, top_list, top_entry)?;
    let bottom = if bottom_list == NO_BOTTOM {
        None
    } else {
        Some(pointer(BOTTOM_LISTS, bottom_list, bottom_entry)?)
    };
    Ok((top, bottom))
}

impl SamusTiles {
    /// The tables from the ROM, for as many frames of each pose as its delay list has, and
    /// for a spin jump through the frame its wall jump check reaches past the list.
    ///
    /// A pose's frames end early at the first whose definition isn't in the image: past
    /// the frames her animation reaches, the lists run into other data.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if the pose table isn't in the image.
    pub fn load(
        rom: Lorom<'_>,
        delays: &AnimationDelays,
        poses: &Poses,
    ) -> Result<Self, AddressError> {
        let mut by_pose = Vec::with_capacity(POSE_COUNT);
        for index in 0..POSE_COUNT {
            let first = word(rom, at(ANIMATION_DEFINITIONS, index.wrapping_mul(2))?)?;
            let pose = PoseId(u16::try_from(index).unwrap_or(u16::MAX));
            let listed = delays.of(pose).map_or(0, super::animation::DelayList::len);
            let count = listed
                .max(reached(poses, pose).map_or(0, |frame| usize::from(frame.0).wrapping_add(1)));
            let frames: Vec<(u16, Option<u16>)> = (0..count)
                .map_while(|frame| {
                    let pointers = frame_pointers(rom, first, frame).ok()?;
                    let (top, bottom) = pointers;
                    let readable = |pointer: u16| {
                        rom.read(RomAddress::new(BANK, pointer), DEFINITION_BYTES)
                            .is_ok()
                    };
                    (readable(top) && bottom.is_none_or(readable)).then_some(pointers)
                })
                .collect();
            by_pose.push(frames);
        }
        let mut pointers: Vec<u16> = by_pose
            .iter()
            .flatten()
            .flat_map(|&(top, bottom)| std::iter::once(top).chain(bottom))
            .collect();
        pointers.sort_unstable();
        pointers.dedup();
        let definitions = pointers
            .iter()
            .map(|&pointer| {
                let address = RomAddress::new(BANK, pointer);
                let bytes = rom
                    .read(address, DEFINITION_BYTES)?
                    .try_into()
                    .map_err(|_| AddressError::PastEnd(address))?;
                Ok(TileDefinition::decode(bytes))
            })
            .collect::<Result<Vec<_>, AddressError>>()?;
        let id = |pointer: u16| {
            let index = pointers.binary_search(&pointer).unwrap_or_default();
            TileDefinitionId(u16::try_from(index).unwrap_or(u16::MAX))
        };
        let frames = by_pose
            .iter()
            .map(|frames| {
                frames
                    .iter()
                    .map(|&(top, bottom)| FrameTiles {
                        top: id(top),
                        bottom: bottom.map(id),
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            frames,
            pointers,
            definitions,
        })
    }

    /// The tables from parts, for tests: each pose's frames and the definitions by id,
    /// definition `n` kept at pointer `n * 7`.
    #[must_use]
    pub fn new(frames: Vec<Vec<FrameTiles>>, definitions: Vec<TileDefinition>) -> Self {
        let pointers = (0..definitions.len())
            .map(|index| u16::try_from(index.wrapping_mul(DEFINITION_BYTES)).unwrap_or(u16::MAX))
            .collect();
        Self {
            frames,
            pointers,
            definitions,
        }
    }

    /// What `pose` uploads at `frame`, or `None` past its frames.
    #[must_use]
    pub fn frame(&self, pose: PoseId, frame: AnimationFrame) -> Option<FrameTiles> {
        self.frames
            .get(usize::from(pose.0))?
            .get(usize::from(frame.0))
            .copied()
    }

    /// The definition `id` names.
    #[must_use]
    pub fn definition(&self, id: TileDefinitionId) -> Option<&TileDefinition> {
        self.definitions.get(usize::from(id.0))
    }

    /// The definition the original keeps at the 16-bit `pointer`.
    #[must_use]
    pub fn at_pointer(&self, pointer: u16) -> Option<TileDefinitionId> {
        let index = self.pointers.binary_search(&pointer).ok()?;
        u16::try_from(index).ok().map(TileDefinitionId)
    }

    /// The 16-bit pointer the original keeps for `id`.
    #[must_use]
    pub fn pointer(&self, id: TileDefinitionId) -> Option<u16> {
        self.pointers.get(usize::from(id.0)).copied()
    }

    /// How many definitions the frames name.
    #[must_use]
    pub const fn definition_count(&self) -> usize {
        self.definitions.len()
    }
}

/// ROM graphics by bank: the ROM halves of the banks they were loaded from.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Graphics {
    banks: Vec<(u8, Vec<u8>)>,
}

/// Bytes in a `LoROM` bank's ROM half.
const BANK_BYTES: usize = 0x8000;
/// Where a bank's ROM half starts.
const ROM_HALF: u16 = 0x8000;

impl Graphics {
    /// The banks `banks` from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] if a bank isn't in the image.
    pub fn load(rom: Lorom<'_>, banks: std::ops::RangeInclusive<u8>) -> Result<Self, AddressError> {
        let banks = banks
            .map(|bank| {
                let bytes = rom.read(RomAddress::new(bank, ROM_HALF), BANK_BYTES)?;
                Ok((bank, bytes.to_vec()))
            })
            .collect::<Result<_, AddressError>>()?;
        Ok(Self { banks })
    }

    /// Graphics from parts, for tests: each bank's ROM half, from offset 0x8000.
    #[must_use]
    pub const fn new(banks: Vec<(u8, Vec<u8>)>) -> Self {
        Self { banks }
    }

    /// The `count` bytes at `from`, or `None` if they aren't all in one loaded bank's ROM
    /// half.
    #[must_use]
    pub fn bytes(&self, from: GraphicsRef, count: usize) -> Option<&[u8]> {
        let (_, bytes) = self.banks.iter().find(|(bank, _)| *bank == from.bank)?;
        let start = usize::from(from.offset.checked_sub(ROM_HALF)?);
        bytes.get(start..start.checked_add(count)?)
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Bank `$92` log (<https://patrickjohnston.org/bank/92>), set Samus tiles definitions for
// current animation `$92:8000`: the pose's first animation definition is
// `[$92:D94E + pose * 2]`, four bytes a frame. Bytes 0 and 1 pick the top half's tiles
// definition, `[$92:D91E + byte 0 * 2] + byte 1 * 7`; bytes 2 and 3 the bottom half's from
// `$92:D938`, none when byte 2 is `FFh`. Bank `$80` log, transfer Samus tiles to VRAM
// `$80:9376`: a tiles definition is a long source address, part 1's size and part 2's
// The definition the original keeps at `pointer` in bank `$92`.
// The bank `$92` pointer the original keeps for `id`.
// Graphics from parts, for tests: each bank's ROM half, from `$8000`.
