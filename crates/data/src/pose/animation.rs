//! Samus's animation delay lists: how long each animation frame of a pose shows, and the
//! instructions that end, loop or leave the animation.
//!
//! A pointer a pose names a list of bytes. A byte below 0x80 is a frame's delay; a byte with
//! its high bit set is an instruction, dispatched on its low nybble, some of them with
//! operand bytes after it. The running list for when Samus has running momentum without the
//! speed booster is a separate list.
//!
//! The animation frame is a byte offset into the list, and the original reads whatever byte
//! it points at, an operand included, so every offset of a list decodes as an item.

use canonical::Canonical;

use super::{MovementType, POSE_COUNT, PoseId, Poses};
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

/// A delay list instruction, by the low nybble of its byte.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    /// Nybbles 0 to 5: nothing, the frame stays; the byte is kept.
    NoOp(u8),
    /// 6: back to the start if Samus's health is at least 30, else on.
    LoopIfHealthy,
    /// 7: the drained Samus falling movement handler, then on.
    DrainedFalling,
    /// 8: outside the demos, unless the prospective pose is a jump, enable the
    /// auto-jump hack and transition to the pose.
    AutoJumpTransition(PoseId),
    /// 9: transition by whether any of the items is equipped, then by Y speed.
    ByItemsAndSpeed {
        /// The items.
        items: ItemBits,
        /// Without them.
        without: BySpeed,
        /// With one of them.
        with: BySpeed,
    },
    /// 0xA (unused): transition by Y speed.
    BySpeed(BySpeed),
    /// 0xB: the wall jump's delay sequence for her items and the liquid.
    WallJump,
    /// 0xC (unused): transition by whether any of the items is equipped.
    ByItems {
        /// The items.
        items: ItemBits,
        /// The pose without them.
        without: PoseId,
        /// The pose with one of them.
        with: PoseId,
    },
    /// 0xD: transition to the pose, the animation finished.
    Transition(PoseId),
    /// 0xE: back this many bytes.
    GoBack(u8),
    /// 0xF: back to the start.
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

impl DelayItem {
    /// The bytes it takes: the instruction and its operands.
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Delay(_)
            | Self::Instruction(
                Instruction::NoOp(_)
                | Instruction::LoopIfHealthy
                | Instruction::DrainedFalling
                | Instruction::WallJump
                | Instruction::Loop,
            ) => 1,
            Self::Instruction(
                Instruction::AutoJumpTransition(_)
                | Instruction::Transition(_)
                | Instruction::GoBack(_),
            ) => 2,
            Self::Instruction(Instruction::BySpeed(_)) => 3,
            Self::Instruction(Instruction::ByItems { .. }) => 5,
            Self::Instruction(Instruction::ByItemsAndSpeed { .. }) => 7,
        }
    }
}

impl Default for DelayItem {
    fn default() -> Self {
        Self::Delay(0)
    }
}

/// The bytes a wall jump's sequence reaches past its instruction: its screw attack
/// sequence starts $15 bytes on (`$90:8477`).
const WALL_JUMP_REACH: usize = 0x16;

/// The frame from which a spin jump can wall jump.
///
/// The wall jump check compares the frame with it and, while the spin hasn't reached it,
/// sets the frame one before it, so the next tick reaches it whatever the delay list says.
pub const WALL_JUMP_FRAME: AnimationFrame = AnimationFrame(0x0B);
/// The screw attack's wall jump frame.
pub const SCREW_ATTACK_WALL_JUMP_FRAME: AnimationFrame = AnimationFrame(0x1B);
/// The screw attack poses, facing right and left.
const SCREW_ATTACK: [PoseId; 2] = [PoseId(0x81), PoseId(0x82)];

/// The frame from which a spin jump in `pose` can wall jump: the screw attack's or the
/// other spins'.
#[must_use]
pub fn wall_jump_frame(pose: PoseId) -> AnimationFrame {
    if SCREW_ATTACK.contains(&pose) {
        SCREW_ATTACK_WALL_JUMP_FRAME
    } else {
        WALL_JUMP_FRAME
    }
}

/// The falls whose animation a fast fall steps to [`FAST_FALL_FRAME`].
pub const FAST_FALLS: [PoseId; 4] = [PoseId(0x29), PoseId(0x2A), PoseId(0x67), PoseId(0x68)];
/// The frame a fast fall steps a fall's animation to.
pub const FAST_FALL_FRAME: AnimationFrame = AnimationFrame(5);
/// The straight jumps whose animation holds [`RISING_FRAME`] while she rises.
pub const RISINGS: [PoseId; 2] = [PoseId(0x4D), PoseId(0x4E)];
/// The frame a straight jump's animation holds while she rises.
pub const RISING_FRAME: AnimationFrame = AnimationFrame(1);

/// The frame `pose`'s animation reaches whatever its delay list says, if any.
///
/// That is a spin jump's wall jump frame, which its wall jump check steps the animation to, a
/// fall's fast fall frame and a straight jump's rising frame. The original reads on past a
/// list's end to any of them as it would the list.
#[must_use]
pub fn reached(poses: &Poses, pose: PoseId) -> Option<AnimationFrame> {
    let spin_jump = matches!(poses.get(pose)?.movement, MovementType::SpinJumping);
    if spin_jump {
        Some(wall_jump_frame(pose))
    } else if FAST_FALLS.contains(&pose) {
        Some(FAST_FALL_FRAME)
    } else {
        RISINGS.contains(&pose).then_some(RISING_FRAME)
    }
}

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
/// the frame timer whatever the item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DelayList {
    bytes: Vec<u8>,
    items: Vec<DelayItem>,
}

impl DelayList {
    /// How many bytes the list has: the frames her animation can reach and the instruction
    /// bytes after them.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the list has no bytes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

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

    /// The items the list steps through from its start, each with its frame: an operand byte is
    /// no item's.
    pub fn steps(&self) -> impl Iterator<Item = (AnimationFrame, DelayItem)> + '_ {
        let mut at = 0_usize;
        core::iter::from_fn(move || {
            let frame = AnimationFrame(u16::try_from(at).ok()?);
            let item = self.at(frame)?;
            at = at.wrapping_add(item.size());
            Some((frame, item))
        })
    }

    /// The list's bytes as items, from `bytes` (which run on past its end): up to the
    /// instruction that ends it, past every byte a wall jump reaches; `None` when no
    /// instruction ends it within [`LIST_LIMIT`] bytes.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        Self::decode_through(bytes, None)
    }

    /// [`Self::decode`], on through `reached` when the animation gets there past the end:
    /// the original reads the bytes after the list as frames like the ones in it.
    #[must_use]
    pub fn decode_through(bytes: &[u8], reached: Option<AnimationFrame>) -> Option<Self> {
        let mut reach = reached.map_or(0, |frame| usize::from(frame.0).wrapping_add(1));
        let mut at = 0_usize;
        let end = loop {
            if at >= LIST_LIMIT {
                return None;
            }
            let item = item(bytes.get(at..)?)?;
            if item == DelayItem::Instruction(Instruction::WallJump) {
                reach = reach.max(at.wrapping_add(WALL_JUMP_REACH));
            }
            let ends = !matches!(
                item,
                DelayItem::Delay(_)
                    | DelayItem::Instruction(
                        Instruction::NoOp(_)
                            | Instruction::LoopIfHealthy
                            | Instruction::DrainedFalling
                            | Instruction::WallJump
                    )
            );
            at = at.wrapping_add(item.size());
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

/// The table's first pointer.
const ANIMATION_DELAYS: RomAddress = RomAddress::new(0x91, 0xB010);

/// The pointer to the running list without the speed booster.
const RUNNING_DELAYS: RomAddress = RomAddress::new(0x91, 0xB5D1);

/// The bank every list is in.
const LIST_BANK: u8 = 0x91;

/// Every pose's delay list, and the running list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimationDelays {
    poses: Vec<DelayList>,
    running: DelayList,
}

impl AnimationDelays {
    /// The table in `rom`, a list for each of the [`POSE_COUNT`] poses, a spin jump's on
    /// through the frame its wall jump check reaches ([`reached`] from `poses`).
    ///
    /// # Errors
    ///
    /// [`AnimationError`] when a pointer or a list is outside the image or a list has no end.
    pub fn load(rom: Lorom<'_>, poses: &Poses) -> Result<Self, AnimationError> {
        let list_at = |pointer: RomAddress, pose: PoseId, reached: Option<AnimationFrame>| {
            let target = rom
                .read(pointer, 2)
                .map_err(AnimationError::Address)?
                .first_chunk::<2>()
                .ok_or(AnimationError::Address(AddressError::PastEnd(pointer)))?;
            let target = RomAddress::new(LIST_BANK, u16::from_le_bytes(*target));
            let bytes = rom.from(target).map_err(AnimationError::Address)?;
            DelayList::decode_through(bytes, reached).ok_or(AnimationError::Unterminated(pose))
        };
        let poses = (0..POSE_COUNT)
            .map(|index| {
                let pointer = ANIMATION_DELAYS.advance(index.wrapping_mul(2)).ok_or(
                    AnimationError::Address(AddressError::PastEnd(ANIMATION_DELAYS)),
                )?;
                let pose = PoseId(u16::try_from(index).unwrap_or(u16::MAX));
                list_at(pointer, pose, reached(poses, pose))
            })
            .collect::<Result<_, _>>()?;
        let running = list_at(RUNNING_DELAYS, PoseId(u16::MAX), None)?;
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

/// The table of the boosted running lists' pointers, one per speed booster stage.
const BOOSTED_RUNNING_DELAYS: RomAddress = RomAddress::new(0x91, 0xB5DE);

/// The speed booster's stages, each with its own running list.
pub const BOOST_STAGES: usize = 5;

/// The most delays a running list holds before its first instruction.
const PACE_LIMIT: usize = 0x20;

/// The paces her run is played at: the delays of the running lists up to their first
/// instruction, one per frame of her running animations, without the speed booster and with
/// it at each of its stages.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunPaces {
    /// The pace without the speed booster.
    pub plain: Vec<u8>,
    /// The pace with it, by stage, from the first.
    pub boosted: [Vec<u8>; BOOST_STAGES],
}

impl RunPaces {
    /// The paces in `rom`.
    ///
    /// # Errors
    ///
    /// [`AnimationError::Address`] when a pointer or a list is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AnimationError> {
        let pace = |pointer: RomAddress| -> Result<Vec<u8>, AnimationError> {
            let target = rom
                .read(pointer, 2)
                .map_err(AnimationError::Address)?
                .first_chunk::<2>()
                .ok_or(AnimationError::Address(AddressError::PastEnd(pointer)))?;
            let list = rom
                .from(RomAddress::new(LIST_BANK, u16::from_le_bytes(*target)))
                .map_err(AnimationError::Address)?;
            Ok(list
                .iter()
                .take(PACE_LIMIT)
                .take_while(|&&byte| (byte & 0x80) == 0)
                .copied()
                .collect())
        };
        let mut boosted: [Vec<u8>; BOOST_STAGES] = Default::default();
        for (index, slot) in boosted.iter_mut().enumerate() {
            let pointer = BOOSTED_RUNNING_DELAYS
                .advance(index.wrapping_mul(2))
                .ok_or(AnimationError::Address(AddressError::PastEnd(
                    BOOSTED_RUNNING_DELAYS,
                )))?;
            *slot = pace(pointer)?;
        }
        Ok(Self {
            plain: pace(RUNNING_DELAYS)?,
            boosted,
        })
    }
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Bank $91 log, `$91:B010` "Animation delay table": a bank $91 pointer a pose to a list of
// instruction, dispatched on its low nybble by bank $90's "Handle Samus animation delay"
// (`$90:82DC`, its table at `$90:8324`), some of them with operand bytes after it. The
// `$91:B5D1` (<https://patrickjohnston.org/bank/91#fB010>,
// <https://patrickjohnston.org/bank/90#f82DC>).
// A delay list instruction, by the low nybble of its byte (`$90:8324`).
// Nybbles 0 to 5 (`$90:8344`): nothing, the frame stays; the byte is kept.
// 6 (`$90:8346`): back to the start if Samus's health is at least 30, else on.
// 7 (`$90:8360`): the drained Samus falling movement handler, then on.
// 8 (`$90:8370`): outside the demos, unless the prospective pose is a jump, enable the
// 9 (`$90:839A`): transition by whether any of the items is equipped, then by Y speed.
// $A (`$90:83F6`, unused): transition by Y speed.
// $B (`$90:841D`): the wall jump's delay sequence for her items and the liquid.
// $C (`$90:848B`, unused): transition by whether any of the items is equipped.
// $D (`$90:84B6`): transition to the pose, the animation finished.
// $E (`$90:84C7`): back this many bytes.
// $F (`$90:84DB`): back to the start.
// Bank `$90` log, wall jump check `$90:9D35`, which compares the frame with it (`$90:9D55`) and, while the spin hasn't
// reached it, sets the frame one before it (`$90:9DC8`), so the next tick reaches it
// The screw attack's (`$90:9D63`, set one before at `$90:9DD7`).
// The screw attack poses, facing right and left, which `$90:9D48` tells apart.
// the frame timer whatever the item (`$90:84E3`, `$90:8315`).
// The table's first pointer (bank $91 log).
// The pointer to the running list without the speed booster (bank $91 log).
// The boosted running lists, one pointer per speed booster stage (bank $91 log, `$91:B5DE`),
// which `$90:84EF` picks by `0B3F`.
// The fast fall (`$90:A58D`: poses 29h, 2Ah, 67h, 68h, frame 5, `$90:A5BC`) and the rising hold
// (`$90:8011`: poses 4Dh, 4Eh, frame 1).
