//! Samus's motion as a harness domain: the samus-movement model (`samus_movement`)
//! checked against the oracle, frame by frame, on the spec's declared fields
//! (`docs/behaviours/samus-movement.md` §1.1).
//!
//! Each frame decodes record N: Samus, the pad, the frame count, the elevator, the level's
//! layer 1 and the enemies' common words, all from work RAM, so every room reads, and the
//! room's door list, which the importer reads from the ROM by record N's room. It steps her
//! through the model over that level and the enemies the selection would touch
//! (`game::samus_movement_bridge`), and compares her motion words with record N+1's, dashing among them,
//! the step's two outputs to the camera: whether she is rising, and the y a placement put her at,
//! which the record shows through the camera's speed down, and three of its outputs to her
//! animation (§1.4): whether she is heading down and her vertical speed as her moves leave her,
//! which record N+1 shows where the pose change writes neither, and whether her animation's end
//! handed over her pose. Nothing answers from record N+1, so no field is unverified; the null
//! model is her State left as record N has it, rising, heading down, her speed after her moves
//! and handed over as record N shows them, and placed nowhere.
//!
//! After her moves the step runs the frame's pose change and her animation as the engine's
//! Tick does (`samus_movement_bridge::step::next`, `pose::change`,
//! `samus_appearance_bridge::animate`), and takes where the camera last saw her from record N
//! (the scroll writes it), so a free run carries her pose, facing, radius and clip into the
//! next frame. Her pose isn't a declared field yet: its writers outside this domain, knockback's
//! start among them, have no masks for it.

use core::cell::RefCell;
use core::fmt;
use std::collections::BTreeMap;

use data::assets::world::Door;
use data::id::DoorwayId;
use data::rom::Lorom;
use data::room::{self, RoomError, RoomKey, decode as decode_room};
use game::Content;
use game::collision::Room;
use game::collision::solid_enemy::Obstacle;
use game::model::num::Fixed;
use game::pose;
use game::samus_appearance_bridge::{self, Start};
use game::samus_movement_bridge::state::{clean_of, named};
use game::samus_movement_bridge::step::{Ran, Reads, Stepped, next, run, settle};
use game::samus_movement_bridge::{SamusFault, SamusWorld};
use game::state::Controller;
use game::state::room::{Block, BlockColumns, ElevatorRide};
use game::state::samus::{
    Bounce, Dashing, Motion, Point, PoseId, Position, Samus, SpeedMode, VerticalMotion,
};
use game::state::strand::Frame;
use oracle_record::{Cut, FrameRecord, NmiKind};
use samus_movement::{
    BehaviourId, CATALOGUE, Dashing as Dashes, Outcome, Phase, Rules as Reference, Samus as Model,
    Seat, StandingAnimation, WallTest,
};
use types::{Action, Block as Kind, Heading, Pose as Shape, Way, World, ZERO};

use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::bundle::{self, ContentError};
use crate::decode::{DecodeError, Rules, buttons};
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// The vertical direction up (`0B36`): she is rising.
const UP: u16 = 1;
/// The vertical direction down (`0B36`): she is heading down.
const DOWN: u16 = 2;
/// No vertical direction (`0B36`).
const NO_DIRECTION: u16 = 0;

/// Bank $91 log, `$91:E8B6`: the pose change command the pad's lookup leaves, and the one her
/// moves' vertical result (a landing, a ceiling, a walk off a ledge, a bounce) writes over it,
/// which writes her vertical direction after her moves.
const POSE_CHANGE_COMMAND: WramWord = WramWord {
    name: "SAMUS_POSE_CHANGE_COMMAND",
    offset: 0x0A2E,
};
const VERTICAL_RESULT: u16 = 5;
/// Bank $91 log, `$91:F36E`: the super special command "transition animation finished", and
/// the bit of its pose that says none is asked.
const ANIMATION_FINISHED: u16 = 3;
const NO_POSE: u16 = 0x8000;

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
/// The spinning movement type: the spin jump, the space jump and the screw attack.
const SPINNING: u8 = 0x03;
/// Bank $90 log, `$90:9A7E`: her speed mode (`0B4A`) skidding, and slowing to a halt.
const SKIDDING: u16 = 1;
const HALTING: u16 = 2;
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
/// keeps her pose and hands her back falling at 0, whatever her motion did (samus-movement decode
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

/// What record N+1 doesn't show of her vertical motion as her moves leave her, on the frame from
/// `from` to `next`: the frame's pose change writes her vertical direction and speed after her
/// moves, for what they met (a landing, a ceiling, a walk off a ledge, a bounce) or for a rise it
/// starts. A rise her moves start (a space jump) keeps her pose, and a rise from no direction
/// hides only her speed: she isn't heading down either way. A walk off a ledge from no direction
/// at 0 hides only her heading: the falling result writes speed 0 over the 0 her ground moves
/// leave (`$90:923F` writes no speed), so record N+1 shows it.
fn after_moves_unseen(from: &FrameRecord, next: &FrameRecord) -> Result<FieldSet, Skip> {
    use Quantity as Q;
    let direction = [
        word(from, source::SAMUS_Y_DIR)?,
        word(next, source::SAMUS_Y_DIR)?,
    ];
    let met = word(next, POSE_CHANGE_COMMAND)? == VERTICAL_RESULT;
    let walks_off_at_rest = direction == [NO_DIRECTION, DOWN] && signed_speed(from)?.0 == 0;
    let rise_starts = direction[0] != UP
        && direction[1] == UP
        && word(from, source::SAMUS_POSE)? != word(next, source::SAMUS_POSE)?;
    Ok(if met && walks_off_at_rest {
        Q::HeadingDown.set()
    } else if met || rise_starts && direction[0] == DOWN {
        Q::all(&[Q::HeadingDown, Q::SpeedAfterMoves])
    } else if rise_starts {
        Q::SpeedAfterMoves.set()
    } else {
        FieldSet::EMPTY
    })
}

/// Her signed vertical speed in `record`, downward positive: its magnitude, downward with
/// direction down and upward otherwise, as the State reads it.
fn signed_speed(record: &FrameRecord) -> Result<Fixed, Skip> {
    let magnitude = fixed(
        &record.wram,
        source::SAMUS_Y_SPEED,
        source::SAMUS_Y_SUBSPEED,
    )?;
    Ok(if word(record, source::SAMUS_Y_DIR)? == DOWN {
        magnitude
    } else {
        Fixed(magnitude.0.wrapping_neg())
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

/// The mask of her speed mode where braking ended a spin's skid and the side she faces, held,
/// then halted her: spinning and skidding in `from`, spinning and halting at no speed in `next`.
/// Braking below zero ends the skid at speed 0 (`$90:9B1F`) and the spin's frame carries on with
/// the pad, so the halt is in her speed mode word alone, which no later frame shows: a spin
/// decides it afresh each frame, every pose a spin changes into sets it again, and the scroll
/// reads only a skid. No writer stands behind it, so a free run carries on.
fn a_spins_skid_that_braking_ends(
    from: &FrameRecord,
    next: &FrameRecord,
) -> Result<Option<Mask>, Skip> {
    let spinning = |record| Ok::<_, Skip>(byte(record, source::SAMUS_MOVEMENT_TYPE)? == SPINNING);
    let mode = |record| word(record, source::SAMUS_X_ACCEL_MODE);
    let halted = spinning(from)?
        && mode(from)? == SKIDDING
        && spinning(next)?
        && mode(next)? == HALTING
        && fixed(
            &next.wram,
            source::SAMUS_X_BASE_SPEED,
            source::SAMUS_X_BASE_SUBSPEED,
        )? == Fixed(0);
    Ok(halted.then_some(Mask {
        reason: "unperceived: a spin's skid that braking ends, halted by the side she faces held, which its next frame decides afresh",
        fields: Quantity::Mode.set(),
        hides: Hides::NoWriter,
    }))
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
        dashing: if source::SAMUS_HAS_MOMENTUM_FLAG.read(wram)? == 0 {
            Dashing::Off
        } else {
            Dashing::On
        },
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
    /// Whether she is heading down: as record N has it (`0B36` = 2), then the step's
    /// `heading_down`.
    heading_down: bool,
    /// Her vertical speed as her moves leave her, downward positive: as record N has it, then
    /// the step's `vertical_speed_after_moves`.
    speed_after_moves: Fixed,
    /// Whether her pose is the one her animation's end handed over: as record N has it, then
    /// the step's `handed_over`.
    handed_over: bool,
}

/// Whether her pose in `record` is the one her animation's end handed over: the super special
/// command "transition animation finished", with a pose.
fn handed_over(record: &FrameRecord) -> Result<bool, Skip> {
    Ok(
        word(record, source::SAMUS_HURT_SWITCH_INDEX)? == ANIMATION_FINISHED
            && word(record, source::SAMUS_NEW_POSE_TRANSITIONAL)? & NO_POSE == 0,
    )
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
        let moved = if types::below(her.pixel(), seen.pixel()) {
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
    /// `dashing`.
    pub dashing: Dashing,
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
    /// `heading_down`.
    pub heading_down: bool,
    /// `vertical_speed_after_moves`, downward positive.
    pub speed_after_moves: Ps,
    /// `handed_over`.
    pub handed_over: bool,
}

/// The outputs the step gives besides her State, or the record shows of them.
#[derive(Clone, Copy, Debug)]
struct Shown {
    rising: bool,
    placed: Fixed,
    heading_down: bool,
    speed_after_moves: Fixed,
    handed_over: bool,
}

impl Declared {
    /// The declared fields of `state`, with the outputs `shown`.
    const fn of(state: &Motion, shown: Shown) -> Self {
        Self {
            x: Ps(state.position.x),
            y: Ps(state.position.y),
            base_speed: Ps(state.base_speed),
            extra_run: Ps(state.extra_run),
            dashing: state.dashing,
            mode: state.mode,
            vertical_speed: Ps(state.vertical_speed),
            vertical: state.vertical,
            bounce: state.bounce,
            rising: shown.rising,
            placed: Ps(shown.placed),
            heading_down: shown.heading_down,
            speed_after_moves: Ps(shown.speed_after_moves),
            handed_over: shown.handed_over,
        }
    }
}

/// The outputs a mask of `fields` reaches as well: the camera's speed down is made from her y,
/// and whether she is rising or heading down, and the sign of her speed after her moves, is her
/// vertical direction. A mask of her frame-end vertical speed doesn't reach her speed after her
/// moves: some of its writers act after her moves, so that speed's own writers mask it.
fn follows(fields: FieldSet) -> FieldSet {
    use Quantity as Q;
    [
        (Q::Y, Q::Placed),
        (Q::Vertical, Q::Rising),
        (Q::Vertical, Q::HeadingDown),
        (Q::Vertical, Q::SpeedAfterMoves),
    ]
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
    let horizontal = [Q::X, Q::BaseSpeed, Q::ExtraRun, Q::Dashing, Q::Mode];
    let vertical = [Q::Y, Q::VerticalSpeed, Q::Vertical, Q::Rising];
    match id {
        B::HowSheIsDrawn
        | B::PoseFollows
        | B::PadChart
        | B::Aiming
        | B::WhatHerMovesMet
        | B::TouchedCells => FieldSet::EMPTY,
        B::AnimationEnds => Q::HandedOver.set(),
        B::Dashing => Q::Dashing.set(),
        B::SpeedBuildsAndBrakes | B::DriftInTheAir => {
            Q::all(&[Q::X, Q::BaseSpeed, Q::Dashing, Q::Mode])
        }
        B::Dash => Q::all(&[Q::X, Q::ExtraRun]),
        B::TurningRound => Q::all(&horizontal),
        B::AimingAndShooting | B::SlopesSlowTheWalk => Q::X.set(),
        B::RidingTheElevator => Q::Y.set(),
        B::ShrinkingLowersHer => Q::all(&[Q::Y, Q::Placed]),
        B::Jumping | B::FollowingTheGround | B::Moonfall => Q::all(&vertical),
        B::RisingAndFalling => {
            Q::all(&vertical).union(Q::all(&[Q::HeadingDown, Q::SpeedAfterMoves]))
        }
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
            Q::Dashing,
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
            Q::Dashing,
            Q::Mode,
            Q::VerticalSpeed,
            Q::Vertical,
            Q::Rising,
            Q::Placed,
        ]),
    }
}

/// A declared field that a mask or an entry names, by its place in [`Movement`]'s `FIELDS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quantity {
    X,
    Y,
    BaseSpeed,
    ExtraRun,
    Dashing,
    Mode,
    VerticalSpeed,
    Vertical,
    Bounce,
    Rising,
    Placed,
    HeadingDown,
    SpeedAfterMoves,
    HandedOver,
}

impl Quantity {
    /// The field alone.
    const fn set(self) -> FieldSet {
        FieldSet::single(self.index())
    }

    /// Its place in [`Movement`]'s `FIELDS`.
    const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::BaseSpeed => 2,
            Self::ExtraRun => 3,
            Self::Dashing => 4,
            Self::Mode => 5,
            Self::VerticalSpeed => 6,
            Self::Vertical => 7,
            Self::Bounce => 8,
            Self::Rising => 9,
            Self::Placed => 10,
            Self::HeadingDown => 11,
            Self::SpeedAfterMoves => 12,
            Self::HandedOver => 13,
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
pub struct Movement {
    content: Content,
    /// The ROM, whose rooms' door lists the frames read.
    image: Vec<u8>,
    /// Each room's door list, by its header's pointer, decoded once.
    doors: RefCell<BTreeMap<u16, Vec<Door>>>,
}

/// The stand-in for the doorway of a door that leads to a room: her moves read only whether a
/// door leads to a room, never through which doorway, and the rows cross rooms the registry
/// doesn't hold yet.
const ANY_DOORWAY: DoorwayId = DoorwayId(u16::MAX);

/// The engine's door for the decoder's `door`. A door whose header is in work RAM leads
/// nowhere the engine can follow, so it is solid, as one that leads nowhere.
const fn engine_door(door: room::Door) -> Door {
    match door {
        room::Door::ToRoom(_) => Door::To(ANY_DOORWAY),
        room::Door::Nowhere | room::Door::HeaderInRam(_) => Door::Nowhere,
    }
}

impl Movement {
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
        let doors: Vec<Door> = room.doors.doors.into_iter().map(engine_door).collect();
        self.doors.borrow_mut().insert(pointer, doors.clone());
        Ok(doors)
    }
}

impl Movement {
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
        // Her current state handler sets her radius before the movement handler, except while
        // the elevator carries her.
        if !pose::carried(samus, inputs.ride) {
            pose::set_radius(samus, content);
        }
        let reads = Reads {
            controller: &inputs.controller,
            ride: inputs.ride,
            completed: inputs.completed,
            content,
        };
        run(samus, reads, level, &Reference::default())
            .map_err(Skip::Fault)?
            .ok_or(Skip::Pose(pose))
    }

    /// The y her motion's frame from record `from` placed her at, if any (`Outcome::placed`,
    /// the samus-movement spec's output to the camera): where pose fit moved her or a change of
    /// shape started, a placement of 0 px included.
    ///
    /// # Errors
    ///
    /// The reason her motion doesn't run in the frame, or the decode's failure.
    pub fn placement(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<Option<types::Fixed>, Skip> {
        let (mut hers, inputs) = self.decode(from, interval)?;
        let level = self.level(&inputs);
        let ran = self.ran(&mut hers.samus, &inputs, &level)?;
        Ok(ran.outcome.placed)
    }

    /// The engine's content the model reads, her clips among it.
    #[must_use]
    pub const fn content(&self) -> &Content {
        &self.content
    }

    /// What her motion's frame from record `from` shows (`Outcome`): among them what it gives
    /// her animation (the samus-movement spec's §1.4, samus-appearance's inputs).
    ///
    /// # Errors
    ///
    /// The reason her motion doesn't run in the frame, or the decode's failure.
    pub fn outcome(&self, from: &FrameRecord, interval: Interval) -> Result<Outcome, Skip> {
        let (mut hers, inputs) = self.decode(from, interval)?;
        let level = self.level(&inputs);
        let ran = self.ran(&mut hers.samus, &inputs, &level)?;
        Ok(ran.outcome)
    }
}

/// What the model reads besides her, from record N: the pad, the elevator, the frame count,
/// the level, its room's door list, the enemies she can touch, and where the camera last saw
/// her.
pub struct Inputs {
    controller: Controller,
    ride: ElevatorRide,
    completed: u16,
    width: BlockColumns,
    blocks: Vec<Block>,
    doors: Vec<Door>,
    enemies: Vec<Obstacle>,
    /// Where the camera last saw her: the scroll (the camera's domain) writes it after her
    /// frame, so a free run takes it from record N rather than carrying her own.
    seen: Point,
}

impl Inputs {
    /// Record N's inputs, in a room of `doors`, with the camera having last seen her at `seen`:
    /// the enemies are the live ones the selection would let her touch, by the camera as record
    /// N has it.
    fn of(wram: &[u8], doors: Vec<Door>, seen: Point) -> Result<Self, DecodeError> {
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
            seen,
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

impl Domain for Movement {
    const NAME: &'static str = "samus movement";
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
        domain_field!("dashing", dashing),
        domain_field!("mode", mode),
        domain_field!("vertical_speed", vertical_speed),
        domain_field!("vertical", vertical),
        domain_field!("bounce", bounce),
        domain_field!("rising", rising),
        domain_field!("placed", placed),
        domain_field!("heading_down", heading_down),
        domain_field!("vertical_speed_after_moves", speed_after_moves),
        domain_field!("handed_over", handed_over),
    ];

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(Hers, Inputs), Skip> {
        runs_after(from)?;
        motion_runs(from)?;
        // A pose outside her domain has no clip to read her animation's step through.
        let pose = word(from, source::SAMUS_POSE)?;
        if data::assets::samus::map::key_of(PoseId(pose)).is_none() {
            return Err(Skip::Pose(pose));
        }
        let samus = Rules::ORIGINAL.samus(&from.wram, &self.content.samus_clips)?;
        let Some(model) = clean_of(&samus) else {
            return Err(Skip::Pose(samus.pose.0));
        };
        if !named(model.pose) {
            return Err(Skip::Fault(SamusFault::UnnamedPose(samus.pose)));
        }
        let direction = word(from, source::SAMUS_Y_DIR)?;
        let hers = Hers {
            samus,
            rising: direction == UP,
            heading_down: direction == DOWN,
            speed_after_moves: signed_speed(from)?,
            handed_over: handed_over(from)?,
        };
        let doors = self.doors(word(from, source::ROOM_PTR)?)?;
        let seen = hers.samus.previous_position;
        Ok((hers, Inputs::of(&from.wram, doors, seen)?))
    }

    fn masks(
        &self,
        from: &FrameRecord,
        next: &FrameRecord,
        _inputs: &Inputs,
    ) -> Result<Vec<Mask>, Skip> {
        use Quantity as Q;
        let mut masks = Vec::new();
        let mut mask = |reason, fields| masks.push(Mask::another(reason, fields));
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
            // The surface's inside reactions at the frame's start write her speed (`$84:B447`
            // grounded: 0, `$84:B45A` moving up: capped); the sand's fall, the submerging
            // quicksand and the treadmills write displacements only.
            let direction = word(from, source::SAMUS_Y_DIR)?;
            if direction == UP || direction == NO_DIRECTION && signed_speed(from)?.0 != 0 {
                mask(
                    "deferred: the quicksand surface sets her vertical speed before her moves (grounded 0, a rise capped)",
                    Q::SpeedAfterMoves.set(),
                );
            }
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
        let unseen = after_moves_unseen(from, next)?;
        if !unseen.is_empty() {
            mask(
                "unverified: the pose change writes her vertical direction and speed after her moves (what they met, or a rise it starts)",
                unseen,
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
                    Q::Dashing,
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
        masks.extend(a_spins_skid_that_braking_ends(from, next)?);
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
        let content = &self.content;
        hers.samus.previous_position = inputs.seen;
        let start = Start::of(&hers.samus);
        let requests = if pose::carried(&hers.samus, inputs.ride) {
            pose::PoseRequests::default()
        } else {
            pose::input(&hers.samus, &inputs.controller, content)
        };
        if let Ok(ran) = self.ran(&mut hers.samus, inputs, &level) {
            settle(&mut hers.samus, &ran);
            hers.rising = ran.outcome.rising;
            hers.heading_down = ran.outcome.heading_down;
            hers.speed_after_moves = Fixed(ran.outcome.vertical_speed_after_moves.0);
            hers.handed_over = ran.outcome.handed_over;
            world.0 = exercised(&ran, &level);
            // The frame's pose change and her animation, as the engine's Tick runs them after
            // her moves: the free run carries her State, so the next frame's moves read the
            // pose, facing, radius and clip this frame left, not the seed's.
            let stepped = Stepped {
                pose: ran.after.pose,
                outcome: ran.outcome,
            };
            let asked = next(&hers.samus, Some(&stepped), requests);
            pose::change(&mut hers.samus, asked, content);
            samus_appearance_bridge::animate(
                &mut hers.samus,
                start,
                Some(&stepped.outcome),
                &inputs.controller,
                &content.samus_clips,
            );
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
        Declared::of(
            &hers.samus.motion,
            Shown {
                rising: hers.rising,
                placed: hers.speed_down(),
                heading_down: hers.heading_down,
                speed_after_moves: hers.speed_after_moves,
                handed_over: hers.handed_over,
            },
        )
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        let direction = word(next, source::SAMUS_Y_DIR)?;
        Ok(Declared::of(
            &state(next)?,
            Shown {
                rising: direction == UP,
                placed: fixed(&next.wram, CAMERA_Y_SPEED, CAMERA_Y_SUBSPEED)?,
                heading_down: direction == DOWN,
                speed_after_moves: signed_speed(next)?,
                handed_over: handed_over(next)?,
            },
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
        (B::AnimationEnds, ran.animation_ends.is_some()),
        (
            B::Dashing,
            before.run.dashing == Dashes::On || after.run.dashing == Dashes::On,
        ),
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
    use super::{Domain, FieldSet, Hides, Inputs, Mask, Movement, Quantity};
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

    /// The masks of the frame from `from` to `next`.
    fn masks(from: &FrameRecord, next: &FrameRecord) -> Vec<Mask> {
        let domain = Movement {
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
            seen: game::state::samus::Point::default(),
        };
        match domain.masks(from, next, &inputs) {
            Ok(masks) => masks,
            Err(skip) => panic!("masks: {skip}"),
        }
    }

    /// The fields the masks of the frame from `from` to `next` take.
    fn masked(from: &FrameRecord, next: &FrameRecord) -> FieldSet {
        masks(from, next)
            .iter()
            .fold(FieldSet::EMPTY, |set, mask| set.union(mask.fields))
    }

    #[test]
    fn a_spins_skid_that_braking_ends_hides_her_speed_mode_alone_with_no_writer() {
        let movement = WramWord {
            name: "SAMUS_MOVEMENT_TYPE",
            offset: source::SAMUS_MOVEMENT_TYPE.offset,
        };
        let spin = (movement, u16::from(super::SPINNING));
        let skidding = (source::SAMUS_X_ACCEL_MODE, super::SKIDDING);
        let halting = (source::SAMUS_X_ACCEL_MODE, super::HALTING);
        let hidden = |from: &[(WramWord, u16)], next: &[(WramWord, u16)]| {
            masks(&record(0, from), &record(1, next))
                .iter()
                .map(|mask| (mask.fields, mask.hides))
                .collect::<Vec<_>>()
        };
        let mode = [(Quantity::Mode.set(), Hides::NoWriter)];
        let nothing: [(FieldSet, Hides); 0] = [];
        assert_eq!(hidden(&[spin, skidding], &[spin, halting]), mode);
        let moving = (source::SAMUS_X_BASE_SUBSPEED, 0x8000);
        assert_eq!(hidden(&[spin, skidding], &[spin, halting, moving]), nothing);
        assert_eq!(hidden(&[spin, skidding], &[halting]), nothing);
        assert_eq!(hidden(&[skidding], &[spin, halting]), nothing);
        assert_eq!(hidden(&[spin, halting], &[spin, halting]), nothing);
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
    fn a_rise_the_pose_change_starts_hides_her_speed_after_moves_and_from_heading_down_her_heading()
    {
        use Quantity as Q;
        let rise = |direction| {
            masked(
                &record(
                    0,
                    &[(source::SAMUS_Y_DIR, direction), (source::SAMUS_POSE, 1)],
                ),
                &record(
                    1,
                    &[(source::SAMUS_Y_DIR, super::UP), (source::SAMUS_POSE, 2)],
                ),
            )
        };
        assert_eq!(rise(0), Q::SpeedAfterMoves.set());
        assert_eq!(
            rise(super::DOWN),
            Q::all(&[Q::HeadingDown, Q::SpeedAfterMoves])
        );
    }

    #[test]
    fn a_walk_off_at_rest_hides_her_heading_alone() {
        use Quantity as Q;
        let walk_off = |speed| {
            masked(
                &record(0, &[(source::SAMUS_Y_SPEED, speed)]),
                &record(
                    1,
                    &[
                        (source::SAMUS_Y_DIR, super::DOWN),
                        (super::POSE_CHANGE_COMMAND, super::VERTICAL_RESULT),
                    ],
                ),
            )
        };
        assert_eq!(walk_off(0), Q::HeadingDown.set());
        assert_eq!(walk_off(1), Q::all(&[Q::HeadingDown, Q::SpeedAfterMoves]));
    }

    #[test]
    fn a_push_hides_her_speed_after_moves_only_where_the_surface_sets_it() {
        let after_moves_hidden = |words: &[(WramWord, u16)]| {
            masked(&record(0, words), &record(1, &[(super::IN_QUICKSAND, 1)]))
                .contains(Quantity::SpeedAfterMoves.index())
        };
        let up = (source::SAMUS_Y_DIR, super::UP);
        let down = (source::SAMUS_Y_DIR, super::DOWN);
        let moving = (source::SAMUS_Y_SPEED, 1);
        assert!(after_moves_hidden(&[up]));
        assert!(after_moves_hidden(&[moving]));
        assert!(!after_moves_hidden(&[]));
        assert!(!after_moves_hidden(&[down, moving]));
    }

    #[test]
    fn a_quantity_names_the_declared_field_at_its_place() {
        use Quantity as Q;
        let named = [
            (Q::X, "position.x"),
            (Q::Y, "position.y"),
            (Q::BaseSpeed, "base_speed"),
            (Q::ExtraRun, "extra_run"),
            (Q::Dashing, "dashing"),
            (Q::Mode, "mode"),
            (Q::VerticalSpeed, "vertical_speed"),
            (Q::Vertical, "vertical"),
            (Q::Bounce, "bounce"),
            (Q::Rising, "rising"),
            (Q::Placed, "placed"),
            (Q::HeadingDown, "heading_down"),
            (Q::SpeedAfterMoves, "vertical_speed_after_moves"),
            (Q::HandedOver, "handed_over"),
        ];
        for (quantity, name) in named {
            let place = Movement::FIELDS.iter().position(|field| field.name == name);
            assert_eq!(place.map(FieldSet::single), Some(quantity.set()), "{name}");
        }
    }
}
