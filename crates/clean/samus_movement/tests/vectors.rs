//! Runs every vector and pose step of `scenarios/vectors/samus-movement.toml` through the model.
//!
//! The file is read with `toml` and `serde` into its shape, declared once below as the file's
//! header gives it: every table refuses a key the shape doesn't know, and every name a value it
//! doesn't know, so an unknown key or name fails the test rather than being skipped. A vector
//! whose frame reads the body of a pose it doesn't give is refused, and so is a pose step that
//! gives a body, which the chart never reads. The model's types are built from the shape; they
//! know nothing of the reader.

#![cfg(test)]

use core::cell::RefCell;
use core::num::Wrapping;
use samus_movement::{
    Bounces, CATALOGUE, Carry, Choices, Chosen, Dashing, Direction, DirectionalTravels,
    DrawnJumpHistory, Frame, FrameBefore, Hit, JumpButton, JumpHeld, Knockback, Met, Moment,
    Outcome, Phase, Push, PushFrames, Rules, Run, Samus, Seat, SpeedCap, TravelDistance,
    TravelKind, TravelledDistances, Vertical, WallTest,
};
use serde::Deserialize;
use types::{
    Action, Aim, Block, Body, Buttons, Cell, Checked, ClipEnd, Facing, Fixed, Gun, Heading, Hitbox,
    Item, LandedFrom, Legs, Pair, Parity, Parts, Pose, Position, Px, Radius, Set, Side, Slope,
    SlopeFaces, Touch, Turn, Way, World, ZERO, button, fixed,
};

/// One vector without a lead-in, played through the single step: her State and the frame's
/// outputs.
fn receipt(vector: &Vector) -> (Samus, Outcome) {
    assert!(vector.lead_in.is_empty());
    let step = step_of(&vector.checked());
    let end = vector.next_pose.as_ref().map(PoseTable::sized);
    let mut known = vec![vector.pose.sized()];
    known.extend(end);
    known.extend(ending(vector));
    let body = |pose| body_size(pose, step, end, &known).unwrap();
    let headings = |_| None;
    let world = room(&vector.world);
    let frame = Frame {
        sprite_before_movement: vector
            .sprite_before_movement
            .unwrap_or(types::Sprite::Plain),
        cinematic_active: vector.cinematic_active,
        pad: step.pad,
        pressed: step.pressed,
        items: vector.items.iter().map(|i| (i.model(), true)).collect(),
        spin_turned_far: step.spin_turned_far,
        elevator_moving: vector.world.elevator_moving,
        parity: vector.frame.model(),
        animation_ends: step.animation_ends,
        body: &body,
        cannon_heading: &headings,
    };
    let mut samus = start(&vector.before, step.pose);
    let rules = applied_rules(vector);
    let moves = samus.moves(&frame, &world, &rules).unwrap();
    let outcome = samus.react(moves, step.hit);
    (samus, outcome)
}

/// Spec: `AnimationEnds`. An end that hands over its pose takes it without pose fit, whatever
/// body the caller gives the pose handed over: her State and every output of the frame are
/// the same at every height, and nothing places her.
#[test]
fn animation_handover_bypasses_fit_at_every_supplied_target_height() {
    let mut vectors = file().vector;
    let vector = vectors
        .iter_mut()
        .find(|v| v.id == "C-attempt-taller-animation-handover")
        .unwrap();
    assert_eq!(run(vector), Vec::<String>::new());
    let (expected_state, expected_outcome) = receipt(vector);
    assert!(expected_outcome.handed_over);
    assert_eq!(expected_outcome.placed, None);
    for radius in [[0, 0], [5, 19], [5, 21], [u16::MAX, u16::MAX]] {
        vector.next_pose.as_mut().unwrap().radius = Some(radius);
        let (state, outcome) = receipt(vector);
        assert_eq!(state, expected_state);
        assert_eq!(outcome, expected_outcome);
    }
}

/// Spec: `MovesAsShePosed`. Her position after her moves is compared wherever a vector gives
/// it, and tells apart her moves from the aim change's step that follows them: standing at
/// 00FF.0000 with the pad asking to run, her moves leave her there and the step takes her on.
#[test]
fn position_after_moves_is_compared_apart_from_the_step() {
    let mut vectors = file().vector;
    let given = vectors
        .iter()
        .filter(|v| v.after.position_after_moves.is_some())
        .count();
    assert!(given > 0, "no vector gives position_after_moves");
    let vector = vectors
        .iter_mut()
        .find(|v| v.id == "C-after-moves-screen-edge")
        .unwrap();
    assert_eq!(run(vector), Vec::<String>::new());
    let (samus, outcome) = receipt(vector);
    assert_ne!(outcome.position_after_moves, samus.position);
    vector.after.position_after_moves = vector.after.position;
    assert!(
        run(vector)
            .iter()
            .any(|m| m.contains("position_after_moves")),
        "her final position passed as her position after her moves"
    );
}

/// Spec: `SolidEnemies`, `SlopesCarry`. Several solid enemies decide in the world's order: the
/// first she reaches and doesn't overlap, not the nearest. Each order of the same two enemies
/// leaves her where its vector says, seated or not, and swapping the order changes the frame.
#[test]
fn the_first_enemy_in_order_decides() {
    let vectors = file().vector;
    let find = |id: &str| vectors.iter().find(|v| v.id == id).unwrap();
    let (gap_first, touch_first) = (
        find("C-attempt-running-gap-touch"),
        find("C-attempt-running-touch-gap"),
    );
    for vector in [gap_first, touch_first] {
        assert_eq!(run(vector), Vec::<String>::new());
    }
    let (gap_state, _) = receipt(gap_first);
    let (touch_state, _) = receipt(touch_first);
    assert_ne!(gap_state.position, touch_state.position);
}

/// Spec §12, `PushStarts`: a frame reports "push started" exactly where its push starts, which
/// her knockback tells: she starts the frame with no push under way and ends it with one.
#[test]
fn push_started_is_reported_where_a_push_starts() {
    let vectors = file().vector;
    let mut started = 0_usize;
    for vector in vectors.iter().filter(|v| v.lead_in.is_empty()) {
        let before = start(&vector.before, vector.pose.pose()).knockback.push;
        let (samus, outcome) = receipt(vector);
        let expected = before.is_none() && samus.knockback.push.is_some();
        assert_eq!(outcome.push_started, expected, "{}", vector.id);
        started += usize::from(expected);
    }
    assert!(started > 0, "no vector starts a push");
}

const VECTORS: &str = include_str!("../../../../scenarios/vectors/samus-movement.toml");

// ---- The file's shape.

/// The vector file: its `[[vector]]` and `[[pose_step]]` tables.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    /// The vectors.
    vector: Vec<Vector>,
    /// Isolated observations of caller-held `TravelledDistances`.
    distance_history: Vec<DistanceHistory>,
    /// Jump history through a draw that may update it, her motion paused.
    jump_history: Vec<JumpHistory>,
    /// The pose steps.
    pose_step: Vec<PoseStep>,
}

/// A `[[jump_history]]`: jump held through across one cut of a passage, which may or may not
/// hold a draw that updates jump history.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JumpHistory {
    /// Its id.
    id: String,
    /// The record it comes from, or "constructed".
    source: String,
    /// The spec's behaviours it exercises.
    entries: Vec<String>,
    /// The history at the cut's start.
    before: HistoryTable,
    /// What the cut brings.
    input: DrawInput,
    /// The history at the cut's end.
    after: HistoryTable,
}

/// Jump history as a passage holds it: the count in draws, the previous draw's jump, and the
/// frame before.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryTable {
    /// The count, in draws.
    frames: u16,
    /// Jump held on the previous draw that updated jump history.
    last_draw_held: bool,
    /// Landed or ended the frame before.
    landed_or_ended: bool,
}

impl HistoryTable {
    /// The model's history.
    const fn model(&self) -> DrawnJumpHistory {
        DrawnJumpHistory {
            jump_held: JumpHeld {
                frames: Wrapping(self.frames),
                before: if self.landed_or_ended {
                    FrameBefore::LandedOrEnded
                } else {
                    FrameBefore::Other
                },
            },
            last_draw: jump_button(self.last_draw_held),
        }
    }
}

/// Jump held or up.
const fn jump_button(held: bool) -> JumpButton {
    if held {
        JumpButton::Held
    } else {
        JumpButton::Up
    }
}

/// A cut's input: whether it holds a draw that updates jump history, jump held, and the
/// controller's own press, which the draw doesn't read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawInput {
    /// The cut holds a draw that updates jump history.
    drawn: bool,
    /// Jump held at that draw.
    held: bool,
    /// The controller's poll saw jump just pressed.
    pressed: Option<bool>,
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
    /// Actual prior figure, independent of items; vector format defaults to Plain.
    sprite_before_movement: Option<types::Sprite>,
    /// Vector format defaults cinematic to inactive.
    #[serde(default)]
    cinematic_active: bool,
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
    /// The end the animation of her pose reaches, if any.
    animation_ends: Option<EndName>,
    /// The input "spin turned far".
    #[serde(default)]
    spin_turned_far: bool,
    /// Imported new pose cannon content.
    cannon_heading: Option<samus_movement::FiringDirection>,
    /// Imported old pose cannon content.
    previous_cannon_heading: Option<samus_movement::FiringDirection>,
    /// The checked frame's hit, if one comes (spec §12).
    hit: Option<HitTable>,
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
            cannon_heading: self.cannon_heading,
            previous_cannon_heading: self.previous_cannon_heading,
            hit: self.hit,
        }
    }
}

/// The frame's hit (the header's `hit`): the side it pushes her to, and how many frames its push
/// lasts.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct HitTable {
    /// The way it pushes her.
    side: SideName,
    /// How many frames its push lasts.
    time: PushTime,
}

impl HitTable {
    /// The model's hit.
    const fn model(self) -> Hit {
        Hit {
            side: self.side.model(),
            push: self.time.0,
        }
    }
}

/// How long a push lasts, as the model admits it (spec §12: at most 45).
#[derive(Deserialize, Clone, Copy)]
#[serde(try_from = "u8")]
struct PushTime(PushFrames);

impl TryFrom<u8> for PushTime {
    type Error = String;

    fn try_from(frames: u8) -> Result<Self, String> {
        PushFrames::new(frames)
            .map(Self)
            .ok_or_else(|| format!("a push time of {frames} frames, past 45"))
    }
}

/// `before.knockback`: the knockback at the start, every part given.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct KnockbackTable {
    /// The push time left.
    time: PushTime,
    /// The side it pushes her to.
    side: SideName,
    /// The push under way.
    push: PushName,
}

/// `after.knockback`: the parts of the knockback compared, each only where given.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AfterKnockback {
    /// The push time left.
    time: Option<PushTime>,
    /// The side it pushes her to.
    side: Option<SideName>,
    /// The push under way.
    push: Option<PushName>,
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
    /// The end the animation of her pose reaches, if any.
    animation_ends: Option<EndName>,
    /// The input "spin turned far".
    #[serde(default)]
    spin_turned_far: bool,
    /// Imported new pose cannon content.
    cannon_heading: Option<samus_movement::FiringDirection>,
    /// Imported old pose cannon content.
    previous_cannon_heading: Option<samus_movement::FiringDirection>,
    /// The frame's hit, if one comes (spec §12).
    hit: Option<HitTable>,
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

/// A vector's `rules`: the parameters it adjusts, each a value of the one `Rules` the model
/// reads; a parameter it doesn't name keeps the game's value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesTable {
    /// `WordwiseTopSpeed`'s switch.
    speed_cap: Option<SpeedCapName>,
    /// `Dash`: the dash gained a frame.
    dash_gain: Option<Quantity>,
    /// `AimingAndShooting`: the step forward into another running pose.
    aim_step: Option<Quantity>,
    /// `RidingTheElevator`: the ride's descent a frame.
    elevator_descent: Option<Quantity>,
    /// `Jumping`: the jump's speed without the hi-jump boots.
    jump_speed: Option<Quantity>,
    /// `FollowingTheGround`: the pull beyond her speed on the ground.
    ground_pull: Option<Quantity>,
    /// `ShrinkingLowersHer`: the crouch's own distance.
    crouch_lowers: Option<Quantity>,
    /// `RisingAndFalling`: the pull a frame.
    gravity: Option<GravityQuantity>,
    /// `WallJump`: the near-wall test's move.
    wall_reach: Option<Quantity>,
    /// `PoseFitAfterMoves`: the old body's half-height under which a change is refused.
    small_body: Option<u16>,
    /// `Landing`: the pixel part from which a landing makes the ball's bounce 1.
    bounce_threshold: Option<u16>,
    /// `BlocksStopFlush`: the horizontal move's limit.
    move_limit: Option<u16>,
    /// `PushStarts`: the push's rise.
    push_rise: Option<Quantity>,
    /// `PushMoves`: the push's gain a frame.
    push_gain: Option<Quantity>,
    /// `PushMoves`: the push's brake a frame.
    push_brake: Option<Quantity>,
    /// `PushMoves`: the push's top speed.
    push_top: Option<Quantity>,
    /// `BackwardLeap`: the leap's gain a frame.
    leap_gain: Option<Quantity>,
    /// `BackwardLeap`: the leap's top speed.
    leap_top: Option<Quantity>,
}

impl RulesTable {
    /// `rules` with each parameter this table names set to its value.
    fn applied(&self, mut rules: Rules) -> Rules {
        let set = |value: &mut Fixed, q: Option<Quantity>| {
            if let Some(q) = q {
                *value = q.0;
            }
        };
        set(&mut rules.dash_gain, self.dash_gain);
        set(&mut rules.aim_step, self.aim_step);
        set(&mut rules.elevator_descent, self.elevator_descent);
        set(&mut rules.jump.without, self.jump_speed);
        set(&mut rules.ground_pull, self.ground_pull);
        set(&mut rules.crouch_lowers, self.crouch_lowers);
        set(&mut rules.gravity, self.gravity.map(|g| Quantity(g.0)));
        set(&mut rules.wall_reach, self.wall_reach);
        set(&mut rules.push_rise, self.push_rise);
        set(&mut rules.push.step, self.push_gain);
        set(&mut rules.push_brake, self.push_brake);
        set(&mut rules.push.top, self.push_top);
        set(&mut rules.leap.step, self.leap_gain);
        set(&mut rules.leap.top, self.leap_top);
        let set_px = |value: &mut Px, n: Option<u16>| {
            if let Some(n) = n {
                *value = Wrapping(n);
            }
        };
        set_px(&mut rules.small_body, self.small_body);
        set_px(&mut rules.bounce_threshold, self.bounce_threshold);
        set_px(&mut rules.move_limit, self.move_limit);
        if let Some(cap) = self.speed_cap {
            rules.speed_cap = cap.model();
        }
        rules
    }
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
    #[serde(default)]
    enemies: Vec<EnemyTable>,
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
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is one yes-or-no key of the header's `before`, read apart"
)]
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
    /// Whether she is dashing.
    #[serde(default)]
    dashing: bool,
    /// The knockback; missing, no push time and no push under way.
    knockback: Option<KnockbackTable>,
    /// Her horizontal motion's memory (spec §1.2), "Follows" where missing in a push's pose.
    motion: Option<MotionName>,
    /// In an airborne pose, no vertical motion at all (a floor stopped the push).
    #[serde(default)]
    no_vertical_motion: bool,
}

/// What a vector compares at the checked frame's end, each only where given.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    /// Whether she is seated on a slope.
    seated: Option<bool>,
    /// The floor response accepted this frame, or "None".
    floor_response: Option<ExpectedFloor>,
    /// The touchdown's sound requests, in order.
    touchdown_requests: Option<Vec<types::SoundIdentity>>,
    /// The touchdown graphics opportunity.
    touchdown_graphics: Option<bool>,
    /// Fresh publication, not a separate test-support calculation or retained state.
    pending_aim: Option<PendingAim>,
    /// Her position, `[x, y]`.
    position: Option<[Quantity; 2]>,
    /// Her position after her moves, `[x, y]`, before the aim change's step and the pose
    /// change move her.
    position_after_moves: Option<[Quantity; 2]>,
    /// Complete latest signed ordinary distances, checked from movement's output.
    travelled: Option<DistanceMap>,
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
    /// Whether she is dashing at the frame's end.
    dashing: Option<bool>,
    /// Whether she is dashing once her moves are made.
    dashing_after_moves: Option<bool>,
    /// Whether her dash began this frame.
    dash_began: Option<bool>,
    /// Whether her dash was lost this frame.
    dash_lost: Option<bool>,
    /// Whether her animation's end handed over her pose.
    handed_over: Option<bool>,
    /// Whether she is heading down as her moves leave her.
    heading_down: Option<bool>,
    /// Her vertical speed as her moves leave her, at `heading_down`'s point.
    speed_y_after_moves: Option<Signed>,
    /// The parts of the knockback it gives.
    knockback: Option<AfterKnockback>,
    /// Whether she ends the frame falling at 0 (spec §1.2).
    falling: Option<bool>,
    /// Her horizontal motion's memory (spec §1.2).
    motion: Option<MotionName>,
}

/// Explicit None or a response; every nested field is consumed, no unknown fields allowed.
#[derive(Deserialize)]
#[serde(untagged)]
enum ExpectedFloor {
    None(NoFloorResponse),
    Response(FloorTable),
}

#[derive(Deserialize)]
enum NoFloorResponse {
    None,
}

#[derive(Deserialize, Debug)]
enum FloorKindName {
    UprightTouchdown,
    BallSettles,
    SpringHop,
    Rebound,
    ChangingBodyStops,
    TurningStops,
    GroundedBallStops,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FloorTable {
    kind: FloorKindName,
    speed: Signed,
    bounces: Option<u8>,
}

impl ExpectedFloor {
    fn show(&self) -> String {
        match self {
            Self::None(_) => "None".to_owned(),
            Self::Response(r) => format!("{:?} {:?} {}", r.kind, r.bounces, r.speed.show()),
        }
    }
}

fn show_floor(response: Option<types::FloorResponse>) -> String {
    let Some(r) = response else {
        return "None".to_owned();
    };
    let (kind, count) = match r.kind {
        types::FloorResponseKind::UprightTouchdown => (FloorKindName::UprightTouchdown, None),
        types::FloorResponseKind::BallSettles => (FloorKindName::BallSettles, None),
        types::FloorResponseKind::SpringHop => (FloorKindName::SpringHop, None),
        types::FloorResponseKind::Rebound(b) => (FloorKindName::Rebound, Some(bounces(b))),
        types::FloorResponseKind::ChangingBodyStops => (FloorKindName::ChangingBodyStops, None),
        types::FloorResponseKind::TurningStops => (FloorKindName::TurningStops, None),
        types::FloorResponseKind::GroundedBallStops => (FloorKindName::GroundedBallStops, None),
    };
    format!("{kind:?} {count:?} {}", speed_y(r.speed.0).show())
}

/// Independent expectation from `TouchdownRequests`' library/allowance prose, NOT the
/// production request constructor or any production constants. Compare all three fields.
fn expected_sound(identity: types::SoundIdentity) -> types::SoundRequest {
    use types::{QueueAllowance, SoundIdentity, SoundLibrary, SoundRequest};
    match identity {
        SoundIdentity::SpinEnded => SoundRequest {
            identity: SoundIdentity::SpinEnded,
            library: SoundLibrary::Samus,
            queue_allowance: QueueAllowance(6),
        },
        SoundIdentity::ScrewAttackEnded => SoundRequest {
            identity: SoundIdentity::ScrewAttackEnded,
            library: SoundLibrary::Samus,
            queue_allowance: QueueAllowance(6),
        },
        SoundIdentity::SamusLandedOrWallJumped => SoundRequest {
            identity: SoundIdentity::SamusLandedOrWallJumped,
            library: SoundLibrary::Library3,
            queue_allowance: QueueAllowance(6),
        },
        SoundIdentity::SamusLandedHard => SoundRequest {
            identity: SoundIdentity::SamusLandedHard,
            library: SoundLibrary::Library3,
            queue_allowance: QueueAllowance(6),
        },
    }
}

/// The touchdown's outputs: the floor response, its sound requests (identity, library and
/// allowance) and its graphics opportunity.
fn compared_new_outputs(after: &After, outcome: &Outcome) -> Vec<Compared> {
    vec![
        (
            "floor_response",
            after.floor_response.as_ref().map(ExpectedFloor::show),
            show_floor(outcome.floor_response),
        ),
        (
            "touchdown_requests",
            after.touchdown_requests.as_ref().map(|list| {
                format!(
                    "{:?}",
                    list.iter().copied().map(expected_sound).collect::<Vec<_>>()
                )
            }),
            format!("{:?}", outcome.touchdown_requests),
        ),
        (
            "touchdown_graphics",
            after.touchdown_graphics.map(|b| b.to_string()),
            outcome.touchdown_graphics.to_string(),
        ),
    ]
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

/// A complete signed distance map: no missing direction or unknown key is accepted.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DistanceMap {
    left: Signed,
    right: Signed,
    up: Signed,
    down: Signed,
}

impl DistanceMap {
    fn model(&self) -> DirectionalTravels {
        [
            (Direction::Left, self.left),
            (Direction::Right, self.right),
            (Direction::Up, self.up),
            (Direction::Down, self.down),
        ]
        .into_iter()
        .map(|(d, s)| (d, TravelDistance(s.value())))
        .collect()
    }
}

/// One isolated lifetime/selection case; collision is checked by full movement vectors.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DistanceHistory {
    id: String,
    source: String,
    entries: Vec<String>,
    before: DistanceMap,
    refresh: bool,
    travels: Vec<MeasuredTravel>,
    after: DistanceMap,
}

/// A measured travel, already excluding independent collision snapping.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MeasuredTravel {
    kind: TravelKindName,
    direction: DirectionName,
    accepted: Signed,
}

#[derive(Deserialize)]
enum TravelKindName {
    Walk,
    Vertical,
    AimStep,
    Placement,
    Look,
}

impl TravelKindName {
    fn model(&self) -> TravelKind {
        match self {
            Self::Walk => TravelKind::Walk,
            Self::Vertical => TravelKind::Vertical,
            Self::AimStep => TravelKind::AimStep,
            Self::Placement => TravelKind::Placement,
            Self::Look => TravelKind::Look,
        }
    }
}

#[derive(Deserialize)]
enum DirectionName {
    Down,
    Left,
    Right,
    Up,
}

impl DirectionName {
    fn model(&self) -> Direction {
        match self {
            Self::Down => Direction::Down,
            Self::Left => Direction::Left,
            Self::Right => Direction::Right,
            Self::Up => Direction::Up,
        }
    }
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

/// Gravity vectors supply both unsigned and signed parameter notation.
#[derive(Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
struct GravityQuantity(Fixed);

impl TryFrom<String> for GravityQuantity {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        if text.starts_with(['+', '-']) {
            Signed::try_from(text).map(|s| Self(s.value()))
        } else {
            Quantity::try_from(text).map(|q| Self(q.0))
        }
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

/// The publication expectation: all named directions unchanged, or explicit no publication.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct PendingAim(Option<samus_movement::FiringDirection>);

impl TryFrom<String> for PendingAim {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        if text == "None" {
            return Ok(Self(None));
        }
        toml::from_str::<AimCheckpoint>(&format!("aim = {text:?}"))
            .map(|value| Self(value.aim))
            .map_err(|error| error.to_string())
    }
}

/// Caller-held checkpoint, also used to read the vector's direction names strictly.
#[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AimCheckpoint {
    aim: Option<samus_movement::FiringDirection>,
}

fn show_aim(aim: Option<samus_movement::FiringDirection>) -> String {
    aim.map_or_else(|| "None".to_owned(), |aim| format!("{aim:?}"))
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
    Leaping,
    Moonwalking,
    Morphing,
    Recoiling,
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
            Self::Leaping => Action::Leaping,
            Self::Moonwalking => Action::Moonwalking,
            Self::Morphing => Action::Morphing,
            Self::Recoiling => Action::Recoiling,
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

/// A side: the way a hit pushes her (spec §12).
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum SideName {
    Left,
    Right,
}

impl SideName {
    const fn model(self) -> Side {
        match self {
            Self::Left => Side::Left,
            Self::Right => Side::Right,
        }
    }
}

/// The push under way (spec §12), the file's `none` for no push under way.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum PushName {
    None,
    Down,
    Up,
}

impl PushName {
    const fn model(self) -> Option<Push> {
        match self {
            Self::None => None,
            Self::Down => Some(Push::Down),
            Self::Up => Some(Push::Up),
        }
    }
}

/// Her horizontal motion's memory (spec §1.2): it follows the pad, carries on along her facing,
/// or carries on against it, a skid.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum MotionName {
    Follows,
    CarriesOn,
    Skids,
}

impl MotionName {
    /// The way her speed points as she remembers it, and whether it carries on, facing `ahead`.
    fn model(self, ahead: Side) -> (Side, Carry) {
        let behind = match ahead {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        };
        match self {
            Self::Follows => (ahead, Carry::FollowsPad),
            Self::CarriesOn => (ahead, Carry::CarriesOn),
            Self::Skids => (behind, Carry::CarriesOn),
        }
    }

    /// The name of her run's memory, facing `ahead`.
    fn of(run: &Run, ahead: Side) -> Self {
        match (run.carry, run.way == ahead) {
            (Carry::FollowsPad, _) => Self::Follows,
            (Carry::CarriesOn, true) => Self::CarriesOn,
            (Carry::CarriesOn, false) => Self::Skids,
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
    AimDown,
    AimUp,
    Down,
    Jump,
    Left,
    Right,
    Run,
    Shoot,
    Up,
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

/// The kind of end her animation reaches (the header's `animation_ends`).
#[derive(Deserialize, Clone, Copy)]
enum EndName {
    End,
    EndUnlessJump,
}

impl EndName {
    const fn model(self) -> ClipEnd {
        match self {
            Self::End => ClipEnd::End,
            Self::EndUnlessJump => ClipEnd::EndUnlessJump,
        }
    }
}

/// What a pose step's moves met.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum MetName {
    AnimationEnd,
    AnimationEndUnlessJump,
    Floor,
    NoFloor,
    Nothing,
    Wall,
    WallKick,
}

impl MetName {
    /// What her moves met, and the end her animation reaches.
    const fn model(self) -> (Option<Met>, Option<ClipEnd>) {
        match self {
            Self::AnimationEnd => (None, Some(ClipEnd::End)),
            Self::AnimationEndUnlessJump => (None, Some(ClipEnd::EndUnlessJump)),
            Self::Floor => (Some(Met::Floor), None),
            Self::NoFloor => (Some(Met::NoFloor), None),
            Self::Nothing => (None, None),
            Self::Wall => (Some(Met::Wall), None),
            Self::WallKick => (Some(Met::WallKick), None),
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
    let factor = types::SlopeFactor(table.factor.as_ref().map_or(0x100, |f| {
        u16::try_from(f.0.0).expect("dimensionless factor")
    }));
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
        .chain(&world.enemies)
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
/// skidding, and she remembers no spin and no seat. In a jump (its
/// take-off included), a spin, a fall or a wall jump (a turn in a jump or a fall included) she
/// is in the air: an upward speed there is a rise, and a vertical speed of 0 means she has just
/// started to fall. In any other pose she is on the ground, keeping any vertical speed she has,
/// an upward one a rise she keeps (`FollowingTheGround`, `RisingAndFalling`), or, where
/// `before.falling` says so, falling at 0 there. Jump held through and dashing are as `before`
/// gives them: a count of 0, nothing the frame before and not dashing where it doesn't, so a
/// dash she starts with and not dashing is one she is to lose (spec `Dash`).
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
    let knockback = before
        .knockback
        .map_or(Knockback::spent(Side::Right), |k| Knockback {
            time_left: k.time.0,
            side: k.side.model(),
            push: k.push.model(),
        });
    let pushed = pushes(pose, knockback);
    if let Some(points) = pushed.filter(|_| speed_x.size != ZERO) {
        assert_eq!(
            speed_x.negative,
            points == Side::Left,
            "a push's pose: speed_x points its side"
        );
    }
    let (way, carry) = match (before.motion, pushed) {
        (Some(motion), _) => motion.model(facing),
        (None, Some(_)) => MotionName::Follows.model(facing),
        (None, None) => {
            let way = if speed_x.size == ZERO {
                facing
            } else if speed_x.negative {
                Side::Left
            } else {
                Side::Right
            };
            let carry = if way == facing {
                Carry::FollowsPad
            } else {
                Carry::CarriesOn
            };
            (way, carry)
        }
    };
    let airborne = matches!(
        pose.action,
        Action::TakingOff
            | Action::Jumping
            | Action::Spinning
            | Action::Falling
            | Action::WallJumping
            | Action::Leaping
            | Action::Turning(Turn::Jump | Turn::Fall)
    );
    // `Recoiling` is read as a grounded pose (spec §1.2: every pose the spec doesn't name as
    // airborne), yet when a push lifts her in it she is rising.
    let phase = if speed_y.negative && (airborne || pose.action == Action::Recoiling) {
        Phase::Rising
    } else if before.no_vertical_motion {
        Phase::Ground
    } else if airborne {
        Phase::Falling
    } else if before.falling {
        Phase::GroundFalling
    } else {
        Phase::Ground
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
            carry,
            dash,
            dashing: if before.dashing {
                Dashing::On
            } else {
                Dashing::Off
            },
        },
        vertical: Vertical {
            speed: speed_y.value(),
            phase,
            bounces,
        },
        pose,
        choices: Choices::starting_in(pose),
        seat: Seat::Unseated,
        jump_held,
        knockback,
    }
}

/// Spec §12, her poses in a push: the side her horizontal speed points to in `pose`, where a push
/// decides it: the knockback's side in `Recoiling`, backward in `Leaping`; elsewhere none, as §1
/// says.
fn pushes(pose: Pose, knockback: Knockback) -> Option<Side> {
    if pose.action == Action::Recoiling {
        Some(knockback.side)
    } else if pose.action == Action::Leaping {
        Some(Side::of(pose.facing.back()))
    } else {
        None
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
    /// `AnimationEnds`: the frame's own `animation_ends` (default none), never inferred.
    animation_ends: Option<ClipEnd>,
    /// `WallJump`: the frame's own `spin_turned_far` (default false), given as `animation_ends`.
    spin_turned_far: bool,
    /// §12: the frame's hit, which the frame's second step reads.
    hit: Option<Hit>,
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
        animation_ends: f.animation_ends.map(EndName::model),
        spin_turned_far: f.spin_turned_far,
        hit: f.hit.map(HitTable::model),
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
/// A push's poses, `Recoiling` and `Leaping`, are content like every pose's (spec §1.3, §12).
/// None where the vector gives no body for the pose.
fn body_size(p: Pose, step: Step, end: Option<SizedPose>, known: &[SizedPose]) -> Option<Radius> {
    (step.pose == p)
        .then_some((step.pose, step.radius))
        .or_else(|| end.filter(|e| e.0 == p))
        .or_else(|| known.iter().find(|k| k.0 == p).copied())
        .map(|k| k.1)
}

/// The pose a vector's checked frame ends in, with her body there, as `after.pose` gives it: that
/// radius is the pose's body, a push's start's `Recoiling` included, the chosen pose's own as pose
/// fit takes it.
fn ending(vector: &Vector) -> Option<SizedPose> {
    vector.after.pose.as_ref().map(PoseTable::sized)
}

fn applied_rules(vector: &Vector) -> Rules {
    vector
        .rules
        .as_ref()
        .map_or_else(Rules::default, |r| r.applied(Rules::default()))
}

/// Runs one vector; returns its mismatches.
fn run(vector: &Vector) -> Vec<String> {
    let name = format!("{} ({})", vector.id, vector.source);
    let rules = applied_rules(vector);
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
    known.extend(ending(vector));
    let mut parity = first_parity(vector.frame.model(), frames.len());
    let mut misses = Vec::new();
    let mut outcome = None;
    for (n, (&step, &end)) in frames.iter().zip(&ends).enumerate() {
        // The poses whose body the frame reads and the vector doesn't give: the vector is
        // refused.
        let ungiven: RefCell<Vec<Pose>> = RefCell::new(Vec::new());
        let body = |p: Pose| {
            body_size(p, step, end, &known).unwrap_or_else(|| {
                ungiven.borrow_mut().push(p);
                step.radius
            })
        };
        let content = if n.wrapping_add(1) == frames.len() {
            &checked
        } else {
            &vector.lead_in[n]
        };
        let cannon_heading = |p: Pose| {
            if p == step.pose {
                content.previous_cannon_heading
            } else {
                end.filter(|e| e.0 == p).and(content.cannon_heading)
            }
        };
        let frame = Frame {
            sprite_before_movement: vector
                .sprite_before_movement
                .unwrap_or(types::Sprite::Plain),
            cinematic_active: vector.cinematic_active,
            pad: step.pad,
            pressed: step.pressed,
            items,
            spin_turned_far: step.spin_turned_far,
            elevator_moving: vector.world.elevator_moving,
            parity,
            animation_ends: step.animation_ends,
            body: &body,
            cannon_heading: &cannon_heading,
        };
        parity = other(parity);
        // The frame's two steps: her moves, then her reaction to the frame's hit.
        let stepped = samus
            .moves(&frame, &room, &rules)
            .map(|moves| samus.react(moves, step.hit));
        if let Some(p) = ungiven.borrow().first() {
            misses.push(format!(
                "{name}: frame {n} reads the body of {p:?}, not given"
            ));
            return misses;
        }
        match stepped {
            Ok(frame_outcome) => outcome = Some(frame_outcome),
            Err(unknown) => {
                misses.push(format!("{name}: frame {n} starts in {unknown:?}"));
                return misses;
            }
        }
        // The first pose that differs; the later ones follow from it. The checked frame's
        // after.pose, where given, is the pose it ends in instead of next_pose.
        let expected = end
            .map(|e| e.0)
            .filter(|_| !(n.wrapping_add(1) == frames.len() && after_pose.is_some()));
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

/// Runs one pose step through the chart (`Moment::next_pose`); returns its mismatch. The chart
/// reads no body, so a pose step's poses give none: one that gives a radius is refused, as a
/// body its frame never reads.
fn run_pose_step(step: &PoseStep) -> Option<String> {
    let id = &step.id;
    if [&step.pose, &step.after.pose]
        .iter()
        .any(|p| p.radius.is_some())
    {
        return Some(format!("{id}: a pose step's poses give no radius"));
    }
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
        items,
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

/// Her horizontal speed as the player reads it, rightward positive: the way her pose points it in
/// a push (spec §12), else the way it points (§1).
fn speed_x(samus: &Samus) -> Signed {
    let way = pushes(samus.pose, samus.knockback).unwrap_or(samus.run.way);
    Signed {
        negative: way == Side::Left,
        size: samus.run.speed,
    }
}

/// A vertical speed as the player reads it, downward positive.
fn speed_y(speed: Fixed) -> Signed {
    Signed {
        negative: speed.negative(),
        size: speed.size(),
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
    let x = after.position.map(|[x, _]| x);
    let landed = samus.jump_held.before == FrameBefore::LandedOrEnded;
    vec![
        (
            "dashing",
            after.dashing.map(|b| b.to_string()),
            (samus.run.dashing == Dashing::On).to_string(),
        ),
        ("position.x", x.map(|q| show(q.0)), show(samus.position.x)),
        (
            "position_x",
            after.position_x.map(|q| show(q.0)),
            show(samus.position.x),
        ),
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
            speed_y(samus.vertical.speed).show(),
        ),
        ("dash", after.dash.map(|d| show(d.0)), show(samus.run.dash)),
        (
            "seated",
            after.seated.map(|s| s.to_string()),
            (samus.seat == Seat::Seated).to_string(),
        ),
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

/// Her §12 and §1.2 memory at the frame's end: the knockback's parts the vector gives, whether
/// she is falling at 0, and her horizontal motion's memory.
fn remembered(after: &After, samus: &Samus) -> Vec<Compared> {
    let k = after.knockback.as_ref();
    let falling = samus.vertical.speed == ZERO
        && matches!(samus.vertical.phase, Phase::Falling | Phase::GroundFalling);
    let facing = Side::of(samus.pose.facing);
    vec![
        (
            "knockback.time",
            k.and_then(|k| k.time).map(|t| t.0.frames().to_string()),
            samus.knockback.time_left.frames().to_string(),
        ),
        (
            "knockback.side",
            k.and_then(|k| k.side).map(|s| format!("{:?}", s.model())),
            format!("{:?}", samus.knockback.side),
        ),
        (
            "knockback.push",
            k.and_then(|k| k.push).map(|p| format!("{:?}", p.model())),
            format!("{:?}", samus.knockback.push),
        ),
        (
            "falling",
            after.falling.map(|b| b.to_string()),
            falling.to_string(),
        ),
        (
            "motion",
            after.motion.map(|m| format!("{m:?}")),
            format!("{:?}", MotionName::of(&samus.run, facing)),
        ),
    ]
}

/// The frame's outputs.
fn outputs(after: &After, outcome: &Outcome) -> Vec<Compared> {
    let cell = |[x, y]: &[u16; 2]| format!("[{x}, {y}]");
    let touch = |t: &TouchTable| format!("{} {:?}", cell(&t.cell), t.way);
    let moved_to = outcome.position_after_moves;
    vec![
        (
            "placed",
            after.placed.as_ref().map(|p| show_placed(p.0)),
            show_placed(outcome.placed),
        ),
        (
            "position_after_moves",
            after
                .position_after_moves
                .map(|[x, y]| format!("[{}, {}]", show(x.0), show(y.0))),
            format!("[{}, {}]", show(moved_to.x), show(moved_to.y)),
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

/// The frame's outputs for her animation (spec §1.4).
fn for_her_animation(after: &After, outcome: &Outcome) -> Vec<Compared> {
    let flag = |name, expected: Option<bool>, actual: bool| {
        (name, expected.map(|b| b.to_string()), actual.to_string())
    };
    vec![
        flag(
            "dashing_after_moves",
            after.dashing_after_moves,
            outcome.dashing_after_moves == Dashing::On,
        ),
        flag("dash_began", after.dash_began, outcome.dash_began),
        flag("dash_lost", after.dash_lost, outcome.dash_lost),
        flag("handed_over", after.handed_over, outcome.handed_over),
        flag("heading_down", after.heading_down, outcome.heading_down),
        (
            "speed_y_after_moves",
            after.speed_y_after_moves.map(Signed::show),
            speed_y(outcome.vertical_speed_after_moves).show(),
        ),
    ]
}

/// Every field a vector's `after` gives, hers and the frame's outputs.
fn compared(after: &After, samus: &Samus, outcome: &Outcome) -> Vec<Compared> {
    let mut fields = hers(after, samus);
    fields.extend(remembered(after, samus));
    fields.push((
        "pending_aim",
        after.pending_aim.as_ref().map(|aim| show_aim(aim.0)),
        show_aim(outcome.pending_aim),
    ));
    fields.extend(compared_new_outputs(after, outcome));
    fields.extend(outputs(after, outcome));
    fields.extend(for_her_animation(after, outcome));
    fields.extend(compared_travels(
        after.travelled.as_ref(),
        &outcome.travelled,
    ));
    fields
}

/// Compare each named distance, using signed magnitude only for displaying vector notation.
fn compared_travels(expected: Option<&DistanceMap>, actual: &DirectionalTravels) -> Vec<Compared> {
    let expected = expected.map(DistanceMap::model);
    [
        ("travelled.left", Direction::Left),
        ("travelled.right", Direction::Right),
        ("travelled.up", Direction::Up),
        ("travelled.down", Direction::Down),
    ]
    .into_iter()
    .map(|(name, direction)| {
        (
            name,
            expected.as_ref().map(|e| speed_y(e[direction].0).show()),
            speed_y(actual[direction].0).show(),
        )
    })
    .collect()
}

#[test]
fn every_distance_history() {
    let histories = file().distance_history;
    assert!(!histories.is_empty(), "no distance histories");
    let mut misses = Vec::new();
    for history in &histories {
        let mut memory = TravelledDistances {
            travels: history.before.model(),
        };
        if history.refresh {
            memory.refresh();
        }
        for travel in &history.travels {
            memory.record(
                travel.kind.model(),
                travel.direction.model(),
                TravelDistance(travel.accepted.value()),
            );
        }
        for (field, expected, actual) in compared_travels(Some(&history.after), &memory.travels) {
            if expected.as_ref() != Some(&actual) {
                misses.push(format!(
                    "{} ({}): {field} expected {expected:?}, actual {actual}",
                    history.id, history.source
                ));
            }
        }
    }
    assert!(
        misses.is_empty(),
        "{} of {} histories fail:\n{}",
        misses.len(),
        histories.len(),
        misses.join("\n")
    );
}

/// Spec: `Jumping`. Each cut of a passage through the draw-only operation alone: a cut with a
/// draw counts jump against the previous draw's, whatever the controller's poll pressed, and a
/// cut without one leaves the history as it is.
#[test]
fn every_jump_history() {
    let histories = file().jump_history;
    assert!(!histories.is_empty(), "no jump histories");
    let mut misses = Vec::new();
    for history in &histories {
        let mut memory = history.before.model();
        if history.input.drawn {
            memory.draw(jump_button(history.input.held));
        }
        let expected = history.after.model();
        if memory != expected {
            misses.push(format!(
                "{} ({}, pressed {:?}): expected {expected:?}, actual {memory:?}",
                history.id, history.source, history.input.pressed
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "{} of {} jump histories fail:\n{}",
        misses.len(),
        histories.len(),
        misses.join("\n")
    );
}

/// The vector file, read into its shape: a key or a name the shape doesn't know fails here.
fn file() -> VectorFile {
    toml::from_str(VECTORS)
        .unwrap_or_else(|error| panic!("vector file: {} at {:?}", error.message(), error.span()))
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

/// C-wall as the file gives it, but for its `next_pose` line and its `after`.
const C_WALL_UNTIL_NEXT_POSE: &str = r#"
id = "C-wall"
source = "constructed"
entries = ["WallsAndCeilings"]
world = { floor_top = 288, wall_right = 256 }
before = { position = ["00FA.0000", "010B.FFFF"], speed_x = "+0002.0000" }
pose = { body = "Upright", action = "Running", facing = "Right", radius = [5, 21] }
pad = ["right"]
jump_pressed = false
"#;

/// Every `rules` key the header defines, with the game's value it gives, and another value.
const RULES_KEYS: [(&str, &str, &str); 18] = [
    ("push_rise", "\"0005.0000\"", "\"0004.0000\""),
    ("push_gain", "\"0001.8000\"", "\"0001.0000\""),
    ("push_brake", "\"0000.8000\"", "\"0000.4000\""),
    ("push_top", "\"0005.0000\"", "\"0003.0000\""),
    ("leap_gain", "\"0000.C000\"", "\"0001.0000\""),
    ("leap_top", "\"0005.0000\"", "\"0003.0000\""),
    ("speed_cap", "\"Wordwise\"", "\"Exact\""),
    ("dash_gain", "\"0000.1000\"", "\"0000.2000\""),
    ("aim_step", "\"0001.0000\"", "\"0003.0000\""),
    ("elevator_descent", "\"0001.0000\"", "\"0002.0000\""),
    ("jump_speed", "\"0004.E000\"", "\"0003.0000\""),
    ("ground_pull", "\"0001.0000\"", "\"0002.0000\""),
    ("crouch_lowers", "\"0005.0000\"", "\"0003.0000\""),
    ("gravity", "\"0000.1C00\"", "\"0000.4000\""),
    ("wall_reach", "\"0008.0000\"", "\"0010.0000\""),
    ("small_body", "8", "17"),
    ("bounce_threshold", "3", "2"),
    ("move_limit", "15", "8"),
];

/// The vector test applies each `rules` key to the one set of rules the model reads: at the
/// game's value the header gives, every key leaves the default rules as they are, and each key
/// at another value changes them.
#[test]
fn every_rules_key_is_applied() {
    let game = Rules::default();
    let read = |text: &str| -> Rules {
        let table: RulesTable = toml::from_str(text).expect("the rules read");
        table.applied(Rules::default())
    };
    let all = RULES_KEYS
        .iter()
        .map(|(key, value, _)| format!("{key} = {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(read(&all), game, "the game's values");
    for (key, _, other) in RULES_KEYS {
        assert_ne!(
            read(&format!("{key} = {other}\n")),
            game,
            "{key} isn't applied"
        );
    }
}

/// A vector whose frame reads a body it doesn't give is refused: C-wall without its
/// `next_pose`, whose wall stop chooses `AgainstWall`, a pose pose fit reads the body of.
#[test]
fn a_vector_missing_a_body_is_refused() {
    let given = format!(
        "{C_WALL_UNTIL_NEXT_POSE}{}\nafter = {{}}\n",
        r#"next_pose = { body = "Upright", action = "AgainstWall", facing = "Right", radius = [5, 21] }"#
    );
    let with_body: Vector = toml::from_str(&given).expect("C-wall reads");
    assert_eq!(run(&with_body), Vec::<String>::new());
    let missing: Vector =
        toml::from_str(&format!("{C_WALL_UNTIL_NEXT_POSE}after = {{}}\n")).expect("it reads");
    let misses = run(&missing);
    assert!(
        misses.iter().any(|m| m.contains("not given")),
        "refused for no body: {misses:?}"
    );
}

/// A vector that has every table the header gives, each with a mark `%n%` where a key may be
/// put in it: `rules`, `world` and each of its blocks, enemy and cells, `before`, a lead-in
/// frame and its pose, the checked pose, `next_pose`, `after`, a reached cell and `after.pose`.
const EVERY_TABLE: &str = r#"
id = "every-table"
source = "constructed"
entries = ["Landing"]
rules = { speed_cap = "Wordwise", dash_gain = "0000.1000", aim_step = "0001.0000", elevator_descent = "0001.0000", jump_speed = "0004.E000", ground_pull = "0001.0000", crouch_lowers = "0005.0000", gravity = "0000.1C00", wall_reach = "0008.0000", small_body = 8, bounce_threshold = 3, move_limit = 15, push_rise = "0005.0000", push_gain = "0001.8000", push_brake = "0000.8000", push_top = "0005.0000", leap_gain = "0000.C000", leap_top = "0005.0000"%0% }
items = ["MorphBall"]
world = { floor_top = 288, slope_floor = { block = [256, 272], surface = [16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16], factor = "1.00"%1% }, slope_ceiling = { block = [256, 0], surface = [17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17]%2% }, stepped = { block = [0, 0], quarters = ["..", ".."]%3% }, enemy = { centre = [8, 8], radius = [1, 1]%4% }, cells = [{ block = [16, 0], kind = "Spikes"%5% }]%6% }
before = { position = ["0100.0000", "0100.0000"], speed_y = "+0000.0000", motion = "Follows", no_vertical_motion = false, knockback = { time = 0, side = "Right", push = "None"%16% }%7% }
lead_in = [
  { pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21]%8% }, pad = [], jump_pressed = false, hit = { side = "Left", time = 5%17% }%9% },
]
pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21]%10% }
next_pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21]%11% }
hit = { side = "Left", time = 5%18% }
pad = []
jump_pressed = false
after = { reached = [{ cell = [0, 0], way = "Down"%12% }], pose = { body = "Upright", action = "Still", facing = "Right"%13% }, travelled = { left = "+0000.0000", right = "+0000.0000", up = "+0000.0000", down = "+0000.0000"%15% }, knockback = { time = 4, side = "Left", push = "Up"%19% }, falling = false, motion = "Follows"%14% }
"#;

/// How many marks `EVERY_TABLE` has.
const TABLES: usize = 20;

/// A pose step whose own pose, `after` and `after.pose` each have a mark, as `EVERY_TABLE`'s.
const STEP_TABLES: &str = r#"
id = "every-table"
source = "constructed"
entries = ["PadChart"]
pose = { body = "Upright", action = "Still", facing = "Right"%0% }
pad = []
jump_pressed = false
met = "nothing"
after = { pose = { body = "Upright", action = "Still", facing = "Right"%1% }%2% }
"#;

/// How many marks `STEP_TABLES` has.
const STEP_MARKS: usize = 3;

/// `text`, whose marks are `%0%` to `%marks - 1%`, with an unknown key in the table of mark `n`
/// alone, or in none.
fn with_unknown_key_in(text: &str, marks: usize, n: Option<usize>) -> String {
    (0..marks).fold(text.to_owned(), |text, mark| {
        let key = if Some(mark) == n { ", bogus = 1" } else { "" };
        text.replace(&format!("%{mark}%"), key)
    })
}

/// Whether `text` reads as a `T`, and as no `T` with an unknown key in any one table it has,
/// nor with one of its own.
fn refuses_every_unknown_key<T: for<'de> Deserialize<'de>>(text: &str, marks: usize) -> bool {
    let known = with_unknown_key_in(text, marks, None);
    let mut unknown = (0..marks).map(|n| with_unknown_key_in(text, marks, Some(n)));
    toml::from_str::<T>(&known).is_ok()
        && unknown.all(|u| toml::from_str::<T>(&u).is_err())
        && toml::from_str::<T>(&format!("{known}bogus = 1\n")).is_err()
}

/// A key the shape doesn't know is refused in every table of a vector, the vector's own
/// included, in a pose step and its `after`, and in the file; a name it doesn't know is refused
/// for each kind of name a vector gives.
#[test]
fn an_unknown_key_or_name_is_refused() {
    assert!(
        refuses_every_unknown_key::<Vector>(EVERY_TABLE, TABLES),
        "a vector's"
    );
    assert!(
        refuses_every_unknown_key::<PoseStep>(STEP_TABLES, STEP_MARKS),
        "a pose step's"
    );
    assert!(
        toml::from_str::<VectorFile>(
            "vector = []\npose_step = []\ndistance_history = []\njump_history = []\nbogus = 1\n"
        )
        .is_err(),
        "the file's key"
    );
    let known = with_unknown_key_in(EVERY_TABLE, TABLES, None);
    let names = [
        ("\"Wordwise\"", "\"Word\""),
        ("\"0000.1C00\"", "\"1C00\""),
        ("small_body = 8", "small_body = \"8\""),
        ("\"MorphBall\"", "\"Morph\""),
        ("\"Spikes\"", "\"Spike\""),
        ("\"Upright\"", "\"Standing\""),
        ("\"Still\"", "\"Rest\""),
        ("\"Right\"", "\"East\""),
        ("\"Down\"", "\"Downward\""),
        ("\"0100.0000\"", "\"100\""),
        ("pad = []", "pad = [\"select\"]"),
        (
            "jump_pressed = false\nafter",
            "animation_ends = \"Ends\"\njump_pressed = false\nafter",
        ),
        ("\"Follows\"", "\"Follow\""),
        ("push = \"None\"", "push = \"Nothing\""),
        ("side = \"Left\"", "side = \"West\""),
        ("time = 5", "time = 46"),
        ("\"Still\", facing", "\"Recoil\", facing"),
    ];
    for (name, unknown) in names {
        let text = known.replacen(name, unknown, 1);
        assert_ne!(text, known, "{name} is in the vector");
        assert!(toml::from_str::<Vector>(&text).is_err(), "{unknown} read");
    }
    let step = with_unknown_key_in(STEP_TABLES, STEP_MARKS, None);
    let met = step.replace("\"nothing\"", "\"nothing_at_all\"");
    assert!(toml::from_str::<PoseStep>(&met).is_err(), "a met read");
}

/// A history with a mark in each nested table and an after.travelled-shaped map.
const HISTORY_TABLES: &str = r#"
id = "history-tables"
source = "constructed"
entries = ["TravelledDistances"]
refresh = false
before = { left = "-0000.4000", right = "+0003.E000", up = "-0002.8000", down = "+0001.2000"%0% }
travels = [{ kind = "Walk", direction = "Left", accepted = "-0000.8000"%1% }]
after = { left = "-0000.8000", right = "+0003.E000", up = "-0002.8000", down = "+0001.2000"%2% }
"#;

#[test]
fn distance_tables_refuse_unknown_or_missing_values() {
    assert!(refuses_every_unknown_key::<DistanceHistory>(
        HISTORY_TABLES,
        STEP_MARKS
    ));
    let known = with_unknown_key_in(HISTORY_TABLES, STEP_MARKS, None);
    for (from, to) in [
        ("kind = \"Walk\"", "kind = \"Teleport\""),
        ("direction = \"Left\"", "direction = \"West\""),
        ("accepted = \"-0000.8000\"", "accepted = \"0000.8000\""),
        (", down = \"+0001.2000\"", ""),
    ] {
        let unknown = known.replace(from, to);
        assert_ne!(unknown, known);
        assert!(
            toml::from_str::<DistanceHistory>(&unknown).is_err(),
            "{unknown}"
        );
    }
    let movement = with_unknown_key_in(EVERY_TABLE, TABLES, None);
    let missing = movement.replace(", down = \"+0000.0000\"", "");
    assert!(toml::from_str::<Vector>(&missing).is_err());
}

/// Every signed wrapping value is restorable, independently of its direction or a pose.
#[test]
fn caller_memory_preserves_complete_wrapping_values() {
    let values = [0_u32, 1, 0xFFFF_C000, 0x8000_0000, u32::MAX];
    for bits in values {
        let travels = DirectionalTravels::from_fn(|_| TravelDistance(Wrapping(bits)));
        let mut memory = TravelledDistances { travels };
        let saved = memory;
        memory.refresh();
        memory = saved;
        assert_eq!(memory.travels, travels);
        memory.record(TravelKind::Walk, Direction::Right, TravelDistance::ZERO);
        for direction in [Direction::Left, Direction::Up, Direction::Down] {
            assert_eq!(memory.travels[direction].0.0, bits);
        }
        assert_eq!(memory.travels[Direction::Right], TravelDistance::ZERO);
    }
    let negative_fraction = TravelDistance(
        Signed::try_from("-0000.4000".to_owned())
            .expect("signed fraction")
            .value(),
    );
    assert_eq!(negative_fraction.0.words(), (0xFFFF, 0xC000));
}

/// A pose step's frame reads no body, so a pose step that gives one is refused.
#[test]
fn a_pose_step_giving_a_body_is_refused() {
    let step: PoseStep = toml::from_str(
        r#"
id = "P21"
source = "r04:473"
entries = ["PadChart"]
pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21] }
pad = ["jump"]
jump_pressed = true
met = "nothing"
after = { pose = { body = "Upright", action = "TakingOff", facing = "Right" } }
"#,
    )
    .expect("the step reads");
    assert!(
        run_pose_step(&step).is_some(),
        "a radius in a pose step passed"
    );
}

/// Spec: `TransitionFiringAim`. Every content direction passes through actual complete
/// movement, for both new-jump and old-moonwalk publications; caller memory round-trips too.
#[test]
fn every_firing_direction_is_published_and_checkpointed_unchanged() {
    use samus_movement::FiringDirection;
    let directions = [
        FiringDirection::Left,
        FiringDirection::Right,
        FiringDirection::DownLeft,
        FiringDirection::DownRight,
        FiringDirection::UpLeft,
        FiringDirection::UpRight,
        FiringDirection::DownFacingLeft,
        FiringDirection::DownFacingRight,
        FiringDirection::UpFacingLeft,
        FiringDirection::UpFacingRight,
    ];
    let mut vectors = file().vector;
    for direction in directions {
        for id in [
            "C-aim-jump-press",
            "C-aim-moonwalk-turn",
            "C-aim-moonwalk-jump",
        ] {
            let vector = vectors
                .iter_mut()
                .find(|v| v.id == id)
                .expect("supplied movement vector");
            vector.cannon_heading = (id == "C-aim-jump-press").then_some(direction);
            vector.previous_cannon_heading = (id != "C-aim-jump-press").then_some(direction);
            vector.after.pending_aim = Some(PendingAim(Some(direction)));
            assert!(run(vector).is_empty(), "{id}: {direction:?}");
        }
        let mut retained = AimCheckpoint {
            aim: Some(direction),
        };
        let saved = toml::to_string(&retained).expect("serialize optional firing direction");
        retained.aim = None;
        assert_eq!(retained.aim, None);
        retained = toml::from_str(&saved).expect("restore optional firing direction");
        assert_eq!(retained.aim, Some(direction));
        // No-publication cases run the complete controller with content available.
        for id in ["C-aim-jump-held", "C-aim-jump-released", "C-aim-same-pose"] {
            let vector = vectors
                .iter_mut()
                .find(|v| v.id == id)
                .expect("supplied no publication case");
            vector.cannon_heading = Some(direction);
            vector.previous_cannon_heading = Some(direction);
            assert!(run(vector).is_empty(), "{id}");
        }
    }
    let empty = AimCheckpoint { aim: None };
    let saved = toml::to_string(&empty).expect("serialize no pending aim");
    assert_eq!(toml::from_str::<AimCheckpoint>(&saved).unwrap(), empty);
}

/// New vector inputs and expectations reject unknown directions rather than rounding them.
#[test]
fn firing_content_and_expectations_are_strict() {
    let known = with_unknown_key_in(EVERY_TABLE, TABLES, None);
    for field in ["cannon_heading", "previous_cannon_heading"] {
        assert!(toml::from_str::<Vector>(&format!("{known}{field} = \"Upward\"\n")).is_err());
    }
    assert!(PendingAim::try_from("Upward".to_owned()).is_err());
    assert!(PendingAim::try_from("none".to_owned()).is_err());
}

/// A complete ordinary call: input/body/room from content, persistent Samus restored,
/// output publications delivered separately. Mutating either transport field must mismatch
/// the independent spec expectations, not merely leave vector identities unchanged.
#[test]
fn ordinary_call_delivers_outputs_and_comparison_checks_transport_fields() {
    let vectors = file().vector;
    let vector = vectors
        .iter()
        .find(|v| v.id == "C-output-spin-soft")
        .unwrap();
    let content = vector.checked();
    let step = step_of(&content);
    let end = vector.next_pose.as_ref().map(PoseTable::sized);
    let known = [
        vector.pose.sized(),
        vector.next_pose.as_ref().unwrap().sized(),
    ];
    let body = |pose| body_size(pose, step, end, &known).unwrap();
    let cannon = |_| None;
    let world = room(&vector.world);
    let mut samus = start(&vector.before, step.pose);
    let frame = Frame {
        sprite_before_movement: types::Sprite::Plain,
        cinematic_active: false,
        pad: step.pad,
        pressed: step.pressed,
        items: Set::default(),
        spin_turned_far: step.spin_turned_far,
        elevator_moving: false,
        parity: Parity::Even,
        animation_ends: step.animation_ends,
        body: &body,
        cannon_heading: &cannon,
    };
    let rules = Rules::default();
    let moves = samus.moves(&frame, &world, &rules).unwrap();
    let outcome = samus.react(moves, None);
    let saved = toml::to_string(&samus).unwrap();
    let restored: Samus = toml::from_str(&saved).unwrap();
    assert_eq!(restored, samus);
    let mut next_firing = TravelledDistances::fresh();
    next_firing.travels = outcome.travelled;
    assert_eq!(next_firing.travels, outcome.travelled);
    let matches = |out: &Outcome| {
        compared_new_outputs(&vector.after, out)
            .iter()
            .all(|(_, expected, actual)| {
                expected.as_ref().is_none_or(|expected| expected == actual)
            })
    };
    assert!(matches(&outcome));
    for index in 0..outcome.touchdown_requests.len() {
        let mut wrong_library = outcome.clone();
        wrong_library.touchdown_requests[index].library =
            match wrong_library.touchdown_requests[index].library {
                types::SoundLibrary::Samus => types::SoundLibrary::Library3,
                types::SoundLibrary::Library3 => types::SoundLibrary::Samus,
            };
        assert!(!matches(&wrong_library), "wrong library must fail");
        let mut wrong_allowance = outcome.clone();
        wrong_allowance.touchdown_requests[index].queue_allowance = types::QueueAllowance(1);
        assert!(!matches(&wrong_allowance), "wrong allowance must fail");
    }
    // Later consumer requests are appended verbatim, even when their identity repeats.
    let later_request = outcome.touchdown_requests[0];
    let mut transport = outcome.touchdown_requests.clone();
    transport.push(later_request);
    assert_eq!(transport[0], *transport.last().unwrap());
    assert_eq!(transport.len(), outcome.touchdown_requests.len() + 1);
    assert!(outcome.touchdown_graphics); // Appearance may still decline graphics.
    assert!(outcome.floor_response.is_some());
}

/// Spec: `PadChart`, `ShrinkingLowersHer`. Down pressed standing starts the crouch and its
/// look down lowers her 5 px, a placement; down held and not pressed keeps her standing,
/// unlowered and unplaced, her ground pull's cell alone reached. Both through the one step.
#[test]
fn lowering_needs_down_pressed_not_merely_held() {
    let mut vectors = file().vector;
    let vector = vectors
        .iter_mut()
        .find(|v| v.id == "C-touch-lowering-look")
        .unwrap();
    assert_eq!(buttons(&vector.pad), button::DOWN);
    assert_eq!(buttons(&vector.pressed), button::DOWN);
    assert_eq!(run(vector), Vec::<String>::new());
    vector.after.pose = vector.next_pose.clone();
    vector.after.position = Some([
        Quantity(fixed(Wrapping(40), 0)),
        Quantity(fixed(Wrapping(176), 0xFFFF)),
    ]);
    vector.after.placed = Some(Placed(Some(fixed(Wrapping(176), 0xFFFF))));
    assert_eq!(run(vector), Vec::<String>::new());

    vector.pressed.clear();
    vector.next_pose = None;
    vector.after = toml::from_str(
        r#"
pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21] }
position = ["0028.0000", "00AB.FFFF"]
placed = "none"
reached = [{ cell = [32, 192], way = "Down" }]
"#,
    )
    .unwrap();
    assert_eq!(buttons(&vector.pad), button::DOWN);
    assert_eq!(run(vector), Vec::<String>::new());
}

#[test]
fn new_output_tables_and_names_are_strict() {
    for text in [
        "kind = \"UprightTouchdown\"\nspeed = \"+0001.1000\"\nbogus = 1",
        "kind = \"Landed\"\nspeed = \"+0001.1000\"",
    ] {
        assert!(toml::from_str::<FloorTable>(text).is_err());
    }
    let known = with_unknown_key_in(EVERY_TABLE, TABLES, None);
    for field in [
        "sprite_before_movement = \"Unknown\"",
        "cinematic_active = \"false\"",
    ] {
        assert!(toml::from_str::<Vector>(&format!("{known}{field}\n")).is_err());
    }
}

/// A standing frame on the floor with nothing held: her situation's last row chooses the pose
/// she already has.
const STANDS_ON: &str = r#"
id = "choice-same-pose"
source = "constructed"
entries = ["WallJump"]
world = { floor_top = 288 }
before = { position = ["0100.0000", "010B.FFFF"] }
pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21] }
pad = []
jump_pressed = false
after = { pose = { body = "Upright", action = "Still", facing = "Right", radius = [5, 21] } }
"#;

/// A frame facing the screen on a moving elevator: the ride keeps her pose and chooses nothing.
const RIDES_ON: &str = r#"
id = "choice-none"
source = "constructed"
entries = ["WallJump"]
world = { floor_top = 288, elevator_moving = true }
before = { position = ["0100.0000", "010B.FFFF"] }
pose = { body = "Upright", action = "Still", facing = "Screen", radius = [5, 21] }
pad = []
jump_pressed = false
after = { pose = { body = "Upright", action = "Still", facing = "Screen", radius = [5, 21] } }
"#;

/// Spec: `WallJump`. The frame reports what its choice recorded in her choices: a frame whose
/// last row chooses the pose she already has records it anew, and her choices move on; the
/// elevator's ride, which keeps the same pose, chooses nothing, reports none and leaves her
/// choices as they were.
#[test]
fn recorded_choice_tells_a_same_pose_choice_from_none() {
    let same: Vector = toml::from_str(STANDS_ON).expect("the standing frame reads");
    assert_eq!(run(&same), Vec::<String>::new());
    let (samus, outcome) = receipt(&same);
    assert_eq!(samus.pose, same.pose.pose(), "the pose stays the same");
    assert_eq!(outcome.recorded_choice, Some(Chosen::Other));
    let started = Choices::starting_in(same.pose.pose());
    assert_eq!(
        samus.choices,
        Choices {
            latest: Chosen::Other,
            before: started.latest,
        }
    );

    let none: Vector = toml::from_str(RIDES_ON).expect("the ride reads");
    assert_eq!(run(&none), Vec::<String>::new());
    let (samus, outcome) = receipt(&none);
    assert_eq!(samus.pose, none.pose.pose(), "the pose stays the same");
    assert_eq!(outcome.recorded_choice, None);
    assert_eq!(samus.choices, Choices::starting_in(none.pose.pose()));
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
        .chain(file.distance_history.iter().flat_map(|h| &h.entries))
        .chain(file.jump_history.iter().flat_map(|h| &h.entries))
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
