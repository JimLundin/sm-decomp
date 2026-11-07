//! Samus's motion as a harness domain.
//!
//! [`State::step`], [`State::arm_pump_step`] and [`State::pose_writes`] are checked against
//! the oracle, frame by frame, on the spec's declared fields
//! (`docs/behaviours/samus-motion.md` §1.1).
//!
//! The decode follows `docs/behaviours/samus-motion.decode.md`. Collision and pose selection
//! aren't under test: [`Replay`] stands in for both and answers from record N+1, so the
//! motion model is tested and the collision is not.

use core::fmt;

use data::movement::{AirMotion, GroundMotion, HorizontalSpeeds, VerticalSpeeds};
use data::rom::{AddressError, Lorom};
use game::model::collision::{
    Blocker, Body, Collision, Enemies, EnemyHit, Heading, Moved, Stop, WallHit,
};
use game::model::num::Fixed;
use game::model::samus::motion::{
    Bounce, Equipment, Facing, FallKind, Inputs, JumpButton, LandKind, Locomotion, MotionRules,
    Pad, PoseChange, PoseCommand, PoseRequest, PoseStart, Position, Radius, Sideways, SpeedMode,
    Spin, State, VerticalMotion, VerticalResult,
};
use game::session::SpeedCap;
use game::state::strand::Frame;
use oracle_record::{Cut, FrameRecord, NmiKind};

use super::{Domain, Field, Interval};
use crate::decode::{DecodeError, Rules};
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// The pad's left and right bits.
const PAD_LEFT: u16 = 0x0200;
const PAD_RIGHT: u16 = 0x0100;

/// The equipment bits motion reads.
const HI_JUMP: u16 = 0x0100;
const SPACE_JUMP: u16 = 0x0200;

/// Why a frame isn't one Samus's motion is checked on.
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
    /// Her movement type at the frame's start is outside the domain.
    MovementType(u8),
    /// Her pose's X direction is no facing.
    Facing(u8),
    /// Her y radius changes, so pose fit moves y, which isn't an input yet.
    PoseFit,
    /// The elevator is moving, and carries her by more than motion's own step.
    Elevator,
    /// A pose change command the original never writes.
    Command(u16),
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::GameState(code) => write!(f, "game state {code:#04X}, not gameplay"),
            Self::NotAtFrameEnd => write!(f, "record N isn't at the main loop's frame end (lag)"),
            Self::Waiting(at) => write!(f, "the main thread waits in {at}: motion doesn't run"),
            Self::MovementType(code) => write!(f, "movement type {code:#04X} isn't in the domain"),
            Self::Facing(code) => write!(f, "pose X direction {code:#04X}"),
            Self::PoseFit => write!(f, "the y radius changes: pose fit moves y"),
            Self::Elevator => write!(f, "the elevator moves her: another domain writes y"),
            Self::Command(code) => write!(f, "pose change command {code:#06X}"),
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

fn state(record: &FrameRecord) -> Result<State, Skip> {
    Ok(decode(&record.wram)?)
}

/// Samus's motion State in `wram`: the decode the Session decoder and this domain share.
///
/// # Errors
///
/// [`DecodeError`] when a word is outside `wram` or holds a code the State has no value for.
pub fn decode(wram: &[u8]) -> Result<State, DecodeError> {
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
    Ok(State {
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

fn radius(record: &FrameRecord) -> Result<Radius, Skip> {
    Ok(Radius {
        x: word(record, source::SAMUS_X_RADIUS)?,
        y: word(record, source::SAMUS_Y_RADIUS)?,
    })
}

/// Her pose, as the decode reads it: the pose, its movement type and its X direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pose {
    code: u16,
    movement: u8,
    facing: u8,
}

impl Pose {
    fn of(record: &FrameRecord) -> Result<Self, Skip> {
        Ok(Self {
            code: word(record, source::SAMUS_POSE)?,
            movement: byte(record, source::SAMUS_MOVEMENT_TYPE)?,
            facing: byte(record, source::SAMUS_POSE_X_DIR)?,
        })
    }

    /// The normal jump's take-off poses.
    const fn take_off(self) -> bool {
        matches!(self.code, 0x4B | 0x4C | 0x55..=0x5A)
    }

    const fn ball(self) -> bool {
        matches!(self.movement, 0x04 | 0x08 | 0x11 | 0x12 | 0x13)
    }

    const fn turn(self) -> bool {
        matches!(self.movement, 0x0E | 0x17 | 0x18)
    }

    const fn spinning(self) -> bool {
        matches!(self.movement, 0x03 | 0x14)
    }

    const fn locomotion(self) -> Result<Locomotion, Skip> {
        Ok(match self.movement {
            0x00 => match self.code {
                0x00 | 0x9B => Locomotion::FacingForward,
                0x01 | 0x02 => Locomotion::Standing { plain: true },
                _ => Locomotion::Standing { plain: false },
            },
            0x01 => Locomotion::Running,
            0x02 => Locomotion::Jumping {
                take_off: self.take_off(),
            },
            0x03 => Locomotion::SpinJumping,
            0x04 => Locomotion::Ball {
                spring: false,
                at_rest: matches!(self.code, 0x1D | 0x41),
            },
            0x05 => Locomotion::Crouching,
            0x06 => Locomotion::Falling,
            0x08 => Locomotion::BallInAir {
                spring: false,
                falling: true,
            },
            0x0E => Locomotion::TurningOnGround,
            0x0F => Locomotion::Transition,
            0x10 => Locomotion::Moonwalking,
            0x11 => Locomotion::Ball {
                spring: true,
                at_rest: matches!(self.code, 0x79 | 0x7A),
            },
            0x12 => Locomotion::BallInAir {
                spring: true,
                falling: false,
            },
            0x13 => Locomotion::BallInAir {
                spring: true,
                falling: true,
            },
            0x14 => Locomotion::WallJumping,
            0x15 => Locomotion::RanIntoWall,
            0x17 | 0x18 => Locomotion::TurningInAir,
            code => return Err(Skip::MovementType(code)),
        })
    }

    const fn facing(self) -> Result<Facing, Skip> {
        match self.facing {
            0x04 => Ok(Facing::Left),
            0x08 => Ok(Facing::Right),
            0x00 => Ok(Facing::Forward),
            code => Err(Skip::Facing(code)),
        }
    }

    /// What the start of `self`, after `old`, writes (spec §7's `PoseStart`), and whether it
    /// replaced the pose.
    fn start_after(self, old: Self) -> (Option<PoseStart>, bool) {
        let reversed = self.facing != old.facing;
        let start = match self.movement {
            _ if self.turn() && !old.turn() => return (Some(PoseStart::Turn), true),
            _ if self.ball() && old.ball() && reversed => Some(PoseStart::BallReverse),
            0x03 if old.spinning() => reversed.then_some(PoseStart::SpinReverse),
            0x03 => Some(PoseStart::SpinStart),
            0x02 if self.take_off() && !old.take_off() => Some(PoseStart::TakeOff {
                from_plain_crouch: matches!(old.code, 0x27 | 0x28),
            }),
            0x02 | 0x06 => Some(PoseStart::JumpOrFall),
            0x12 if old.movement == 0x11 => Some(PoseStart::SpringLaunch),
            0x14 if old.movement != 0x14 => Some(PoseStart::WallJumpStart),
            _ => None,
        };
        (start, false)
    }
}

/// The prospective pose change command, as pose selection gives it with a new pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    None,
    Decelerate,
    Stop,
    SolidVerticalCollision,
    KillXSpeed,
    StartTransition,
    KillRunSpeed,
}

impl TryFrom<u16> for Command {
    type Error = Skip;
    fn try_from(code: u16) -> Result<Self, Skip> {
        Ok(match code {
            0 | 3 | 4 => Self::None,
            1 => Self::Decelerate,
            2 => Self::Stop,
            5 => Self::SolidVerticalCollision,
            6 => Self::KillXSpeed,
            7 => Self::StartTransition,
            8 => Self::KillRunSpeed,
            code => return Err(Skip::Command(code)),
        })
    }
}

/// The new pose of the frame, as pose selection started it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NewPose {
    start: Option<PoseStart>,
    replaced: bool,
    command: Command,
    started_falling: bool,
    running: bool,
}

/// The harness's stand-in for collision and pose selection over one frame.
///
/// **Collision.** Each move is answered from where record N+1 has her (`target`), on the
/// move's axis: when the remaining way to the target, from the body's position, is the
/// requested distance, the move is made unblocked; when it is shorter in the same direction
/// (or zero), the move is stopped by a block there, written as the target's subpixel and its
/// whole-pixel difference; otherwise (longer, or the other way) the move is made unblocked, so
/// a wrong speed shows in the position. A zero move is never stopped. No enemy is solid
/// (`enemy_ahead` and the probe find nothing), so the wall jump never fires.
///
/// **Pose selection.** The new pose is record N+1's when it differs from record N's: its start
/// comes from the two poses (spec §7), its command from record N+1's prospective pose change
/// command, and the arm-pump step's request is that pose. No pose is forced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Replay {
    target: Position,
    radius: Radius,
    pose: Option<NewPose>,
}

impl Replay {
    /// The move from `at` by `distance` on an axis whose target is `target`.
    fn answer(at: Fixed, target: Fixed, distance: Fixed, side: Heading) -> Moved {
        let remaining = i64::from(target.0.wrapping_sub(at.0).cast_signed());
        let asked = i64::from(distance.0.cast_signed());
        let landed = Fixed(at.0.wrapping_add(distance.0));
        // Short of the move, or pushed back from it: a stop. A landing on the target's pixel
        // is no stop (a probe may still write the subpixel); beyond it, the model's error.
        let stopped = match asked.signum() {
            _ if landed.pixel() == target.pixel() => false,
            1 => remaining < asked,
            -1 => remaining > asked,
            _ => false,
        };
        if !stopped {
            return Moved {
                distance,
                stop: None,
            };
        }
        Moved {
            distance: Fixed::new(target.pixel().wrapping_sub(at.pixel()), 0),
            stop: Some(Stop {
                side,
                blocker: Blocker::Block {
                    subpixel: target.sub(),
                },
            }),
        }
    }
}

impl Collision for Replay {
    fn move_horizontal(
        &mut self,
        body: Body,
        heading: Heading,
        distance: Fixed,
        _enemies: Enemies,
    ) -> Moved {
        Self::answer(body.position.x, self.target.x, distance, heading)
    }

    fn move_down(
        &mut self,
        body: Body,
        heading: Heading,
        distance: Fixed,
        _enemies: Enemies,
    ) -> Moved {
        let signed = match heading {
            Heading::Up => Fixed(distance.0.wrapping_neg()),
            Heading::Down | Heading::Left | Heading::Right => distance,
        };
        Self::answer(body.position.y, self.target.y, signed, heading)
    }

    fn move_up(&mut self, body: Body, distance: Fixed) -> Moved {
        Self::answer(
            body.position.y,
            self.target.y,
            Fixed(distance.0.wrapping_neg()),
            Heading::Up,
        )
    }

    fn enemy_ahead(
        &mut self,
        _body: Body,
        _heading: Heading,
        _distance: Fixed,
    ) -> Option<EnemyHit> {
        None
    }

    fn probe_wall(&mut self, body: Body, _heading: Heading, reach: u16) -> Option<WallHit> {
        let fires = self
            .pose
            .is_some_and(|pose| pose.start == Some(PoseStart::WallJumpStart));
        let subpixel =
            (body.position.x.sub() != self.target.x.sub()).then_some(self.target.x.sub());
        (fires || subpixel.is_some()).then_some(WallHit {
            distance_px: if fires { 0 } else { reach },
            subpixel,
        })
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
}

impl From<&State> for Declared {
    fn from(state: &State) -> Self {
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

/// Samus's motion domain, with the speed tables from the ROM.
#[derive(Clone, Debug)]
pub struct SamusMotion {
    speeds: HorizontalSpeeds,
    vertical: VerticalSpeeds,
}

impl SamusMotion {
    /// The domain with the speeds in `rom`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        Ok(Self {
            speeds: HorizontalSpeeds::load(rom)?,
            vertical: VerticalSpeeds::load(rom)?,
        })
    }

    const fn rules(&self) -> MotionRules<'_> {
        MotionRules {
            speeds: &self.speeds,
            vertical: self.vertical,
            ground: GroundMotion::VANILLA,
            air: AirMotion::VANILLA,
            speed_cap: SpeedCap::Wordwise,
        }
    }
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
    let main = Rules::ORIGINAL.strand.main(
        nmi.interrupted_pc,
        record.cpu.s,
        record.cpu.x,
        &record.wram,
    )?;
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

const fn jump(held: bool, pressed: bool) -> JumpButton {
    match (held, pressed) {
        (_, true) => JumpButton::Pressed,
        (true, false) => JumpButton::Held,
        (false, false) => JumpButton::Released,
    }
}

impl Domain for SamusMotion {
    const NAME: &'static str = "samus motion";
    type State = State;
    type Inputs = Inputs;
    type World = Replay;
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

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(State, Inputs), Skip> {
        runs_after(from)?;
        let pose = Pose::of(from)?;
        // The pad the frame's gameplay reads is the one the NMI ending record N's interval
        // stored, which record N holds.
        let held = word(from, source::JOYPAD1_LASTKEYS)?;
        let pressed = word(from, source::JOYPAD1_NEWKEYS)?;
        let bound = |binding| word(from, binding);
        let (jump_bit, run_bit, shoot_bit) = (
            bound(source::BINDING_JUMP)?,
            bound(source::BINDING_RUN)?,
            bound(source::BINDING_SHOOT)?,
        );
        let items = word(from, source::EQUIPPED_ITEMS)?;
        let frame = word(from, source::SAMUS_ANIM_FRAME)?;
        let ready_from = if matches!(pose.code, 0x81 | 0x82) {
            0x1B
        } else {
            0x0B
        };
        let inputs = Inputs {
            locomotion: pose.locomotion()?,
            facing: pose.facing()?,
            radius: radius(from)?,
            pad: Pad {
                run: held & run_bit != 0,
                shoot: held & shoot_bit != 0,
                jump: jump(held & jump_bit != 0, pressed & jump_bit != 0),
                sideways: Sideways {
                    left: held & PAD_LEFT != 0,
                    right: held & PAD_RIGHT != 0,
                },
            },
            spin: Spin {
                ready: frame >= ready_from,
                was_spinning: matches!(
                    byte(from, source::SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE)?,
                    0x03 | 0x14
                ),
            },
            equipment: Equipment {
                hi_jump: items & HI_JUMP != 0,
                space_jump: items & SPACE_JUMP != 0,
            },
            elevator_moving: word(from, source::ELEVATOR_STATUS)? != 0,
            on_slope: word(from, source::SAMUS_SLOPE_ADJUSTED)? != 0,
        };
        Ok((state(from)?, inputs))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Replay, Skip> {
        if word(from, source::ELEVATOR_STATUS)? != 0 {
            return Err(Skip::Elevator);
        }
        let (before, after) = (radius(from)?, radius(next)?);
        if before.y != after.y {
            return Err(Skip::PoseFit);
        }
        let (old, new) = (Pose::of(from)?, Pose::of(next)?);
        let pose = if new == old {
            None
        } else {
            let (start, replaced) = new.start_after(old);
            Some(NewPose {
                start,
                replaced,
                command: Command::try_from(word(next, source::SAMUS_POSE_CHANGE_COMMAND)?)?,
                started_falling: new.movement == 0x06 && old.movement != 0x06,
                running: new.movement == 0x01,
            })
        };
        Ok(Replay {
            target: position(&next.wram)?,
            radius: after,
            pose,
        })
    }

    fn step(&self, state: &mut State, inputs: &Inputs, world: &mut Replay) {
        let rules = self.rules();
        // The record lags the pose by a frame: the radius is record N+1's.
        let inputs = Inputs {
            radius: world.radius,
            ..*inputs
        };
        let Ok(contacts) = state.step(&inputs, &rules, world) else {
            return;
        };
        let request = world
            .pose
            .map_or(PoseRequest::None, |pose| PoseRequest::Pose {
                running: pose.running,
            });
        state.arm_pump_step(&inputs, contacts.wall_stop, request, &rules, world);
        let vertical = contacts.vertical;
        let passed = !matches!(
            vertical,
            VerticalResult::None
                | VerticalResult::Falling(FallKind::NoChange)
                | VerticalResult::Landed(LandKind::NoChange)
        );
        let command = |command| match command {
            _ if passed => PoseCommand::Vertical(vertical),
            Command::Decelerate => PoseCommand::Decelerate,
            Command::Stop => PoseCommand::Stop,
            Command::KillXSpeed => PoseCommand::KillXSpeed,
            Command::KillRunSpeed => PoseCommand::KillRunSpeed,
            Command::None | Command::SolidVerticalCollision | Command::StartTransition => {
                PoseCommand::None
            }
        };
        let change = match world.pose {
            Some(pose) => PoseChange {
                start: pose.start,
                replaced: pose.replaced,
                command: command(pose.command),
                started_falling: pose.started_falling,
            },
            None if passed => PoseChange {
                start: None,
                replaced: false,
                command: PoseCommand::Vertical(vertical),
                started_falling: false,
            },
            None => return,
        };
        let jump_held = !matches!(inputs.pad.jump, JumpButton::Released);
        state.pose_writes(change, jump_held, inputs.equipment, &rules);
    }

    fn fields(&self, state: &State) -> Declared {
        Declared::from(state)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        Ok(Declared::from(&state(next)?))
    }
}
