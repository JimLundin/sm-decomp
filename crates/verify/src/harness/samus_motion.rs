//! Samus's motion as a harness domain: the samus-motion model (`clean_model_samus_motion`)
//! checked against the oracle, frame by frame, on the spec's declared fields
//! (`docs/behaviours/samus-motion.md` §1.1).
//!
//! Each frame decodes record N: Samus, the pad, the frame count, the elevator, the level's
//! layer 1 and the enemies' common words, all from work RAM, so every room reads, and the
//! room's door list, which the importer reads from the ROM by record N's room. It steps her
//! through the model over that level and the enemies the selection would touch
//! (`game::samus_world`), and compares her motion words with record N+1's, and the step's two
//! outputs to the camera: whether she is rising, and the y a placement put her at, which the
//! record shows through the camera's speed down. Nothing answers from record N+1, so no field is
//! unverified; the null model is her State left as record N has it, rising as she was, and
//! placed nowhere.

use core::cell::RefCell;
use core::fmt;
use std::collections::BTreeMap;

use clean_model_samus_motion::{
    BehaviourId, CATALOGUE, Phase, Rules as Reference, Samus as Model, Seat, StandingAnimation,
    WallTest,
};
use clean_types::{Action, Block as Kind, Heading, Pose as Shape, Way, World, ZERO};
use data::rom::Lorom;
use data::room::{Door, RoomError, RoomKey, decode as decode_room};
use game::Content;
use game::collision::Room;
use game::collision::solid_enemy::Obstacle;
use game::model::num::Fixed;
use game::pose::{self, PoseRequests};
use game::samus_world::state::{clean_of, named};
use game::samus_world::step::{Ran, Reads, run, settle};
use game::samus_world::{SamusFault, SamusWorld};
use game::state::Controller;
use game::state::room::{Block, BlockColumns, ElevatorRide};
use game::state::samus::{Bounce, Motion, Position, Samus, SpeedMode, VerticalMotion};
use game::state::strand::Frame;
use oracle_record::{Cut, FrameRecord, NmiKind};

use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use crate::bundle::{self, ContentError};
use crate::decode::{DecodeError, Rules, buttons};
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// The vertical direction up (`0B36`): she is rising.
const UP: u16 = 1;

/// The equipment bit the liquid mask reads.
const GRAVITY_SUIT: u16 = 0x0020;

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
    /// The adapter refuses the frame: a pose no situation of the model names, or a stand-in
    /// chain that loops (a harness defect: the original ran the frame).
    Fault(SamusFault),
    /// The room record N is in, whose header is at `$8F:` this pointer, doesn't decode, so its
    /// door list can't be read (a harness defect).
    Room(u16, RoomError),
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
    /// The knockback's end: the frame whose knockback time ran out the frame before, on which the
    /// knockback has the last word on her pose, y and vertical motion (spec §10, gap 33).
    KnockbackEnds,
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
            Self::KnockbackEnds => {
                "the knockback's end (its time run out, the knockback on under the normal handler)"
            }
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
            Self::Fault(fault) => write!(f, "the adapter refuses the frame: {fault:?}"),
            Self::Room(pointer, error) => {
                write!(f, "room {pointer:#06X} doesn't decode: {error:?}")
            }
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
/// The running movement type, which the running let-go row leaves.
const RUNNING: u8 = 0x01;
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

/// Bank $90 log, `$90:96FF`: the camera's speed down, made from where it last saw her (`0B14`,
/// a placement's pixel where one put her) as the scroll reads her y.
const CAMERA_Y_SPEED: WramWord = WramWord {
    name: "CAMERA_Y_SPEED",
    offset: 0x0DA6,
};
const CAMERA_Y_SUBSPEED: WramWord = WramWord {
    name: "CAMERA_Y_SUBSPEED",
    offset: 0x0DA8,
};
/// The extra pixel the camera's speed adds to her move.
const ONE_PIXEL: u32 = 0x0001_0000;

/// Whether frame `next` (its record: the pushes stay until the next frame's start) ran a
/// block-inside push. The camera's harness masks y on those of them that place her (camera C2).
///
/// # Errors
///
/// The [`DecodeError`] of a word the record doesn't hold.
pub fn pushed(next: &FrameRecord) -> Result<bool, DecodeError> {
    let words = [
        EXTRA_X_DISPLACEMENT,
        EXTRA_X_SUBDISPLACEMENT,
        EXTRA_Y_DISPLACEMENT,
        EXTRA_Y_SUBDISPLACEMENT,
        IN_QUICKSAND,
    ];
    for source in words {
        if source.read(&next.wram)? != 0 {
            return Ok(true);
        }
    }
    Ok(X_DECEL_MULTIPLIER.read(&next.wram)? != 0)
}

/// Bank $90 log: the knockback's direction, 0 when she takes none.
const KNOCKBACK_DIRECTION: WramWord = WramWord {
    name: "KNOCKBACK_DIR",
    offset: 0x0A52,
};
/// Bank $A0 log: the knockback's time left, counted down at the frame's end (`$A0:9169`).
const KNOCKBACK_TIMER: WramWord = WramWord {
    name: "KNOCKBACK_TIMER",
    offset: 0x18AA,
};
/// Bank $90 log: the normal movement handler, the one this domain's model runs.
const NORMAL_MOVEMENT: u16 = 0xA337;

/// Whether the frame from `from` is the knockback's end.
///
/// It starts with the knockback on under the normal movement handler and its time run out, so
/// the hit's pose change after her motion ends the knockback (`$90:DDE9`, then `$91:F31D`): it
/// keeps her pose and hands her back falling at 0, whatever her motion did (samus-motion decode
/// C3). This harness defers it (gap 33), and the camera's masks y on those of them that place her
/// (camera C3). The tail's frames before it run her own motion, and are compared.
///
/// # Errors
///
/// The [`DecodeError`] of a word the record doesn't hold.
pub fn knockback_ends(from: &FrameRecord) -> Result<bool, DecodeError> {
    Ok(KNOCKBACK_DIRECTION.read(&from.wram)? != 0
        && KNOCKBACK_TIMER.read(&from.wram)? == 0
        && SAMUS_MOVEMENT_HANDLER.read(&from.wram)? == NORMAL_MOVEMENT)
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
        NORMAL_MOVEMENT if knockback_ends(record)? => Deferred::KnockbackEnds,
        NORMAL_MOVEMENT => return Ok(()),
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

/// Her State as the harness steps it, with the two outputs the camera reads of her frame.
#[derive(Clone, Debug)]
pub struct Hers {
    /// Her State; after the step, where the camera last saw her down holds the pixel a
    /// placement put her at, as the engine's `settle` writes it.
    samus: Samus,
    /// Whether she is rising: as record N has it (`0B36` = 1), then the step's `rising`.
    rising: bool,
}

impl Hers {
    /// The camera's speed down after her frame, as the scroll works it out from her y and where
    /// it last saw her, a placement's pixel included (camera `FollowingHer`): her move plus a
    /// pixel, its way read from the pixel parts. The scroll works it out every frame, also one
    /// on which her pixel part stays where it last saw her.
    const fn speed_down(&self) -> Fixed {
        let her = self.samus.motion.position.y;
        let seen = self.samus.previous_position.y;
        let seen = Fixed::new(seen.pixels, seen.subpixels);
        let moved = if clean_types::below(her.pixel(), seen.pixel()) {
            seen.0.wrapping_sub(her.0)
        } else {
            her.0.wrapping_sub(seen.0)
        };
        Fixed(moved.wrapping_add(ONE_PIXEL))
    }
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
    /// `rising`.
    pub rising: bool,
    /// `placed`, as the record shows it: the camera's speed down, measured from the y a
    /// placement put her at, or, with none, from where the camera last saw her.
    pub placed: Ps,
}

impl Declared {
    /// The declared fields of `state`, rising or not, with the camera's speed down `placed`
    /// makes.
    const fn of(state: &Motion, rising: bool, placed: Fixed) -> Self {
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
            rising,
            placed: Ps(placed),
        }
    }
}

/// The outputs a mask of `fields` reaches as well: the camera's speed down is made from her y,
/// and whether she is rising is her vertical direction up.
fn follows(fields: FieldSet) -> FieldSet {
    use Quantity as Q;
    [(Q::Y, Q::Placed), (Q::Vertical, Q::Rising)]
        .into_iter()
        .filter(|(by, _)| fields.contains(by.index()))
        .fold(FieldSet::EMPTY, |set, (_, output)| set.union(output.set()))
}

/// The declared fields a catalogue entry's output reaches. Her pose, what she is drawn as, the
/// standing animation and the cells she touched aren't declared fields, so an entry whose output
/// is only those (the pose's choice, its chart, its aim, an animation's end) is credited nothing.
fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    use Quantity as Q;
    let horizontal = [Q::X, Q::BaseSpeed, Q::ExtraRun, Q::Momentum, Q::Mode];
    let vertical = [Q::Y, Q::VerticalSpeed, Q::Vertical, Q::Rising];
    match id {
        B::HowSheIsDrawn
        | B::PoseFollows
        | B::PadChart
        | B::Aiming
        | B::WhatHerMovesMet
        | B::AnimationEnds
        | B::TouchedCells => FieldSet::EMPTY,
        B::SpeedBuildsAndBrakes | B::DriftInTheAir => {
            Q::all(&[Q::X, Q::BaseSpeed, Q::Momentum, Q::Mode])
        }
        B::Dash => Q::all(&[Q::X, Q::ExtraRun]),
        B::TurningRound => Q::all(&horizontal),
        B::AimingAndShooting | B::SlopesSlowTheWalk => Q::X.set(),
        B::RidingTheElevator => Q::Y.set(),
        B::ShrinkingLowersHer => Q::all(&[Q::Y, Q::Placed]),
        B::Jumping | B::FollowingTheGround | B::RisingAndFalling | B::Moonfall => Q::all(&vertical),
        B::Landing => Q::all(&[Q::Y, Q::VerticalSpeed, Q::Vertical, Q::Bounce, Q::Rising]),
        B::MovesAsShePosed
        | B::BlocksStopFlush
        | B::SolidEnemies
        | B::SlopesCarry
        | B::OtherBlockKinds => Q::all(&[Q::X, Q::Y]),
        B::WallsAndCeilings => Q::all(&[Q::X, Q::Y, Q::BaseSpeed, Q::VerticalSpeed, Q::Rising]),
        B::WordwiseTopSpeed => Q::BaseSpeed.set(),
        B::WallJump => Q::all(&[
            Q::X,
            Q::Y,
            Q::BaseSpeed,
            Q::ExtraRun,
            Q::Momentum,
            Q::Mode,
            Q::VerticalSpeed,
            Q::Vertical,
            Q::Rising,
        ]),
        B::PoseFitAfterMoves => Q::all(&[
            Q::X,
            Q::Y,
            Q::BaseSpeed,
            Q::ExtraRun,
            Q::Momentum,
            Q::Mode,
            Q::VerticalSpeed,
            Q::Vertical,
            Q::Rising,
            Q::Placed,
        ]),
    }
}

/// A declared field that a mask or an entry names, by its place in [`SamusMotion`]'s `FIELDS`.
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
    Bounce,
    Rising,
    Placed,
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
            Self::Bounce => 8,
            Self::Rising => 9,
            Self::Placed => 10,
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
/// the slope shapes, and each room's door list.
#[derive(Clone, Debug)]
pub struct SamusMotion {
    content: Content,
    /// The ROM, whose rooms' door lists the frames read.
    image: Vec<u8>,
    /// Each room's door list, by its header's pointer, decoded once.
    doors: RefCell<BTreeMap<u16, Vec<Door>>>,
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
            image: image.to_vec(),
            doors: RefCell::default(),
        })
    }

    /// The door list of the room whose header is at `$8F:pointer`.
    fn doors(&self, pointer: u16) -> Result<Vec<Door>, Skip> {
        if let Some(doors) = self.doors.borrow().get(&pointer) {
            return Ok(doors.clone());
        }
        let room = decode_room(Lorom::new(&self.image), RoomKey::from_room_pointer(pointer))
            .map_err(|error| Skip::Room(pointer, error))?;
        let doors = room.doors.doors;
        self.doors.borrow_mut().insert(pointer, doors.clone());
        Ok(doors)
    }
}

impl SamusMotion {
    /// Where her moves happen: record N's level and the enemies she can touch.
    fn level<'a>(&'a self, inputs: &'a Inputs) -> SamusWorld<'a> {
        let room = Room {
            blocks: &inputs.blocks,
            width: inputs.width,
            slopes: Some(&self.content.slopes),
            doors: &inputs.doors,
        };
        SamusWorld::of(room, inputs.enemies.iter().copied())
    }

    /// Her frame through the model over `level`, as the engine's Tick runs it; the skip when her
    /// pose is outside it or the adapter refuses the frame.
    fn ran(&self, samus: &mut Samus, inputs: &Inputs, level: &SamusWorld) -> Result<Ran, Skip> {
        let pose = samus.pose.0;
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
            .map_err(Skip::Fault)?
            .ok_or(Skip::Pose(pose))
    }

    /// The y her motion's frame from record `from` placed her at, if any (`Outcome::placed`,
    /// the samus-motion spec's output to the camera): where pose fit moved her or a change of
    /// shape started, a placement of 0 px included.
    ///
    /// # Errors
    ///
    /// The reason her motion doesn't run in the frame, or the decode's failure.
    pub fn placement(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<Option<clean_types::Fixed>, Skip> {
        let (mut hers, inputs) = self.decode(from, interval)?;
        let level = self.level(&inputs);
        let ran = self.ran(&mut hers.samus, &inputs, &level)?;
        Ok(ran.outcome.placed)
    }
}

/// What the model reads besides her, from record N: the pad, the elevator, the frame count,
/// the level, its room's door list and the enemies she can touch.
pub struct Inputs {
    controller: Controller,
    ride: ElevatorRide,
    completed: u16,
    width: BlockColumns,
    blocks: Vec<Block>,
    doors: Vec<Door>,
    enemies: Vec<Obstacle>,
}

impl Inputs {
    /// Record N's inputs, in a room of `doors`: the enemies are the live ones the selection
    /// would let her touch, by the camera as record N has it.
    fn of(wram: &[u8], doors: Vec<Door>) -> Result<Self, DecodeError> {
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
            doors,
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
    type State = Hers;
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
        domain_field!("rising", rising),
        domain_field!("placed", placed),
    ];

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(Hers, Inputs), Skip> {
        runs_after(from)?;
        motion_runs(from)?;
        let samus = Rules::ORIGINAL.samus(&from.wram)?;
        let Some(model) = clean_of(&samus) else {
            return Err(Skip::Pose(samus.pose.0));
        };
        if !named(model.pose) {
            return Err(Skip::Fault(SamusFault::UnnamedPose(samus.pose)));
        }
        let hers = Hers {
            samus,
            rising: word(from, source::SAMUS_Y_DIR)? == UP,
        };
        let doors = self.doors(word(from, source::ROOM_PTR)?)?;
        Ok((hers, Inputs::of(&from.wram, doors)?))
    }

    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        _inputs: &Inputs,
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
        // The windup's start runs in the frame's pose change, after the motion, and writes the
        // shinespark's own speeds ($90:CFFA): dash 8.0, base speed 0, vertical speed 0, rising.
        if word(next, SAMUS_MOVEMENT_HANDLER)? == SHINESPARK_WINDUP {
            mask(
                "deferred: the shinespark's start sets her dash, base and vertical speeds",
                Q::all(&[Q::BaseSpeed, Q::ExtraRun, Q::VerticalSpeed, Q::Vertical]),
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
        // The camera's speed down is made from her y at the scroll, so where another domain
        // writes y the speed is that domain's too (below). Record N+1 shows the scroll's speed
        // only once the frame has reached it: an item's message box holds the frame between her
        // motion and the scroll. Once the scroll has run, something that moves her after it
        // leaves her y in record N+1 off the one the scroll read; before it, that y is only her
        // motion's, not yet read.
        let scrolled = !matches!(
            runs_after(next),
            Err(Skip::Waiting(_) | Skip::NotAtFrameEnd)
        );
        if !scrolled {
            mask(
                "the scroll hasn't run: record N+1's main thread waits before it (the message box) or is mid-frame",
                Q::Placed.set(),
            );
        }
        if word(from, source::SLOW_GRAPPLE_SCROLLING)? != 0
            || word(next, source::SLOW_GRAPPLE_SCROLLING)? != 0
        {
            mask(
                "the slow grapple scroll: the scroll leaves the camera's speed down as it was ($90:94F2)",
                Q::Placed.set(),
            );
        }
        if scrolled
            && fixed(&next.wram, source::SAMUS_Y_POS, source::SAMUS_Y_SUBPOS)?
                != fixed(
                    &next.wram,
                    source::SAMUS_PREV_Y_POS,
                    source::SAMUS_PREV_Y_SUBPOS,
                )?
        {
            mask(
                "another domain moves her after the scroll (a phase the gameplay pass runs after it): the camera's speed down is from the y it read",
                Q::Placed.set(),
            );
        }
        Ok(masks
            .into_iter()
            .map(|mask| Mask {
                fields: mask.fields.union(follows(mask.fields)),
                ..mask
            })
            .collect())
    }

    fn world(&self, _from: &FrameRecord, _next: &FrameRecord) -> Result<Exercised, Skip> {
        Ok(Exercised::default())
    }

    fn step(&self, hers: &mut Hers, inputs: &Inputs, world: &mut Exercised) {
        let level = self.level(inputs);
        if let Ok(ran) = self.ran(&mut hers.samus, inputs, &level) {
            settle(&mut hers.samus, &ran);
            hers.rising = ran.outcome.rising;
            world.0 = exercised(&ran, &level);
        }
    }

    fn behaviours(&self) -> Vec<Entry> {
        CATALOGUE
            .iter()
            .map(|entry| Entry {
                name: format!("{:?}", entry.id),
                fields: shown_by(entry.id),
            })
            .collect()
    }

    fn exercised(&self, _: &Hers, _: &Inputs, world: &Exercised) -> Vec<usize> {
        world.0.clone()
    }

    fn fields(&self, hers: &Hers) -> Declared {
        Declared::of(&hers.samus.motion, hers.rising, hers.speed_down())
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::of(
            &state(next)?,
            word(next, source::SAMUS_Y_DIR)? == UP,
            fixed(&next.wram, CAMERA_Y_SPEED, CAMERA_Y_SUBSPEED)?,
        ))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Unreadable(_) | Skip::Fault(_) | Skip::Room(..) => {
                Uncompared::Defect(skip.to_string())
            }
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
    let airborne = before.vertical.phase != Phase::Ground;
    let landed = airborne && after.vertical.phase == Phase::Ground;
    let walked_off = !airborne && after.vertical.phase != Phase::Ground;
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
    let moonfall = |samus: &Model| samus.vertical.phase == Phase::Moonfall;
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
            aimed || stepped || matches!(outcome.standing_animation, StandingAnimation::Restarts),
        ),
        (
            B::WallJump,
            outcome.wall_test != WallTest::Neither || after.pose.action == Action::WallJumping,
        ),
        (B::Moonfall, moonfall(before) || moonfall(after)),
        (
            B::PoseFitAfterMoves,
            changed && ran.body_after.y > ran.body_before.y,
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

pub mod cases;

#[cfg(test)]
mod tests {
    use super::{Domain, FieldSet, Inputs, Quantity, SamusMotion};
    use crate::source::{self, WramWord};
    use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

    /// A record of work RAM alone, every word 0 but `words`; no liquid in the room.
    fn record(index: u64, words: &[(WramWord, u16)]) -> FrameRecord {
        let mut wram = vec![0_u8; 0x2_0000];
        let no_liquid = [(super::FX_Y_POS, 0xFFFF), (super::LAVA_ACID_Y_POS, 0xFFFF)];
        for &(word, value) in no_liquid.iter().chain(words) {
            let at = usize::from(word.offset);
            if let Some(bytes) = wram.get_mut(at..at + 2) {
                bytes.copy_from_slice(&value.to_le_bytes());
            }
        }
        FrameRecord {
            index,
            video_frame: 0,
            pad: Pad(0),
            cut: Cut::NmiReturn(Nmi {
                kind: NmiKind::Requested,
                interrupted_pc: 0,
                polled: true,
            }),
            joypad_reads: 0,
            wait_entry: None,
            overscan: false,
            cpu: CpuRegisters::default(),
            wram: wram.into_boxed_slice(),
            vram: Box::default(),
            cgram: Box::default(),
            oam: Box::default(),
            sram: Box::default(),
            ppu: PpuRegisters::default(),
            apu_log: Vec::new(),
            picture: None,
            spc: None,
            bank_instructions: None,
            writes: Vec::new(),
            hits: Vec::new(),
            legacy_ppu_digest: 0,
        }
    }

    /// The fields the masks of the frame from `from` to `next` take.
    fn masked(from: &FrameRecord, next: &FrameRecord) -> FieldSet {
        let domain = SamusMotion {
            content: game::Content::default(),
            image: Vec::new(),
            doors: core::cell::RefCell::default(),
        };
        let inputs = Inputs {
            controller: game::state::Controller::default(),
            ride: game::state::room::ElevatorRide::default(),
            completed: 0,
            width: game::state::room::BlockColumns(0),
            blocks: Vec::new(),
            doors: Vec::new(),
            enemies: Vec::new(),
        };
        match domain.masks(from, next, &inputs) {
            Ok(masks) => masks
                .iter()
                .fold(FieldSet::EMPTY, |set, mask| set.union(mask.fields)),
            Err(skip) => panic!("masks: {skip}"),
        }
    }

    #[test]
    fn the_slow_grapple_scroll_masks_the_placement_alone() {
        let on = [(source::SLOW_GRAPPLE_SCROLLING, 1)];
        assert_eq!(masked(&record(0, &[]), &record(1, &[])), FieldSet::EMPTY);
        assert_eq!(
            masked(&record(0, &on), &record(1, &[])),
            Quantity::Placed.set()
        );
        assert_eq!(
            masked(&record(0, &[]), &record(1, &on)),
            Quantity::Placed.set()
        );
    }

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
            (Q::Bounce, "bounce"),
            (Q::Rising, "rising"),
            (Q::Placed, "placed"),
        ];
        for (quantity, name) in named {
            let place = SamusMotion::FIELDS
                .iter()
                .position(|field| field.name == name);
            assert_eq!(place.map(FieldSet::single), Some(quantity.set()), "{name}");
        }
    }
}
