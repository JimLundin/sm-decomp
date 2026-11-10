//! Runs every vector and pose step of `scenarios/vectors/samus-motion.toml` through the model.
//!
//! The file is read with `toml` and `serde` into its shape, declared once below as the file's
//! header gives it: every table refuses a key the shape doesn't know, and every name a value it
//! doesn't know, so an unknown key or name fails the test rather than being skipped. The model's
//! types are built from the shape; they know nothing of the reader.

#![cfg(test)]

use clean_model_samus_motion::{
    Bounces, CATALOGUE, Carry, DashIs, Frame, FrameBefore, JumpHeld, Met, Moment, Outcome, Phase,
    Rules, Run, Samus, Seat, SpeedCap, Spin, Vertical, WallTest,
};
use clean_types::{
    Action, Aim, Block, Body, Buttons, Cell, Checked, Facing, Fixed, Gun, Heading, Hitbox, Item,
    LandedFrom, Legs, Pair, Parity, Parts, Pose, Position, Px, Radius, Set, Side, Slope,
    SlopeFaces, Touch, Turn, Way, World, ZERO, button, fixed,
};
use core::num::Wrapping;
use serde::Deserialize;

const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-motion.toml");

// ---- The file's shape.

/// The vector file: its `[[vector]]` and `[[pose_step]]` tables.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    /// The vectors.
    vector: Vec<Vector>,
    /// The pose steps.
    pose_step: Vec<PoseStep>,
}

/// A `[[vector]]`: a short run of frames, the lead-in's and then the one it checks.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    /// Its id.
    id: String,
    /// The record holding the checked frame's result, or "constructed".
    source: String,
    /// The spec's behaviours it exercises.
    entries: Vec<String>,
    /// The rules, where they aren't the reference's.
    rules: Option<RulesTable>,
    /// The items switched on.
    #[serde(default)]
    items: Vec<ItemName>,
    /// The stand-in room.
    world: WorldTable,
    /// Samus at the start.
    before: Before,
    /// The frames before the checked one, oldest first.
    #[serde(default)]
    lead_in: Vec<FrameTable>,
    /// The pose at the checked frame's start.
    pose: PoseTable,
    /// The pose chosen during the checked frame, where it changes.
    next_pose: Option<PoseTable>,
    /// The checked frame's parity.
    #[serde(default)]
    frame: ParityName,
    /// The buttons held.
    pad: Vec<ButtonName>,
    /// Jump just pressed.
    jump_pressed: bool,
    /// The other buttons just pressed.
    #[serde(default)]
    pressed: Vec<ButtonName>,
    /// The animation of her pose ends.
    #[serde(default)]
    animation_ends: bool,
    /// The input "spin turned far".
    #[serde(default)]
    spin_turned_far: bool,
    /// What is compared at the checked frame's end.
    after: After,
}

impl Vector {
    /// The checked frame, as a lead-in frame gives one.
    fn checked(&self) -> FrameTable {
        FrameTable {
            pose: self.pose.clone(),
            pad: self.pad.clone(),
            jump_pressed: self.jump_pressed,
            pressed: self.pressed.clone(),
            animation_ends: self.animation_ends,
            spin_turned_far: self.spin_turned_far,
        }
    }
}

/// A lead-in frame: the pose it starts in, and what the frame brings.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameTable {
    /// The pose at its start.
    pose: PoseTable,
    /// The buttons held.
    pad: Vec<ButtonName>,
    /// Jump just pressed.
    jump_pressed: bool,
    /// The other buttons just pressed.
    #[serde(default)]
    pressed: Vec<ButtonName>,
    /// The animation of her pose ends.
    #[serde(default)]
    animation_ends: bool,
    /// The input "spin turned far".
    #[serde(default)]
    spin_turned_far: bool,
}

/// A `[[pose_step]]`: one frame of the pose chart.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PoseStep {
    /// Its id.
    id: String,
    /// The record it comes from, or "constructed".
    source: String,
    /// The spec's behaviours it exercises.
    entries: Vec<String>,
    /// The items switched on.
    #[serde(default)]
    items: Vec<ItemName>,
    /// The pose at the frame's start.
    pose: PoseTable,
    /// The buttons held.
    pad: Vec<ButtonName>,
    /// Jump just pressed.
    jump_pressed: bool,
    /// The other buttons just pressed.
    #[serde(default)]
    pressed: Vec<ButtonName>,
    /// Her horizontal speed at the frame's start, 0 where missing.
    speed_x: Option<Signed>,
    /// In the air once the frame's moves are made; where missing, as her pose and what her moves
    /// met say (`PoseStep::in_the_air`).
    in_the_air: Option<bool>,
    /// What the frame's moves met.
    met: MetName,
    /// The pose the frame ends in.
    after: StepAfter,
}

impl PoseStep {
    /// Whether she is in the air once the frame's moves are made: as the step says, else in the
    /// air in `Jumping` (the spring ball's hop too), `Spinning`, `Falling` and `WallJumping` and
    /// after walking off a ledge, and on the ground after a landing and in every other pose (a
    /// take-off and a turn are on the ground).
    fn in_the_air(&self, met: Option<Met>) -> bool {
        let in_air_pose = matches!(
            self.pose.action,
            ActionName::Jumping
                | ActionName::Spinning
                | ActionName::Falling
                | ActionName::WallJumping
        );
        self.in_the_air
            .unwrap_or_else(|| met == Some(Met::NoFloor) || met != Some(Met::Floor) && in_air_pose)
    }
}

/// A pose step's `after`: the pose alone.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepAfter {
    /// The pose the frame ends in.
    pose: PoseTable,
}

/// A vector's `rules`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesTable {
    /// `WordwiseTopSpeed`'s switch.
    speed_cap: SpeedCapName,
}

/// A pose (spec §2), with her body's size there where a vector gives it.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct PoseTable {
    /// The body.
    body: BodyName,
    /// The action.
    action: ActionName,
    /// The facing.
    facing: FacingName,
    /// Her body's half-width and half-height, `[x, y]` in px.
    radius: Option<[u16; 2]>,
    /// The aim.
    #[serde(default)]
    aim: AimName,
    /// The gun.
    #[serde(default)]
    gun: GunName,
    /// The turn, for `Turning`.
    #[serde(default)]
    turn: TurnName,
    /// What she landed from, for `Landing`.
    landed_from: Option<LandedFromName>,
    /// The legs.
    #[serde(default)]
    legs: LegsName,
}

/// The stand-in room (the header's `world`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldTable {
    /// The y of a solid floor's top face; every row from it down is solid.
    floor_top: Option<u32>,
    /// The x of a solid wall's left face; every column from it rightward is solid.
    wall_right: Option<u32>,
    /// The y of a solid ceiling's bottom face; every row above it is solid.
    ceiling_bottom: Option<u32>,
    /// One sloped floor block, or a list of them.
    slope_floor: Option<OneOrMore<SlopeTable>>,
    /// One sloped ceiling block.
    slope_ceiling: Option<SlopeTable>,
    /// One solid block's top-left corner.
    solid_block: Option<[u16; 2]>,
    /// One stepped block.
    stepped: Option<SteppedTable>,
    /// One solid enemy.
    enemy: Option<EnemyTable>,
    /// She stands on an elevator that is moving.
    #[serde(default)]
    elevator_moving: bool,
    /// The other kinds of cell.
    #[serde(default)]
    cells: Vec<CellTable>,
}

/// One table of a kind, or a list of them.
#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMore<T> {
    /// One.
    One(T),
    /// A list.
    More(Vec<T>),
}

/// A sloped block.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlopeTable {
    /// Its top-left corner.
    block: [u16; 2],
    /// The surface's row at each of its 16 columns; 16 an empty column, 17 one far past.
    surface: [u16; 16],
    /// The walk factor, 1.00 by default.
    factor: Option<Factor>,
}

/// A stepped block.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SteppedTable {
    /// Its top-left corner.
    block: [u16; 2],
    /// Its upper row of quarters then its lower.
    quarters: [QuarterRow; 2],
}

/// A stepped block's row of two 8 px quarters, from the left, as the header writes it: each
/// character "#" (solid) or "." (air), and no other.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(try_from = "String")]
struct QuarterRow([bool; 2]);

impl TryFrom<String> for QuarterRow {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let quarter = |c: char| match c {
            '#' => Ok(true),
            '.' => Ok(false),
            other => Err(format!("{text:?}: {other:?} is neither \"#\" nor \".\"")),
        };
        let solid = text
            .chars()
            .map(quarter)
            .collect::<Result<Vec<bool>, String>>()?;
        match solid[..] {
            [left, right] => Ok(Self([left, right])),
            _ => Err(format!("{text:?} is no row of two quarters")),
        }
    }
}

/// A solid enemy.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyTable {
    /// Its centre, in px.
    centre: [u16; 2],
    /// Its half-width and half-height, in px.
    radius: [u16; 2],
}

/// One of the other kinds of cell.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CellTable {
    /// Its top-left corner.
    block: [u16; 2],
    /// Its kind.
    kind: CellKind,
    /// The world's answer for every touch this frame, for a cell that answers.
    #[serde(default)]
    answer: AnswerName,
    /// The top-left corner of the cell a stand-in names.
    names: Option<[u16; 2]>,
}

/// What the player can tell about Samus at the start (the header's `before`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Before {
    /// Her position, `[x, y]`.
    position: [Quantity; 2],
    /// Her horizontal speed, 0 where missing.
    speed_x: Option<Signed>,
    /// Her dash, 0 where missing.
    dash: Option<Quantity>,
    /// Her vertical speed, 0 where missing.
    speed_y: Option<Signed>,
    /// The ball's bounces.
    #[serde(default)]
    bounces: u8,
    /// On the ground, falling at a vertical speed of 0.
    #[serde(default)]
    falling: bool,
    /// Jump held through's count.
    #[serde(default)]
    jump_held: u16,
    /// Jump held through's frame before.
    #[serde(default)]
    landed_or_ended: bool,
}

/// What a vector compares at the checked frame's end, each only where given.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    /// Her position, `[x, y]`.
    position: Option<[Quantity; 2]>,
    /// Her x alone.
    position_x: Option<Quantity>,
    /// Her horizontal speed.
    speed_x: Option<Signed>,
    /// Her vertical speed.
    speed_y: Option<Signed>,
    /// Her dash.
    dash: Option<Quantity>,
    /// The ball's bounces.
    bounces: Option<u8>,
    /// The pose she ends in, where pose fit changed the one chosen.
    pose: Option<PoseTable>,
    /// Jump held through's count.
    jump_held: Option<u16>,
    /// Jump held through's frame before.
    landed_or_ended: Option<bool>,
    /// Where a placement put her.
    placed: Option<Placed>,
    /// Whether she is rising.
    rising: Option<bool>,
    /// Whether she skids.
    skidding: Option<bool>,
    /// The cells her moves and looks reach.
    reached: Option<Vec<TouchTable>>,
    /// The cells she is inside.
    inside: Option<Vec<[u16; 2]>>,
    /// The near-wall test's outcome.
    wall_test: Option<WallTestName>,
}

/// A reached cell: its top-left corner, and the way.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TouchTable {
    /// Its top-left corner.
    cell: [u16; 2],
    /// The way it was reached.
    way: WayName,
}

// ---- Quantities, as the vectors write them.

/// A `P.S` quantity, "PPPP.SSSS" in hex.
#[derive(Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
struct Quantity(Fixed);

impl TryFrom<String> for Quantity {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let (pixel, sub) = text
            .split_once('.')
            .ok_or_else(|| format!("{text:?} is no P.S quantity"))?;
        let part = |p: &str| u16::from_str_radix(p, 16).map_err(|e| format!("{text:?}: {e}"));
        Ok(Self(fixed(Wrapping(part(pixel)?), part(sub)?)))
    }
}

/// A signed `P.S` quantity: "+" or "-", then its size; rightward or downward positive.
#[derive(Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
struct Signed {
    /// Leftward or upward.
    negative: bool,
    /// The size.
    size: Fixed,
}

/// No speed.
const STILL: Signed = Signed {
    negative: false,
    size: ZERO,
};

impl TryFrom<String> for Signed {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let (negative, size) = match text.split_at_checked(1) {
            Some(("+", size)) => (false, size),
            Some(("-", size)) => (true, size),
            _ => return Err(format!("{text:?} has no sign")),
        };
        let size = Quantity::try_from(size.to_owned())?.0;
        Ok(Self { negative, size })
    }
}

impl Signed {
    /// The quantity, in two's complement.
    fn value(self) -> Fixed {
        if self.negative { -self.size } else { self.size }
    }

    /// As the vectors write it, 0 with a "+".
    fn show(self) -> String {
        let sign = if self.negative && self.size != ZERO {
            '-'
        } else {
            '+'
        };
        format!("{sign}{}", show(self.size))
    }
}

/// The y a placement put her at, or "none".
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Placed(Option<Fixed>);

impl TryFrom<String> for Placed {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        if text == "none" {
            Ok(Self(None))
        } else {
            Quantity::try_from(text).map(|y| Self(Some(y.0)))
        }
    }
}

/// A sloped floor's walk factor, "P.SS" in hex: a pixel and 1/256ths.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Factor(Fixed);

impl TryFrom<String> for Factor {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        match text.split_once('.') {
            Some((pixel, sub)) if sub.len() == 2 => {
                u32::from_str_radix(&format!("{pixel}{sub}"), 16)
                    .map(|f| Self(Wrapping(f)))
                    .map_err(|e| format!("{text:?}: {e}"))
            }
            _ => Err(format!("{text:?} is no P.SS factor")),
        }
    }
}

// ---- Names, as the vectors give them, and the model's values they stand for.

/// §2 body.
#[derive(Deserialize, Clone, Copy)]
enum BodyName {
    Upright,
    Crouched,
    Ball,
    SpringBall,
}

impl BodyName {
    const fn model(self) -> Body {
        match self {
            Self::Upright => Body::Upright,
            Self::Crouched => Body::Crouched,
            Self::Ball => Body::Ball,
            Self::SpringBall => Body::SpringBall,
        }
    }
}

/// §2 action; a turn and what she landed from come apart.
#[derive(Deserialize, Clone, Copy)]
enum ActionName {
    AgainstWall,
    Crouching,
    Falling,
    Jumping,
    Landing,
    Moonwalking,
    Morphing,
    Rolling,
    Running,
    Spinning,
    StandingUp,
    Still,
    TakingOff,
    Turning,
    Unmorphing,
    WallJumping,
}

impl ActionName {
    /// The action, `Turning` with `turn` and `Landing` from `landed_from`.
    fn model(self, turn: Turn, landed_from: Option<LandedFrom>) -> Action {
        match self {
            Self::AgainstWall => Action::AgainstWall,
            Self::Crouching => Action::Crouching,
            Self::Falling => Action::Falling,
            Self::Jumping => Action::Jumping,
            Self::Landing => Action::Landing(landed_from.expect("a landing's landed_from")),
            Self::Moonwalking => Action::Moonwalking,
            Self::Morphing => Action::Morphing,
            Self::Rolling => Action::Rolling,
            Self::Running => Action::Running,
            Self::Spinning => Action::Spinning,
            Self::StandingUp => Action::StandingUp,
            Self::Still => Action::Still,
            Self::TakingOff => Action::TakingOff,
            Self::Turning => Action::Turning(turn),
            Self::Unmorphing => Action::Unmorphing,
            Self::WallJumping => Action::WallJumping,
        }
    }
}

/// §2 facing.
#[derive(Deserialize, Clone, Copy)]
enum FacingName {
    Left,
    Right,
    Screen,
}

impl FacingName {
    const fn model(self) -> Facing {
        match self {
            Self::Left => Facing::Left,
            Self::Right => Facing::Right,
            Self::Screen => Facing::Screen,
        }
    }
}

/// §2 aim, `Forward` by default.
#[derive(Deserialize, Clone, Copy, Default)]
enum AimName {
    #[default]
    Forward,
    Up,
    UpForward,
    DownForward,
    Down,
}

impl AimName {
    const fn model(self) -> Aim {
        match self {
            Self::Forward => Aim::Forward,
            Self::Up => Aim::Up,
            Self::UpForward => Aim::UpForward,
            Self::DownForward => Aim::DownForward,
            Self::Down => Aim::Down,
        }
    }
}

/// §2 gun, `Stowed` by default.
#[derive(Deserialize, Clone, Copy, Default)]
enum GunName {
    Drawn,
    #[default]
    Stowed,
}

impl GunName {
    const fn model(self) -> Gun {
        match self {
            Self::Drawn => Gun::Drawn,
            Self::Stowed => Gun::Stowed,
        }
    }
}

/// §2 turn, `Ground` by default.
#[derive(Deserialize, Clone, Copy, Default)]
enum TurnName {
    Fall,
    #[default]
    Ground,
    Jump,
    MoonwalkJump,
}

impl TurnName {
    const fn model(self) -> Turn {
        match self {
            Self::Fall => Turn::Fall,
            Self::Ground => Turn::Ground,
            Self::Jump => Turn::Jump,
            Self::MoonwalkJump => Turn::MoonwalkJump,
        }
    }
}

/// §2 landed from.
#[derive(Deserialize, Clone, Copy)]
enum LandedFromName {
    Jump,
    Spin,
}

impl LandedFromName {
    const fn model(self) -> LandedFrom {
        match self {
            Self::Jump => LandedFrom::Jump,
            Self::Spin => LandedFrom::Spin,
        }
    }
}

/// §2 legs, `Together` by default.
#[derive(Deserialize, Clone, Copy, Default)]
enum LegsName {
    Striding,
    #[default]
    Together,
}

impl LegsName {
    const fn model(self) -> Legs {
        match self {
            Self::Striding => Legs::Striding,
            Self::Together => Legs::Together,
        }
    }
}

/// A button of the pad.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ButtonName {
    Run,
    Jump,
    Shoot,
    Left,
    Right,
    Up,
    Down,
    AimUp,
    AimDown,
}

impl ButtonName {
    const fn model(self) -> Buttons {
        match self {
            Self::Run => button::RUN,
            Self::Jump => button::JUMP,
            Self::Shoot => button::SHOOT,
            Self::Left => button::LEFT,
            Self::Right => button::RIGHT,
            Self::Up => button::UP,
            Self::Down => button::DOWN,
            Self::AimUp => button::AIM_UP,
            Self::AimDown => button::AIM_DOWN,
        }
    }
}

/// An item switched on, named as progress names its `Item`s.
#[derive(Deserialize, Clone, Copy)]
enum ItemName {
    Bombs,
    GrappleBeam,
    GravitySuit,
    HiJumpBoots,
    MorphBall,
    ScrewAttack,
    SpaceJump,
    SpeedBooster,
    SpringBall,
    VariaSuit,
    XRayScope,
}

impl ItemName {
    const fn model(self) -> Item {
        match self {
            Self::Bombs => Item::Bombs,
            Self::GrappleBeam => Item::GrappleBeam,
            Self::GravitySuit => Item::GravitySuit,
            Self::HiJumpBoots => Item::HiJumpBoots,
            Self::MorphBall => Item::MorphBall,
            Self::ScrewAttack => Item::ScrewAttack,
            Self::SpaceJump => Item::SpaceJump,
            Self::SpeedBooster => Item::SpeedBooster,
            Self::SpringBall => Item::SpringBall,
            Self::VariaSuit => Item::VariaSuit,
            Self::XRayScope => Item::XRayScope,
        }
    }
}

/// The checked frame's parity, `Even` by default.
#[derive(Deserialize, Clone, Copy, Default)]
enum ParityName {
    #[default]
    Even,
    Odd,
}

impl ParityName {
    const fn model(self) -> Parity {
        match self {
            Self::Even => Parity::Even,
            Self::Odd => Parity::Odd,
        }
    }
}

/// `WordwiseTopSpeed`'s switch.
#[derive(Deserialize, Clone, Copy)]
enum SpeedCapName {
    Wordwise,
    Exact,
}

impl SpeedCapName {
    const fn model(self) -> SpeedCap {
        match self {
            Self::Wordwise => SpeedCap::Wordwise,
            Self::Exact => SpeedCap::Exact,
        }
    }
}

/// The kinds of cell of spec `OtherBlockKinds`.
#[derive(Deserialize, Clone, Copy)]
enum CellKind {
    DoorToRoom,
    DoorToNowhere,
    Spikes,
    AirWithSpikes,
    BreaksToShots,
    HoldsGrapple,
    AirShotsActOn,
    BreaksToBombs,
    AirBombsActOn,
    ReactingBlock,
    ReactingAir,
    StandIn,
}

impl CellKind {
    /// `OtherBlockKinds`: how the kind meets her motion: as solid, as air, as it answers, or as
    /// the cell a stand-in `names`.
    const fn model(self, names: Option<Cell>) -> Block {
        match self {
            Self::DoorToRoom
            | Self::AirWithSpikes
            | Self::AirShotsActOn
            | Self::AirBombsActOn
            | Self::ReactingAir => Block::Air,
            Self::DoorToNowhere | Self::Spikes | Self::BreaksToShots | Self::HoldsGrapple => {
                Block::Solid
            }
            Self::BreaksToBombs | Self::ReactingBlock => Block::Answered,
            Self::StandIn => Block::StandsFor(names),
        }
    }
}

/// The world's answer for a cell that answers, `Solid` by default.
#[derive(Deserialize, Clone, Copy, Default, PartialEq, Eq)]
enum AnswerName {
    #[default]
    Solid,
    Air,
}

/// What a pose step's moves met.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum MetName {
    AnimationEnd,
    Floor,
    NoFloor,
    Nothing,
    Wall,
    WallKick,
}

impl MetName {
    /// What her moves met, and whether her animation ends.
    const fn model(self) -> (Option<Met>, bool) {
        match self {
            Self::AnimationEnd => (None, true),
            Self::Floor => (Some(Met::Floor), false),
            Self::NoFloor => (Some(Met::NoFloor), false),
            Self::Nothing => (None, false),
            Self::Wall => (Some(Met::Wall), false),
            Self::WallKick => (Some(Met::WallKick), false),
        }
    }
}

/// The way a cell is reached.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum WayName {
    Down,
    Left,
    Look,
    Right,
    Up,
}

impl WayName {
    /// The name of the model's way.
    const fn of(way: Way) -> Self {
        match way {
            Way::Move(Heading::Down) => Self::Down,
            Way::Move(Heading::Left) => Self::Left,
            Way::Move(Heading::Right) => Self::Right,
            Way::Move(Heading::Up) => Self::Up,
            Way::Look => Self::Look,
        }
    }
}

/// The near-wall test's outcome.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum WallTestName {
    Kicked,
    Neither,
    Readied,
}

impl WallTestName {
    /// The name of the model's outcome.
    const fn of(test: WallTest) -> Self {
        match test {
            WallTest::Kicked => Self::Kicked,
            WallTest::Neither => Self::Neither,
            WallTest::Readied => Self::Readied,
        }
    }
}

// ---- The model's values, from the shape.

/// The buttons a list names.
fn buttons(names: &[ButtonName]) -> Buttons {
    names
        .iter()
        .fold(Buttons::default(), |pad, b| Buttons(pad.0 | b.model().0))
}

/// A pair of pixels, `[x, y]`, as the model's pixels.
const fn px_pair([x, y]: [u16; 2]) -> Pair<Px> {
    Pair {
        x: Wrapping(x),
        y: Wrapping(y),
    }
}

/// A pose with her body's size there, as a vector gives one.
type SizedPose = (Pose, Radius);

impl PoseTable {
    /// The pose (spec §2): `turn` for `Turning` alone, `landed_from` for `Landing` alone.
    fn pose(&self) -> Pose {
        let landed_from = self.landed_from.map(LandedFromName::model);
        Pose {
            body: self.body.model(),
            action: self.action.model(self.turn.model(), landed_from),
            facing: self.facing.model(),
            aim: self.aim.model(),
            gun: self.gun.model(),
            legs: self.legs.model(),
        }
    }

    /// The pose with her body's size there, which every pose of a vector gives.
    fn sized(&self) -> SizedPose {
        let radius = self.radius.expect("a vector's pose gives its radius");
        (self.pose(), px_pair(radius))
    }
}

/// The vectors' stand-in room: a floor from `floor_top` down, a wall from `wall_right`
/// rightward, a ceiling above `ceiling_bottom`, single blocks at cells (sloped, solid,
/// stepped or another kind), solid enemies, air elsewhere.
struct Room {
    floor_top: Option<u32>,
    wall_right: Option<u32>,
    ceiling_bottom: Option<u32>,
    blocks: Vec<((u16, u16), Block)>,
    /// The cells whose reaction lets her through this frame.
    through: Vec<(u16, u16)>,
    enemies: Vec<Hitbox>,
}

impl World for Room {
    fn block(&self, cell: Cell) -> Block {
        let Cell { x: column, y: row } = cell;
        if let Some(&(_, block)) = self.blocks.iter().find(|(at, _)| *at == (column, row)) {
            return block;
        }
        let from = |cell: u16, face: Option<u32>| {
            face.is_some_and(|face| u32::from(cell).wrapping_mul(16) >= face)
        };
        let above = |cell: u16, face: Option<u32>| {
            face.is_some_and(|face| u32::from(cell).wrapping_add(1).wrapping_mul(16) <= face)
        };
        if from(row, self.floor_top)
            || from(column, self.wall_right)
            || above(row, self.ceiling_bottom)
        {
            Block::Solid
        } else {
            Block::Air
        }
    }
    fn lets_through(&self, cell: Cell, _: Way) -> bool {
        self.through.contains(&(cell.x, cell.y))
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        &self.enemies
    }
}

/// The cell whose top-left corner is `[x, y]`.
const fn corner([x, y]: [u16; 2]) -> (u16, u16) {
    (x / 16, y / 16)
}

/// A sloped block: its corner and its surface at its 16 columns; 16 is an empty column (the
/// surface just past the far edge), 17 a column far past (spec `SlopesCarry`). The model holds
/// a surface as its depth from the block's open side (spec `SlopesCarry`): a floor's top row as
/// it is, a ceiling's bottom row counted up from the block's bottom row (15 less it), 16 and 17
/// the same for both. The walk factor is 1.00 by default.
fn slope(table: &SlopeTable, faces: SlopeFaces) -> ((u16, u16), Block) {
    let depth = table.surface.map(|n| {
        assert!(n <= 17, "surface row {n}");
        Wrapping(if faces == SlopeFaces::Ceiling && n <= 15 {
            15_u16.saturating_sub(n)
        } else {
            n
        })
    });
    let factor = table.factor.as_ref().map_or(Wrapping(0x100), |f| f.0);
    (
        corner(table.block),
        Block::Slope(Slope {
            faces,
            depth,
            factor,
        }),
    )
}

/// A stepped block: its corner and its quarters, upper row then lower.
fn stepped(table: &SteppedTable) -> ((u16, u16), Block) {
    (
        corner(table.block),
        Block::Stepped(table.quarters.map(|row| row.0)),
    )
}

/// The world's room: the other kinds of cell, the sloped, solid and stepped blocks, the solid
/// enemy, and the faces of the floor, the wall and the ceiling.
fn room(world: &WorldTable) -> Room {
    let cells = world.cells.iter().map(|c| {
        let names = c.names.map(|n| {
            let (x, y) = corner(n);
            Cell { x, y }
        });
        (corner(c.block), c.kind.model(names))
    });
    let floors: Vec<&SlopeTable> = match &world.slope_floor {
        Some(OneOrMore::One(one)) => vec![one],
        Some(OneOrMore::More(more)) => more.iter().collect(),
        None => Vec::new(),
    };
    let blocks = cells
        .chain(floors.into_iter().map(|s| slope(s, SlopeFaces::Floor)))
        .chain(
            world
                .slope_ceiling
                .iter()
                .map(|s| slope(s, SlopeFaces::Ceiling)),
        )
        .chain(world.solid_block.map(|b| (corner(b), Block::Solid)))
        .chain(world.stepped.iter().map(stepped))
        .collect();
    let through = world
        .cells
        .iter()
        .filter(|c| c.answer == AnswerName::Air)
        .map(|c| corner(c.block))
        .collect();
    let enemies = world
        .enemy
        .iter()
        .map(|e| Hitbox {
            centre: px_pair(e.centre),
            radius: px_pair(e.radius),
        })
        .collect();
    Room {
        floor_top: world.floor_top,
        wall_right: world.wall_right,
        ceiling_bottom: world.ceiling_bottom,
        blocks,
        through,
        enemies,
    }
}

/// Samus at a vector's start (spec §1.2) in `pose`: her motion follows the pad unless she is
/// skidding, her dash is hers to keep, and she remembers no spin and no seat. In a jump (its
/// take-off included), a spin, a fall or a wall jump (a turn in a jump or a fall included) she
/// is in the air: an upward speed there is a rise, and a vertical speed of 0 means she has just
/// started to fall. In any other pose she is on the ground, keeping any vertical speed she has,
/// an upward one a rise she keeps (`FollowingTheGround`, `RisingAndFalling`), or, where
/// `before.falling` says so, falling at 0 there. Jump held through is as `before` gives it, a
/// count of 0 and nothing the frame before where it doesn't.
fn start(before: &Before, pose: Pose) -> Samus {
    // A missing quantity is 0.
    let (speed_x, speed_y) = (
        before.speed_x.unwrap_or(STILL),
        before.speed_y.unwrap_or(STILL),
    );
    let dash = before.dash.map_or(ZERO, |d| d.0);
    let [x, y] = before.position;
    let bounces = match before.bounces {
        0 => Bounces::None,
        1 => Bounces::Once,
        2 => Bounces::Twice,
        n => panic!("bounces {n}, past 2"),
    };
    let facing = Side::of(pose.facing);
    let way = if speed_x.size == ZERO {
        facing
    } else if speed_x.negative {
        Side::Left
    } else {
        Side::Right
    };
    let airborne = matches!(
        pose.action,
        Action::TakingOff
            | Action::Jumping
            | Action::Spinning
            | Action::Falling
            | Action::WallJumping
            | Action::Turning(Turn::Jump | Turn::Fall)
    );
    let phase = match (airborne, speed_y.negative) {
        (true, true) => Phase::Rising,
        (true, false) => Phase::Falling,
        (false, _) if before.falling => Phase::GroundFalling,
        (false, _) => Phase::Ground,
    };
    let jump_held = JumpHeld {
        frames: Wrapping(before.jump_held),
        before: if before.landed_or_ended {
            FrameBefore::LandedOrEnded
        } else {
            FrameBefore::Other
        },
    };
    Samus {
        position: Position { x: x.0, y: y.0 },
        run: Run {
            speed: speed_x.size,
            way,
            carry: if way == facing {
                Carry::FollowsPad
            } else {
                Carry::CarriesOn
            },
            dash,
            dash_is: DashIs::Hers,
        },
        vertical: Vertical {
            speed: speed_y.value(),
            phase,
            bounces,
        },
        pose,
        spin: Spin::Off,
        seat: Seat::Unseated,
        jump_held,
    }
}

/// One of a vector's frames: the pose it starts in and her body's size there, the buttons
/// held and those just pressed, whether her pose's animation ends on it, and whether the spin
/// has turned far.
#[derive(Clone, Copy)]
struct Step {
    /// The pose at the frame's start.
    pose: Pose,
    /// Her body's size there.
    radius: Radius,
    /// The buttons held.
    pad: Buttons,
    /// The buttons just pressed.
    pressed: Buttons,
    /// `AnimationEnds`: the frame's own `animation_ends` (default false), never inferred.
    animation_ends: bool,
    /// `WallJump`: the frame's own `spin_turned_far` (default false), given as `animation_ends`.
    spin_turned_far: bool,
}

/// One frame of a vector or its lead-in. The buttons just pressed are `pressed`, and jump as
/// `jump_pressed` says.
fn step_of(f: &FrameTable) -> Step {
    let (pose, radius) = f.pose.sized();
    let jump = if f.jump_pressed {
        button::JUMP
    } else {
        Buttons::default()
    };
    Step {
        pose,
        radius,
        pad: buttons(&f.pad),
        pressed: Buttons(buttons(&f.pressed).0 | jump.0),
        animation_ends: f.animation_ends,
        spin_turned_far: f.spin_turned_far,
    }
}

/// The frame parity a vector's first frame has: the lead-in's frames alternate back from
/// the checked frame's.
const fn first_parity(checked: Parity, frames: usize) -> Parity {
    if frames % 2 == 1 {
        checked
    } else {
        other(checked)
    }
}

/// The other parity.
const fn other(p: Parity) -> Parity {
    match p {
        Parity::Even => Parity::Odd,
        Parity::Odd => Parity::Even,
    }
}

/// A pose's body size in a frame (spec §1.3, the game's data): for the pose the frame starts in,
/// the size its start gives, the body the frame's moves use; for another, the frame's own end
/// where it names that pose, else any the vector gives. A record's body can lag its pose, so the
/// same pose may come with two sizes, one at a frame's end and one at the next frame's start.
fn body_size(p: Pose, step: Step, end: Option<SizedPose>, known: &[SizedPose]) -> Radius {
    (step.pose == p)
        .then_some((step.pose, step.radius))
        .or_else(|| end.filter(|e| e.0 == p))
        .or_else(|| known.iter().find(|k| k.0 == p).copied())
        .map_or(step.radius, |k| k.1)
}

/// Runs one vector; returns its mismatches.
fn run(vector: &Vector) -> Vec<String> {
    let name = format!("{} ({})", vector.id, vector.source);
    let speed_cap = vector
        .rules
        .as_ref()
        .map_or(SpeedCapName::Wordwise, |r| r.speed_cap);
    let rules = Rules {
        speed_cap: speed_cap.model(),
        ..Rules::REFERENCE
    };
    let items: Set<Item> = vector.items.iter().map(|i| (i.model(), true)).collect();
    let room = room(&vector.world);
    let checked = vector.checked();
    let frames: Vec<Step> = vector
        .lead_in
        .iter()
        .chain([&checked])
        .map(step_of)
        .collect();
    let mut samus = start(&vector.before, frames[0].pose);
    // The pose each frame ends in, where the vector says it: the next frame's pose, and the
    // checked frame's next_pose. Every pose the vector gives a body size for: each frame's
    // start, the checked frame's next_pose, and the pose it ends in.
    let last_next = vector.next_pose.as_ref().map(PoseTable::sized);
    let after_pose = vector.after.pose.as_ref().map(PoseTable::sized);
    let ends: Vec<Option<SizedPose>> = frames
        .iter()
        .skip(1)
        .map(|f| Some((f.pose, f.radius)))
        .chain([last_next])
        .collect();
    let mut known: Vec<SizedPose> = frames.iter().map(|f| (f.pose, f.radius)).collect();
    known.extend(last_next);
    known.extend(after_pose);
    let mut parity = first_parity(vector.frame.model(), frames.len());
    let mut misses = Vec::new();
    let mut outcome = None;
    for (n, (&step, &end)) in frames.iter().zip(&ends).enumerate() {
        let body = |p: Pose| body_size(p, step, end, &known);
        let frame = Frame {
            pad: step.pad,
            pressed: step.pressed,
            items,
            spin_turned_far: step.spin_turned_far,
            elevator_moving: vector.world.elevator_moving,
            parity,
            animation_ends: step.animation_ends,
            body: &body,
        };
        parity = other(parity);
        match samus.step(&frame, &room, &rules) {
            Ok(frame_outcome) => outcome = Some(frame_outcome),
            Err(unknown) => {
                misses.push(format!("{name}: frame {n} starts in {unknown:?}"));
                return misses;
            }
        }
        // The first pose that differs; the later ones follow from it. The checked frame's
        // after.pose, where given, is the pose it ends in instead of next_pose.
        let checked = n.wrapping_add(1) == frames.len();
        let expected = end
            .map(|e| e.0)
            .filter(|_| !(checked && after_pose.is_some()));
        if misses.is_empty() && expected.is_some_and(|e| e != samus.pose) {
            misses.push(format!(
                "{name}: frame {n} ends in pose {:?}, expected {expected:?}",
                samus.pose
            ));
        }
    }
    let outcome = outcome.expect("a checked frame");
    let differ = compared(&vector.after, &samus, &outcome)
        .into_iter()
        .filter_map(|(field, expected, actual)| {
            let expected = expected.filter(|e| *e != actual)?;
            Some(format!(
                "{name}: {field} expected {expected}, actual {actual}"
            ))
        });
    misses.extend(differ);
    misses
}

/// Runs one pose step through the chart (`Moment::next_pose`); returns its mismatch.
fn run_pose_step(step: &PoseStep) -> Option<String> {
    let id = &step.id;
    let given = [&step.pose, &step.after.pose].map(|p| p.radius);
    assert_eq!(
        given,
        [None, None],
        "{id}: a pose step's poses give no radius"
    );
    let start = step.pose.pose();
    let held = buttons(&step.pad);
    let jump = if step.jump_pressed {
        button::JUMP
    } else {
        Buttons::default()
    };
    let pressed = Buttons(buttons(&step.pressed).0 | jump.0);
    let (met, animation_ends) = step.met.model();
    let items: Set<Item> = step.items.iter().map(|i| (i.model(), true)).collect();
    // A step has no lead-in, so nothing makes jump held through a press.
    let moment = Moment {
        pad: held,
        pressed,
        spring_ball: items[Item::SpringBall],
        animation_ends,
        speed: step.speed_x.unwrap_or(STILL).size,
        airborne: step.in_the_air(met),
        met,
    };
    let chosen = moment.next_pose(start);
    let expected = Ok(step.after.pose.pose());
    (chosen != expected).then(|| {
        format!(
            "{id} ({}): pose expected {expected:?}, actual {chosen:?}",
            step.source
        )
    })
}

/// A cell as the vectors write it, `[x, y]`, its top-left corner in px.
fn show_cell(cell: Cell) -> String {
    format!("[{}, {}]", u32::from(cell.x) * 16, u32::from(cell.y) * 16)
}

/// A reached cell as the vectors write it.
fn show_touch(touch: Touch) -> String {
    format!("{} {:?}", show_cell(touch.cell), WayName::of(touch.way))
}

/// A list as the vectors' cells are compared, each item written by `show`.
fn listed<T>(items: &[T], show: impl Fn(&T) -> String) -> String {
    items.iter().map(show).collect::<Vec<_>>().join("; ")
}

/// A `P.S` quantity as the vectors write it.
fn show(f: Fixed) -> String {
    format!("{:04X}.{:04X}", f.pixel(), f.sub())
}

/// Where a placement put her, as the vectors write it.
fn show_placed(placed: Option<Fixed>) -> String {
    placed.map_or_else(|| "none".to_owned(), show)
}

/// Her horizontal speed as the player reads it, rightward positive.
const fn speed_x(samus: &Samus) -> Signed {
    Signed {
        negative: matches!(samus.run.way, Side::Left),
        size: samus.run.speed,
    }
}

/// Her vertical speed as the player reads it, downward positive.
fn speed_y(samus: &Samus) -> Signed {
    Signed {
        negative: samus.vertical.speed.negative(),
        size: samus.vertical.speed.size(),
    }
}

/// The ball's bounces as the vectors count them.
const fn bounces(b: Bounces) -> u8 {
    match b {
        Bounces::None => 0,
        Bounces::Once => 1,
        Bounces::Twice => 2,
    }
}

/// One field a vector can compare: its name, the expected value where the vector gives it, and
/// the actual one, as text.
type Compared = (&'static str, Option<String>, String);

/// What she is at the frame's end: her quantities, her pose, and jump held through's two parts,
/// as `before` gives them. The radius `after.pose` gives is that pose's body (spec §1.3), data
/// the frames read like every pose's radius, not a quantity of hers.
fn hers(after: &After, samus: &Samus) -> Vec<Compared> {
    let x = after.position.map(|[x, _]| x).or(after.position_x);
    let landed = samus.jump_held.before == FrameBefore::LandedOrEnded;
    vec![
        ("position.x", x.map(|q| show(q.0)), show(samus.position.x)),
        (
            "position.y",
            after.position.map(|[_, y]| show(y.0)),
            show(samus.position.y),
        ),
        (
            "speed_x",
            after.speed_x.map(Signed::show),
            speed_x(samus).show(),
        ),
        (
            "speed_y",
            after.speed_y.map(Signed::show),
            speed_y(samus).show(),
        ),
        ("dash", after.dash.map(|d| show(d.0)), show(samus.run.dash)),
        (
            "bounces",
            after.bounces.map(|b| b.to_string()),
            bounces(samus.vertical.bounces).to_string(),
        ),
        (
            "pose",
            after.pose.as_ref().map(|p| format!("{:?}", p.pose())),
            format!("{:?}", samus.pose),
        ),
        (
            "jump_held",
            after.jump_held.map(|n| n.to_string()),
            samus.jump_held.frames.to_string(),
        ),
        (
            "landed_or_ended",
            after.landed_or_ended.map(|b| b.to_string()),
            landed.to_string(),
        ),
    ]
}

/// The frame's outputs.
fn outputs(after: &After, outcome: &Outcome) -> Vec<Compared> {
    let cell = |[x, y]: &[u16; 2]| format!("[{x}, {y}]");
    let touch = |t: &TouchTable| format!("{} {:?}", cell(&t.cell), t.way);
    vec![
        (
            "placed",
            after.placed.as_ref().map(|p| show_placed(p.0)),
            show_placed(outcome.placed),
        ),
        (
            "rising",
            after.rising.map(|b| b.to_string()),
            outcome.rising.to_string(),
        ),
        (
            "skidding",
            after.skidding.map(|b| b.to_string()),
            outcome.skidding.to_string(),
        ),
        (
            "reached",
            after.reached.as_ref().map(|r| listed(r, touch)),
            listed(&outcome.reached, |t| show_touch(*t)),
        ),
        (
            "inside",
            after.inside.as_ref().map(|i| listed(i, cell)),
            listed(&outcome.inside, |c| show_cell(*c)),
        ),
        (
            "wall_test",
            after.wall_test.map(|w| format!("{w:?}")),
            format!("{:?}", WallTestName::of(outcome.wall_test)),
        ),
    ]
}

/// Every field a vector's `after` gives, hers and the frame's outputs.
fn compared(after: &After, samus: &Samus, outcome: &Outcome) -> Vec<Compared> {
    let mut fields = hers(after, samus);
    fields.extend(outputs(after, outcome));
    fields
}

/// The vector file, read into its shape: a key or a name the shape doesn't know fails here.
fn file() -> VectorFile {
    toml::from_str(VECTORS).expect("the vector file reads as its shape")
}

#[test]
fn every_vector() {
    let vectors = file().vector;
    assert!(!vectors.is_empty(), "no vectors");
    let failing: Vec<Vec<String>> = vectors.iter().map(run).filter(|m| !m.is_empty()).collect();
    assert!(
        failing.is_empty(),
        "{} of {} vectors fail:\n{}",
        failing.len(),
        vectors.len(),
        failing.concat().join("\n")
    );
}

#[test]
fn every_pose_step() {
    let steps = file().pose_step;
    assert!(!steps.is_empty(), "no pose steps");
    let misses: Vec<String> = steps.iter().filter_map(run_pose_step).collect();
    assert!(
        misses.is_empty(),
        "{} of {} pose steps fail:\n{}",
        misses.len(),
        steps.len(),
        misses.join("\n")
    );
}

/// The header allows only "#" and "." in a stepped block's quarters, two to a row: the reader
/// refuses any other.
#[test]
fn stepped_quarters_are_hash_or_dot() {
    let read = |text: &str| QuarterRow::try_from(text.to_owned());
    assert_eq!(read("#."), Ok(QuarterRow([true, false])));
    assert_eq!(read(".#"), Ok(QuarterRow([false, true])));
    for refused in ["#x", "# ", "X.", "o#", "##.", "#", ""] {
        assert!(read(refused).is_err(), "{refused:?} read as quarters");
    }
}

#[test]
fn catalogue_matches_the_vectors() {
    let ids: Vec<String> = CATALOGUE.iter().map(|b| format!("{:?}", b.id)).collect();
    for (n, id) in ids.iter().enumerate() {
        assert!(!ids[..n].contains(id), "{id} twice in the catalogue");
    }
    let file = file();
    let exercised: Vec<&str> = file
        .vector
        .iter()
        .flat_map(|v| &v.entries)
        .chain(file.pose_step.iter().flat_map(|s| &s.entries))
        .map(String::as_str)
        .collect();
    for entry in &exercised {
        assert!(
            ids.iter().any(|id| id == entry),
            "{entry} is not in the catalogue"
        );
    }
    // An entry the spec checks on the original alone needs no vector; one it checks by vectors
    // needs one.
    let unexercised: Vec<String> = CATALOGUE
        .iter()
        .filter(|b| b.checked == Checked::ByVectors)
        .map(|b| format!("{:?}", b.id))
        .filter(|id| !exercised.contains(&id.as_str()))
        .collect();
    assert!(
        unexercised.is_empty(),
        "entries no vector exercises: {unexercised:?}"
    );
}
