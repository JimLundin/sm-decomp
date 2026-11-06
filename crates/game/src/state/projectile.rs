//! Samus's projectiles: her beams, missiles and bombs in the original's ten projectile slots
//! (bank $90 log, `$90:AC1C` onward).

use canonical::Canonical;

use super::samus::Point;
use super::slots::{Flag, FlagSet, Frames, SlotStore};

/// Projectile slots: five for beams and missiles, then five for bombs (`$90:ADED`).
pub const PROJECTILE_SLOTS: usize = 10;
/// The slots beams and missiles take, from the first; bombs take the rest.
pub const SHOT_SLOTS: usize = 5;

/// The projectile family marker, which a [`super::slots::SlotIndex`] into
/// [`Projectiles::slots`] names.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Projectile;

/// The fields a projectile slot keeps across a clear, whoever occupies it: the clear
/// (`$90:ADB7`) zeroes everything else.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectileCommon {
    /// Frames until the beam leaves its next trail (`$0C90`); the beam's pre-instruction
    /// counts it down and sets it back to 4 as it spawns one (`$90:AF00`).
    pub trail_timer: Frames,
}

/// A speed in 1/256ths of a pixel a frame, signed, as the original keeps a projectile's.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubpixelSpeed(pub i16);

/// A projectile's speeds.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Velocity {
    /// Rightward.
    pub x: SubpixelSpeed,
    /// Downward.
    pub y: SubpixelSpeed,
}

/// A projectile's half extents, in pixels: its hitbox reaches this far from its position.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Radius {
    /// Horizontally.
    pub x: u16,
    /// Vertically.
    pub y: u16,
}

/// The damage a projectile deals an enemy.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Damage(pub u16);

/// Which way a projectile flies: the original's direction 0 to 9, from up clockwise round to
/// up again, with the straight up and down shots keeping which way Samus faced.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// 0: up, fired facing right.
    #[default]
    UpFacingRight,
    /// 1.
    UpRight,
    /// 2.
    Right,
    /// 3.
    DownRight,
    /// 4: down, fired facing right.
    DownFacingRight,
    /// 5: down, fired facing left.
    DownFacingLeft,
    /// 6.
    DownLeft,
    /// 7.
    Left,
    /// 8.
    UpLeft,
    /// 9: up, fired facing left.
    UpFacingLeft,
}

/// The way a projectile flies, and whether something marked it for deletion.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heading {
    /// The direction.
    pub direction: Direction,
    /// An enemy hit it (`$0C04`'s bit 4): its next pre-instruction clears it (`$90:AEF3`).
    pub deleted: bool,
}

/// A beam Samus has equipped, as a shot carries it.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beam {
    /// Wave.
    Wave,
    /// Ice.
    Ice,
    /// Spazer.
    Spazer,
    /// Plasma.
    Plasma,
}

impl Flag for Beam {
    fn bit(self) -> u32 {
        let place = match self {
            Self::Wave => 0,
            Self::Ice => 1,
            Self::Spazer => 2,
            Self::Plasma => 3,
        };
        1 << place
    }
}

/// The beams a shot carries; none is the power beam.
pub type Beams = FlagSet<Beam>;

/// A beam shot's make-up.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeamShot {
    /// The beams it combines.
    pub beams: Beams,
    /// Whether it was charged.
    pub charged: bool,
}

/// The beam explosion's frames read so far (`$93:A007`): its instruction list shows six,
/// then deletes it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExplosionStep(pub u8);

/// What a projectile is (the type word's bits 8 to 11, with the beam's bits below them).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shot {
    /// A beam.
    Beam(BeamShot),
    /// A missile.
    Missile,
    /// A super missile.
    SuperMissile,
    /// A power bomb.
    PowerBomb,
    /// A bomb.
    Bomb,
    /// A beam's explosion where it hit.
    BeamExplosion {
        /// The beam that exploded.
        beam: BeamShot,
        /// How far its frames have run.
        step: ExplosionStep,
    },
    /// A missile's explosion where it hit.
    MissileExplosion,
}

impl Default for Shot {
    fn default() -> Self {
        Self::Beam(BeamShot::default())
    }
}

/// A projectile's type.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectileType {
    /// What it is.
    pub shot: Shot,
    /// Whether it can touch Samus: the type word's bit 15 is clear.
    pub touches_samus: bool,
}

/// One live projectile (`$0B64` onward, a word per slot in each table).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectileBody {
    /// Where it is.
    pub position: Point,
    /// Its hitbox.
    pub radius: Radius,
    /// How fast it flies.
    pub velocity: Velocity,
    /// Which way.
    pub heading: Heading,
    /// What it is.
    pub kind: ProjectileType,
    /// What it deals.
    pub damage: Damage,
    /// Frames until its instruction list shows its next frame (`$0C54`).
    pub frame_timer: Frames,
}

/// How many beams and missiles are in flight.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShotCount(pub u16);

/// Samus's projectiles.
// open: the instruction lists (`$0C40`, `$0C54`), the spritemaps (`$0CB8`), the variables
// and bomb timers (`$0C7C`) and the bomb counter (`$0CD2`) join with drawing and the bombs.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Projectiles {
    /// The slots: occupied while the slot's instruction list runs (`$90:AED8`).
    pub slots: SlotStore<ProjectileCommon, ProjectileBody, PROJECTILE_SLOTS>,
    /// Beams and missiles in flight, which fire checks against five (`$90:AC39`).
    pub count: ShotCount,
    /// Frames until Samus can fire again (`$0CCC`).
    pub cooldown: Frames,
}
