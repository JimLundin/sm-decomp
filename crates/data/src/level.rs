//! A room's level: its blocks, what each does when touched, and the camera's scroll areas.
//!
//! The decompressed level data (bank `$82` log, `$82:E7D3`, into `$7F:0000`) is layer 1's
//! size in bytes, then layer 1, a word a block, then the BTS, a byte a block, then layer 2,
//! a word a block, when the room has one. A block word is the tile number in bits 0-9, the
//! flips in bits 10 and 11 and the block type in bits 12-15; the BTS is the type's argument
//! (bank `$94` log, the block reaction tables).

use canonical::Canonical;

use crate::rom::{AddressError, Lorom, RomAddress};
use crate::room::Screens;

pub mod slope;

/// A tile number's bits in a block word.
const TILE_NUMBER: u16 = 0x03ff;
/// The horizontal flip's bit.
const X_FLIP: u16 = 0x0400;
/// The vertical flip's bit.
const Y_FLIP: u16 = 0x0800;
/// Where the block type starts.
const TYPE_SHIFT: u16 = 12;

/// The tilemap block a level block draws: an index into the tileset's tile table.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileNumber(pub u16);

/// The tilemap block a level block draws and how it's flipped.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Tile {
    /// The tile table entry, ten bits.
    pub number: TileNumber,
    /// Mirrored left to right.
    pub x_flip: bool,
    /// Mirrored top to bottom.
    pub y_flip: bool,
}

impl Tile {
    /// The tile of a block word, whose type bits it ignores.
    #[must_use]
    pub const fn from_word(word: u16) -> Self {
        Self {
            number: TileNumber(word & TILE_NUMBER),
            x_flip: word & X_FLIP != 0,
            y_flip: word & Y_FLIP != 0,
        }
    }

    /// The tile's twelve bits of a block word.
    #[must_use]
    pub const fn word(self) -> u16 {
        let x_flip = if self.x_flip { X_FLIP } else { 0 };
        let y_flip = if self.y_flip { Y_FLIP } else { 0 };
        (self.number.0 & TILE_NUMBER) | x_flip | y_flip
    }
}

/// A BTS whose meaning for its block type the engine doesn't name yet, as the original
/// stores it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bts(pub u8);

/// A slope's shape and flips (bank `$94` log, the slope collision routines `$94:8D2B` on).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Slope {
    /// The slope definition, BTS bits 0-4: shapes below 5 are the square slopes.
    pub shape: u8,
    /// BTS bit 5.
    // open: no slope routine in bank $94 reads bit 5; kept so the BTS goes back out exact.
    pub bit5: bool,
    /// Mirrored left to right, bit 6.
    pub x_flip: bool,
    /// Mirrored top to bottom, bit 7.
    pub y_flip: bool,
}

impl Slope {
    const SHAPE: u8 = 0x1f;
    const BIT5: u8 = 0x20;
    const X_FLIP: u8 = 0x40;
    const Y_FLIP: u8 = 0x80;

    const fn from_bts(bts: u8) -> Self {
        Self {
            shape: bts & Self::SHAPE,
            bit5: bts & Self::BIT5 != 0,
            x_flip: bts & Self::X_FLIP != 0,
            y_flip: bts & Self::Y_FLIP != 0,
        }
    }

    fn bts(self) -> u8 {
        let flag = |set: bool, bit: u8| if set { bit } else { 0 };
        (self.shape & Self::SHAPE)
            | flag(self.bit5, Self::BIT5)
            | flag(self.x_flip, Self::X_FLIP)
            | flag(self.y_flip, Self::Y_FLIP)
    }
}

/// The block an extension block stands for: this many blocks on, signed, along its axis.
/// Zero acts as air (bank `$94` log, `$94:9411`, `$94:9447`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Offset(pub i8);

/// A door block's door: its index in the room's door list (bank `$94` log, the door
/// collision reaction).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DoorIndex(pub u8);

/// The PLM a shootable or special block spawns when it reacts (bank `$94` log, `$94:9E73`).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlmChoice {
    /// BTS bit 7 clear: entry `n` of the block type's PLM table (`$94:9EA6` shootable,
    /// `$94:9139` special).
    Common(u8),
    /// BTS bit 7 set: entry `n` (bits 0-6) of the current area's table (`$94:9FC6` shootable).
    Area(u8),
}

/// What a bombable block does when bombed (bank `$94` log, `$94:9FD6`, `$94:9FF4`).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BombableBts {
    /// BTS bit 7 clear: spawns entry `n` of `$94:A012`.
    Plm(u8),
    /// BTS bit 7 set: spawns nothing; bits 0-6 as stored.
    Inert(u8),
}

const HIGH_BIT: u8 = 0x80;

impl PlmChoice {
    const fn from_bts(bts: u8) -> Self {
        if bts & HIGH_BIT == 0 {
            Self::Common(bts)
        } else {
            Self::Area(bts & !HIGH_BIT)
        }
    }

    const fn bts(self) -> u8 {
        match self {
            Self::Common(index) => index & !HIGH_BIT,
            Self::Area(index) => index | HIGH_BIT,
        }
    }
}

impl BombableBts {
    const fn from_bts(bts: u8) -> Self {
        if bts & HIGH_BIT == 0 {
            Self::Plm(bts)
        } else {
            Self::Inert(bts & !HIGH_BIT)
        }
    }

    const fn bts(self) -> u8 {
        match self {
            Self::Plm(index) => index & !HIGH_BIT,
            Self::Inert(bits) => bits | HIGH_BIT,
        }
    }
}

/// A block's type, the top nybble of its word, with its BTS (bank `$94` log, the type
/// tables at `$94:82E1` and `$94:8301`): types 0-7 let Samus through, 8-F stop her.
// open: the BTS of air, spike air, special air, unused air, solid, spike, and grapple
// blocks stays raw until the reactions that read it are built (C4 on): spike BTS 0 is the
// generic spike, 1 Kraid's, 3 Draygon's broken turret; special air 8-B are treadmills.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// Type 0.
    Air(Bts),
    /// Type 1.
    Slope(Slope),
    /// Type 2.
    SpikeAir(Bts),
    /// Type 3.
    SpecialAir(Bts),
    /// Type 4.
    ShootableAir(PlmChoice),
    /// Type 5.
    HorizontalExtension(Offset),
    /// Type 6.
    UnusedAir(Bts),
    /// Type 7.
    BombableAir(BombableBts),
    /// Type 8.
    Solid(Bts),
    /// Type 9.
    Door(DoorIndex),
    /// Type A.
    Spike(Bts),
    /// Type B: crumble and speed blocks.
    Special(PlmChoice),
    /// Type C.
    Shootable(PlmChoice),
    /// Type D.
    VerticalExtension(Offset),
    /// Type E.
    Grapple(Bts),
    /// Type F.
    Bombable(BombableBts),
}

impl Default for BlockKind {
    fn default() -> Self {
        Self::Air(Bts::default())
    }
}

impl BlockKind {
    /// The kind of a block of type `nybble` (its low four bits) with `bts`.
    #[must_use]
    pub const fn new(nybble: u8, bts: u8) -> Self {
        let raw = Bts(bts);
        let offset = Offset(i8::from_le_bytes([bts]));
        match nybble & 0x0f {
            0x0 => Self::Air(raw),
            0x1 => Self::Slope(Slope::from_bts(bts)),
            0x2 => Self::SpikeAir(raw),
            0x3 => Self::SpecialAir(raw),
            0x4 => Self::ShootableAir(PlmChoice::from_bts(bts)),
            0x5 => Self::HorizontalExtension(offset),
            0x6 => Self::UnusedAir(raw),
            0x7 => Self::BombableAir(BombableBts::from_bts(bts)),
            0x8 => Self::Solid(raw),
            0x9 => Self::Door(DoorIndex(bts)),
            0xa => Self::Spike(raw),
            0xb => Self::Special(PlmChoice::from_bts(bts)),
            0xc => Self::Shootable(PlmChoice::from_bts(bts)),
            0xd => Self::VerticalExtension(offset),
            0xe => Self::Grapple(raw),
            _ => Self::Bombable(BombableBts::from_bts(bts)),
        }
    }

    /// The block type, as the word's top nybble holds it.
    #[must_use]
    pub const fn nybble(self) -> u8 {
        match self {
            Self::Air(_) => 0x0,
            Self::Slope(_) => 0x1,
            Self::SpikeAir(_) => 0x2,
            Self::SpecialAir(_) => 0x3,
            Self::ShootableAir(_) => 0x4,
            Self::HorizontalExtension(_) => 0x5,
            Self::UnusedAir(_) => 0x6,
            Self::BombableAir(_) => 0x7,
            Self::Solid(_) => 0x8,
            Self::Door(_) => 0x9,
            Self::Spike(_) => 0xa,
            Self::Special(_) => 0xb,
            Self::Shootable(_) => 0xc,
            Self::VerticalExtension(_) => 0xd,
            Self::Grapple(_) => 0xe,
            Self::Bombable(_) => 0xf,
        }
    }

    /// The BTS, as the original stores it.
    #[must_use]
    pub fn bts(self) -> u8 {
        match self {
            Self::Air(raw)
            | Self::SpikeAir(raw)
            | Self::SpecialAir(raw)
            | Self::UnusedAir(raw)
            | Self::Solid(raw)
            | Self::Spike(raw)
            | Self::Grapple(raw) => raw.0,
            Self::Slope(slope) => slope.bts(),
            Self::ShootableAir(choice) | Self::Special(choice) | Self::Shootable(choice) => {
                choice.bts()
            }
            Self::HorizontalExtension(offset) | Self::VerticalExtension(offset) => {
                let [byte] = offset.0.to_le_bytes();
                byte
            }
            Self::BombableAir(bombable) | Self::Bombable(bombable) => bombable.bts(),
            Self::Door(door) => door.0,
        }
    }
}

/// A layer 1 block: what it draws and what it does.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Block {
    /// Its type and BTS.
    pub kind: BlockKind,
    /// The tile it draws.
    pub tile: Tile,
}

impl Block {
    /// The block of a level data word and its BTS.
    #[must_use]
    pub const fn new(word: u16, bts: u8) -> Self {
        let [nybble, _] = (word >> TYPE_SHIFT).to_le_bytes();
        Self {
            kind: BlockKind::new(nybble, bts),
            tile: Tile::from_word(word),
        }
    }

    /// The level data word, as the original stores it.
    #[must_use]
    pub fn word(self) -> u16 {
        self.tile.word() | (u16::from(self.kind.nybble()) << TYPE_SHIFT)
    }
}

/// A layer 2 block: only its tile is drawn.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Layer2Block {
    /// The tile it draws.
    pub tile: Tile,
    /// The word's type bits, which nothing reads for layer 2, as stored.
    pub unread: u8,
}

impl Layer2Block {
    /// The block a level data word holds.
    #[must_use]
    pub const fn from_word(word: u16) -> Self {
        let [unread, _] = (word >> TYPE_SHIFT).to_le_bytes();
        Self {
            tile: Tile::from_word(word),
            unread,
        }
    }

    /// The level data word, as the original stores it.
    #[must_use]
    pub fn word(self) -> u16 {
        self.tile.word() | (u16::from(self.unread) << TYPE_SHIFT)
    }
}

/// A room's decoded level data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Level {
    /// Layer 1, row-major from the top left.
    pub layer1: Vec<Block>,
    /// Layer 2, when the level data has one; otherwise the room draws its library background.
    pub layer2: Option<Vec<Layer2Block>>,
}

/// Why level data doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelError {
    /// Shorter than the size word.
    NoSize,
    /// Layer 1's size is odd: not a word a block.
    OddSize(u16),
    /// The length is neither layer 1 and the BTS nor those and layer 2.
    Length {
        /// Blocks, from the size word.
        blocks: usize,
        /// The data's length.
        len: usize,
    },
}

impl Level {
    /// The level in `bytes`, decompressed level data; the length says whether layer 2 is
    /// there: 2 + 3n bytes without it, 2 + 5n with it, n the blocks the size word gives.
    ///
    /// # Errors
    ///
    /// [`LevelError`] when the size word is missing or odd, or the length fits neither.
    pub fn decode(bytes: &[u8]) -> Result<Self, LevelError> {
        let (size, rest) = bytes.split_first_chunk::<2>().ok_or(LevelError::NoSize)?;
        let size = u16::from_le_bytes(*size);
        if size & 1 != 0 {
            return Err(LevelError::OddSize(size));
        }
        let blocks = usize::from(size >> 1);
        let length = LevelError::Length {
            blocks,
            len: bytes.len(),
        };
        let (layer1, rest) = rest.split_at_checked(usize::from(size)).ok_or(length)?;
        let (bts, layer2) = rest.split_at_checked(blocks).ok_or(length)?;
        let words = |data: &[u8]| -> Vec<u16> {
            data.as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes(*pair))
                .collect()
        };
        let layer2 = match layer2.len() {
            0 => None,
            len if len == usize::from(size) => Some(
                words(layer2)
                    .into_iter()
                    .map(Layer2Block::from_word)
                    .collect(),
            ),
            _ => return Err(length),
        };
        let layer1 = words(layer1)
            .into_iter()
            .zip(bts)
            .map(|(word, &bts)| Block::new(word, bts))
            .collect();
        Ok(Self { layer1, layer2 })
    }
}

/// A screen's scroll permission (bank `$80` log, the camera's scroll limits).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollArea {
    /// 0: the camera may not enter.
    #[default]
    Red,
    /// 1: the camera may enter; the HUD hides its bottom.
    Blue,
    /// 2: the camera may enter freely.
    Green,
}

impl TryFrom<u8> for ScrollArea {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::Red),
            1 => Ok(Self::Blue),
            2 => Ok(Self::Green),
            other => Err(other),
        }
    }
}

impl From<ScrollArea> for u8 {
    fn from(area: ScrollArea) -> Self {
        match area {
            ScrollArea::Red => 0,
            ScrollArea::Blue => 1,
            ScrollArea::Green => 2,
        }
    }
}

/// Why a scroll table doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollError {
    /// The table is outside the image.
    Address(AddressError),
    /// A screen's byte, or a generated table's bottom row, is no scroll area.
    Area {
        /// The screen, row-major.
        screen: usize,
        /// The byte.
        value: u8,
    },
}

impl From<AddressError> for ScrollError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The scroll areas a state's scroll pointer gives a room of `size`, a screen each,
/// row-major (bank `$82` log, `$82:EA73`, on loading a room from a save).
///
/// A pointer in `$8F:8000`-`$FFFF` names a table of a byte a screen; one below `$8000` has
/// no table: every screen is green but the bottom row, which is the pointer's low byte plus
/// one.
///
/// # Errors
///
/// [`ScrollError`] when the table is outside the image or holds a byte no area has.
pub fn scroll_areas(
    rom: Lorom<'_>,
    pointer: RomAddress,
    size: Screens,
) -> Result<Vec<ScrollArea>, ScrollError> {
    let width = usize::from(size.width);
    let screens = width.wrapping_mul(usize::from(size.height));
    let area = |screen: usize, value: u8| {
        ScrollArea::try_from(value).map_err(|value| ScrollError::Area { screen, value })
    };
    if pointer.offset & 0x8000 == 0 {
        let [low, _] = pointer.offset.to_le_bytes();
        let bottom = screens.saturating_sub(width);
        return (0..screens)
            .map(|screen| {
                if screen < bottom {
                    Ok(ScrollArea::Green)
                } else {
                    area(screen, low.wrapping_add(1))
                }
            })
            .collect();
    }
    rom.read(pointer, screens)?
        .iter()
        .enumerate()
        .map(|(screen, &value)| area(screen, value))
        .collect()
}

#[cfg(test)]
mod tests;
