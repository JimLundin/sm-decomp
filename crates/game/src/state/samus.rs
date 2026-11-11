//! Samus as orthogonal facts: where she is, how she moves, what she has, how she's drawn.

use canonical::Canonical;

use super::slots::{FlagSet, Frames};
use crate::model::num::Fixed as Exact;

/// Where Samus's centre is.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position {
    /// Rightward.
    pub x: Exact,
    /// Downward.
    pub y: Exact,
}

/// What the base speed is doing; pose selection sets it when a pose starts.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpeedMode {
    /// Speeding up to the row's top speed.
    #[default]
    Accelerating,
    /// Braking through a turn, still sliding the old way.
    TurningAround,
    /// Braking (on the ground; in the air it still speeds up).
    Decelerating,
}

/// Which way she moves vertically; anything but `None` is airborne.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VerticalMotion {
    /// On the ground.
    #[default]
    None,
    /// Rising.
    Up,
    /// Falling.
    Down,
}

/// How many times the ball has bounced since it landed.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bounce {
    /// Not bouncing.
    #[default]
    None,
    /// In the first bounce.
    First,
    /// In the second.
    Second,
}

/// Whether she is dashing: on from a run with run held until a halt, a turn or the pad's
/// let-go ends it, whatever her dash (crouched from a run she goes on dashing with none).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dashing {
    /// Not dashing.
    #[default]
    Off,
    /// Dashing.
    On,
}

/// Her motion as the game keeps it: the quantities a later frame reads.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Motion {
    /// Her centre.
    pub position: Position,
    /// The row's own speed, never negative.
    pub base_speed: Exact,
    /// The speed running adds, never negative.
    pub extra_run: Exact,
    /// Whether she is dashing.
    pub dashing: Dashing,
    /// What the base speed is doing.
    pub mode: SpeedMode,
    /// The vertical speed's magnitude.
    pub vertical_speed: Exact,
    /// Its direction.
    pub vertical: VerticalMotion,
    /// The ball's bounces.
    pub bounce: Bounce,
}

/// A pixel position or speed with its subpixel fraction, as the original keeps them.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fixed {
    /// Whole pixels.
    pub pixels: u16,
    /// 1/65536ths of a pixel.
    pub subpixels: u16,
}

/// A room position.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// Rightward from the room's left edge.
    pub x: Fixed,
    /// Downward from the room's top edge.
    pub y: Fixed,
}

impl From<Exact> for Fixed {
    fn from(value: Exact) -> Self {
        Self {
            pixels: value.pixel(),
            subpixels: value.sub(),
        }
    }
}

impl From<Fixed> for Exact {
    fn from(value: Fixed) -> Self {
        Self::new(value.pixels, value.subpixels)
    }
}

impl From<Position> for Point {
    fn from(position: Position) -> Self {
        Self {
            x: position.x.into(),
            y: position.y.into(),
        }
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            x: point.x.into(),
            y: point.y.into(),
        }
    }
}

pub use data::movement::Speed;
pub use data::pose::{MovementType, PoseFacing as Facing, PoseId};

/// Where Samus aims her arm cannon.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aim {
    /// Straight ahead.
    #[default]
    Forward,
    /// Up and ahead.
    UpForward,
    /// Straight up.
    Up,
    /// Down and ahead.
    DownForward,
    /// Straight down.
    Down,
}

/// A pose with the facing and movement type its definition gave, as the original copies
/// them beside the pose word (bank $90 log).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseRecord {
    /// The pose.
    pub pose: PoseId,
    /// Its facing.
    pub facing: Facing,
    /// Its movement type.
    pub movement: MovementType,
}

/// An item Samus can collect and equip.
// open: the beams and the charge beam (the original's beam words), with their combination
// rules, join with weapons.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    /// The Varia suit.
    VariaSuit,
    /// The spring ball.
    SpringBall,
    /// The morph ball.
    MorphBall,
    /// The screw attack.
    ScrewAttack,
    /// The gravity suit.
    GravitySuit,
    /// The hi-jump boots.
    HiJumpBoots,
    /// The space jump.
    SpaceJump,
    /// Bombs.
    Bombs,
    /// The speed booster.
    SpeedBooster,
    /// The grapple beam.
    GrappleBeam,
    /// The X-ray scope.
    XRayScope,
}

/// A set of items.
pub type Items = FlagSet<Item>;

impl Item {
    /// The item's bit in an [`Items`] set: its place in declaration order.
    #[must_use]
    pub const fn bit(self) -> u32 {
        let place = match self {
            Self::VariaSuit => 0,
            Self::SpringBall => 1,
            Self::MorphBall => 2,
            Self::ScrewAttack => 3,
            Self::GravitySuit => 4,
            Self::HiJumpBoots => 5,
            Self::SpaceJump => 6,
            Self::Bombs => 7,
            Self::SpeedBooster => 8,
            Self::GrappleBeam => 9,
            Self::XRayScope => 10,
        };
        1 << place
    }
}

impl Items {
    /// Add `item` to the set.
    pub const fn insert(&mut self, item: Item) {
        self.bits |= item.bit();
    }

    /// Whether the set holds `item`.
    #[must_use]
    pub const fn contains(&self, item: Item) -> bool {
        (self.bits & item.bit()) != 0
    }
}

/// A quantity with a ceiling: energy, a reserve, an ammunition type.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gauge {
    /// What's left.
    pub current: u16,
    /// The ceiling.
    pub max: u16,
}

/// What Samus has.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Equipment {
    /// Collected.
    pub collected: Items,
    /// Collected and switched on.
    pub equipped: Items,
    /// Energy.
    pub energy: Gauge,
    /// The reserve tanks' energy.
    pub reserve: Gauge,
    /// Missiles.
    pub missiles: Gauge,
    /// Super missiles.
    pub super_missiles: Gauge,
    /// Power bombs.
    pub power_bombs: Gauge,
    /// How the reserve tanks refill energy.
    pub reserve_mode: ReserveMode,
}

/// How the reserve tanks refill energy (bank `$80` log, `$80:9B4E`: the HUD draws AUTO only
/// in the automatic mode).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReserveMode {
    /// No reserve tank collected yet.
    #[default]
    None,
    /// They refill energy when it runs out.
    Automatic,
    /// The player refills from the equipment screen.
    Manual,
}

pub use data::pose::animation::AnimationFrame;

/// Her step in her pose's clip, numbered from 0 without the original's operand bytes
/// (samus-looks §1.4); `samus_looks_world` gives the original's frame of it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClipStep(pub u8);

/// Her run's speed booster stage (samus-looks §1).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoostStage {
    /// Stage 0, where every dash begins.
    #[default]
    Stage0,
    /// Stage 1.
    Stage1,
    /// Stage 2.
    Stage2,
    /// Stage 3.
    Stage3,
    /// Stage 4: boosting.
    Boosting,
}

/// Her run's boost: its stage and the laps the stage still lasts (samus-looks §1).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boost {
    /// The stage.
    pub stage: BoostStage,
    /// The laps of her run the stage still lasts.
    pub laps_left: u8,
}

/// Samus's animation (samus-looks §1): where her clip is and her run's boost, all it keeps
/// from one frame to the next.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// Her step in her pose's clip.
    pub step: ClipStep,
    /// The frames her step still shows, 0 to 255.
    pub frames_left: u8,
    /// Her run's boost.
    pub boost: Boost,
}

/// What the pose update does after it takes the super special prospective pose (bank $91
/// log, the super special prospective pose change commands).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SuperSpecialCommand {
    /// Nothing (0).
    #[default]
    None,
    /// 1 (`$91:F31D`): knockback finished.
    KnockbackFinished,
    /// 2 (`$91:F34E`): shinespark finished.
    ShinesparkFinished,
    /// 3 (`$91:F36E`): the transition animation finished.
    TransitionAnimationFinished,
    /// 4 (`$91:F37C`): Samus to the screen's centre.
    CentreOnScreen,
    /// 5 (`$91:F397`): Samus 5 pixels down.
    MoveDown,
    /// 6 (`$91:F3A5`): start the grapple wall jump.
    StartGrappleWallJump,
    /// 7 (`$91:F3AA`): start the release from the grapple swing.
    StartReleaseFromGrappleSwing,
    /// 8 (`$91:F3FD`): knockback and the transition animation both finished.
    KnockbackAndTransitionAnimationFinished,
}

/// The super special prospective pose request the frame made.
///
/// The pose update takes it before every other (bank $91 log, `$91:EB88`). The original clears it at the start
/// of every frame (`$90:E695`) and it keeps the last frame's until then.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SuperSpecial {
    /// The pose asked for.
    pub pose: Option<PoseId>,
    /// What the update does after taking it.
    pub command: SuperSpecialCommand,
}

/// Which of the visor's cycle colours shows next (bank $91 log, handle visor palette
/// `$91:D83F`): the cycle steps bright, dim, dark, then back to bright.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisorShade {
    /// The cycle's first colour, where a new game starts it (`$91:E0BF`).
    #[default]
    Bright,
    /// The second.
    Dim,
    /// The third.
    Dark,
}

impl VisorShade {
    /// The shade after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Bright => Self::Dim,
            Self::Dim => Self::Dark,
            Self::Dark => Self::Bright,
        }
    }

    /// The shade's place in the cycle's colours.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Bright => 0,
            Self::Dim => 1,
            Self::Dark => 2,
        }
    }
}

/// Frames until the visor's next colour, a byte that counts down to 0.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisorTimer(pub u8);

impl Default for VisorTimer {
    /// A new game shows the first colour on its first frame (`$91:E0BF`).
    fn default() -> Self {
        Self(1)
    }
}

/// The visor's colour cycle in rooms whose colour math adds the backdrop.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Visor {
    /// The colour it shows next.
    pub shade: VisorShade,
    /// Frames until it does.
    pub timer: VisorTimer,
}

/// What draws Samus each frame: her drawing handler, which draw Samus sprites jumps through
/// after the arm cannon's open state (bank $90 log, `$90:EB4B`).
///
/// The handler only RTS unused code sets (`$90:E35A`) has no variant.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawingHandler {
    /// Her body, arm cannon and echoes (`$90:EB52`); the Samus commands that end a special
    /// state set it back (`$90:F295`, `$90:F4FF`).
    #[default]
    Default,
    /// While the grapple beam fires: its flare and beam with her (`$90:EB86`).
    FiringGrapple,
    /// The shinespark crash's echoes circling her (`$90:EBF3`, set by `$90:D315`).
    ShinesparkCrashEchoCircle,
    /// Riding an elevator: she is drawn inanimate on even frames only (`$90:EC14`, set by
    /// Samus command 7, `$90:F1C8`).
    UsingElevator,
    /// Inanimate Samus every frame (`$90:EC1D`, set by `$90:F308`).
    Inanimate,
    /// Not drawn: Samus command Ah stops drawing her (`$90:F28D`), as does the placement
    /// mode toggle (`$90:E795`).
    Hidden,
}

/// Jump held through (samus-motion `Jumping`): how long jump has stayed held since its press,
/// and what the frame before left her pose input.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JumpHeld {
    /// The frames jump has stayed held since its press (`$0AF4`), counted at the frame's draw
    /// (`$90:EAB3`) and wrapping from 65,535 to 0.
    pub frames: u16,
    /// What the frame before left her pose input (`$0A60`).
    pub before: FrameBefore,
}

/// What the frame before left her pose input, as jump held through reads it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameBefore {
    /// She landed, or a turn's or a landing's animation ended: the pose input that may count
    /// jump held through as a press (`$90:E926`, installed by `$91:F1EC` and `$90:8370`), for
    /// one frame.
    LandedOrEnded,
    /// Any other pose input: the normal one (`$90:E913`), or one the engine doesn't run (the
    /// demo's). None of them reads the count.
    #[default]
    Other,
}

/// Samus.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Samus {
    /// Where she was when the camera last followed her: the main scrolling routine keeps it
    /// at its end (bank $90 log, `$90:956E`), and the next frame's camera speed is the
    /// distance from it.
    pub previous_position: Point,
    /// Her pose: the pose definitions table (`data::pose`) gives its facing, movement type,
    /// shot direction and radius, and behaviour that keys on one pose (landing, the
    /// elevator's facing forward) reads it.
    pub pose: PoseId,
    /// The pose before the last pose change, and what it gave.
    pub previous: PoseRecord,
    /// The last pose that differed from the current one, and what it gave.
    pub last_different: PoseRecord,
    /// Which way she faces.
    pub facing: Facing,
    /// Where she aims.
    // open: the aim is the pose's shot direction (`data::pose::Shot`); whether the State
    // keeps it beside the pose or reads it from the table joins with weapons.
    pub aim: Aim,
    /// Her pose's movement type, as the pose change copies it from the definition.
    pub movement: MovementType,
    /// Her motion: where she is and how she moves.
    pub motion: Motion,
    /// What she has.
    pub equipment: Equipment,
    /// Frames of invincibility left after a hit.
    pub invincibility: Frames,
    /// Her animation.
    pub animation: Animation,
    /// The super special prospective pose request.
    pub super_special: SuperSpecial,
    /// Her visor's colour cycle.
    pub visor: Visor,
    /// What draws her each frame.
    pub drawing: DrawingHandler,
    /// The Y radius her block collision reads (`$0B00`): her pose's, as the normal current
    /// state handler last set it (`$90:EC22`). The locked handler that replaces it for an
    /// elevator ride doesn't, so the ride keeps the radius she called it with (`$91:E44D`).
    pub y_radius: u16,
    /// Whether a slope adjusted her position (`$0DBA`): what her last vertical move left, so
    /// the next frame's ground movement steps her down a pixel alone (`$90:924C`).
    // open: the slope alignment after a horizontal move (`$94:87F4`) sets it too; it joins
    // with the horizontal slope detection flag (`$0A46`).
    pub slope_adjusted: bool,
    /// Jump held through.
    pub jump_held: JumpHeld,
}
