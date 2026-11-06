//! Samus's poses: the bank $91 pose definitions table, decoded into typed records.
//!
//! Bank $91 log, `$91:B629` "Pose definitions": eight bytes a pose, for poses 0 to $FC:
//! the X direction, the movement type, the pose to take when no button changes it, the
//! direction shots are fired in, the Y offset of the graphics and projectile origin, an
//! unused byte, the Y radius and another unused byte
//! (<https://patrickjohnston.org/bank/91#fB629>).

use canonical::Canonical;

use crate::rom::{AddressError, Lorom, RomAddress};

pub mod animation;
pub mod transition;

/// A pose: the index into the pose definitions, the number the original keeps in its pose
/// word, so it goes back out unchanged.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoseId(pub u16);

/// How a pose moves Samus: the movement type the bank $90 movement handler dispatches on.
///
/// The names follow the bank $90 log's movement type list.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MovementType {
    /// Standing.
    #[default]
    Standing,
    /// Running.
    Running,
    /// Normal jumping.
    NormalJumping,
    /// Spin jumping.
    SpinJumping,
    /// Morph ball, on the ground.
    MorphBallOnGround,
    /// Crouching.
    Crouching,
    /// Falling.
    Falling,
    /// Unused type 7.
    Unused07,
    /// Morph ball, falling.
    MorphBallFalling,
    /// Unused type 9.
    Unused09,
    /// Knockback, or the end of a crystal flash.
    KnockbackOrCrystalFlashEnding,
    /// Unused type $B.
    Unused0B,
    /// Unused type $C.
    Unused0C,
    /// Unused type $D.
    Unused0D,
    /// Turning around on the ground.
    TurningAroundOnGround,
    /// The crouching, standing, morphing and unmorphing transitions.
    CrouchingTransition,
    /// Moonwalking.
    Moonwalking,
    /// Spring ball, on the ground.
    SpringBallOnGround,
    /// Spring ball, in the air.
    SpringBallInAir,
    /// Spring ball, falling.
    SpringBallFalling,
    /// Wall jumping.
    WallJumping,
    /// Ran into a wall.
    RanIntoWall,
    /// Grappling.
    Grappling,
    /// Turning around while jumping.
    TurningAroundJumping,
    /// Turning around while falling.
    TurningAroundFalling,
    /// A damage boost.
    DamageBoost,
    /// Grabbed by Draygon.
    GrabbedByDraygon,
    /// Shinesparking, crystal flashing, drained by a metroid or hit by Mother Brain.
    Shinespark,
}

/// Every movement type, in the original's numbering.
const MOVEMENT_TYPES: [MovementType; 0x1C] = [
    MovementType::Standing,
    MovementType::Running,
    MovementType::NormalJumping,
    MovementType::SpinJumping,
    MovementType::MorphBallOnGround,
    MovementType::Crouching,
    MovementType::Falling,
    MovementType::Unused07,
    MovementType::MorphBallFalling,
    MovementType::Unused09,
    MovementType::KnockbackOrCrystalFlashEnding,
    MovementType::Unused0B,
    MovementType::Unused0C,
    MovementType::Unused0D,
    MovementType::TurningAroundOnGround,
    MovementType::CrouchingTransition,
    MovementType::Moonwalking,
    MovementType::SpringBallOnGround,
    MovementType::SpringBallInAir,
    MovementType::SpringBallFalling,
    MovementType::WallJumping,
    MovementType::RanIntoWall,
    MovementType::Grappling,
    MovementType::TurningAroundJumping,
    MovementType::TurningAroundFalling,
    MovementType::DamageBoost,
    MovementType::GrabbedByDraygon,
    MovementType::Shinespark,
];

impl TryFrom<u8> for MovementType {
    type Error = u8;

    /// The movement type numbered `value`; the movement handler's jump table has no entry
    /// past $1B, so a larger value is an error.
    fn try_from(value: u8) -> Result<Self, u8> {
        MOVEMENT_TYPES.get(usize::from(value)).copied().ok_or(value)
    }
}

impl From<MovementType> for u8 {
    fn from(movement: MovementType) -> Self {
        match movement {
            MovementType::Standing => 0x00,
            MovementType::Running => 0x01,
            MovementType::NormalJumping => 0x02,
            MovementType::SpinJumping => 0x03,
            MovementType::MorphBallOnGround => 0x04,
            MovementType::Crouching => 0x05,
            MovementType::Falling => 0x06,
            MovementType::Unused07 => 0x07,
            MovementType::MorphBallFalling => 0x08,
            MovementType::Unused09 => 0x09,
            MovementType::KnockbackOrCrystalFlashEnding => 0x0A,
            MovementType::Unused0B => 0x0B,
            MovementType::Unused0C => 0x0C,
            MovementType::Unused0D => 0x0D,
            MovementType::TurningAroundOnGround => 0x0E,
            MovementType::CrouchingTransition => 0x0F,
            MovementType::Moonwalking => 0x10,
            MovementType::SpringBallOnGround => 0x11,
            MovementType::SpringBallInAir => 0x12,
            MovementType::SpringBallFalling => 0x13,
            MovementType::WallJumping => 0x14,
            MovementType::RanIntoWall => 0x15,
            MovementType::Grappling => 0x16,
            MovementType::TurningAroundJumping => 0x17,
            MovementType::TurningAroundFalling => 0x18,
            MovementType::DamageBoost => 0x19,
            MovementType::GrabbedByDraygon => 0x1A,
            MovementType::Shinespark => 0x1B,
        }
    }
}

/// Which way a pose faces: the pose X direction byte.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PoseFacing {
    /// Toward the screen: the two facing-forward poses (0 and $9B), byte 0.
    Forward,
    /// Toward the left edge, byte 4.
    Left,
    /// Toward the right edge, byte 8.
    #[default]
    Right,
    /// Another byte: the unused poses $21 and $22 hold 1 and 2. The original's facing
    /// tests read the left and right bits, so it tolerates them (docs/LEGACY.md).
    Unknown(u8),
}

impl From<u8> for PoseFacing {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Forward,
            4 => Self::Left,
            8 => Self::Right,
            other => Self::Unknown(other),
        }
    }
}

impl From<PoseFacing> for u8 {
    fn from(facing: PoseFacing) -> Self {
        match facing {
            PoseFacing::Forward => 0,
            PoseFacing::Left => 4,
            PoseFacing::Right => 8,
            PoseFacing::Unknown(other) => other,
        }
    }
}

/// A direction shots leave in, numbered clockwise from up as the original does (0 to 9).
/// Up and down each have two, one per facing.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShotDirection {
    /// Up, facing right (0).
    UpFacingRight,
    /// Up and right (1).
    UpRight,
    /// Right (2).
    #[default]
    Right,
    /// Down and right (3).
    DownRight,
    /// Down, facing right (4).
    DownFacingRight,
    /// Down, facing left (5).
    DownFacingLeft,
    /// Down and left (6).
    DownLeft,
    /// Left (7).
    Left,
    /// Up and left (8).
    UpLeft,
    /// Up, facing left (9).
    UpFacingLeft,
}

/// Every shot direction, in the original's numbering.
const SHOT_DIRECTIONS: [ShotDirection; 10] = [
    ShotDirection::UpFacingRight,
    ShotDirection::UpRight,
    ShotDirection::Right,
    ShotDirection::DownRight,
    ShotDirection::DownFacingRight,
    ShotDirection::DownFacingLeft,
    ShotDirection::DownLeft,
    ShotDirection::Left,
    ShotDirection::UpLeft,
    ShotDirection::UpFacingLeft,
];

/// Whether a pose can fire, and where.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shot {
    /// Shots leave in this direction.
    Fires(ShotDirection),
    /// The high nybble is set: no shot, and the grapple beam is cancelled. The byte is
    /// kept whole ($FA, $FB, $FC or $FF in the table) so it goes back out unchanged.
    Withheld(u8),
}

impl Default for Shot {
    fn default() -> Self {
        Self::Fires(ShotDirection::default())
    }
}

impl TryFrom<u8> for Shot {
    type Error = u8;

    /// A byte with the high nybble clear names a direction 0 to 9; one from $A to $F isn't
    /// a direction the projectile code has a table entry for, so it is an error.
    fn try_from(value: u8) -> Result<Self, u8> {
        if value & 0xF0 != 0 {
            return Ok(Self::Withheld(value));
        }
        SHOT_DIRECTIONS
            .get(usize::from(value))
            .map(|&direction| Self::Fires(direction))
            .ok_or(value)
    }
}

impl From<Shot> for u8 {
    fn from(shot: Shot) -> Self {
        match shot {
            Shot::Fires(direction) => match direction {
                ShotDirection::UpFacingRight => 0,
                ShotDirection::UpRight => 1,
                ShotDirection::Right => 2,
                ShotDirection::DownRight => 3,
                ShotDirection::DownFacingRight => 4,
                ShotDirection::DownFacingLeft => 5,
                ShotDirection::DownLeft => 6,
                ShotDirection::Left => 7,
                ShotDirection::UpLeft => 8,
                ShotDirection::UpFacingLeft => 9,
            },
            Shot::Withheld(byte) => byte,
        }
    }
}

/// One pose's definition.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PoseDefinition {
    /// Which way it faces.
    pub facing: PoseFacing,
    /// How it moves Samus.
    pub movement: MovementType,
    /// The pose to take when no button changes it; `None` keeps this one ($FF).
    pub next: Option<PoseId>,
    /// Whether and where it fires.
    pub shot: Shot,
    /// The Y offset of Samus's graphics and projectile origin, not of her hitbox.
    pub y_offset: i8,
    /// Half her height in this pose, in pixels.
    pub y_radius: u8,
}

/// The table's first entry (bank $91 log).
pub const POSE_DEFINITIONS: RomAddress = RomAddress::new(0x91, 0xB629);

/// The poses the table defines: 0 to $FC, ending where the next table begins (`$91:BE11`).
pub const POSE_COUNT: usize = 0xFD;

/// Bytes a definition.
const DEFINITION_BYTES: usize = 8;

/// Why the pose table doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoseError {
    /// The table isn't in the image.
    Address(AddressError),
    /// A pose's movement type is past the movement handler's table.
    MovementType {
        /// The pose.
        pose: PoseId,
        /// The byte.
        value: u8,
    },
    /// A pose's shot byte has a clear high nybble but no direction.
    Shot {
        /// The pose.
        pose: PoseId,
        /// The byte.
        value: u8,
    },
}

/// Every pose's definition, indexed by [`PoseId`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Poses {
    definitions: Vec<PoseDefinition>,
}

impl Poses {
    /// The table in `rom`.
    ///
    /// # Errors
    ///
    /// [`PoseError`] when the table is outside the image or holds a byte no pose can.
    pub fn load(rom: Lorom<'_>) -> Result<Self, PoseError> {
        let bytes = rom
            .read(POSE_DEFINITIONS, POSE_COUNT.wrapping_mul(DEFINITION_BYTES))
            .map_err(PoseError::Address)?;
        Self::decode(bytes)
    }

    /// The table from its bytes, eight a pose; a trailing partial entry is ignored.
    ///
    /// # Errors
    ///
    /// [`PoseError`] when an entry holds a movement type or shot byte no pose can.
    pub fn decode(bytes: &[u8]) -> Result<Self, PoseError> {
        let (entries, _) = bytes.as_chunks::<DEFINITION_BYTES>();
        let definitions = entries
            .iter()
            .zip(0_u16..)
            .map(|(entry, number)| definition(PoseId(number), *entry))
            .collect::<Result<_, _>>()?;
        Ok(Self { definitions })
    }

    /// The definition of `pose`, `None` past the table.
    #[must_use]
    pub fn get(&self, pose: PoseId) -> Option<&PoseDefinition> {
        self.definitions.get(usize::from(pose.0))
    }

    /// How many poses the table defines.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Whether the table defines no pose.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

/// One entry; its two unused bytes are not kept.
fn definition(
    pose: PoseId,
    [facing, movement, next, shot, y_offset, _, y_radius, _]: [u8; DEFINITION_BYTES],
) -> Result<PoseDefinition, PoseError> {
    Ok(PoseDefinition {
        facing: PoseFacing::from(facing),
        movement: MovementType::try_from(movement)
            .map_err(|value| PoseError::MovementType { pose, value })?,
        next: (next != 0xFF).then_some(PoseId(u16::from(next))),
        shot: Shot::try_from(shot).map_err(|value| PoseError::Shot { pose, value })?,
        y_offset: i8::from_le_bytes([y_offset]),
        y_radius,
    })
}

#[cfg(test)]
mod tests;
