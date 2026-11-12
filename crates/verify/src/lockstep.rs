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
use std::sync::{Arc, LazyLock};

use canonical::{Canonical, Difference, Segment};
use data::assets::world::{Back, Door, World};
use data::id::RoomId;
use data::plm::{PlmPopulation, ScrollLists};
use data::rom::Lorom;
use data::room::RoomKey;
use game::run_progress_bridge;
use game::state::atmospheric::AtmosphericGraphic;
use game::state::projectile::ProjectileBody;
use game::state::room::{
    ENEMY_SLOTS, Elevator, EnemyBody, FaceBlock, ItemGfxSlot, ItemStep, MorphBallEye, PLM_SLOTS,
    PlmBody, ReactionPlace, ScrollPlm, ScrollStep, UnmodelledEnemy,
};
use game::state::slots::{Frames, Occupancy};
use game::state::{NmiTiming, State};
use game::{AutoRepeat, Content, OpenError, Rules, Session, SessionFault, Switches, TickInput};
use import::world::WorldError;
use oracle_record::{Cut, FrameRecord};
use run_progress::Progress;

use crate::cache::{Cache, CacheError, CutKind};
use crate::coverage::{FIELDS, Status};
use crate::decode::{
    DecodeError, Decoded, Quiescence, RoomHeader, RoomMap, StateHeader, StateLevel, Unsupported,
    decode,
};
use crate::enemy::AiVariables;
use crate::lock::Frame;
use crate::source::{self, WramWord};

/// The rooms the engine loads and the species their enemies are.
///
/// The rooms have the IDs the room registry gives them (`import::world::LOADED`: the Morph
/// Ball Room and Ceres's six), the species theirs (`data::enemy::SpeciesMap::slice_zero`, so
/// an enemy of a species slice 0 doesn't hold, such as Ceres's doors, still doesn't decode).
/// It holds no enemy programs, tiles definitions or PLM populations:
/// [`RoomMap::with_programs`], [`RoomMap::with_samus_tiles`] and [`RoomMap::with_plms`] add
/// them.
#[must_use]
pub fn loaded_rooms() -> RoomMap {
    RoomMap {
        rooms: import::world::LOADED
            .iter()
            .filter_map(|&id| {
                let pointer = import::world::key(id)?.room_pointer()?;
                Some((RoomHeader(pointer), id))
            })
            .collect(),
        species: data::enemy::SpeciesMap::slice_zero(),
        programs: data::enemy::program::EnemyPrograms::default(),
        samus_tiles: data::pose::tiles::SamusTiles::default(),
        samus_clips: game::samus_appearance_bridge::Clips::default(),
        plms: Vec::new(),
        scroll_lists: Vec::new(),
        states: Vec::new(),
        reactions: data::blocks::Reactions::default(),
    }
}

/// The rooms the engine loads, imported from the ROM `image`.
///
/// # Errors
///
/// [`WorldError`] if a room doesn't import.
pub fn loaded_world(image: &[u8]) -> Result<World, WorldError> {
    import::world::import(Lorom::new(image), &import::world::LOADED)
}

/// What each version of each room of `world` loads that RAM doesn't keep, from its record,
/// by its state header's pointer, which the importer says for the ROM `image`.
///
/// # Errors
///
/// [`WorldError`] if a room doesn't decode from `image`.
pub fn loaded_states(
    world: &World,
    image: &[u8],
) -> Result<Vec<(StateHeader, StateLevel)>, WorldError> {
    let mut states = Vec::new();
    for room in world.rooms() {
        let places = import::world::version_places(Lorom::new(image), room.id)?;
        states.extend(places.iter().enumerate().filter_map(|(place, held)| {
            let version = room.version(place)?;
            // A state header sits in the room headers' bank, which a room key names.
            let pointer = RoomKey(held.state).room_pointer()?;
            Some((
                StateHeader(pointer),
                StateLevel {
                    tileset: version.tileset,
                    layer2: version.back == Back::Level,
                },
            ))
        }));
    }
    Ok(states)
}

/// The PLM population of each room of `world`, as it loads on a new file entered by no door:
/// the version the rows play in.
#[must_use]
pub fn loaded_plms(world: &World) -> Vec<(RoomId, PlmPopulation)> {
    world
        .rooms()
        .filter_map(|room| {
            let version = run_progress_bridge::room_version(room, &Progress::default(), None)?;
            Some((room.id, version.plms.clone()))
        })
        .collect()
}

/// Where the scroll change lists are that the PLMs of [`loaded_plms`] name, room by room:
/// the importer's, for the room arguments the work RAM holds (K1).
///
/// # Errors
///
/// [`WorldError`] if a room doesn't decode from `image`.
pub fn loaded_scroll_lists(
    world: &World,
    image: &[u8],
) -> Result<Vec<(RoomId, ScrollLists)>, WorldError> {
    let mut lists = Vec::new();
    for room in world.rooms() {
        let Some(place) = run_progress_bridge::version_place(room, &Progress::default(), None)
        else {
            continue;
        };
        let mut places = import::world::version_places(Lorom::new(image), room.id)?;
        if place < places.len() {
            lists.push((room.id, places.swap_remove(place).scroll_lists));
        }
    }
    Ok(lists)
}

/// The door list of each room of `world`.
#[must_use]
pub fn loaded_doors(world: &World) -> Vec<(RoomId, Vec<Door>)> {
    world
        .rooms()
        .map(|room| (room.id, room.doors.clone()))
        .collect()
}

/// One compared field: its coverage path and how to compare it.
#[derive(Clone, Copy)]
struct Comparator {
    path: &'static str,
    compare: fn(&State, &State) -> Option<Difference>,
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

/// A comparator for one common field of every PLM slot, compared slot by slot.
macro_rules! plm {
    ($path:literal, $($name:ident).+) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| state.room.plms.slots.map(|slot| slot.common.$($name).+);
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// A comparator for one field of every atmospheric graphic slot, absent in a free slot.
macro_rules! atmospheric {
    ($path:literal, $part:expr) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| state.atmospherics.slots.map(|slot| slot.map($part));
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// A comparator for one part of every PLM slot's body, absent where `$part` finds none.
macro_rules! plm_body {
    ($path:literal, $part:expr) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| {
                    state.room.plms.slots.map(|slot| match slot.occupancy {
                        Occupancy::Free => None,
                        Occupancy::Live(body) | Occupancy::Deleting(body) => $part(body),
                    })
                };
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// A comparator for one part of every projectile slot's body, absent in a free slot.
macro_rules! projectile {
    ($path:literal, $part:expr) => {
        Comparator {
            path: $path,
            compare: |engine: &State, oracle: &State| {
                let part = |state: &State| {
                    state
                        .projectiles
                        .slots
                        .slots
                        .map(|slot| match slot.occupancy {
                            Occupancy::Free => None,
                            Occupancy::Live(body) | Occupancy::Deleting(body) => Some($part(body)),
                        })
                };
                part(engine).diff(&part(oracle))
            },
        }
    };
}

/// Each PLM slot's occupancy and its body's kind, without the parts their own rows compare.
fn plm_occupant(state: &State) -> [Occupancy<PlmBody>; PLM_SLOTS] {
    let quiet = |body: PlmBody| match body {
        PlmBody::Scroll(_) => PlmBody::Scroll(ScrollPlm::default()),
        PlmBody::ScrollTouch => PlmBody::ScrollTouch,
        PlmBody::Item { item, .. } => PlmBody::Item {
            item,
            collected_bit: 0,
            step: ItemStep::Spawned,
            slot: ItemGfxSlot::default(),
        },
        PlmBody::ItemCollision => PlmBody::ItemCollision,
        PlmBody::Held => PlmBody::Held,
        PlmBody::Unmodelled(unmodelled) => PlmBody::Unmodelled(unmodelled),
        PlmBody::Reaction(running) => PlmBody::Reaction(ReactionPlace {
            next: 0,
            grows_back_as: None,
            ..running
        }),
    };
    state.room.plms.slots.map(|slot| match slot.occupancy {
        Occupancy::Free => Occupancy::Free,
        Occupancy::Live(body) => Occupancy::Live(quiet(body)),
        Occupancy::Deleting(body) => Occupancy::Deleting(quiet(body)),
    })
}

/// A scroll PLM's own part.
const fn scroll(body: PlmBody) -> Option<ScrollPlm> {
    match body {
        PlmBody::Scroll(scroll) => Some(scroll),
        PlmBody::ScrollTouch
        | PlmBody::Item { .. }
        | PlmBody::ItemCollision
        | PlmBody::Held
        | PlmBody::Reaction(_)
        | PlmBody::Unmodelled(_) => None,
    }
}

/// Where a PLM is in its instruction list, for the kinds that keep a step.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum PlmStep {
    /// A scroll PLM's.
    Scroll(ScrollStep),
    /// An item's.
    Item(ItemStep),
}

/// A PLM's step.
const fn plm_step(body: PlmBody) -> Option<PlmStep> {
    match body {
        PlmBody::Scroll(scroll) => Some(PlmStep::Scroll(scroll.step)),
        PlmBody::Item { step, .. } => Some(PlmStep::Item(step)),
        PlmBody::ScrollTouch
        | PlmBody::ItemCollision
        | PlmBody::Held
        | PlmBody::Reaction(_)
        | PlmBody::Unmodelled(_) => None,
    }
}

/// An item's collected bit.
const fn collected_bit(body: PlmBody) -> Option<u16> {
    match body {
        PlmBody::Item { collected_bit, .. } => Some(collected_bit),
        PlmBody::Scroll(_)
        | PlmBody::ScrollTouch
        | PlmBody::ItemCollision
        | PlmBody::Held
        | PlmBody::Reaction(_)
        | PlmBody::Unmodelled(_) => None,
    }
}

/// A reaction's place: its reaction and script, its next step and what its cell grows back as.
const fn reaction(body: PlmBody) -> Option<ReactionPlace> {
    match body {
        PlmBody::Reaction(running) => Some(running),
        PlmBody::Scroll(_)
        | PlmBody::ScrollTouch
        | PlmBody::Item { .. }
        | PlmBody::ItemCollision
        | PlmBody::Held
        | PlmBody::Unmodelled(_) => None,
    }
}

/// Each enemy slot's occupancy without its AI words, which their own row compares.
fn occupant(state: &State) -> [Occupancy<EnemyBody>; ENEMY_SLOTS] {
    let quiet = |body: EnemyBody| match body {
        EnemyBody::MorphBallEye(_) => EnemyBody::MorphBallEye(MorphBallEye::default()),
        EnemyBody::Elevator(_) => EnemyBody::Elevator(Elevator::default()),
        EnemyBody::FaceBlock(_) => EnemyBody::FaceBlock(FaceBlock::default()),
        EnemyBody::Unmodelled(unmodelled) => EnemyBody::Unmodelled(UnmodelledEnemy {
            variables: [0; 6],
            ..unmodelled
        }),
        // A door's State is its body: its AI words are constant.
        EnemyBody::CeresDoor(door) => EnemyBody::CeresDoor(door),
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
            EnemyBody::MorphBallEye(eye) => crate::enemy::eye_words(eye),
            EnemyBody::FaceBlock(block) => crate::enemy::face_block_words(block),
            EnemyBody::Elevator(elevator) => crate::enemy::elevator_words(elevator),
            EnemyBody::Unmodelled(unmodelled) => crate::enemy::unmodelled_words(unmodelled),
            EnemyBody::CeresDoor(_) => crate::enemy::from_words(crate::enemy::ceres_door::WORDS),
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
    field!("video.playfield.main_screen", video.playfield.main_screen),
    field!("video.playfield.bg3_tilemap", video.playfield.bg3_tilemap),
    field!("video.message_box.tilemap", video.message_box.tilemap),
    field!(
        "video.message_box.bg3_tilemap",
        video.message_box.bg3_tilemap
    ),
    field!(
        "video.playfield.colour_math.select",
        video.playfield.colour_math.select
    ),
    field!(
        "video.playfield.colour_math.layers",
        video.playfield.colour_math.layers
    ),
    field!(
        "video.batch.playfield_colour_math.select",
        video.batch.playfield_colour_math.select
    ),
    field!(
        "video.batch.playfield_colour_math.layers",
        video.batch.playfield_colour_math.layers
    ),
    field!("video.batch.samus_tiles", video.batch.samus_tiles),
    field!("video.batch.samus_tiles.top", video.batch.samus_tiles.top),
    field!(
        "video.batch.samus_tiles.bottom",
        video.batch.samus_tiles.bottom
    ),
    field!(
        "video.batch.level.bg1.column.left",
        video.batch.level.bg1.column.left
    ),
    field!(
        "video.batch.level.bg1.column.right",
        video.batch.level.bg1.column.right
    ),
    field!(
        "video.batch.level.bg1.column.first.to",
        video.batch.level.bg1.column.first.to
    ),
    field!(
        "video.batch.level.bg1.column.first.bytes",
        video.batch.level.bg1.column.first.bytes
    ),
    field!(
        "video.batch.level.bg1.column.wrapped.to",
        video.batch.level.bg1.column.wrapped.to
    ),
    field!(
        "video.batch.level.bg1.column.wrapped.bytes",
        video.batch.level.bg1.column.wrapped.bytes
    ),
    field!(
        "video.batch.level.bg1.column.requests",
        video.batch.level.bg1.column.requests
    ),
    field!(
        "video.batch.level.bg1.row.top",
        video.batch.level.bg1.row.top
    ),
    field!(
        "video.batch.level.bg1.row.bottom",
        video.batch.level.bg1.row.bottom
    ),
    field!(
        "video.batch.level.bg1.row.first.to",
        video.batch.level.bg1.row.first.to
    ),
    field!(
        "video.batch.level.bg1.row.first.bytes",
        video.batch.level.bg1.row.first.bytes
    ),
    field!(
        "video.batch.level.bg1.row.wrapped.to",
        video.batch.level.bg1.row.wrapped.to
    ),
    field!(
        "video.batch.level.bg1.row.wrapped.bytes",
        video.batch.level.bg1.row.wrapped.bytes
    ),
    field!(
        "video.batch.level.bg1.row.requests",
        video.batch.level.bg1.row.requests
    ),
    field!(
        "video.batch.level.bg2.column.left",
        video.batch.level.bg2.column.left
    ),
    field!(
        "video.batch.level.bg2.column.right",
        video.batch.level.bg2.column.right
    ),
    field!(
        "video.batch.level.bg2.column.first.to",
        video.batch.level.bg2.column.first.to
    ),
    field!(
        "video.batch.level.bg2.column.first.bytes",
        video.batch.level.bg2.column.first.bytes
    ),
    field!(
        "video.batch.level.bg2.column.wrapped.to",
        video.batch.level.bg2.column.wrapped.to
    ),
    field!(
        "video.batch.level.bg2.column.wrapped.bytes",
        video.batch.level.bg2.column.wrapped.bytes
    ),
    field!(
        "video.batch.level.bg2.column.requests",
        video.batch.level.bg2.column.requests
    ),
    field!(
        "video.batch.level.bg2.row.top",
        video.batch.level.bg2.row.top
    ),
    field!(
        "video.batch.level.bg2.row.bottom",
        video.batch.level.bg2.row.bottom
    ),
    field!(
        "video.batch.level.bg2.row.first.to",
        video.batch.level.bg2.row.first.to
    ),
    field!(
        "video.batch.level.bg2.row.first.bytes",
        video.batch.level.bg2.row.first.bytes
    ),
    field!(
        "video.batch.level.bg2.row.wrapped.to",
        video.batch.level.bg2.row.wrapped.to
    ),
    field!(
        "video.batch.level.bg2.row.wrapped.bytes",
        video.batch.level.bg2.row.wrapped.bytes
    ),
    field!(
        "video.batch.level.bg2.row.requests",
        video.batch.level.bg2.row.requests
    ),
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
    // Samus's motion State, the motion spec's declared fields (`samus-movement.md` §1.1).
    field!("samus.motion.position.x", samus.motion.position.x),
    field!("samus.motion.position.y", samus.motion.position.y),
    field!("samus.motion.base_speed", samus.motion.base_speed),
    field!("samus.motion.extra_run", samus.motion.extra_run),
    field!("samus.motion.dashing", samus.motion.dashing),
    field!("samus.motion.mode", samus.motion.mode),
    field!("samus.motion.vertical_speed", samus.motion.vertical_speed),
    field!("samus.motion.vertical", samus.motion.vertical),
    field!("samus.motion.bounce", samus.motion.bounce),
    field!(
        "samus.previous_position.x.pixels",
        samus.previous_position.x.pixels
    ),
    field!(
        "samus.previous_position.x.subpixels",
        samus.previous_position.x.subpixels
    ),
    field!(
        "samus.previous_position.y.pixels",
        samus.previous_position.y.pixels
    ),
    field!(
        "samus.previous_position.y.subpixels",
        samus.previous_position.y.subpixels
    ),
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
    field!("samus.equipment.collected", samus.equipment.collected),
    field!("samus.animation.step", samus.animation.step),
    field!("samus.animation.frames_left", samus.animation.frames_left),
    field!("samus.animation.boost.stage", samus.animation.boost.stage),
    field!(
        "samus.animation.boost.laps_left",
        samus.animation.boost.laps_left
    ),
    field!("samus.visor.shade", samus.visor.shade),
    field!("samus.visor.timer", samus.visor.timer),
    field!("samus.drawing", samus.drawing),
    field!("samus.y_radius", samus.y_radius),
    field!("samus.slope_adjusted", samus.slope_adjusted),
    field!("samus.jump_held.frames", samus.jump_held.frames),
    field!("samus.jump_held.before", samus.jump_held.before),
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
    field!("room.height", room.height),
    field!("room.layer2", room.layer2),
    field!("room.level_tail", room.level_tail),
    field!("room.area", room.area),
    field!("room.tileset", room.tileset),
    field!("room.tilemap_gap", room.tilemap_gap),
    field!("room.layer_blending", room.layer_blending),
    field!("room.scroll_areas", room.scroll_areas),
    field!("room.scrolls_below", room.scrolls_below),
    field!("room.camera.x.pixels", room.camera.x.pixels),
    field!("room.camera.x.subpixels", room.camera.x.subpixels),
    field!("room.camera.y.pixels", room.camera.y.pixels),
    field!("room.camera.y.subpixels", room.camera.y.subpixels),
    field!("room.camera.layer2.x", room.camera.layer2.x),
    field!("room.camera.layer2.y", room.camera.layer2.y),
    field!("room.camera.bg1_offset.x", room.camera.bg1_offset.x),
    field!("room.camera.bg1_offset.y", room.camera.bg1_offset.y),
    field!("room.camera.bg2_offset.x", room.camera.bg2_offset.x),
    field!("room.camera.bg2_offset.y", room.camera.bg2_offset.y),
    field!("room.camera.distance", room.camera.distance),
    field!("room.camera.scrollers.up", room.camera.scrollers.up),
    field!("room.camera.scrollers.down", room.camera.scrollers.down),
    field!("room.camera.layer2_scroll.x", room.camera.layer2_scroll.x),
    field!("room.camera.layer2_scroll.y", room.camera.layer2_scroll.y),
    field!("room.camera.drawn.layer1.x", room.camera.drawn.layer1.x),
    field!("room.camera.drawn.layer1.y", room.camera.drawn.layer1.y),
    field!("room.camera.drawn.layer2.x", room.camera.drawn.layer2.x),
    field!("room.camera.drawn.layer2.y", room.camera.drawn.layer2.y),
    plm!("room.plms.slots.common.block", block),
    plm!("room.plms.slots.common.cursor.delay", cursor.delay),
    Comparator {
        path: "room.plms.slots.occupancy",
        compare: |engine: &State, oracle: &State| plm_occupant(engine).diff(&plm_occupant(oracle)),
    },
    plm_body!("room.plms.slots.occupancy.changes", |body| scroll(body)
        .map(|scroll| scroll.changes)),
    plm_body!("room.plms.slots.occupancy.triggered", |body| scroll(body)
        .map(|scroll| scroll.triggered)),
    plm_body!("room.plms.slots.occupancy.step", plm_step),
    plm_body!("room.plms.slots.occupancy.slot", |body| match body {
        PlmBody::Item { slot, .. } => Some(slot),
        PlmBody::Scroll(_)
        | PlmBody::ScrollTouch
        | PlmBody::ItemCollision
        | PlmBody::Held
        | PlmBody::Reaction(_)
        | PlmBody::Unmodelled(_) => None,
    }),
    plm_body!("room.plms.slots.occupancy.collected_bit", collected_bit),
    plm_body!("room.plms.slots.occupancy.reaction", reaction),
    field!("room.item_gfx.next", room.item_gfx.next),
    field!("room.item_gfx.loaded", room.item_gfx.loaded),
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
    enemy!("room.enemies.slots.common.cursor.at", cursor.at),
    enemy!("room.enemies.slots.common.cursor.delay", cursor.delay),
    enemy!("room.enemies.slots.common.spritemap", spritemap),
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
    atmospheric!("atmospherics.slots", |graphic: AtmosphericGraphic| graphic
        .kind),
    atmospheric!("atmospherics.slots.frame", |graphic: AtmosphericGraphic| {
        graphic.frame
    }),
    atmospheric!("atmospherics.slots.timer", |graphic: AtmosphericGraphic| {
        graphic.timer
    }),
    atmospheric!("atmospherics.slots.x", |graphic: AtmosphericGraphic| {
        graphic.x
    }),
    atmospheric!("atmospherics.slots.y", |graphic: AtmosphericGraphic| {
        graphic.y
    }),
    field!("elevator.status", elevator.status),
    field!("elevator.armed", elevator.armed),
    field!("elevator.direction", elevator.direction),
    field!("room.drawing.hook", room.drawing.hook),
    field!("enemy_palette_cycle.palette", enemy_palette_cycle.palette),
    field!("enemy_palette_cycle.set", enemy_palette_cycle.set),
    field!("enemy_palette_cycle.timer", enemy_palette_cycle.timer),
    field!("hud.tilemap", hud.tilemap),
    field!("hud.selected", hud.selected),
    field!("hud.auto_cancel", hud.auto_cancel),
    field!("hud.drawn.energy", hud.drawn.energy),
    field!("hud.drawn.missiles", hud.drawn.missiles),
    field!("hud.drawn.super_missiles", hud.drawn.super_missiles),
    field!("hud.drawn.power_bombs", hud.drawn.power_bombs),
    field!("hud.drawn.selection", hud.drawn.selection),
    field!("hud.mini_map.explored", hud.mini_map.explored),
    field!("hud.mini_map.area", hud.mini_map.area),
    field!("hud.mini_map.room.x", hud.mini_map.room.x),
    field!("hud.mini_map.room.y", hud.mini_map.room.y),
    field!("hud.mini_map.map_collected", hud.mini_map.map_collected),
    field!("hud.mini_map.disabled", hud.mini_map.disabled),
    field!("message", message),
    field!("ceres", ceres),
    field!("collected_items", collected_items),
    field!("sram", sram),
    field!("projectiles.count", projectiles.count),
    field!("projectiles.cooldown", projectiles.cooldown),
    Comparator {
        path: "projectiles.slots.common.trail_timer",
        compare: |engine: &State, oracle: &State| {
            let part = |state: &State| {
                state
                    .projectiles
                    .slots
                    .slots
                    .map(|slot| slot.common.trail_timer)
            };
            part(engine).diff(&part(oracle))
        },
    },
    projectile!("projectiles.slots.occupancy", |_: ProjectileBody| true),
    projectile!(
        "projectiles.slots.occupancy.position.x.pixels",
        |body: ProjectileBody| body.position.x.pixels
    ),
    projectile!(
        "projectiles.slots.occupancy.position.x.subpixels",
        |body: ProjectileBody| body.position.x.subpixels
    ),
    projectile!(
        "projectiles.slots.occupancy.position.y.pixels",
        |body: ProjectileBody| body.position.y.pixels
    ),
    projectile!(
        "projectiles.slots.occupancy.position.y.subpixels",
        |body: ProjectileBody| body.position.y.subpixels
    ),
    projectile!(
        "projectiles.slots.occupancy.radius.x",
        |body: ProjectileBody| body.radius.x
    ),
    projectile!(
        "projectiles.slots.occupancy.radius.y",
        |body: ProjectileBody| body.radius.y
    ),
    projectile!(
        "projectiles.slots.occupancy.velocity.x",
        |body: ProjectileBody| body.velocity.x
    ),
    projectile!(
        "projectiles.slots.occupancy.velocity.y",
        |body: ProjectileBody| body.velocity.y
    ),
    projectile!(
        "projectiles.slots.occupancy.heading",
        |body: ProjectileBody| body.heading
    ),
    projectile!(
        "projectiles.slots.occupancy.kind",
        |body: ProjectileBody| body.kind
    ),
    projectile!(
        "projectiles.slots.occupancy.damage",
        |body: ProjectileBody| body.damage
    ),
    projectile!(
        "projectiles.slots.occupancy.frame_timer",
        |body: ProjectileBody| body.frame_timer
    ),
];

/// Whether the coverage table marks `path` decoded.
fn decoded(path: &str) -> bool {
    FIELDS
        .iter()
        .any(|field| field.path == path && matches!(field.status, Status::Decoded(_)))
}

/// The comparators of the fields the coverage table marks decoded, in its order. Both tables
/// are constant, so the search by path runs once rather than on every Tick.
static DECODED: LazyLock<Vec<&'static Comparator>> = LazyLock::new(|| {
    COMPARATORS
        .iter()
        .filter(|comparator| decoded(comparator.path))
        .collect()
});

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
pub(crate) fn differences(
    engine: &State,
    oracle: &State,
    quiescence: Quiescence,
) -> Vec<Difference> {
    DECODED
        .iter()
        .filter(|comparator| compared_at(comparator.path, quiescence))
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
    /// [`DecodeError::UnknownRoomState`].
    UnknownRoomState,
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
            DecodeError::UnknownRoomState(_) => Self::UnknownRoomState,
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
        DECODED
            .iter()
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
/// timing yet.
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
        switches: Switches::REFERENCE,
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
    for item in cache.records_without_pictures() {
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
        // The Session takes frame k's State and frame k+1's is kept for the next Tick, so
        // neither is copied.
        let from = previous.take();
        if let Some(from) = from.filter(|_| frames.contains(&frame.0)) {
            match step(from, &decoded, rules, content, CutKind::of(record.cut)) {
                Ok(differences) => report.steps.push(Step { frame, differences }),
                Err(skip) => tally(&mut report.skipped, skip),
            }
        }
        previous = Some(decoded);
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
    to: &Result<Decoded, ErrorKind>,
    rules: Rules,
    content: &Arc<Content>,
    cut: CutKind,
) -> Result<Vec<Difference>, Skip> {
    let to = to.as_ref().map_err(|kind| Skip::Undecodable(*kind))?;
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

    /// The records' tileset and layer 2, by state header, are what the decoder reads from the
    /// ROM for every state of every loaded room, and the scroll change lists are the default
    /// version's, which every row plays.
    #[test]
    fn each_loaded_state_reads_its_record_as_the_decoder_reads_the_rom() {
        use super::{loaded_scroll_lists, loaded_states, loaded_world};
        use crate::decode::StateHeader;
        use data::rom::Lorom;
        use data::room::RoomKey;
        use import::tileset::TilesetIndex;
        let Some(image) = std::env::var_os("SM_ROM").and_then(|path| std::fs::read(path).ok())
        else {
            return;
        };
        let world = loaded_world(&image).unwrap();
        let mut decoded = Vec::new();
        let mut lists = Vec::new();
        for id in import::world::LOADED {
            let key = import::world::key(id).unwrap();
            let room = data::room::decode(Lorom::new(&image), key).unwrap();
            decoded.extend(room.states().map(|(_, state)| {
                (
                    StateHeader(RoomKey(state.at).room_pointer().unwrap()),
                    Some(TilesetIndex(state.tileset)),
                    state.layers.layer2.is_some(),
                )
            }));
            lists.push((id, room.default.scroll_lists.clone()));
        }
        let loaded: Vec<_> = loaded_states(&world, &image)
            .unwrap()
            .into_iter()
            .map(|(header, level)| {
                (
                    header,
                    import::world::tileset_index(level.tileset),
                    level.layer2,
                )
            })
            .collect();
        assert_eq!(loaded, decoded);
        assert_eq!(loaded_scroll_lists(&world, &image).unwrap(), lists);
    }
}

#[cfg(test)]
mod recorded;
