//! Samus's motion as a harness domain: the samus-motion model (`clean_model_samus_motion`)
//! checked against the oracle, frame by frame, on the spec's declared fields
//! (`docs/behaviours/samus-motion.md` §1.1).
//!
//! Each frame decodes record N: Samus, the pad, the frame count, the elevator, the level's
//! layer 1 and the enemies' common words, all from work RAM, so every room reads. It steps her
//! through the model over that level and the enemies the selection would touch
//! (`game::samus_world`), and compares her motion words with record N+1's. Nothing answers
//! from record N+1, so no field is unverified; the null model is her State left as record N
//! has it.

use core::fmt;

use clean_model_samus_motion::{
    BehaviourId, CATALOGUE, Rules as Reference, Samus as Model, Seat, Standing, Vertical, WallTest,
};
use clean_types::{Action, Block as Kind, Heading, Pose as Shape, Way, World, ZERO};
use game::Content;
use game::collision::Room;
use game::collision::solid_enemy::Obstacle;
use game::model::num::Fixed;
use game::pose::{self, PoseRequests};
use game::samus_world::SamusWorld;
use game::samus_world::state::clean_of;
use game::samus_world::step::{Ran, Reads, run};
use game::state::Controller;
use game::state::room::{Block, BlockColumns, BlockKind, ElevatorRide};
use game::state::samus::{Bounce, Motion, Position, Samus, SpeedMode, VerticalMotion};
use game::state::strand::Frame;
use oracle_record::{Cut, FrameRecord, NmiKind};

use super::{Domain, Field, FieldSet, Interval, Mask, Uncompared};
use crate::bundle::{self, ContentError};
use crate::decode::{DecodeError, Rules, buttons};
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// A block is 16 px.
const BLOCK_SHIFT: u16 = 4;

/// The equipment bits the masks read.
const GRAVITY_SUIT: u16 = 0x0020;
const SPEED_BOOSTER: u16 = 0x2000;

/// Why a frame isn't one Samus's motion is compared on: her motion doesn't run in it, or the
/// harness couldn't decode it (`Unreadable`: a harness defect).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a code the model has no value for.
    Unreadable(DecodeError),
    /// The main loop isn't in gameplay.
    GameState(u16),
    /// Record N's NMI found the main thread mid-frame, so the frame from N is the rest of a
    /// lag frame, or the NMI didn't run.
    NotAtFrameEnd,
    /// The main thread resumes inside a routine that waits for NMI before the gameplay pass
    /// finishes (the message box): Samus's motion doesn't run.
    Waiting(&'static str),
    /// Her pose at the frame's start is outside the model's domain.
    Pose(u16),
    /// Samus's per-frame handler at the frame's start runs no motion: she is locked (the
    /// gunship, a station, a cutscene) or time is frozen (the x-ray scope).
    NotRunning(u16),
    /// Her movement handler at the frame's start isn't the normal one and is no deferred
    /// behaviour this harness names.
    MovementHandler(u16),
    /// A deferred behaviour (spec §10) moves her instead of the normal movement handler.
    Deferred(Deferred),
}

/// A behaviour the spec defers (§10, "Out of scope") whose own movement handler runs in the
/// frame instead of the motion model's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deferred {
    /// The grapple beam's movement (the swing release).
    Grapple,
    /// Knockback from a hit.
    Knockback,
    /// The bomb jump's movement.
    BombJump,
    /// The shinespark's or the crystal flash's movement.
    Shinespark,
    /// The damage boost's movement.
    DamageBoost,
}

impl fmt::Display for Deferred {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Grapple => "grapple (the swing release's movement)",
            Self::Knockback => "knockback",
            Self::BombJump => "the bomb jump's movement",
            Self::Shinespark => "the shinespark or crystal flash movement",
            Self::DamageBoost => "the damage boost's movement",
        })
    }
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::GameState(code) => write!(f, "game state {code:#04X}, not gameplay"),
            Self::NotAtFrameEnd => write!(f, "record N isn't at the main loop's frame end (lag)"),
            Self::Waiting(at) => write!(f, "the main thread waits in {at}: motion doesn't run"),
            Self::Pose(code) => write!(f, "pose {code:#04X} isn't in the domain"),
            Self::NotRunning(code) => {
                write!(
                    f,
                    "her frame handler {code:#06X} runs no motion: {}",
                    stopped(*code)
                )
            }
            Self::MovementHandler(code) => write!(f, "movement handler {code:#06X}"),
            Self::Deferred(behaviour) => write!(f, "deferred: {behaviour}"),
        }
    }
}

impl From<DecodeError> for Skip {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

fn word(record: &FrameRecord, source: WramWord) -> Result<u16, Skip> {
    Ok(source.read(&record.wram)?)
}

fn byte(record: &FrameRecord, source: WramByte) -> Result<u8, Skip> {
    Ok(source.read(&record.wram)?)
}

/// Bank $90 log: the handler gameplay runs for Samus once a frame ($90:E722).
const SAMUS_NEW_STATE_HANDLER: WramWord = WramWord {
    name: "FRAME_HANDLER_BETA",
    offset: 0x0A44,
};
/// Bank $90 log: the movement handler the normal state handler runs ($90:E94B).
const SAMUS_MOVEMENT_HANDLER: WramWord = WramWord {
    name: "SAMUS_MOVEMENT_HANDLER",
    offset: 0x0A58,
};
/// Bank $90 log: the water's surface, negative when the room has none ($90:9741).
const FX_Y_POS: WramWord = WramWord {
    name: "FX_Y_POS",
    offset: 0x195E,
};
/// Bank $90 log: the lava's or acid's surface, read when the water's is negative.
const LAVA_ACID_Y_POS: WramWord = WramWord {
    name: "LAVA_ACID_Y_POS",
    offset: 0x1962,
};
/// Bank $90 log: bit 2 turns the water's physics off.
const FX_LIQUID_OPTIONS: WramWord = WramWord {
    name: "FX_LIQUID_OPTIONS",
    offset: 0x197E,
};
const LIQUID_PHYSICS_OFF: u16 = 0x0004;
/// Bank $91 log: the knockback's movement handler, which the knockback's start sets
/// ($91:EE1F, $91:EE40), morphed or not.
const KNOCKBACK: u16 = 0xDF38;
/// The running movement type, the only one whose dash the speed booster grows.
const RUNNING: u8 = 0x01;
/// The spin jump's movement type, whose falling spin the space jump restarts.
const SPIN_JUMPING: u8 = 0x03;
/// The standing movement type, which the running let-go row stands her in.
const STANDING: u8 = 0x00;
/// The pad's left and right (`JOYPAD1_LASTKEYS`), which no binding moves.
const SIDES: u16 = 0x0300;
/// Bank $90 log: the shinespark windup's movement handler, which the windup's start sets
/// ($90:CFFA).
const SHINESPARK_WINDUP: u16 = 0xD068;
/// Bank $94 log: the pushes the block-inside reactions (quicksand, treadmills) set at the start
/// of each frame, before the motion ($94:9B60): the extra displacements and the X deceleration
/// multiplier.
const EXTRA_X_DISPLACEMENT: WramWord = WramWord {
    name: "EXTRA_SAMUS_X_DISPLACEMENT",
    offset: 0x0B58,
};
const EXTRA_X_SUBDISPLACEMENT: WramWord = WramWord {
    name: "EXTRA_SAMUS_X_SUBDISPLACEMENT",
    offset: 0x0B56,
};
const EXTRA_Y_DISPLACEMENT: WramWord = WramWord {
    name: "EXTRA_SAMUS_Y_DISPLACEMENT",
    offset: 0x0B5C,
};
const EXTRA_Y_SUBDISPLACEMENT: WramWord = WramWord {
    name: "EXTRA_SAMUS_Y_SUBDISPLACEMENT",
    offset: 0x0B5A,
};
const X_DECEL_MULTIPLIER: WramByte = WramByte {
    name: "SAMUS_X_DECEL_MULT",
    offset: 0x0B4C,
};
/// Bank $84 log: the quicksand's reactions set this, and zero her speeds.
const IN_QUICKSAND: WramWord = WramWord {
    name: "SAMUS_IN_QUICKSAND",
    offset: 0x1E71,
};

/// Whether frame `next` (its record: the pushes stay until the next frame's start) ran a
/// block-inside push.
fn pushed(next: &FrameRecord) -> Result<bool, Skip> {
    let words = [
        EXTRA_X_DISPLACEMENT,
        EXTRA_X_SUBDISPLACEMENT,
        EXTRA_Y_DISPLACEMENT,
        EXTRA_Y_SUBDISPLACEMENT,
        IN_QUICKSAND,
    ];
    for source in words {
        if word(next, source)? != 0 {
            return Ok(true);
        }
    }
    Ok(byte(next, X_DECEL_MULTIPLIER)? != 0)
}

/// What the handler `code`, one that runs no motion, is.
const fn stopped(code: u16) -> &'static str {
    match code {
        0xE8DC => "she is locked",
        0xE8D6 => "she is locked into a station",
        0xE8D9 => "she is being drained",
        0xE902 => "she enters or leaves the gunship",
        0xE8CD => "none",
        0xE94F => "the x-ray scope freezes time",
        0xE90E => "no movement",
        _ => "an unnamed handler",
    }
}

/// Whether Samus's handlers in `record` run the motion of this domain in the next frame: the
/// normal, Ceres or elevator state handler (each runs the movement handler), and the normal
/// movement handler.
fn motion_runs(record: &FrameRecord) -> Result<(), Skip> {
    match word(record, SAMUS_NEW_STATE_HANDLER)? {
        0xE725 | 0xE8AA | 0xE8EC => {}
        code => return Err(Skip::NotRunning(code)),
    }
    let deferred = match word(record, SAMUS_MOVEMENT_HANDLER)? {
        0xA337 => return Ok(()),
        0x946E => Deferred::Grapple,
        KNOCKBACK => Deferred::Knockback,
        0xE025 | 0xE032 | 0xE04C | 0xE066 => Deferred::BombJump,
        0xD068 | 0xD0AB | 0xD0D7 | 0xD106 | 0xD346 | 0xD383 | 0xD396 | 0xD3CC | 0xD3F3 | 0xD40D
        | 0xD678 | 0xD6CE | 0xD75B => Deferred::Shinespark,
        code @ (0xE94F | 0xE90E) => return Err(Skip::NotRunning(code)),
        code => return Err(Skip::MovementHandler(code)),
    };
    Err(Skip::Deferred(deferred))
}

/// Whether she is in a liquid in `record` with the y radius `radius`, as the speed tables and
/// the jump test it: her bottom boundary below the water's surface (unless its physics are
/// off), or, with no water, below the lava's or acid's. The gravity suit ignores both.
fn submerged(record: &FrameRecord, radius: u16) -> Result<bool, Skip> {
    if word(record, source::EQUIPPED_ITEMS)? & GRAVITY_SUIT != 0 {
        return Ok(false);
    }
    let bottom = word(record, source::SAMUS_Y_POS)?
        .wrapping_add(radius)
        .wrapping_sub(1);
    let above = |surface: u16| surface.wrapping_sub(bottom).cast_signed() < 0;
    let water = word(record, FX_Y_POS)?;
    Ok(if water.cast_signed() >= 0 {
        above(water) && word(record, FX_LIQUID_OPTIONS)? & LIQUID_PHYSICS_OFF == 0
    } else {
        let lava = word(record, LAVA_ACID_Y_POS)?;
        lava.cast_signed() >= 0 && above(lava)
    })
}

/// The fields the speed booster writes in the frame from `from` to `next`, with it equipped.
/// Its dash grows past 2.0 only while she runs with run held ($90:9789): from a dash of 2.0 or
/// more there, the dash and x are its. A dash of any size elsewhere is only a value, which the
/// domain's moves carry. A rise she starts with a dash gets half the dash added to its speed
/// ($90:9919, $90:99A9): the vertical speed of that frame. A jump or a wall jump starts after
/// the frame's movement ($91:EB88), so she doesn't move by the boosted speed until the next
/// frame, which starts from the speed record N+1 holds: y stays the domain's. The space jump
/// starts inside the falling spin's movement ($90:A4C4) and moves her by the boosted speed in
/// the same frame ($90:A50F), so a rise from a falling spin that is still a spin takes y too. A
/// wall jump kicked from a falling spin (a wall jump's movement type next) starts after the
/// movement, which didn't move her by it ($90:9D35 skips $90:90E2).
fn boosted(from: &FrameRecord, next: &FrameRecord) -> Result<FieldSet, Skip> {
    if word(from, source::EQUIPPED_ITEMS)? & SPEED_BOOSTER == 0 {
        return Ok(FieldSet::EMPTY);
    }
    let (before, after) = (state(from)?, state(next)?);
    let mut fields = FieldSet::EMPTY;
    let run_held = word(from, source::JOYPAD1_LASTKEYS)? & word(from, source::BINDING_RUN)? != 0;
    if byte(from, source::SAMUS_MOVEMENT_TYPE)? == RUNNING
        && run_held
        && before.extra_run.pixel() >= 2
    {
        fields = fields.union(Quantity::ExtraRun.set().union(Quantity::X.set()));
    }
    if rise_starts(&before, &after) && (before.extra_run.0 != 0 || after.extra_run.0 != 0) {
        fields = fields.union(Quantity::VerticalSpeed.set());
        let movement = |record| byte(record, source::SAMUS_MOVEMENT_TYPE);
        if movement(from)? == SPIN_JUMPING
            && movement(next)? == SPIN_JUMPING
            && before.vertical == VerticalMotion::Down
        {
            fields = fields.union(Quantity::Y.set());
        }
    }
    Ok(fields)
}

/// Whether the running let-go row stands her (`PadChart`, AS3 and AS4): running with neither
/// side held in `from`, standing in `next`. Whether she carries on or follows the pad is then
/// in her speed mode word alone, which standing halts either way, so no later frame shows it
/// (gap 26).
fn let_go_stands(from: &FrameRecord, next: &FrameRecord) -> Result<bool, Skip> {
    let movement = |record| byte(record, source::SAMUS_MOVEMENT_TYPE);
    Ok(movement(from)? == RUNNING
        && word(from, source::JOYPAD1_LASTKEYS)? & SIDES == 0
        && movement(next)? == STANDING)
}

/// Whether a sloped floor block (not a ceiling's) lies under her path from `from` to `next`:
/// in a column her centre covers between the two records' x, inclusive, and in her bottom's
/// row at `from` or either neighbour. The model's x lies between the two, since its capped dash
/// falls short of the game's the same way, so this covers every x either reaches (gap 31).
fn sloped_floor_under(
    from: &FrameRecord,
    next: &FrameRecord,
    inputs: &Inputs,
) -> Result<bool, Skip> {
    let (start, end) = (
        word(from, source::SAMUS_X_POS)? >> BLOCK_SHIFT,
        word(next, source::SAMUS_X_POS)? >> BLOCK_SHIFT,
    );
    let columns = start.min(end)..=start.max(end);
    let bottom = word(from, source::SAMUS_Y_POS)?
        .wrapping_add(radius(from)?)
        .wrapping_sub(1)
        >> BLOCK_SHIFT;
    let width = usize::from(inputs.width.0);
    let near = |centre: u16| centre.saturating_sub(1)..=centre.saturating_add(1);
    Ok(near(bottom).any(|row| {
        columns.clone().any(|column| {
            usize::from(row)
                .checked_mul(width)
                .and_then(|row_start| row_start.checked_add(usize::from(column)))
                .and_then(|index| inputs.blocks.get(index))
                .is_some_and(|block| matches!(block.kind, BlockKind::Slope(slope) if !slope.y_flip))
        })
    }))
}

/// Whether a jump, a spin's jump or a wall jump's kick starts a rise from `before` to `after`:
/// she rises after it and didn't before, or rises faster than before, which the pull never
/// does.
fn rise_starts(before: &Motion, after: &Motion) -> bool {
    after.vertical == VerticalMotion::Up
        && (before.vertical != VerticalMotion::Up
            || after.vertical_speed.0 > before.vertical_speed.0)
}

fn fixed(wram: &[u8], pixel: WramWord, sub: WramWord) -> Result<Fixed, DecodeError> {
    Ok(Fixed::new(pixel.read(wram)?, sub.read(wram)?))
}

/// A code the State has no value for.
const fn unrepresentable(source: WramWord, value: u16) -> DecodeError {
    DecodeError::Unrepresentable {
        source: source.name,
        value,
    }
}

/// The work RAM words Samus's motion State decodes from.
pub const SOURCES: [WramWord; 14] = [
    source::SAMUS_X_POS,
    source::SAMUS_X_SUBPOS,
    source::SAMUS_Y_POS,
    source::SAMUS_Y_SUBPOS,
    source::SAMUS_X_BASE_SPEED,
    source::SAMUS_X_BASE_SUBSPEED,
    source::SAMUS_X_EXTRA_RUN_SPEED,
    source::SAMUS_X_EXTRA_RUN_SUBSPEED,
    source::SAMUS_HAS_MOMENTUM_FLAG,
    source::SAMUS_X_ACCEL_MODE,
    source::SAMUS_Y_SPEED,
    source::SAMUS_Y_SUBSPEED,
    source::SAMUS_Y_DIR,
    source::SAMUS_BOUNCE_STATE,
];

fn state(record: &FrameRecord) -> Result<Motion, Skip> {
    Ok(decode(&record.wram)?)
}

/// Samus's motion State in `wram`: the decode the Session decoder and this domain share.
///
/// # Errors
///
/// [`DecodeError`] when a word is outside `wram` or holds a code the State has no value for.
pub fn decode(wram: &[u8]) -> Result<Motion, DecodeError> {
    let mode = match source::SAMUS_X_ACCEL_MODE.read(wram)? {
        0 => SpeedMode::Accelerating,
        1 => SpeedMode::TurningAround,
        2 => SpeedMode::Decelerating,
        code => return Err(unrepresentable(source::SAMUS_X_ACCEL_MODE, code)),
    };
    let vertical = match source::SAMUS_Y_DIR.read(wram)? {
        0 => VerticalMotion::None,
        1 => VerticalMotion::Up,
        2 => VerticalMotion::Down,
        code => return Err(unrepresentable(source::SAMUS_Y_DIR, code)),
    };
    // The high byte marks the spring ball, which nothing in the domain reads.
    let [count, _spring] = source::SAMUS_BOUNCE_STATE.read(wram)?.to_le_bytes();
    let bounce = match count {
        0 => Bounce::None,
        1 => Bounce::First,
        2 => Bounce::Second,
        code => {
            return Err(unrepresentable(source::SAMUS_BOUNCE_STATE, u16::from(code)));
        }
    };
    Ok(Motion {
        position: position(wram)?,
        base_speed: fixed(
            wram,
            source::SAMUS_X_BASE_SPEED,
            source::SAMUS_X_BASE_SUBSPEED,
        )?,
        extra_run: fixed(
            wram,
            source::SAMUS_X_EXTRA_RUN_SPEED,
            source::SAMUS_X_EXTRA_RUN_SUBSPEED,
        )?,
        momentum: source::SAMUS_HAS_MOMENTUM_FLAG.read(wram)? != 0,
        mode,
        vertical_speed: fixed(wram, source::SAMUS_Y_SPEED, source::SAMUS_Y_SUBSPEED)?,
        vertical,
        bounce,
    })
}

fn position(wram: &[u8]) -> Result<Position, DecodeError> {
    Ok(Position {
        x: fixed(wram, source::SAMUS_X_POS, source::SAMUS_X_SUBPOS)?,
        y: fixed(wram, source::SAMUS_Y_POS, source::SAMUS_Y_SUBPOS)?,
    })
}

fn radius(record: &FrameRecord) -> Result<u16, Skip> {
    word(record, source::SAMUS_Y_RADIUS)
}

/// A `P.S` value, shown as the spec writes it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ps(pub Fixed);

impl fmt::Debug for Ps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04X}.{:04X}", self.0.pixel(), self.0.sub())
    }
}

/// The declared fields: spec §1.1's State quantities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declared {
    /// `position.x`.
    pub x: Ps,
    /// `position.y`.
    pub y: Ps,
    /// `base_speed`.
    pub base_speed: Ps,
    /// `extra_run`.
    pub extra_run: Ps,
    /// `momentum`.
    pub momentum: bool,
    /// `mode`.
    pub mode: SpeedMode,
    /// `vertical_speed`.
    pub vertical_speed: Ps,
    /// `vertical`.
    pub vertical: VerticalMotion,
    /// `bounce`.
    pub bounce: Bounce,
}

impl From<&Motion> for Declared {
    fn from(state: &Motion) -> Self {
        Self {
            x: Ps(state.position.x),
            y: Ps(state.position.y),
            base_speed: Ps(state.base_speed),
            extra_run: Ps(state.extra_run),
            momentum: state.momentum,
            mode: state.mode,
            vertical_speed: Ps(state.vertical_speed),
            vertical: state.vertical,
            bounce: state.bounce,
        }
    }
}

/// A declared field that a mask names, by its place in [`SamusMotion`]'s `FIELDS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quantity {
    X,
    Y,
    BaseSpeed,
    ExtraRun,
    Momentum,
    Mode,
    VerticalSpeed,
    Vertical,
}

impl Quantity {
    /// The field alone.
    const fn set(self) -> FieldSet {
        FieldSet::single(self.index())
    }

    /// Its place in [`SamusMotion`]'s `FIELDS`.
    const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::BaseSpeed => 2,
            Self::ExtraRun => 3,
            Self::Momentum => 4,
            Self::Mode => 5,
            Self::VerticalSpeed => 6,
            Self::Vertical => 7,
        }
    }

    /// The fields in `quantities`.
    fn all(quantities: &[Self]) -> FieldSet {
        quantities
            .iter()
            .fold(FieldSet::EMPTY, |set, quantity| set.union(quantity.set()))
    }
}

/// Samus's motion domain, with the content the model reads: the pose table, the delay lists,
/// the slope shapes.
#[derive(Clone, Debug)]
pub struct SamusMotion {
    content: Content,
}

impl SamusMotion {
    /// The domain over the ROM `image`.
    ///
    /// # Errors
    ///
    /// [`ContentError`] when the content can't be read.
    pub fn load(image: &[u8]) -> Result<Self, ContentError> {
        Ok(Self {
            content: bundle::content(image.to_vec())?,
        })
    }
}

impl SamusMotion {
    /// Where her moves happen: record N's level and the enemies she can touch.
    fn level<'a>(&'a self, inputs: &'a Inputs) -> SamusWorld<'a> {
        let room = Room {
            blocks: &inputs.blocks,
            width: inputs.width,
            slopes: Some(&self.content.slopes),
        };
        SamusWorld::of(room, inputs.enemies.iter().copied())
    }

    /// Her frame through the model over `level`, as the engine's Tick runs it; `None` when her
    /// pose is outside it.
    fn ran(&self, samus: &mut Samus, inputs: &Inputs, level: &SamusWorld) -> Option<Ran> {
        let content = &self.content;
        // Her current state handler sets her radius and reads the pad's pose request before
        // the movement handler, except while the elevator carries her.
        let requests = if pose::carried(samus, inputs.ride) {
            PoseRequests::default()
        } else {
            pose::set_radius(samus, content);
            pose::input(samus, &inputs.controller, content)
        };
        let reads = Reads {
            controller: &inputs.controller,
            ride: inputs.ride,
            completed: inputs.completed,
            requests,
            content,
        };
        run(samus, reads, level, &Reference::REFERENCE)
    }

    /// Where her motion's frame from record `from` placed the y the camera follows her from
    /// (`Outcome::camera_from`, the samus-motion spec's output to the camera): `Some` where pose
    /// fit or a change of shape moved it off her start y, `None` where it is her start y.
    ///
    /// # Errors
    ///
    /// The reason her motion doesn't run in the frame, or the decode's failure.
    pub fn placement(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<Option<clean_types::Fixed>, Skip> {
        let (mut samus, inputs) = self.decode(from, interval)?;
        let level = self.level(&inputs);
        let ran = self
            .ran(&mut samus, &inputs, &level)
            .ok_or(Skip::Pose(samus.pose.0))?;
        let from = ran.outcome.camera_from;
        Ok((from != ran.before.position.y).then_some(from))
    }
}

/// What the model reads besides her, from record N: the pad, the elevator, the frame count,
/// the level and the enemies she can touch.
pub struct Inputs {
    controller: Controller,
    ride: ElevatorRide,
    completed: u16,
    width: BlockColumns,
    blocks: Vec<Block>,
    enemies: Vec<Obstacle>,
}

impl Inputs {
    /// Record N's inputs: the enemies are the live ones the selection would let her touch,
    /// by the camera as record N has it.
    fn of(wram: &[u8]) -> Result<Self, DecodeError> {
        let rules = Rules::ORIGINAL;
        let (camera_x, camera_y) = (rules.camera_x.read(wram)?, rules.camera_y.read(wram)?);
        let enemies = rules
            .enemies
            .commons(wram)?
            .iter()
            .flatten()
            .filter(|common| game::enemy::touchable(common, camera_x, camera_y))
            .map(Obstacle::from)
            .collect();
        Ok(Self {
            controller: Controller {
                held: buttons(rules.held.read(wram)?),
                pressed: buttons(rules.pressed.read(wram)?),
                ..Controller::default()
            },
            ride: rules.elevator.read(wram)?,
            completed: rules.completed.read(wram)?,
            width: BlockColumns(rules.level.width.read(wram)?),
            blocks: rules.level.layer1(wram)?,
            enemies,
        })
    }
}

/// The catalogue entries the step exercised, which the attribution reads.
#[derive(Default)]
pub struct Exercised(Vec<usize>);

/// Bank $85 log: the message box's bank, whose wait loops ($85:8271, $85:8425) hold the main
/// loop while a box is up.
const MESSAGE_BOX_BANK: u32 = 0x85;
/// Bank $84 and $A2 logs: the returns of the calls of the message box ($85:8080) from the item
/// pick-ups ($84:88EB..$84:8CF6, $84:B029) and the gunship's save ($A2:AB22).
const MESSAGE_BOX_RETURNS: [u32; 15] = [
    0x84_88EF, 0x84_8916, 0x84_893E, 0x84_8965, 0x84_8983, 0x84_89A6, 0x84_89CF, 0x84_89F8,
    0x84_8A21, 0x84_8CA6, 0x84_8CC0, 0x84_8CE1, 0x84_8CFA, 0x84_B02D, 0xA2_AB26,
];

/// Whether the main thread's wait at `pc`, or a return on its chain, is the message box's.
fn in_message_box(pc: u32) -> bool {
    pc >> 16 == MESSAGE_BOX_BANK || MESSAGE_BOX_RETURNS.contains(&pc)
}

/// Whether Samus's motion runs in the frame after `record`: gameplay, and the main thread
/// waiting at the main loop's end, not inside a routine that waits mid-pass.
fn runs_after(record: &FrameRecord) -> Result<(), Skip> {
    let game_state = word(record, source::GAME_STATE)?;
    if game_state != MAIN_GAMEPLAY {
        return Err(Skip::GameState(game_state));
    }
    let Cut::NmiReturn(nmi) = record.cut else {
        return Err(Skip::NotAtFrameEnd);
    };
    match nmi.kind {
        NmiKind::Requested => {}
        NmiKind::Overrun => return Err(Skip::NotAtFrameEnd),
    }
    let main = match Rules::ORIGINAL.strand.main(
        nmi.interrupted_pc,
        record.cpu.s,
        record.cpu.x,
        &record.wram,
    ) {
        Ok(main) => main,
        // The strand's tables don't hold every message box's waits yet: these are its own loops
        // and the returns of the item pick-ups' calls of it. The main loop is held there, so
        // her motion doesn't run, as in the waits the tables do know.
        Err(DecodeError::UnknownYieldSite { pc }) if in_message_box(pc) => {
            return Err(Skip::Waiting("the message box"));
        }
        Err(error) => return Err(error.into()),
    };
    // The innermost routine the main thread waits in, the strand's last frame.
    main.frames
        .iter()
        .rev()
        .find_map(|frame| match frame {
            Frame::MainLoop(_) => None,
            Frame::Gameplay(_) => Some("the gameplay pass"),
            Frame::Plms { .. } => Some("the PLM pass"),
            Frame::MessageBox(_) => Some("the message box"),
        })
        .map_or(Ok(()), |at| Err(Skip::Waiting(at)))
}

impl Domain for SamusMotion {
    const NAME: &'static str = "samus motion";
    type State = Samus;
    type Inputs = Inputs;
    type World = Exercised;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("position.x", x),
        domain_field!("position.y", y),
        domain_field!("base_speed", base_speed),
        domain_field!("extra_run", extra_run),
        domain_field!("momentum", momentum),
        domain_field!("mode", mode),
        domain_field!("vertical_speed", vertical_speed),
        domain_field!("vertical", vertical),
        domain_field!("bounce", bounce),
    ];

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(Samus, Inputs), Skip> {
        runs_after(from)?;
        motion_runs(from)?;
        let samus = Rules::ORIGINAL.samus(&from.wram)?;
        if clean_of(&samus).is_none() {
            return Err(Skip::Pose(samus.pose.0));
        }
        Ok((samus, Inputs::of(&from.wram)?))
    }

    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        inputs: &Inputs,
    ) -> Result<Vec<Mask>, Skip> {
        use Quantity as Q;
        let mut masks = Vec::new();
        let mut mask = |reason, fields| masks.push(Mask { reason, fields });
        if word(from, source::ELEVATOR_STATUS)? != 0 {
            mask(
                "the elevator moves her: another domain writes y",
                Q::Y.set(),
            );
        }
        if pushed(next)? {
            mask(
                "deferred: a block-inside push (quicksand, treadmill)",
                Q::all(&[Q::X, Q::Y, Q::BaseSpeed, Q::ExtraRun, Q::VerticalSpeed]),
            );
        }
        if submerged(from, radius(from)?)? {
            mask("deferred: liquid physics (submerged)", FieldSet::ALL);
        }
        let boost = boosted(from, next)?;
        if boost != FieldSet::EMPTY {
            mask(
                "deferred: the speed booster (its dash past 2.0 running with run held, or a boosted rise's speed)",
                boost,
            );
        }
        // The windup's start runs in the frame's pose change, after the motion, and writes the
        // shinespark's own speeds ($90:CFFA): dash 8.0, base speed 0, vertical speed 0, rising.
        if word(next, SAMUS_MOVEMENT_HANDLER)? == SHINESPARK_WINDUP {
            mask(
                "deferred: the shinespark's start sets her dash, base and vertical speeds",
                Q::all(&[Q::BaseSpeed, Q::ExtraRun, Q::VerticalSpeed, Q::Vertical]),
            );
        }
        // The speed booster's dash moves her further (x, above); on a sloped floor the
        // surface's height at her centre column, which the walk and the ground pull end on,
        // follows that x. Following the ground down (the horizontal move not adjusted by the
        // slope) the pull is |speed| + dash + 1, so the dash reaches y through the pull too;
        // where the slope adjusted the move the pull is 1.0 ($90:924C, $90:925E) and the dash
        // reaches y through x alone. Either way y is the booster's there (gap 31: with the
        // dash's cap lifted, each such frame of the five rows that still differs has x a pixel
        // off, the booster's own growth of the dash).
        if boost.contains(Q::X.index()) && sloped_floor_under(from, next, inputs)? {
            mask(
                "deferred: the speed booster on a sloped floor (its dash sets x, where the slope's surface leaves her, and the pull)",
                Q::Y.set(),
            );
        }
        if let_go_stands(from, next)? {
            mask(
                "unperceived: the running let-go row's speed mode, which standing halts either way",
                Q::Mode.set(),
            );
        }
        // A hit starts knockback in the frame's pose change, after the motion ($90:DDE9,
        // $91:EB88): its pose (53h or 54h, or hers morphed) is taken before the pad's, so the
        // pad's pose-change command and the aim change's step don't run, which leaves x and her
        // speeds as the motion left them; the knockback's taller body is fitted (y), and its
        // start sets her vertical speed upward ($90:99D6). Her health drops in the same frame.
        // The frames after it run the knockback's movement handler or its poses, not this domain.
        if word(next, SAMUS_MOVEMENT_HANDLER)? == KNOCKBACK {
            mask(
                "deferred: knockback's start takes her pose before the pad's (no pose command, no aim step), fits its body and sets her vertical speed",
                Q::all(&[
                    Q::X,
                    Q::Y,
                    Q::BaseSpeed,
                    Q::ExtraRun,
                    Q::Momentum,
                    Q::Mode,
                    Q::VerticalSpeed,
                    Q::Vertical,
                ]),
            );
        }
        Ok(masks)
    }

    fn world(&self, _from: &FrameRecord, _next: &FrameRecord) -> Result<Exercised, Skip> {
        Ok(Exercised::default())
    }

    fn step(&self, samus: &mut Samus, inputs: &Inputs, world: &mut Exercised) {
        let level = self.level(inputs);
        if let Some(ran) = self.ran(samus, inputs, &level) {
            world.0 = exercised(&ran, &level);
        }
    }

    fn behaviours(&self) -> Vec<String> {
        CATALOGUE
            .iter()
            .map(|entry| format!("{:?}", entry.id))
            .collect()
    }

    fn exercised(&self, _: &Samus, _: &Inputs, world: &Exercised) -> Vec<usize> {
        world.0.clone()
    }

    fn fields(&self, samus: &Samus) -> Declared {
        Declared::from(&samus.motion)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&state(next)?))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) => Uncompared::Defect(skip.to_string()),
            Skip::GameState(_)
            | Skip::NotAtFrameEnd
            | Skip::Waiting(_)
            | Skip::Pose(_)
            | Skip::NotRunning(_)
            | Skip::MovementHandler(_)
            | Skip::Deferred(_) => Uncompared::Skip(skip.to_string()),
        }
    }
}

/// Whether `pose` turns her round.
const fn turning(pose: Shape) -> bool {
    matches!(pose.action, Action::Turning(_))
}

/// The catalogue entries a frame exercised, from her before and after the step, its frame and
/// what it showed: each entry whose rule the frame's situation or outcome brings into play.
/// What her moves met is read from her phases and the cells her moves reached: she landed when
/// she was in the air and isn't, and a move met a block when it reached one that isn't air.
fn exercised(ran: &Ran, level: &SamusWorld) -> Vec<usize> {
    use BehaviourId as B;
    let Ran {
        before,
        after,
        outcome,
        ..
    } = ran;
    let airborne = before.vertical != Vertical::Ground;
    let landed = airborne && after.vertical == Vertical::Ground;
    let walked_off = !airborne && after.vertical != Vertical::Ground;
    let met = |ways: &[Heading]| {
        outcome.reached.iter().any(|touch| {
            matches!(touch.way, Way::Move(heading) if ways.contains(&heading))
                && !matches!(level.block(touch.cell), Kind::Air)
        })
    };
    let wall_or_ceiling = met(&[Heading::Left, Heading::Right, Heading::Up]);
    let blocked = wall_or_ceiling || met(&[Heading::Down]);
    let changed = after.pose != before.pose;
    let aimed = after.pose.aim != before.pose.aim;
    let stepped = changed && matches!(after.pose.action, Action::Running | Action::AgainstWall);
    let moonfall = |samus: &Model| samus.vertical == Vertical::Moonfall;
    [
        (B::MovesAsShePosed, true),
        (B::PoseFollows, true),
        (B::PadChart, true),
        (B::SpeedBuildsAndBrakes, !airborne),
        (B::FollowingTheGround, !airborne),
        (B::DriftInTheAir, airborne),
        (B::RisingAndFalling, airborne),
        (B::Dash, before.run.dash != ZERO || after.run.dash != ZERO),
        (B::TurningRound, turning(before.pose) || turning(after.pose)),
        (
            B::Jumping,
            changed && matches!(after.pose.action, Action::TakingOff | Action::Spinning),
        ),
        (B::Landing, landed),
        (B::WhatHerMovesMet, landed || walked_off || wall_or_ceiling),
        (B::WallsAndCeilings, wall_or_ceiling),
        (B::BlocksStopFlush, blocked),
        (B::Aiming, aimed),
        (
            B::AimingAndShooting,
            aimed || stepped || matches!(outcome.standing, Standing::Restarts),
        ),
        (
            B::WallJump,
            outcome.wall_test != WallTest::Neither || after.pose.action == Action::WallJumping,
        ),
        (B::Moonfall, moonfall(before) || moonfall(after)),
        (
            B::PoseFitAfterMoves,
            changed && after.radius.y > before.radius.y,
        ),
        (
            B::ShrinkingLowersHer,
            changed && matches!(after.pose.action, Action::Crouching | Action::Morphing),
        ),
        (B::AnimationEnds, ran.animation_ends),
        (B::RidingTheElevator, ran.elevator_moving),
        (B::SolidEnemies, !level.solid_enemies().is_empty()),
        (
            B::SlopesCarry,
            before.seat == Seat::Seated || after.seat == Seat::Seated,
        ),
        (B::TouchedCells, !outcome.reached.is_empty()),
    ]
    .into_iter()
    .filter(|&(_, exercised)| exercised)
    .filter_map(|(id, _)| {
        CATALOGUE
            .iter()
            .position(|entry| core::mem::discriminant(&entry.id) == core::mem::discriminant(&id))
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{Domain, FieldSet, Quantity, SamusMotion};

    #[test]
    fn a_quantity_names_the_declared_field_at_its_place() {
        use Quantity as Q;
        let named = [
            (Q::X, "position.x"),
            (Q::Y, "position.y"),
            (Q::BaseSpeed, "base_speed"),
            (Q::ExtraRun, "extra_run"),
            (Q::Momentum, "momentum"),
            (Q::Mode, "mode"),
            (Q::VerticalSpeed, "vertical_speed"),
            (Q::Vertical, "vertical"),
        ];
        for (quantity, name) in named {
            let place = SamusMotion::FIELDS
                .iter()
                .position(|field| field.name == name);
            assert_eq!(place.map(FieldSet::single), Some(quantity.set()), "{name}");
        }
    }
}
