//! Samus's animation delay lists: how long each animation frame of a pose shows, and the
//! instructions that end, loop or leave the animation.
//!
//! Bank $91 log, `$91:B010` "Animation delay table": a bank $91 pointer a pose to a list of
//! bytes. A byte below $80 is a frame's delay; a byte with its high bit set is an
//! instruction, dispatched on its low nybble by bank $90's "Handle Samus animation delay"
//! (`$90:82DC`, its table at `$90:8324`), some of them with operand bytes after it. The
//! running list for when Samus has running momentum without the speed booster is at
//! `$91:B5D1` (<https://patrickjohnston.org/bank/91#fB010>,
//! <https://patrickjohnston.org/bank/90#f82DC>).
//!
//! The animation frame is a byte offset into the list, and the original reads whatever byte
//! it points at, an operand included, so every offset of a list decodes as an item.

use canonical::Canonical;

use super::{POSE_COUNT, PoseId};
use crate::rom::{AddressError, Lorom, RomAddress};

/// An animation frame: the byte offset into the pose's delay list the original keeps.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimationFrame(pub u16);

/// Equipment bits as the original's equipped items word holds them, an operand of the
/// item-dependent instructions.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemBits(pub u16);

/// The poses a transition picks between by Samus's Y speed.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BySpeed {
    /// When her Y speed is zero.
    pub still: PoseId,
    /// Otherwise.
    pub moving: PoseId,
}

/// A delay list instruction, by the low nybble of its byte (`$90:8324`).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    /// Nybbles 0 to 5 (`$90:8344`): nothing, the frame stays; the byte is kept.
    NoOp(u8),
    /// 6 (`$90:8346`): back to the start if Samus's health is at least 30, else on.
    LoopIfHealthy,
    /// 7 (`$90:8360`): the drained Samus falling movement handler, then on.
    DrainedFalling,
    /// 8 (`$90:8370`): outside the demos, unless the prospective pose is a jump, enable the
    /// auto-jump hack and transition to the pose.
    AutoJumpTransition(PoseId),
    /// 9 (`$90:839A`): transition by whether any of the items is equipped, then by Y speed.
    ByItemsAndSpeed {
        /// The items.
        items: ItemBits,
        /// Without them.
        without: BySpeed,
        /// With one of them.
        with: BySpeed,
    },
    /// $A (`$90:83F6`, unused): transition by Y speed.
    BySpeed(BySpeed),
    /// $B (`$90:841D`): the wall jump's delay sequence for her items and the liquid.
    WallJump,
    /// $C (`$90:848B`, unused): transition by whether any of the items is equipped.
    ByItems {
        /// The items.
        items: ItemBits,
        /// The pose without them.
        without: PoseId,
        /// The pose with one of them.
        with: PoseId,
    },
    /// $D (`$90:84B6`): transition to the pose, the animation finished.
    Transition(PoseId),
    /// $E (`$90:84C7`): back this many bytes.
    GoBack(u8),
    /// $F (`$90:84DB`): back to the start.
    Loop,
}

/// What a delay list holds at a frame.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DelayItem {
    /// Show the frame this many frames.
    Delay(u8),
    /// Run an instruction.
    Instruction(Instruction),
}

impl Default for DelayItem {
    fn default() -> Self {
        Self::Delay(0)
    }
}

/// The bytes a wall jump's sequence reaches past its instruction: its screw attack
/// sequence starts $15 bytes on (`$90:8477`).
const WALL_JUMP_REACH: usize = 0x16;

/// The longest list the decoder walks before it gives up on finding the end.
const LIST_LIMIT: usize = 0x100;

/// Why the delay lists don't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationError {
    /// A pointer or a list is outside the image.
    Address(AddressError),
    /// A pose's list runs on past [`LIST_LIMIT`] bytes without an instruction that ends it.
    Unterminated(PoseId),
}

/// One delay list: the item at each frame, and the byte it is, which the original adds to
/// the frame timer whatever the item (`$90:84E3`, `$90:8315`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DelayList {
    bytes: Vec<u8>,
    items: Vec<DelayItem>,
}

impl DelayList {
    /// The item at `frame`; `None` past the list's end.
    #[must_use]
    pub fn at(&self, frame: AnimationFrame) -> Option<DelayItem> {
        self.items.get(usize::from(frame.0)).copied()
    }

    /// The byte at `frame`; `None` past the list's end.
    #[must_use]
    pub fn byte(&self, frame: AnimationFrame) -> Option<u8> {
        self.bytes.get(usize::from(frame.0)).copied()
    }

    /// The list's bytes as items, from `bytes` (which run on past its end): up to the
    /// instruction that ends it, past every byte a wall jump reaches; `None` when no
    /// instruction ends it within [`LIST_LIMIT`] bytes.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut reach = 0_usize;
        let mut at = 0_usize;
        let end = loop {
            if at >= LIST_LIMIT {
                return None;
            }
            let item = item(bytes.get(at..)?)?;
            let (ends, size) = match item {
                DelayItem::Delay(_)
                | DelayItem::Instruction(
                    Instruction::NoOp(_) | Instruction::LoopIfHealthy | Instruction::DrainedFalling,
                ) => (false, 1),
                DelayItem::Instruction(Instruction::WallJump) => {
                    reach = reach.max(at.wrapping_add(WALL_JUMP_REACH));
                    (false, 1)
                }
                DelayItem::Instruction(Instruction::Loop) => (true, 1),
                DelayItem::Instruction(
                    Instruction::AutoJumpTransition(_)
                    | Instruction::Transition(_)
                    | Instruction::GoBack(_),
                ) => (true, 2),
                DelayItem::Instruction(Instruction::BySpeed(_)) => (true, 3),
                DelayItem::Instruction(Instruction::ByItems { .. }) => (true, 5),
                DelayItem::Instruction(Instruction::ByItemsAndSpeed { .. }) => (true, 7),
            };
            at = at.wrapping_add(size);
            if ends && at >= reach {
                break at;
            }
        };
        let items = (0..end)
            .map(|offset| bytes.get(offset..).and_then(item))
            .collect::<Option<_>>()?;
        Some(Self {
            bytes: bytes.get(..end)?.to_vec(),
            items,
        })
    }
}

/// The item a list's bytes start with, its operands read from the bytes after it; `None`
/// when they run out.
fn item(bytes: &[u8]) -> Option<DelayItem> {
    let (&byte, operands) = bytes.split_first()?;
    if byte & 0x80 == 0 {
        return Some(DelayItem::Delay(byte));
    }
    let pose = |index: usize| operands.get(index).map(|&pose| PoseId(u16::from(pose)));
    let items = || {
        Some(ItemBits(u16::from_le_bytes([
            *operands.first()?,
            *operands.get(1)?,
        ])))
    };
    let instruction = match byte & 0x0F {
        6 => Instruction::LoopIfHealthy,
        7 => Instruction::DrainedFalling,
        8 => Instruction::AutoJumpTransition(pose(0)?),
        9 => Instruction::ByItemsAndSpeed {
            items: items()?,
            without: BySpeed {
                still: pose(2)?,
                moving: pose(3)?,
            },
            with: BySpeed {
                still: pose(4)?,
                moving: pose(5)?,
            },
        },
        0x0A => Instruction::BySpeed(BySpeed {
            still: pose(0)?,
            moving: pose(1)?,
        }),
        0x0B => Instruction::WallJump,
        0x0C => Instruction::ByItems {
            items: items()?,
            without: pose(2)?,
            with: pose(3)?,
        },
        0x0D => Instruction::Transition(pose(0)?),
        0x0E => Instruction::GoBack(*operands.first()?),
        0x0F => Instruction::Loop,
        _ => Instruction::NoOp(byte),
    };
    Some(DelayItem::Instruction(instruction))
}

/// The table's first pointer (bank $91 log).
pub const ANIMATION_DELAYS: RomAddress = RomAddress::new(0x91, 0xB010);

/// The pointer to the running list without the speed booster (bank $91 log).
pub const RUNNING_DELAYS: RomAddress = RomAddress::new(0x91, 0xB5D1);

/// The bank every list is in.
const LIST_BANK: u8 = 0x91;

/// Every pose's delay list, and the running list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimationDelays {
    poses: Vec<DelayList>,
    running: DelayList,
}

impl AnimationDelays {
    /// The table in `rom`, a list for each of the [`POSE_COUNT`] poses.
    ///
    /// # Errors
    ///
    /// [`AnimationError`] when a pointer or a list is outside the image or a list has no end.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AnimationError> {
        let list_at = |pointer: RomAddress, pose: PoseId| {
            let target = rom
                .read(pointer, 2)
                .map_err(AnimationError::Address)?
                .first_chunk::<2>()
                .ok_or(AnimationError::Address(AddressError::PastEnd(pointer)))?;
            let target = RomAddress::new(LIST_BANK, u16::from_le_bytes(*target));
            let bytes = rom.from(target).map_err(AnimationError::Address)?;
            DelayList::decode(bytes).ok_or(AnimationError::Unterminated(pose))
        };
        let poses = (0..POSE_COUNT)
            .map(|index| {
                let pointer = ANIMATION_DELAYS.advance(index.wrapping_mul(2)).ok_or(
                    AnimationError::Address(AddressError::PastEnd(ANIMATION_DELAYS)),
                )?;
                let pose = PoseId(u16::try_from(index).unwrap_or(u16::MAX));
                list_at(pointer, pose)
            })
            .collect::<Result<_, _>>()?;
        let running = list_at(RUNNING_DELAYS, PoseId(u16::MAX))?;
        Ok(Self { poses, running })
    }

    /// The table from each pose's list and the running list.
    #[must_use]
    pub const fn new(poses: Vec<DelayList>, running: DelayList) -> Self {
        Self { poses, running }
    }

    /// The list of `pose`; `None` past the table.
    #[must_use]
    pub fn of(&self, pose: PoseId) -> Option<&DelayList> {
        self.poses.get(usize::from(pose.0))
    }

    /// The running list for when Samus has running momentum without the speed booster.
    #[must_use]
    pub const fn running(&self) -> &DelayList {
        &self.running
    }
}

#[cfg(test)]
mod tests;
