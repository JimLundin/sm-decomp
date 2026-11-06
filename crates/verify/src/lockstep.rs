//! Lockstep (ARCHITECTURE §11.5): the engine stepped one Tick from each decoded oracle cut,
//! compared with the next cut's decoded State field by field.
//!
//! For each pair of consecutive cached frames (k, k+1) that both decode, with k quiescent so
//! its main strand is known, a [`Session`] opens on decode(k) and steps one Tick with frame
//! k+1's pad and its cut's NMI timing. The engine's State is then compared with decode(k+1)
//! on the fields [`crate::coverage::FIELDS`] lists as decoded; every other field keeps its
//! default on the oracle side, so it isn't compared.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

use canonical::{Canonical, Difference, Segment};
use game::state::room::{AiVariables, ENEMY_SLOTS, Elevator, EnemyBody};
use game::state::slots::{Frames, Occupancy};
use game::state::{NmiTiming, State};
use game::{AutoRepeat, Content, OpenError, Rules, Session, SessionFault, TickInput};
use oracle_record::{Cut, FrameRecord};

use crate::cache::{Cache, CacheError, CutKind};
use crate::coverage::{FIELDS, Status};
use crate::decode::{DecodeError, Decoded, Quiescence, RoomHeader, RoomMap, Unsupported, decode};
use crate::lock::Frame;
use crate::source::{self, WramWord};

/// The rooms slice 0's rows play in and the species their enemies are.
///
/// The rooms have the IDs the content gives them (`data::room::RoomMap::slice_zero`, the
/// Morph Ball Room at `$8F:9E9F`), the species theirs (`data::enemy::SpeciesMap::slice_zero`).
#[must_use]
pub fn slice_zero_rooms() -> RoomMap {
    RoomMap {
        rooms: data::room::RoomMap::slice_zero()
            .iter()
            .filter_map(|(key, id)| key.room_pointer().map(|pointer| (RoomHeader(pointer), id)))
            .collect(),
        species: data::enemy::SpeciesMap::slice_zero(),
    }
}

/// One compared field: its coverage path and how to compare it.
#[derive(Clone, Copy)]
struct Comparator {
    path: &'static str,
    compare: fn(&State, &State) -> Option<Difference>,
}

/// A comparator for one part of the horizontal motion Samus's movement keeps.
macro_rules! speed {
    ($path:literal, $part:ident) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| state.samus.movement.horizontal().map(|h| h.$part);
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// A comparator for one part of the vertical motion Samus's movement keeps.
macro_rules! vertical {
    ($path:literal, $part:ident) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| state.samus.movement.vertical().map(|v| v.$part);
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// A comparator for one common field of every enemy slot, compared slot by slot.
macro_rules! enemy {
    ($path:literal, $($name:ident).+) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| state.room.enemies.slots.map(|slot| slot.common.$($name).+);
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// Each enemy slot's occupancy without its AI words, which their own row compares.
fn occupant(state: &State) -> [Occupancy<EnemyBody>; ENEMY_SLOTS] {
    let quiet = |body: EnemyBody| match body {
        EnemyBody::MorphBallEye { .. } => EnemyBody::MorphBallEye {
            variables: AiVariables::default(),
        },
        EnemyBody::Elevator(_) => EnemyBody::Elevator(Elevator::default()),
        EnemyBody::FaceBlock { .. } => EnemyBody::FaceBlock {
            variables: AiVariables::default(),
        },
    };
    state.room.enemies.slots.map(|slot| match slot.occupancy {
        Occupancy::Free => Occupancy::Free,
        Occupancy::Live(body) => Occupancy::Live(quiet(body)),
        Occupancy::Deleting(body) => Occupancy::Deleting(quiet(body)),
    })
}

/// Each enemy slot's AI words, absent in a free slot.
fn variables(state: &State) -> [Option<AiVariables>; ENEMY_SLOTS] {
    state.room.enemies.slots.map(|slot| match slot.occupancy {
        Occupancy::Free => None,
        Occupancy::Live(body) | Occupancy::Deleting(body) => Some(match body {
            EnemyBody::MorphBallEye { variables } | EnemyBody::FaceBlock { variables } => variables,
            EnemyBody::Elevator(elevator) => crate::enemy::elevator_words(elevator),
        }),
    })
}

macro_rules! field {
    ($path:literal, $($name:ident).+) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| engine.$($name).+.diff(&oracle.$($name).+),
        }
    };
}

/// A comparator for every field the coverage table marks decoded, in its order.
const COMPARATORS: &[Comparator] = &[
    field!("clock.counters.nmi", clock.counters.nmi),
    field!("clock.counters.completed", clock.counters.completed),
    field!("clock.counters.lag", clock.counters.lag),
    field!("clock.counters.max_lag", clock.counters.max_lag),
    field!(
        "clock.counters.requested_byte",
        clock.counters.requested_byte
    ),
    field!("clock.game_time.hours", clock.game_time.hours),
    field!("clock.game_time.minutes", clock.game_time.minutes),
    field!("clock.game_time.seconds", clock.game_time.seconds),
    field!("clock.game_time.frames", clock.game_time.frames),
    field!("rng.word", rng.word),
    field!("controller.auto_read", controller.auto_read),
    field!("controller.held", controller.held),
    field!("controller.pressed", controller.pressed),
    field!("controller.previous", controller.previous),
    field!("controller.repeat", controller.repeat),
    field!("controller.repeat_delay", controller.repeat_delay),
    field!("mode", mode),
    field!("main.frames", main.frames),
    field!("main.waiting", main.waiting),
    field!("irq.command", irq.command),
    field!("irq.line", irq.line),
    field!("timing.last", timing.last),
    field!("video.memory.vram", video.memory.vram),
    field!("video.memory.cgram", video.memory.cgram),
    field!("video.memory.oam", video.memory.oam),
    field!("video.committed", video.committed),
    field!("video.dma", video.dma),
    field!(
        "video.batch.registers.forced_blank",
        video.batch.registers.forced_blank
    ),
    field!(
        "video.batch.registers.brightness",
        video.batch.registers.brightness
    ),
    field!(
        "video.batch.registers.main_screen",
        video.batch.registers.main_screen
    ),
    field!(
        "video.batch.registers.sub_screen",
        video.batch.registers.sub_screen
    ),
    field!("video.batch.registers.scroll", video.batch.registers.scroll),
    field!(
        "video.batch.registers.objects",
        video.batch.registers.objects
    ),
    field!("video.batch.registers.mode", video.batch.registers.mode),
    field!("video.batch.registers.mosaic", video.batch.registers.mosaic),
    field!(
        "video.batch.registers.tilemaps",
        video.batch.registers.tilemaps
    ),
    field!(
        "video.batch.registers.tile_bases",
        video.batch.registers.tile_bases
    ),
    field!(
        "video.batch.registers.mode7.select",
        video.batch.registers.mode7.select
    ),
    field!(
        "video.batch.registers.mode7.matrix",
        video.batch.registers.mode7.matrix
    ),
    field!(
        "video.batch.registers.mode7.centre",
        video.batch.registers.mode7.centre
    ),
    field!(
        "video.batch.registers.windows.select",
        video.batch.registers.windows.select
    ),
    field!(
        "video.batch.registers.windows.edges",
        video.batch.registers.windows.edges
    ),
    field!(
        "video.batch.registers.windows.background_logic",
        video.batch.registers.windows.background_logic
    ),
    field!(
        "video.batch.registers.windows.object_logic",
        video.batch.registers.windows.object_logic
    ),
    field!(
        "video.batch.registers.main_window_mask",
        video.batch.registers.main_window_mask
    ),
    field!(
        "video.batch.registers.sub_window_mask",
        video.batch.registers.sub_window_mask
    ),
    field!(
        "video.batch.registers.colour_math.select",
        video.batch.registers.colour_math.select
    ),
    field!(
        "video.batch.registers.colour_math.layers",
        video.batch.registers.colour_math.layers
    ),
    field!(
        "video.batch.registers.settings",
        video.batch.registers.settings
    ),
    field!("video.batch.fixed_colour", video.batch.fixed_colour),
    field!("video.batch.oam", video.batch.oam),
    field!("video.batch.cgram", video.batch.cgram),
    field!("video.batch.vram", video.batch.vram),
    field!("video.nmi_requested", video.nmi_requested),
    field!("samus.position.x.pixels", samus.position.x.pixels),
    field!("samus.position.x.subpixels", samus.position.x.subpixels),
    field!("samus.position.y.pixels", samus.position.y.pixels),
    field!("samus.position.y.subpixels", samus.position.y.subpixels),
    field!("samus.pose", samus.pose),
    field!("samus.previous.pose", samus.previous.pose),
    field!("samus.previous.facing", samus.previous.facing),
    field!("samus.previous.movement", samus.previous.movement),
    field!("samus.last_different.pose", samus.last_different.pose),
    field!("samus.last_different.facing", samus.last_different.facing),
    field!(
        "samus.last_different.movement",
        samus.last_different.movement
    ),
    field!("samus.facing", samus.facing),
    field!("samus.movement", samus.movement),
    // The speed words live inside the movement's variant; each row compares the part it names
    // (a 16.16 speed's pixels and subpixels are one value), absent when the movement keeps none.
    speed!("samus.movement.speed.base.pixels", base),
    speed!("samus.movement.speed.base.subpixels", base),
    speed!("samus.movement.speed.extra_run.pixels", extra_run),
    speed!("samus.movement.speed.extra_run.subpixels", extra_run),
    speed!("samus.movement.speed.acceleration", acceleration),
    speed!("samus.movement.speed.momentum", momentum),
    vertical!("samus.movement.vertical.speed.pixels", speed),
    vertical!("samus.movement.vertical.speed.subpixels", speed),
    vertical!("samus.movement.vertical.direction", direction),
    field!("samus.equipment.collected", samus.equipment.collected),
    field!("samus.animation.frame", samus.animation.frame),
    field!("samus.animation.timer", samus.animation.timer),
    field!(
        "samus.animation.new_pose_frame",
        samus.animation.new_pose_frame
    ),
    field!("samus.super_special.pose", samus.super_special.pose),
    field!("samus.super_special.command", samus.super_special.command),
    field!("samus.equipment.equipped", samus.equipment.equipped),
    field!(
        "samus.equipment.energy.current",
        samus.equipment.energy.current
    ),
    field!("samus.equipment.energy.max", samus.equipment.energy.max),
    field!(
        "samus.equipment.reserve.current",
        samus.equipment.reserve.current
    ),
    field!("samus.equipment.reserve.max", samus.equipment.reserve.max),
    field!(
        "samus.equipment.missiles.current",
        samus.equipment.missiles.current
    ),
    field!("samus.equipment.missiles.max", samus.equipment.missiles.max),
    field!(
        "samus.equipment.super_missiles.current",
        samus.equipment.super_missiles.current
    ),
    field!(
        "samus.equipment.super_missiles.max",
        samus.equipment.super_missiles.max
    ),
    field!(
        "samus.equipment.power_bombs.current",
        samus.equipment.power_bombs.current
    ),
    field!(
        "samus.equipment.power_bombs.max",
        samus.equipment.power_bombs.max
    ),
    field!("samus.equipment.reserve_mode", samus.equipment.reserve_mode),
    field!("room.room", room.room),
    field!("room.blocks", room.blocks),
    field!("room.width", room.width),
    field!("room.scroll_areas", room.scroll_areas),
    field!("room.camera.x.pixels", room.camera.x.pixels),
    field!("room.camera.x.subpixels", room.camera.x.subpixels),
    field!("room.camera.y.pixels", room.camera.y.pixels),
    field!("room.camera.y.subpixels", room.camera.y.subpixels),
    enemy!("room.enemies.slots.common.species", species),
    enemy!(
        "room.enemies.slots.common.position.x.pixels",
        position.x.pixels
    ),
    enemy!(
        "room.enemies.slots.common.position.x.subpixels",
        position.x.subpixels
    ),
    enemy!(
        "room.enemies.slots.common.position.y.pixels",
        position.y.pixels
    ),
    enemy!(
        "room.enemies.slots.common.position.y.subpixels",
        position.y.subpixels
    ),
    enemy!("room.enemies.slots.common.radius_x", radius_x),
    enemy!("room.enemies.slots.common.radius_y", radius_y),
    enemy!("room.enemies.slots.common.properties", properties),
    enemy!(
        "room.enemies.slots.common.extra_properties",
        extra_properties
    ),
    enemy!("room.enemies.slots.common.ai_handlers", ai_handlers),
    enemy!("room.enemies.slots.common.health", health),
    enemy!("room.enemies.slots.common.timer", timer),
    enemy!("room.enemies.slots.common.cursor.delay", cursor.delay),
    enemy!("room.enemies.slots.common.palette", palette),
    enemy!("room.enemies.slots.common.tiles", tiles),
    enemy!("room.enemies.slots.common.layer", layer),
    enemy!("room.enemies.slots.common.flash", flash),
    enemy!("room.enemies.slots.common.frozen", frozen),
    enemy!("room.enemies.slots.common.invincibility", invincibility),
    enemy!("room.enemies.slots.common.shake", shake),
    enemy!("room.enemies.slots.common.frame_counter", frame_counter),
    enemy!("room.enemies.slots.common.parameter_1", parameter_1),
    enemy!("room.enemies.slots.common.parameter_2", parameter_2),
    Comparator {
        path: "room.enemies.slots.occupancy",
        compare: |engine: &State, oracle: &State| occupant(engine).diff(&occupant(oracle)),
    },
    Comparator {
        path: "room.enemies.slots.occupancy.variables",
        compare: |engine: &State, oracle: &State| variables(engine).diff(&variables(oracle)),
    },
    field!("elevator.status", elevator.status),
    field!("elevator.armed", elevator.armed),
    field!("elevator.direction", elevator.direction),
    field!("hud.tilemap", hud.tilemap),
    field!("hud.selected", hud.selected),
    field!("hud.auto_cancel", hud.auto_cancel),
    field!("hud.drawn.energy", hud.drawn.energy),
    field!("hud.drawn.missiles", hud.drawn.missiles),
    field!("hud.drawn.super_missiles", hud.drawn.super_missiles),
    field!("hud.drawn.power_bombs", hud.drawn.power_bombs),
    field!("hud.drawn.selection", hud.drawn.selection),
    field!("hud.item_palette", hud.item_palette),
    field!("message", message),
];

/// Whether the coverage table marks `path` decoded.
fn decoded(path: &str) -> bool {
    FIELDS
        .iter()
        .any(|field| field.path == path && matches!(field.status, Status::Decoded(_)))
}

/// Whether `path` compares at a cut of this quiescence. At a busy cut the NMI interrupted the
/// main thread mid-routine, so no engine strand corresponds and the decoder leaves `main` at
/// its default.
fn compared_at(path: &str, quiescence: Quiescence) -> bool {
    match quiescence {
        Quiescence::Quiet => true,
        Quiescence::Busy => path.split('.').next() != Some("main"),
    }
}

/// The compared fields where `engine` and `oracle` differ, each as a difference whose path
/// starts at the State, in the coverage table's order.
fn differences(engine: &State, oracle: &State, quiescence: Quiescence) -> Vec<Difference> {
    COMPARATORS
        .iter()
        .filter(|comparator| decoded(comparator.path) && compared_at(comparator.path, quiescence))
        .filter_map(|comparator| {
            (comparator.compare)(engine, oracle).map(|difference| {
                comparator
                    .path
                    .rsplit('.')
                    .fold(difference, |inner, name| inner.within(Segment::Field(name)))
            })
        })
        .collect()
}

/// Why a frame has no decoded State, by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// [`DecodeError::WramTooShort`].
    WramTooShort,
    /// [`DecodeError::WrongSize`].
    WrongSize,
    /// [`DecodeError::UnknownRoom`].
    UnknownRoom,
    /// [`Unsupported::GameState`].
    GameState,
    /// [`Unsupported::NoNmi`]: a vblank without an NMI.
    VblankWithoutNmi,
    /// [`Unsupported::NoNmi`]: an NMI still running at the end of vblank.
    NmiStillRunning,
    /// [`Unsupported::IrqCommand`].
    IrqCommand,
    /// [`DecodeError::Unrepresentable`].
    Unrepresentable,
    /// [`DecodeError::UnknownYieldSite`].
    UnknownYieldSite,
}

impl ErrorKind {
    /// The kind of `error`.
    #[must_use]
    pub const fn of(error: &DecodeError) -> Self {
        match error {
            DecodeError::WramTooShort { .. } => Self::WramTooShort,
            DecodeError::WrongSize { .. } => Self::WrongSize,
            DecodeError::UnknownRoom(_) => Self::UnknownRoom,
            DecodeError::Unsupported(Unsupported::GameState(_)) => Self::GameState,
            DecodeError::Unsupported(Unsupported::NoNmi(Cut::VblankWithoutNmi)) => {
                Self::VblankWithoutNmi
            }
            DecodeError::Unsupported(Unsupported::NoNmi(
                Cut::NmiStillRunning { .. } | Cut::NmiReturn(_),
            )) => Self::NmiStillRunning,
            DecodeError::Unsupported(Unsupported::IrqCommand { .. }) => Self::IrqCommand,
            DecodeError::Unrepresentable { .. } => Self::Unrepresentable,
            DecodeError::UnknownYieldSite { .. } => Self::UnknownYieldSite,
        }
    }
}

/// Why a pair of frames wasn't compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// One of the two frames doesn't decode; the error is the later frame's when both don't.
    Undecodable(ErrorKind),
    /// Frame k's cut is busy, so its main strand isn't known and no Session opens on it.
    BusyStart,
    /// The Session refused frame k's State.
    Open(OpenError),
    /// The Tick faulted.
    Fault(SessionFault),
    /// Frame k+1's cut has no NMI timing the Tick can take.
    NoTiming(CutKind),
}

/// One Tick's comparison: the frame it ran to and the compared fields that differ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// Frame k+1: the cut the Tick ran to.
    pub frame: Frame,
    /// The differing fields in the coverage table's order; empty when the States agree.
    pub differences: Vec<Difference>,
}

impl Step {
    /// The first differing field, `None` when the States agree.
    #[must_use]
    pub fn first(&self) -> Option<&Difference> {
        self.differences.first()
    }
}

/// One field over the run: how often it compared and differed, and where it first did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSummary {
    /// Its coverage path.
    pub path: &'static str,
    /// The Ticks it compared on.
    pub compared: u64,
    /// The Ticks it differed on.
    pub differed: u64,
    /// The first Tick it differed on, with the difference.
    pub first: Option<(Frame, Difference)>,
}

/// A lockstep run over a row's cache.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The auto-repeat delays the Session ran under, read from the first record.
    pub auto_repeat: AutoRepeat,
    /// The decoded frames whose auto-repeat words differ from the first record's: nonzero
    /// means the delays are State, not rules.
    pub auto_repeat_changes: u64,
    /// Every compared Tick, in frame order.
    pub steps: Vec<Step>,
    /// The pairs not compared, by reason, in the order each reason first occurred.
    pub skipped: Vec<(Skip, u64)>,
    /// The frames that don't decode, by kind.
    pub undecodable: BTreeMap<ErrorKind, u64>,
}

impl Report {
    /// The Ticks compared.
    #[must_use]
    pub const fn compared(&self) -> usize {
        self.steps.len()
    }

    /// The Ticks whose compared fields all agree.
    #[must_use]
    pub fn equal(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.differences.is_empty())
            .count()
    }

    /// The first differing field's path, counted over the Ticks.
    #[must_use]
    pub fn first_differences(&self) -> BTreeMap<String, u64> {
        let mut histogram = BTreeMap::new();
        for difference in self.steps.iter().filter_map(Step::first) {
            let count: &mut u64 = histogram.entry(path(difference)).or_default();
            *count = count.saturating_add(1);
        }
        histogram
    }

    /// Every compared field over the run, in the coverage table's order.
    #[must_use]
    pub fn fields(&self) -> Vec<FieldSummary> {
        COMPARATORS
            .iter()
            .filter(|comparator| decoded(comparator.path))
            .map(|comparator| {
                let mut summary = FieldSummary {
                    path: comparator.path,
                    compared: 0,
                    differed: 0,
                    first: None,
                };
                for step in &self.steps {
                    summary.compared = summary.compared.saturating_add(1);
                    let hit = step
                        .differences
                        .iter()
                        .find(|difference| within(difference, comparator.path));
                    if let Some(difference) = hit {
                        summary.differed = summary.differed.saturating_add(1);
                        if summary.first.is_none() {
                            summary.first = Some((step.frame, difference.clone()));
                        }
                    }
                }
                summary
            })
            .collect()
    }
}

/// A difference's path down to its field, without element indices: `video.memory.vram`.
fn path(difference: &Difference) -> String {
    let names: Vec<&str> = difference
        .path()
        .iter()
        .filter_map(|segment| match segment {
            Segment::Field(name) | Segment::Variant(name) => Some(*name),
            Segment::Index(_) | Segment::Len => None,
        })
        .collect();
    names.join(".")
}

/// Whether `difference` lies in the field at `field`.
fn within(difference: &Difference, field: &str) -> bool {
    let names = field.split('.');
    let count = names.clone().count();
    difference.path().len() >= count
        && difference
            .path()
            .iter()
            .zip(names)
            .all(|(segment, name)| matches!(segment, Segment::Field(field) if *field == name))
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "auto-repeat {} then {} frames ({} frames change them)",
            self.auto_repeat.initial.0, self.auto_repeat.subsequent.0, self.auto_repeat_changes
        )?;
        writeln!(
            f,
            "ticks compared {}, equal {}",
            self.compared(),
            self.equal()
        )?;
        writeln!(f, "undecodable frames {:?}", self.undecodable)?;
        writeln!(f, "pairs skipped {:?}", self.skipped)?;
        writeln!(f, "first difference by field:")?;
        for (field, count) in self.first_differences() {
            writeln!(f, "  {field}: {count}")?;
        }
        writeln!(f, "fields (compared, differed, first frame: difference):")?;
        for summary in self.fields() {
            write!(
                f,
                "  {} {} {}",
                summary.path, summary.compared, summary.differed
            )?;
            match &summary.first {
                Some((frame, difference)) => writeln!(f, " {}: {difference}", frame.0)?,
                None => writeln!(f, " equal")?,
            }
        }
        Ok(())
    }
}

/// How a Tick's NMI arrived, from the cut it ran to. A cut with the NMI still running has no
/// timing yet (SLICE0 B1 status).
const fn timing(cut: CutKind) -> Option<NmiTiming> {
    match cut {
        CutKind::Requested => Some(NmiTiming::OnTime),
        CutKind::Overrun => Some(NmiTiming::Overrun),
        CutKind::VblankWithoutNmi => Some(NmiTiming::Disabled),
        CutKind::NmiStillRunning => None,
    }
}

/// The auto-repeat delays a record's work RAM holds, the Session's rules for a run from it.
///
/// # Errors
///
/// A [`DecodeError`] when the record's work RAM is too short for the two words.
pub fn auto_repeat(record: &FrameRecord) -> Result<AutoRepeat, DecodeError> {
    let read = |word: WramWord| word.read(&record.wram).map(Frames);
    Ok(AutoRepeat {
        initial: read(source::JOYPAD_CTR_REPEAT_FIRST)?,
        subsequent: read(source::JOYPAD_CTR_REPEAT_NEXT)?,
    })
}

/// Why lockstep couldn't run.
#[derive(Debug)]
pub enum LockstepError {
    /// The cache didn't read.
    Cache(CacheError),
    /// The cache holds no frames.
    Empty,
    /// The first record has no auto-repeat words.
    AutoRepeat(DecodeError),
}

impl fmt::Display for LockstepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(error) => write!(f, "{error}"),
            Self::Empty => write!(f, "the cache holds no frames"),
            Self::AutoRepeat(error) => write!(f, "the auto-repeat delays: {error}"),
        }
    }
}

impl std::error::Error for LockstepError {}

impl From<CacheError> for LockstepError {
    fn from(error: CacheError) -> Self {
        Self::Cache(error)
    }
}

/// Lockstep over the Ticks to the frames in `frames` of `cache`.
///
/// The Session's auto-repeat delays come from the first record's work RAM
/// (`JOYPAD_CTR_REPEAT_FIRST`, `JOYPAD_CTR_REPEAT_NEXT`, which boot sets).
///
/// # Errors
///
/// A [`LockstepError`] when the cache doesn't read or its first record has no auto-repeat
/// words.
pub fn lockstep(
    cache: &Cache,
    rooms: &RoomMap,
    content: &Arc<Content>,
    frames: Range<u64>,
) -> Result<Report, LockstepError> {
    let first = cache.entries().first().ok_or(LockstepError::Empty)?;
    let delays = auto_repeat(&cache.record(first)?).map_err(LockstepError::AutoRepeat)?;
    let rules = Rules {
        auto_repeat: delays,
    };
    let mut report = Report {
        auto_repeat: delays,
        auto_repeat_changes: 0,
        steps: Vec::new(),
        skipped: Vec::new(),
        undecodable: BTreeMap::new(),
    };
    let mut previous: Option<Result<Decoded, ErrorKind>> = None;
    for item in cache.records() {
        let (frame, record) = item?;
        let decoded = decode(&record, rooms).map_err(|error| ErrorKind::of(&error));
        match &decoded {
            Ok(_) => {
                if auto_repeat(&record).ok() != Some(delays) {
                    report.auto_repeat_changes = report.auto_repeat_changes.saturating_add(1);
                }
            }
            Err(kind) => count(&mut report.undecodable, *kind),
        }
        let from = previous.replace(decoded.clone());
        if !frames.contains(&frame.0) {
            continue;
        }
        let Some(from) = from else { continue };
        match step(from, decoded, rules, content, CutKind::of(record.cut)) {
            Ok(differences) => report.steps.push(Step { frame, differences }),
            Err(skip) => tally(&mut report.skipped, skip),
        }
    }
    Ok(report)
}

fn tally(counts: &mut Vec<(Skip, u64)>, skip: Skip) {
    match counts.iter_mut().find(|(seen, _)| *seen == skip) {
        Some((_, count)) => *count = count.saturating_add(1),
        None => counts.push((skip, 1)),
    }
}

fn count<K: Ord>(map: &mut BTreeMap<K, u64>, key: K) {
    let entry = map.entry(key).or_default();
    *entry = entry.saturating_add(1);
}

/// One Tick from `from` compared with `to`, the decoded States of frames k and k+1.
fn step(
    from: Result<Decoded, ErrorKind>,
    to: Result<Decoded, ErrorKind>,
    rules: Rules,
    content: &Arc<Content>,
    cut: CutKind,
) -> Result<Vec<Difference>, Skip> {
    let to = to.map_err(Skip::Undecodable)?;
    let from = from.map_err(Skip::Undecodable)?;
    if from.quiescence == Quiescence::Busy {
        return Err(Skip::BusyStart);
    }
    let timing = timing(cut).ok_or(Skip::NoTiming(cut))?;
    let mut session = Session::open(from.state, rules, Arc::clone(content)).map_err(Skip::Open)?;
    session
        .step(TickInput {
            pad: to.state.controller.auto_read,
            timing,
        })
        .map_err(Skip::Fault)?;
    Ok(differences(session.state(), &to.state, to.quiescence))
}

#[cfg(test)]
mod tests {
    use super::{COMPARATORS, decoded, differences, within};
    use crate::coverage::{FIELDS, Status};
    use crate::decode::Quiescence;
    use game::state::State;
    use game::state::{FrameCount, slots::Frames};

    #[test]
    fn every_decoded_field_has_a_comparator() {
        for field in FIELDS {
            if matches!(field.status, Status::Decoded(_)) {
                assert!(
                    COMPARATORS.iter().any(|c| c.path == field.path),
                    "{} has no comparator",
                    field.path
                );
            }
        }
        for comparator in COMPARATORS {
            assert!(
                FIELDS.iter().any(|field| field.path == comparator.path),
                "{} isn't in the coverage table",
                comparator.path
            );
        }
    }

    #[test]
    fn differences_name_every_differing_field_in_order() {
        let oracle = State::default();
        let mut engine = State::default();
        engine.clock.counters.lag = FrameCount(3);
        engine.controller.repeat_delay = Frames(9);
        engine.rng.calls = 5;
        let found = differences(&engine, &oracle, Quiescence::Quiet);
        assert_eq!(found.len(), 2, "rng.calls isn't compared");
        assert!(within(&found[0], "clock.counters.lag"));
        assert!(within(&found[1], "controller.repeat_delay"));
        assert!(decoded("clock.counters.lag"));
        assert!(!decoded("rng.calls"));
    }

    #[test]
    fn a_busy_cut_compares_no_strand() {
        let oracle = State::default();
        let mut engine = State::default();
        engine.main.frames.clear();
        assert_eq!(differences(&engine, &oracle, Quiescence::Busy).len(), 0);
        assert_eq!(differences(&engine, &oracle, Quiescence::Quiet).len(), 1);
    }
}
