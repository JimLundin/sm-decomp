//! The Morph Ball Room eye's instruction lists, by what its AI starts them for.

use crate::rom::RomAddress;

use super::program::{EnemyPrograms, ProgramError, Spritemap, SpritemapId, Target};

/// The eye's AI bank.
const BANK: u8 = 0xa8;
/// The active list (`$A8:8FAC`): one four-byte frame for each 10h of the angle to Samus.
const ACTIVE: u16 = 0x8fac;
/// The bytes an active frame takes.
const ACTIVE_FRAME_BYTES: u16 = 4;
/// The active frames: one for each 10h of an 8-bit angle.
pub const ACTIVE_FRAMES: usize = 16;
/// The empty spritemap the init sets (`$A8:9058`).
const EMPTY: u16 = 0x804d;

/// Which way the eye faces: the first parameter's bit 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    /// Bit 0 clear.
    Right,
    /// Bit 0 set.
    Left,
}

/// A list for each facing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ByFacing {
    /// Facing right.
    pub right: Target,
    /// Facing left.
    pub left: Target,
}

impl ByFacing {
    /// The list for `facing`.
    #[must_use]
    pub const fn get(self, facing: Facing) -> Target {
        match facing {
            Facing::Right => self.right,
            Facing::Left => self.left,
        }
    }
}

/// Which side of the eye a mount is on: the second parameter's low nybble, in the order of
/// the init's tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountSide {
    /// 0: 8 pixels left.
    Left,
    /// 1: 8 pixels right.
    Right,
    /// 2: 8 pixels up.
    Up,
    /// 3: 8 pixels down.
    Down,
}

/// A list for each mount side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ByMountSide {
    /// Looking left.
    pub left: Target,
    /// Looking right.
    pub right: Target,
    /// Looking up.
    pub up: Target,
    /// Looking down.
    pub down: Target,
}

impl ByMountSide {
    /// The list for `side`.
    #[must_use]
    pub const fn get(self, side: MountSide) -> Target {
        match side {
            MountSide::Left => self.left,
            MountSide::Right => self.right,
            MountSide::Up => self.up,
            MountSide::Down => self.down,
        }
    }
}

/// The eye's lists as places in the loaded programs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EyeLists {
    /// The active frames, by the angle's high nybble, chosen every frame.
    pub active: [Target; ACTIVE_FRAMES],
    /// Closed, by facing.
    pub closed: ByFacing,
    /// Opening, by facing.
    pub activating: ByFacing,
    /// Closing, by facing.
    pub deactivating: ByFacing,
    /// A mounted eye's, by its side.
    pub mount: ByMountSide,
    /// The empty spritemap its init sets.
    pub empty: SpritemapId,
}

impl EyeLists {
    /// The eye's lists in `programs`.
    ///
    /// # Errors
    ///
    /// [`ProgramError::Unloaded`] for a list or spritemap `programs` doesn't hold.
    pub fn resolve(programs: &EnemyPrograms) -> Result<Self, ProgramError> {
        let place = |offset: u16| {
            let at = RomAddress::new(BANK, offset);
            programs.target(at).ok_or(ProgramError::Unloaded(at))
        };
        let by_facing = |right, left| -> Result<ByFacing, ProgramError> {
            Ok(ByFacing {
                right: place(right)?,
                left: place(left)?,
            })
        };
        let mut active = [Target::default(); ACTIVE_FRAMES];
        for (frame, offset) in active
            .iter_mut()
            .zip((ACTIVE..).step_by(usize::from(ACTIVE_FRAME_BYTES)))
        {
            *frame = place(offset)?;
        }
        let empty = RomAddress::new(BANK, EMPTY);
        Ok(Self {
            active,
            closed: by_facing(0x8ffc, 0x900e)?,
            activating: by_facing(0x9014, 0x9026)?,
            deactivating: by_facing(0x8ff0, 0x9002)?,
            mount: ByMountSide {
                left: place(0x9044)?,
                right: place(0x9038)?,
                up: place(0x904a)?,
                down: place(0x903e)?,
            },
            empty: programs
                .spritemap_id(Spritemap(empty))
                .ok_or(ProgramError::Unloaded(empty))?,
        })
    }
}

impl TryFrom<u16> for MountSide {
    type Error = u16;

    /// The side the second parameter's low nybble names; a nybble past the init's four-entry
    /// tables is the error.
    fn try_from(nybble: u16) -> Result<Self, u16> {
        match nybble {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            2 => Ok(Self::Up),
            3 => Ok(Self::Down),
            other => Err(other),
        }
    }
}

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// The Morph Ball Room eye's instruction lists, by what its AI starts them for (bank $A8
// log, <https://patrickjohnston.org/bank/A8>, `$A8:8FAC` to `$A8:904A` and `$A8:90DA`).
// Which way the eye faces: the first parameter's bit 0 (`$A8:90F1`).
// the init's tables (`$A8:90CA`, `$A8:90D2`, `$A8:90DA`).
// Left (`$A8:9044`).
// Right (`$A8:9038`).
// Up (`$A8:904A`).
// Down (`$A8:903E`).
// The active frames, by the angle's high nybble (`$A8:8FAC`, "indexed by [enemy angle]
// Closed (`$A8:8FFC`, `$A8:900E`).
// Activating (`$A8:9014`, `$A8:9026`).
// Deactivating (`$A8:8FF0`, `$A8:9002`).
// A mount's, by its side (`$A8:90DA`).
