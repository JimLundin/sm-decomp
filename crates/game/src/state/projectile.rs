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

/// Complete clean power State, pinned for checkpoint shape compatibility.
pub struct PowerPin;
impl canonical::model::Pinned for PowerPin {
    type State = weapons_projectiles::State;
    const NAME: &'static str = "weapons-projectiles";
    const SHAPE: u64 = 0x98F6_AF73_265D_2556;
}

/// Whether the charge beam is equipped, independent of any fired projectile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeEquipment {
    /// Charge beam is absent.
    Unequipped,
    /// Charge beam is equipped.
    Equipped,
}
/// Actual hyper activity at the seed edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HyperActivity {
    /// The neighbouring producer is inactive.
    Inactive,
    /// The neighbouring producer is active.
    Active,
}
/// Actual grapple activity; equipment ownership alone cannot establish inactivity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrappleActivity {
    /// The neighbouring producer is inactive.
    Inactive,
    /// The grapple producer lies outside this ordinary path.
    Unsupported,
}
/// Running time is required through the ordinary trail destination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerTime {
    /// Ordinary updates and trail presentation run.
    Running,
    /// Time stops ordinary trail retirement.
    Stopped,
}
/// A decoded producer pool at the seed cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerPool {
    /// No competing producer occupies the decoded pool.
    Empty,
    /// At least one competing producer occupies the decoded pool.
    Occupied,
}

/// Independently decoded facts establishing the bounded ordinary producer path.
#[derive(Clone, Copy, Debug)]
pub struct PowerSeedFacts {
    /// Actually equipped ordinary beam combination.
    pub beams: Beams,
    /// Actually equipped charge beam.
    pub charge: ChargeEquipment,
    /// Current and previous charge counters at the seed.
    pub charge_history: (Frames, Frames),
    /// Actual hyper producer activity.
    pub hyper: HyperActivity,
    /// Actual grapple producer activity.
    pub grapple: GrappleActivity,
    /// Whether ordinary time is running.
    pub time: PowerTime,
    /// Sprite-object occupancy at the seed.
    pub sprite_objects: ProducerPool,
    /// Enemy-projectile occupancy at the seed.
    pub enemy_projectiles: ProducerPool,
    /// Occupancy of both sides of every trail pair.
    pub trail_effects: ProducerPool,
    /// Occupancy outside the five owned power slots.
    pub other_player_slots: ProducerPool,
}

/// A checked ordinary route; private construction rejects unavailable neighbouring facts.
///
/// Checkpoint reopening must revalidate the owned state/content and its current caller.
/// Competing pools are absent seed facts rather than inert runtime arrays: this owner
/// can create only Power and Impact bodies. Admitted doors create no player/effect
/// producers; equipped items, other live actors, occupied PLMs and reactive block
/// classes are rejected. Ordinary trails are transient requests delivered before
/// the uninterrupted pass completes. A restored capability alone is insufficient:
/// Session reopening checks the complete typed path and imported references again.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinaryCapability {
    checked: CheckedOrdinary,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct CheckedOrdinary;

impl OrdinaryCapability {
    /// Establish actual no-charge, power-only applicability and absent competing producers.
    /// # Errors
    /// Any unsupported or occupied producer fact.
    pub fn from_seed(
        facts: PowerSeedFacts,
    ) -> Result<Self, crate::weapons_projectiles_bridge::Fault> {
        let supported = facts.beams == Beams::default()
            && facts.charge == ChargeEquipment::Unequipped
            && facts.charge_history == (Frames(0), Frames(0))
            && facts.hyper == HyperActivity::Inactive
            && facts.grapple == GrappleActivity::Inactive
            && facts.time == PowerTime::Running
            && [
                facts.sprite_objects,
                facts.enemy_projectiles,
                facts.trail_effects,
                facts.other_player_slots,
            ]
            .into_iter()
            .all(|pool| pool == ProducerPool::Empty);
        supported
            .then_some(Self {
                checked: CheckedOrdinary,
            })
            .ok_or(crate::weapons_projectiles_bridge::Fault::Applicability)
    }
}

/// Ordinary power's sole projectile owner. No legacy first-five bodies coexist with it.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryPower {
    /// Complete persistent clean power state.
    pub model: canonical::model::Model<PowerPin>,
    capability: OrdinaryCapability,
}
impl OrdinaryPower {
    /// Restore complete independently decoded model State after the capability edge checks.
    /// # Errors
    /// Invalid references or unsupported ordinary seed producers.
    pub fn from_seed(
        state: weapons_projectiles::State,
        facts: PowerSeedFacts,
        content: &crate::weapons_projectiles_bridge::PowerContent,
    ) -> Result<Self, crate::weapons_projectiles_bridge::Fault> {
        let capability = OrdinaryCapability::from_seed(facts)?;
        content.validate_state(&state)?;
        Ok(Self {
            model: canonical::model::Model(state),
            capability,
        })
    }
}

/// Disjoint ownership, with no automatic conversion or switching between weapon families.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum ProjectileOwner {
    /// Unsupported families retain their sole existing owner.
    Legacy(Projectiles),
    /// Checked ordinary power runs clean W1 and C1.
    OrdinaryPower(OrdinaryPower),
}
impl Default for ProjectileOwner {
    fn default() -> Self {
        Self::Legacy(Projectiles::default())
    }
}

impl ProjectileOwner {
    /// The legacy family owner, when it alone owns player projectiles.
    #[must_use]
    pub const fn legacy(&self) -> Option<&Projectiles> {
        match self {
            Self::Legacy(value) => Some(value),
            Self::OrdinaryPower(_) => None,
        }
    }
    /// Mutably borrow the legacy family owner without switching ownership.
    pub const fn legacy_mut(&mut self) -> Option<&mut Projectiles> {
        match self {
            Self::Legacy(value) => Some(value),
            Self::OrdinaryPower(_) => None,
        }
    }
    /// The ordinary power owner, when the checked path was selected.
    #[must_use]
    pub const fn ordinary(&self) -> Option<&OrdinaryPower> {
        match self {
            Self::Legacy(_) => None,
            Self::OrdinaryPower(value) => Some(value),
        }
    }
}
