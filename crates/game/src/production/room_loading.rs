//! The room's loading: what the passage's calls load of the next room, from our own room,
//! tileset, sprite and species formats.
//!
//! The passage says when each call's work is done; this says what it loads. The room that
//! loads is the one the doorway leads to, in the version progress chooses as she comes in
//! through it. Building it ([`room`]) gives what play reads of her room: what her moves meet,
//! its doors to other rooms by cell, and the camera's room.

use super::connected::inputs::{CameraInputs, Lead, Screen};
use super::played::passage::{ActorFetch, Doorway, Room};
use super::shaft_entry;
use super::world::{Boundary, MovementWorld};
use boss_ceres_ridley::{
    CeresDoor, CeresRidley, ColourSet, DoorColours, DoorKind, DoorPalette, GoalColours,
    RidleyPlacing, Stage, Vent, VentFacing,
};
use core::num::Wrapping;
use data::assets::actors::{self, Actors, SpriteRow};
use data::assets::art::{Colour, ROW_COLOURS};
use data::assets::samus::palettes::SamusPalettes;
use data::assets::tilesets::Tilesets;
use data::assets::world::{self, CeresDoorKind, CeresVentFacing, Door, ItemLighting, World};
use data::id::DoorwayId;
use data::level::slope::{Shape, Slopes};
use data::level::{BlockKind, PlmChoice, ScrollArea};
use data::plm::PlmKind;
use data::room::Area;
use run_progress::{EnteredRoom, Progress, RoomStates, Version};
use types::{BossKind, Item, Pair, Px, RandomSequence, Side};
use world_doors::{COLOURS, Palette, RoomCall};

/// The rooms, tilesets and sprite colours a passage loads from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rooms {
    /// The rooms and the doorways between them.
    pub world: World,
    /// The tilesets their versions draw with.
    pub tilesets: Tilesets,
    /// The colours of the sprites other than hers.
    pub actors: Actors,
    /// Her suit's and her shots' colours.
    pub samus: SamusPalettes,
    /// The slopes' shapes, which a slope block names.
    pub slopes: Slopes,
}

/// What a room's loading reads of the run.
#[derive(Clone, Copy, Debug)]
pub struct Run<'a> {
    /// Her progress, which chooses the next room's version, her suit and her beams, and says
    /// whether the station's boss is beaten.
    pub progress: &'a Progress,
    /// The Ceres station's stage, whose colour set the doors a room places take.
    pub ceres: Stage,
}

impl Run<'_> {
    /// Whether the station's boss counts as beaten: Ceres's main boss is defeated.
    #[must_use]
    pub fn ridley_beaten(&self) -> bool {
        self.progress.defeated[types::Area::Ceres][BossKind::MainBoss]
    }
}

/// Why the next room did not load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadFault {
    /// The doorway or the room it leads to is not in the world.
    Room(DoorwayId),
    /// The room is in an area progress has no place for.
    Area(Area),
    /// The room's version draws with a tileset that is not loaded.
    Tileset,
    /// A colour of the content has more than 15 bits.
    Colour,
    /// No palette is for the beams she has switched on.
    Beams,
    /// The version loads a species that has no palette of its own.
    Species(data::id::SpeciesId),
    /// The version places an actor with no owner in the room loader.
    OtherActors,
    /// A steam vent attached to the turning shaft.
    TurningShaftVent,
    /// The version places a PLM whose blocks no owner here places.
    Plm,

    /// A slope block names a shape the slopes don't hold.
    Slope,
    /// The new game's start room, or the version it starts in, is not in the world.
    Start,
    /// The level doesn't make geometry: its size or a chain of stand-ins.
    Geometry,
}

/// The screen's first sprite row: the sprite rows follow the eight background rows.
const SPRITE_ROW: usize = 8;

/// What the room's loading leaves of the screen, once the work `call` names is done.
///
/// Fetching the tiles sets the colours the fade in heads for to the tileset's, from the
/// screen's first colour on; the colours past the tileset's keep the goal they had. Fetching
/// the contents sets the sprite rows': [`contents`].
///
/// # Errors
/// A doorway, room, tileset, species or beam palette the content lacks leaves the colours as
/// they were.
pub fn load(
    call: Option<RoomCall>,
    through: DoorwayId,
    run: Run<'_>,
    rooms: &Rooms,
    palette: &mut Palette,
) -> Result<(), LoadFault> {
    let progress = run.progress;
    match call {
        Some(RoomCall::FetchContents) => {
            fetch_palette(through, run, rooms, palette, ActorFetch::Place)
        }
        Some(RoomCall::FetchTiles) => {
            let version = version(rooms, through, progress)?;
            let tileset = rooms
                .tilesets
                .get(version.tileset)
                .ok_or(LoadFault::Tileset)?;
            let mut goal = palette.goal;
            let colours = tileset.palettes.0.iter().flat_map(|row| row.0);
            for (goal, colour) in goal.iter_mut().zip(colours) {
                *goal = types::Colour::new(colour.0).ok_or(LoadFault::Colour)?;
            }
            palette.goal = goal;
            Ok(())
        }
        None
        | Some(
            RoomCall::LeaveRoom
            | RoomCall::DrawTopRow
            | RoomCall::BuildRoom
            | RoomCall::AnimateTiles
            | RoomCall::PlayMusic
            | RoomCall::ReadLiquid,
        ) => Ok(()),
    }
}

/// The Ceres actors a room places as its contents are fetched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CeresActors {
    /// Ceres Ridley, its first enemy, as his placing leaves him and what it hands on.
    pub ridley: Option<RidleyPlacing>,
    /// Its Ceres doors, in its order of its enemies.
    pub doors: Vec<CeresDoor>,
    /// Its steam vents, in enemy slot order.
    pub vents: Vec<Vent>,
    /// Population slots for the vents and doors during retirement.
    pub slots: ActorSlots,
    /// Population slots of vents mounted to the shaft's affine view.
    pub mounted_vents: Vec<usize>,
    /// The placed overlay's anchor in room pixels.
    pub shaft_overlay: Option<Pair<Px>>,
}

/// The active actors' original population indices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActorSlots {
    /// Slot of each vent in the same order as its state.
    pub vents: Vec<usize>,
    /// Slot of each door in the same order as its state.
    pub doors: Vec<usize>,
}

/// What a version's start places of the station's actors, in its order of its enemies: Ridley
/// first, then the doors, each with the colours its start sets.
struct Placed {
    ridley: Option<RidleyPlacing>,
    doors: Vec<CeresDoor>,
    /// The doors' colours each door actor's start sets, the shaft's overlay's among them.
    door_colours: Vec<DoorColours>,
    shaft_overlay: Option<Pair<Px>>,
}

/// The station's actors `version` places, as its contents are fetched (`RidleyPlaced`,
/// `RoomStart`, `TheDoorsColours`). The shaft's overlay is an actor of the doors' species:
/// its start sets the doors' colours as a door's does.
fn placed(version: &world::Version, run: Run<'_>) -> Placed {
    let ridley = version
        .ceres_ridley
        .then(|| CeresRidley::place(run.ridley_beaten()))
        .flatten();
    let mut placed = Placed {
        ridley,
        doors: Vec::new(),
        door_colours: Vec::new(),
        shaft_overlay: None,
    };
    // His placing sets the stage the doors after him read.
    let stage = ridley.map_or(run.ceres, |ridley| ridley.stage);
    for door in &version.ceres_doors {
        let (kind, facing) = match door.kind {
            CeresDoorKind::Door(facing) => (DoorKind::Door, facing),
            CeresDoorKind::GetawayWall(side) => (DoorKind::GetawayWall, side),
            CeresDoorKind::RidleyDoor => (DoorKind::RidleyDoor, Side::Right),
            CeresDoorKind::ShaftDoor => (DoorKind::ShaftDoor, Side::Left),
            CeresDoorKind::ShaftOverlay => {
                placed.shaft_overlay = Some(door.position);
                (DoorKind::Door, Side::Left)
            }
        };
        let placement = CeresDoor::place(kind, facing, door.position, stage);
        placed.door_colours.push(placement.colours);
        if door.kind != CeresDoorKind::ShaftOverlay {
            placed.doors.push(placement.door);
        }
    }
    placed
}

/// The station actors the room `through` leads to places as its contents are fetched.
///
/// Ridley places before the doors because his placing can change the stage they read. Each
/// steam vent draws once from `random`, in slot order. Doorway and room setup belong to the
/// preceding static contents installation.
///
/// # Errors
/// A missing room or unowned actor leaves `random`
/// unchanged.
pub fn ceres_actors(
    through: DoorwayId,
    run: Run<'_>,
    rooms: &Rooms,
    random: &mut RandomSequence,
) -> Result<CeresActors, LoadFault> {
    let version = version(rooms, through, run.progress)?;
    actors_of(version, run, false, random)
}

/// Prepare the escape shaft population from the built room's entry.
///
/// # Errors
/// The mounted population requires the active escape view and overlay. An error leaves
/// the random sequence unchanged.
pub fn ceres_shaft_actors(
    through: DoorwayId,
    run: Run<'_>,
    rooms: &Rooms,
    view: &shaft_entry::View,
    random: &mut RandomSequence,
) -> Result<CeresActors, LoadFault> {
    let version = version(rooms, through, run.progress)?;
    let mounts = run.ceres == Stage::Escape && view.affine;
    actors_of(version, run, mounts, random)
}

/// The entry selected independently by the incoming doorway and room version.
///
/// # Errors
/// The door or its selected version must exist in the imported world.
pub fn setup(
    rooms: &Rooms,
    through: DoorwayId,
    progress: &Progress,
) -> Result<shaft_entry::Setup, LoadFault> {
    Ok(shaft_entry::Setup {
        door: rooms
            .world
            .doorway(through)
            .ok_or(LoadFault::Room(through))?
            .rule,
        room: version(rooms, through, progress)?.setup,
        ridley_beaten: progress.defeated[types::Area::Ceres][BossKind::MainBoss],
    })
}

/// The Ceres actors `version` places, refusing other actors.
fn actors_of(
    version: &world::Version,
    run: Run<'_>,
    mount_vents: bool,
    random: &mut RandomSequence,
) -> Result<CeresActors, LoadFault> {
    let placed = placed(version, run);
    let owned = |species| {
        species == actors::CERES_DOOR || (version.ceres_ridley && species == actors::CERES_RIDLEY)
    };
    if !version.enemies.iter().all(|loaded| owned(loaded.species)) {
        return Err(LoadFault::OtherActors);
    }
    let mut drawn = *random;
    let mut vents = Vec::with_capacity(version.ceres_vents.len());
    let mut mounted_vents = Vec::new();
    for vent in &version.ceres_vents {
        let (facing, mounted) = match vent.facing {
            CeresVentFacing::Up => (VentFacing::Up, false),
            CeresVentFacing::Left => (VentFacing::Left, false),
            CeresVentFacing::Down => (VentFacing::Down, false),
            CeresVentFacing::Right => (VentFacing::Right, false),
            CeresVentFacing::ShaftLeft => (VentFacing::Left, true),
            CeresVentFacing::ShaftRight => (VentFacing::Right, true),
        };
        if mounted {
            if !mount_vents || placed.shaft_overlay.is_none() {
                return Err(LoadFault::TurningShaftVent);
            }
            mounted_vents.push(vent.slot);
        }
        game_random::advance_random(&mut drawn);
        vents.push(Vent::place(facing, vent.position, drawn.number.0));
    }
    *random = drawn;
    Ok(CeresActors {
        vents,
        mounted_vents,
        slots: ActorSlots {
            vents: version.ceres_vents.iter().map(|vent| vent.slot).collect(),
            doors: version
                .ceres_doors
                .iter()
                .filter(|door| door.kind != CeresDoorKind::ShaftOverlay)
                .map(|door| door.slot)
                .collect(),
        },
        ridley: placed.ridley,
        doors: placed.doors,
        shaft_overlay: placed.shaft_overlay,
    })
}

/// The room `through` leads to, as building it leaves it for play, with the scroll areas past
/// its screens kept from `before`.
///
/// What her moves meet is the version's level, with the blocks its scroll PLMs and extensions
/// place as the room starts, and no actor's box: fetching its contents places no actor but the
/// Ceres doors ([`ceres_actors`]), which play carries itself. The original places those blocks as
/// the contents are fetched; nothing reads her room between. Its doors to other rooms are the
/// door blocks of that level; a door block lets her through when its door leads to a room. The
/// camera's room is the version's scroll areas, the room's size and its scrollers, and building
/// it turns the camera's lead back to ahead. The following contents fetch applies the doorway rule and room setup to this built room.
///
/// # Errors
/// A doorway or room the content lacks, a version placing PLMs no owner here places, or a level
/// that doesn't make geometry builds none.
pub fn room(
    through: DoorwayId,
    run: Run<'_>,
    rooms: &Rooms,
    before: &CameraInputs,
) -> Result<Room, LoadFault> {
    let version = version(rooms, through, run.progress)?;
    let room = entered(rooms, through)?;
    built(room, Some(through), version, rooms, before)
}

/// What play reads of `room` built in `version`, with the scroll areas past its screens kept
/// from `before`.
fn built(
    room: &world::Room,
    entered_through: Option<DoorwayId>,
    version: &world::Version,
    rooms: &Rooms,
    before: &CameraInputs,
) -> Result<Room, LoadFault> {
    let unplaced = |kind| matches!(kind, PlmKind::Item(..) | PlmKind::Unmodelled { .. });
    if version
        .plms
        .entries
        .iter()
        .any(|entry| unplaced(entry.kind))
    {
        return Err(LoadFault::Plm);
    }
    let width = room.size.blocks_across();
    let mut level = version.level.clone();
    for (index, kind) in version.plms.placed_blocks(width) {
        if let Some(slot) = level.get_mut(usize::from(index)) {
            slot.kind = kind;
        }
    }
    let mut screens: Vec<Screen> = version.scrolls.iter().copied().map(screen).collect();
    screens.extend(before.screens.iter().skip(screens.len()));
    let kinds: Vec<_> = level.iter().map(|block| block.kind).collect();
    let door_cells = kinds.iter().zip(0_u16..).filter_map(|(kind, at)| {
        let BlockKind::Door(door) = kind else {
            return None;
        };
        Some((cell(at, width), door.place()))
    });
    let leads = |place| matches!(room.doors.get(place), Some(Door::To(_)));
    let (cells, lets_through) = blocks(&kinds, width, &rooms.slopes, leads)?;
    Ok(Room {
        id: room.id,
        area: room.place.area,
        entered_through,
        tileset: Some(version.tileset),
        entry: shaft_entry::State {
            width,
            cells: level,
            ..shaft_entry::State::default()
        },
        geometry: MovementWorld::new(width, cells, lets_through, Vec::new(), Boundary::Air)
            .map_err(|_| LoadFault::Geometry)?,
        doorways: doorways(&rooms.world, room, door_cells)?,
        camera: CameraInputs {
            screens,
            size: [room.size.width, room.size.height].map(u16::from),
            heights: [room.scrollers.up, room.scrollers.down].map(u16::from),
            lead: Lead::Ahead,
        },
    })
}

/// The camera's screen for a scroll area: the camera tests only for red and blue.
const fn screen(area: ScrollArea) -> Screen {
    match area {
        ScrollArea::Red => Screen::Closed,
        ScrollArea::Blue => Screen::Short,
        ScrollArea::Green | ScrollArea::Unknown(_) => Screen::Open,
    }
}

/// The cell at `index` of a room `width` blocks across, row-major.
fn cell(index: u16, width: u16) -> types::Cell {
    types::Cell {
        x: index.checked_rem(width).unwrap_or(index),
        y: index.checked_div(width).unwrap_or(0),
    }
}

/// What her moves meet of each block of a level of `kinds`, `width` blocks across, row-major.
///
/// Each also says whether it lets her through whichever way she goes: the speed blocks' special
/// block alone does. A door block lets her through when `leads` says its door, by its place in
/// the room's doors, leads to a room.
///
/// # Errors
/// A slope block naming a shape `slopes` lacks.
pub fn blocks(
    kinds: &[BlockKind],
    width: u16,
    slopes: &Slopes,
    leads: impl Fn(usize) -> bool,
) -> Result<(Vec<types::Block>, Vec<bool>), LoadFault> {
    let cells = kinds
        .iter()
        .zip(0_u16..)
        .map(|(&kind, index)| block(kind, slopes, index, width, &leads))
        .collect::<Result<_, _>>()?;
    let lets_through = kinds
        .iter()
        .map(|kind| matches!(kind, BlockKind::Special(PlmChoice::Common(0x45))))
        .collect();
    Ok((cells, lets_through))
}

/// What her moves meet of a block of `kind` at `index`.
fn block(
    kind: BlockKind,
    slopes: &Slopes,
    index: u16,
    width: u16,
    leads: impl Fn(usize) -> bool,
) -> Result<types::Block, LoadFault> {
    Ok(match kind {
        BlockKind::Air(_)
        | BlockKind::UnusedAir(_)
        | BlockKind::SpikeAir(_)
        | BlockKind::ShootableAir(_)
        | BlockKind::BombableAir(_)
        | BlockKind::SpecialAir(_) => types::Block::Air,
        BlockKind::Solid(_)
        | BlockKind::Spike(_)
        | BlockKind::Shootable(_)
        | BlockKind::Grapple(_) => types::Block::Solid,
        BlockKind::Bombable(_) | BlockKind::Special(_) => types::Block::Answered,
        BlockKind::Door(door) => {
            if leads(door.place()) {
                types::Block::Air
            } else {
                types::Block::Solid
            }
        }
        BlockKind::HorizontalExtension(_) | BlockKind::VerticalExtension(_) => {
            types::Block::StandsFor(kind.stands_for(index, width).map(|at| cell(at, width)))
        }
        BlockKind::Slope(slope) => match slopes.shape(slope.shape).ok_or(LoadFault::Slope)? {
            Shape::Square(square) => types::Block::Stepped(core::array::from_fn(|row| {
                core::array::from_fn(|column| {
                    square.solid_at((column != 0) ^ slope.x_flip, (row != 0) ^ slope.y_flip)
                })
            })),
            Shape::Surface(surface) => {
                let mut tops = surface.tops;
                if slope.x_flip {
                    tops.reverse();
                }
                types::Block::Slope(types::Slope {
                    faces: if slope.y_flip {
                        types::SlopeFaces::Ceiling
                    } else {
                        types::SlopeFaces::Floor
                    },
                    depth: tops.map(|depth| Wrapping(u16::from(depth.min(17)))),
                    factor: types::SlopeFactor(surface.factor),
                })
            }
        },
    })
}

/// The doors to other rooms of `room`, by the cell of each of its door blocks `door_cells`.
///
/// Each door cell comes with the block's place in the room's doors. A door gives the doorway it
/// leads through, where she arrives, whether either room keeps the items dark, and where the
/// next room sits on her map when it is in the same area. A door that leads nowhere is no door
/// to another room.
///
/// # Errors
/// A doorway or room the world lacks.
pub fn doorways(
    world: &World,
    room: &world::Room,
    door_cells: impl IntoIterator<Item = (types::Cell, usize)>,
) -> Result<Vec<Doorway>, LoadFault> {
    let mut doorways = Vec::new();
    for (cell, place) in door_cells {
        let Some(&Door::To(through)) = room.doors.get(place) else {
            continue;
        };
        let doorway = world.doorway(through).ok_or(LoadFault::Room(through))?;
        let next = world.room(doorway.room).ok_or(LoadFault::Room(through))?;
        doorways.push(Doorway {
            cell,
            through,
            arrival: world_doors::Arrival {
                way: doorway.arrival.way,
                view: types::Pair {
                    x: doorway.arrival.view.x.pixels(),
                    y: doorway.arrival.view.y.pixels(),
                },
                speed: doorway.arrival.speed,
            },
            items_dark: [room, next]
                .iter()
                .any(|room| room.item_lighting == ItemLighting::Dark),
            map_room: (next.place.area == room.place.area).then(|| map_room(next)),
        });
    }
    Ok(doorways)
}

/// Where `room` sits on her map, as the HUD reads it: its row counts from the row above its
/// first cell.
#[must_use]
pub fn map_room(room: &world::Room) -> interface_hud::Room {
    interface_hud::Room {
        cell: types::Pair {
            x: i32::from(room.place.x),
            y: i32::from(room.place.y).saturating_add(1),
        },
        blocks: types::Pair {
            x: room.size.blocks_across(),
            y: u16::from(room.size.height).wrapping_mul(16),
        },
    }
}

/// What a new game's start loads of the room it starts in, `room`: what play reads of it, the
/// Ceres actors it places, and the colours the fade in heads for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Started {
    /// What play reads of the room.
    pub room: Room,
    /// Its Ceres actors.
    pub actors: CeresActors,
    /// The colours a fade heads for.
    pub goal: [types::Colour; COLOURS],
    /// The selected version’s writes into the cold-start enemy tile sheet.
    pub enemy_tiles: Vec<actors::EnemyTilePatch>,
    /// Where the room sits on her map, and its area.
    pub map: (interface_hud::Room, Area),
}

/// A new game's start in `room`, entered by no door, with the start's own colours for her row
/// `her_row`.
///
/// The colours the fade heads for are black, then the tileset's from the first colour, then
/// what fetching the room's contents sets, but for her suit's row: as the game starts, that
/// takes the start's own (`her_row`), not her suit's.
///
/// # Errors
/// A room the world lacks, or what a passage's loading of it refuses.
pub fn start(
    room: data::id::RoomId,
    run: Run<'_>,
    rooms: &Rooms,
    her_row: &[Colour; ROW_COLOURS],
    random: &mut RandomSequence,
) -> Result<Started, LoadFault> {
    let room = rooms.world.room(room).ok_or(LoadFault::Start)?;
    let version = version_of(room, None, run.progress)?.ok_or(LoadFault::Start)?;
    let mut palette = Palette {
        screen: [types::Colour::BLACK; COLOURS],
        goal: [types::Colour::BLACK; COLOURS],
    };
    let tileset = rooms
        .tilesets
        .get(version.tileset)
        .ok_or(LoadFault::Tileset)?;
    let colours = tileset.palettes.0.iter().flat_map(|row| row.0);
    for (goal, colour) in palette.goal.iter_mut().zip(colours) {
        *goal = types::Colour::new(colour.0).ok_or(LoadFault::Colour)?;
    }
    contents(version, run, rooms, &mut palette, ActorFetch::Place)?;
    if let Some(row) = suit_row(&mut palette.goal) {
        for (goal, colour) in row.iter_mut().zip(her_row) {
            *goal = types::Colour::new(colour.0).ok_or(LoadFault::Colour)?;
        }
    }
    Ok(Started {
        room: built(
            room,
            None,
            version,
            rooms,
            &CameraInputs {
                screens: Vec::new(),
                size: [0, 0],
                heights: [0, 0],
                lead: Lead::Ahead,
            },
        )?,
        actors: actors_of(version, run, false, random)?,
        goal: palette.goal,
        enemy_tiles: version.starting_enemy_tiles.clone(),
        map: (map_room(room), room.place.area),
    })
}

/// Her suit's row of `colours`, the screen's or the ones they head for.
pub(crate) fn suit_row(
    colours: &mut [types::Colour; COLOURS],
) -> Option<&mut [types::Colour; ROW_COLOURS]> {
    colours
        .as_chunks_mut::<ROW_COLOURS>()
        .0
        .iter_mut()
        .skip(SPRITE_ROW)
        .nth(usize::from(SpriteRow::SUIT.get()))
}

/// The room `through` leads to.
fn entered(rooms: &Rooms, through: DoorwayId) -> Result<&world::Room, LoadFault> {
    rooms
        .world
        .doorway(through)
        .and_then(|doorway| rooms.world.room(doorway.room))
        .ok_or(LoadFault::Room(through))
}

/// Publish the portion of the destination colours reached by the fetch.
pub(super) fn fetch_palette(
    through: DoorwayId,
    run: Run<'_>,
    rooms: &Rooms,
    palette: &mut Palette,
    fetched: ActorFetch,
) -> Result<(), LoadFault> {
    contents(
        version(rooms, through, run.progress)?,
        run,
        rooms,
        palette,
        fetched,
    )
}

/// Fetching the contents, in the original's order, so a later row wins where two share one:
/// her shots' palette for the beams switched on, shown at once as well as headed for; the
/// flashing enemies' white and the common sprites'; each species' the version loads, in its
/// row; for a version that places the Ceres doors, the stage's colour set in colours 1 to 15
/// of their row; and her suit's, the gravity suit's over the varia suit's over the power
/// suit's.
fn contents(
    version: &world::Version,
    run: Run<'_>,
    rooms: &Rooms,
    palette: &mut Palette,
    fetched: ActorFetch,
) -> Result<(), LoadFault> {
    let inventory = &run.progress.inventory;
    let mut screen = palette.screen;
    let mut goal = palette.goal;
    let beam = rooms
        .samus
        .beam(inventory.beams_switched_on)
        .ok_or(LoadFault::Beams)?;
    set(&mut screen, SpriteRow::BEAM, 0, &beam.0)?;
    set(&mut goal, SpriteRow::BEAM, 0, &beam.0)?;
    set(&mut goal, SpriteRow::FLASH, 0, &rooms.actors.flash.0)?;
    set(&mut goal, SpriteRow::COMMON, 0, &rooms.actors.common.0)?;
    if !matches!(fetched, ActorFetch::Clearing(_)) {
        for loaded in &version.enemies {
            let own = rooms
                .actors
                .species(loaded.species)
                .and_then(|species| species.palette)
                .ok_or(LoadFault::Species(loaded.species))?;
            set(&mut goal, loaded.row, 0, &own.0)?;
        }
    }
    if fetched == ActorFetch::Place {
        let placed = placed(version, run);
        if let Some(ridley) = placed.ridley {
            room_colours(ridley, &rooms.actors.ceres_ridley, &mut goal)?;
        }
        for colours in placed.door_colours {
            door_goal(colours, &rooms.actors, &mut goal)?;
        }
        set(&mut goal, SpriteRow::SUIT, 0, &suit(rooms, inventory).0)?;
    }
    palette.screen = screen;
    palette.goal = goal;
    Ok(())
}

/// Her suit's colours as it stands: the gravity suit's over the varia suit's over the power
/// suit's.
#[must_use]
pub fn suit<'a>(
    rooms: &'a Rooms,
    inventory: &run_progress::Inventory,
) -> &'a data::assets::art::Palette {
    let suits = &rooms.samus.suits;
    let on = &inventory.items_switched_on;
    if on[Item::GravitySuit] {
        &suits.gravity
    } else if on[Item::VariaSuit] {
        &suits.varia
    } else {
        &suits.power
    }
}

/// `RidleyPlaced`: his room's colours, the goal the passage's fade brings the screen to. Each
/// palette takes his room's content, or turns black but for its first colour.
fn room_colours(
    ridley: RidleyPlacing,
    content: &actors::CeresRidleyColours,
    goal: &mut [types::Colour; COLOURS],
) -> Result<(), LoadFault> {
    const BLACK: [Colour; ROW_COLOURS] = [Colour(0); ROW_COLOURS];
    let colours = ridley.colours;
    for (palette, row, own) in [
        (colours.baby, SpriteRow::BABY, &content.baby.0),
        (colours.door, SpriteRow::RIDLEY_DOOR, &content.door.0),
        (colours.ridley, SpriteRow::CERES_DOORS, &BLACK),
    ] {
        match palette {
            GoalColours::RoomContent => set(goal, row, 0, own)?,
            GoalColours::BlackButFirst => set(goal, row, 1, &BLACK)?,
        }
    }
    Ok(())
}

/// Set the goal colours selected by a door's placement.
pub(super) fn door_goal(
    colours: DoorColours,
    actors: &Actors,
    goal: &mut [types::Colour; COLOURS],
) -> Result<(), LoadFault> {
    let shades = match colours.set {
        ColourSet::Calm => &actors.ceres_doors.calm,
        ColourSet::Escape => &actors.ceres_doors.escape,
    };
    let row = match colours.palette {
        DoorPalette::Doors => SpriteRow::CERES_DOORS,
        DoorPalette::Own => SpriteRow::RIDLEY_DOOR,
    };
    set(goal, row, 1, shades)
}

/// Set `colours` in sprite row `row` of `screen`, from its colour `from` on.
fn set(
    screen: &mut [types::Colour; COLOURS],
    row: SpriteRow,
    from: usize,
    colours: &[Colour],
) -> Result<(), LoadFault> {
    let rows = screen.as_chunks_mut::<ROW_COLOURS>().0.iter_mut();
    let Some(target) = rows.skip(SPRITE_ROW).nth(usize::from(row.get())) else {
        return Ok(());
    };
    for (place, colour) in target.iter_mut().skip(from).zip(colours) {
        *place = types::Colour::new(colour.0).ok_or(LoadFault::Colour)?;
    }
    Ok(())
}

/// The Mode 7 sheet loaded when the return passage fetches the escape shaft's tiles.
pub(super) fn shaft_sheet<'a>(
    rooms: &'a Rooms,
    through: DoorwayId,
    progress: &Progress,
) -> Result<Option<&'a data::assets::art::TileSheet>, LoadFault> {
    let version = version(rooms, through, progress)?;
    if !matches!(version.setup, world::RoomSetup::CeresHazeDoorShut { .. }) {
        return Ok(None);
    }
    rooms
        .tilesets
        .get(version.tileset)
        .map(|tileset| Some(&tileset.sheet))
        .ok_or(LoadFault::Tileset)
}

/// Select the graphics of the room version entered, including a fresh start.
///
/// # Errors
/// The room must be in an area progress tracks.
pub fn tileset_of(
    room: &world::Room,
    entrance: Option<DoorwayId>,
    progress: &Progress,
) -> Result<Option<data::id::TilesetId>, LoadFault> {
    version_of(room, entrance.map(|door| types::Entrance(door.0)), progress)
        .map(|version| version.map(|version| version.tileset))
}

/// The version of the room `through` leads to that loads: the first whose condition holds for
/// `progress` as she comes in through it, or the default.
fn version<'a>(
    rooms: &'a Rooms,
    through: DoorwayId,
    progress: &Progress,
) -> Result<&'a world::Version, LoadFault> {
    let room = entered(rooms, through)?;
    version_of(room, Some(types::Entrance(through.0)), progress)?.ok_or(LoadFault::Room(through))
}

/// The version of `room` that loads as she comes in by `entered_by`, or with no entrance as a
/// game starts in it: the first whose condition holds for `progress`, or the default; none for
/// a version the room lacks.
///
/// # Errors
/// A room in an area progress has no place for.
fn version_of<'a>(
    room: &'a world::Room,
    entered_by: Option<types::Entrance>,
    progress: &Progress,
) -> Result<Option<&'a world::Version>, LoadFault> {
    let conditions: Vec<_> = room
        .conditional
        .iter()
        .map(|entry| condition(entry.condition))
        .collect();
    let entered = EnteredRoom {
        area: area(room.place.area)?,
        entered_by,
    };
    Ok(
        match (RoomStates {
            conditional: &conditions,
        })
        .choose(progress, entered)
        {
            Version::Conditional(place) => room.version(place),
            Version::Default => Some(&room.default),
        },
    )
}

/// A version's condition in progress's terms; a doorway is the entrance of the same number.
const fn condition(condition: world::Condition) -> run_progress::Condition {
    use run_progress::Condition as Progress;
    match condition {
        world::Condition::EnteredBy(doorway) => Progress::EnteredBy(types::Entrance(doorway.0)),
        world::Condition::Happened(event) => Progress::Happened(event),
        world::Condition::Defeated(kinds) => Progress::Defeated(kinds),
        world::Condition::Has(item) => Progress::Has(item),
        world::Condition::MorphBallAndMissiles => Progress::MorphBallAndMissiles,
        world::Condition::PowerBombs => Progress::PowerBombs,
    }
}

/// A map area in progress's terms; the debug rooms' area has none.
pub(super) const fn area(area: Area) -> Result<types::Area, LoadFault> {
    Ok(match area {
        Area::Crateria => types::Area::Crateria,
        Area::Brinstar => types::Area::Brinstar,
        Area::Norfair => types::Area::Norfair,
        Area::WreckedShip => types::Area::WreckedShip,
        Area::Maridia => types::Area::Maridia,
        Area::Tourian => types::Area::Tourian,
        Area::Ceres => types::Area::Ceres,
        Area::Debug => return Err(LoadFault::Area(area)),
    })
}

#[cfg(test)]
mod tests {
    use super::{LoadFault, Rooms, Run, load, room};
    use crate::production::connected::inputs::{CameraInputs, Lead, Screen};
    use core::num::Wrapping;
    use data::assets::actors::{
        Actors, CERES_DOOR, CERES_RIDLEY, CeresDoorColours, CeresRidleyColours, EnemyGraphics,
        Species, SpriteRow,
    };
    use data::assets::art::{Colour, Palette as Row, PaletteSet};
    use data::assets::samus::palettes::{BeamPalette, SamusPalettes, Suits};
    use data::assets::tilesets::{Tileset, Tilesets};
    use data::assets::world::{
        Arrival, CeresDoorKind, Conditional, Door, DoorRule, Doorway, Heading, ItemLighting,
        MapPlace, Pair, Room, RoomSetup, ScreenCoordinate, Version, World,
    };
    use data::id::{DoorwayId, RoomId, SpeciesId, TilesetId};
    use data::level::{Block, ScrollArea};
    use data::room::{Area, Screens, Scrollers};
    use types::Side;
    use world_doors::{COLOURS, Palette, RoomCall};

    const fn test_random() -> types::RandomSequence {
        types::RandomSequence {
            number: types::RandomNumber(0),
        }
    }

    const ROOM: RoomId = RoomId(1);
    /// The doorway the room's conditional version is entered by, and another into it.
    const CHOSEN: DoorwayId = DoorwayId(1);
    const OTHER: DoorwayId = DoorwayId(2);

    /// A version drawing from `tileset`: a level of air but for a solid block at (1, 0), a door
    /// block of the room's first door at (1, 1) and of a door it lacks at (2, 1); the first
    /// screen short.
    fn version(tileset: u16) -> Version {
        let block = |at: usize| match at {
            1 => Block::new(0x8000, 0),
            17 => Block::new(0x9000, 0),
            18 => Block::new(0x9000, 5),
            _ => Block::default(),
        };
        Version {
            level: (0..256).map(block).collect(),
            scrolls: vec![ScrollArea::Blue],
            tileset: TilesetId(tileset),
            back: data::assets::world::Back::Level,
            plms: data::plm::PlmPopulation::default(),
            enemies: vec![EnemyGraphics {
                species: CERES_DOOR,
                row: SpriteRow::new(2).unwrap(),
            }],
            starting_enemy_tiles: Vec::new(),
            ceres_doors: vec![placed(CeresDoorKind::Door(Side::Left))],
            ceres_ridley: false,
            ceres_vents: Vec::new(),
            setup: RoomSetup::CeresHaze,
        }
    }

    /// A Ceres actor of `kind`, placed at an invented position.
    fn placed(kind: CeresDoorKind) -> data::assets::world::CeresDoorPlacement {
        data::assets::world::CeresDoorPlacement {
            slot: 0,
            kind,
            position: types::Pair {
                x: Wrapping(128),
                y: Wrapping(64),
            },
        }
    }

    fn tileset(id: u16, bits: u16) -> Tileset {
        Tileset {
            id: TilesetId(id),
            palettes: PaletteSet(vec![Row([Colour(bits); 16])]),
            ..Tileset::default()
        }
    }

    fn rooms() -> Rooms {
        rooms_of(version(1), version(0))
    }

    /// The rooms of one room whose version entered by [`CHOSEN`] is `chosen`.
    fn rooms_of(chosen: Version, default: Version) -> Rooms {
        let doorway = |id| Doorway {
            id,
            room: ROOM,
            arrival: Arrival {
                way: Heading::Right,
                view: Pair {
                    x: ScreenCoordinate(0),
                    y: ScreenCoordinate(0),
                },
                speed: None,
            },
            rule: DoorRule::Nothing,
        };
        let room = Room {
            id: ROOM,
            place: MapPlace {
                area: Area::Ceres,
                x: 0,
                y: 0,
            },
            size: Screens {
                width: 1,
                height: 1,
            },
            scrollers: Scrollers {
                up: 0x70,
                down: 0xa0,
            },
            item_lighting: ItemLighting::Lit,
            doors: vec![Door::To(CHOSEN), Door::To(OTHER)],
            conditional: vec![Conditional {
                condition: data::assets::world::Condition::EnteredBy(CHOSEN),
                version: chosen,
            }],
            default,
        };
        Rooms {
            world: World::new(vec![room], vec![doorway(CHOSEN), doorway(OTHER)]).unwrap(),
            tilesets: Tilesets {
                sets: vec![tileset(0, 0x0011), tileset(1, 0x0022)],
                ..Tilesets::default()
            },
            actors: Actors {
                species: (0..=CERES_RIDLEY.0)
                    .map(|id| Species {
                        id: SpeciesId(id),
                        palette: Some(Row([Colour(0x0100 | id); 16])),
                    })
                    .collect(),
                common: Row([Colour(0x0055); 16]),
                flash: Row([Colour(0x0044); 16]),
                ceres_doors: CeresDoorColours {
                    calm: [Colour(0x0066); 15],
                    escape: [Colour(0x0077); 15],
                },
                ceres_ridley: CeresRidleyColours {
                    door: Row([Colour(0x00c0); 16]),
                    baby: Row([Colour(0x00c1); 16]),
                    rest: [Colour(0x00c2); 15],
                    glow1: [Colour(0x00c3); 15],
                    glow2: [Colour(0x00c4); 15],
                    glow3: [Colour(0x00c5); 15],
                    ..CeresRidleyColours::default()
                },
                ..Actors::default()
            },
            slopes: data::level::slope::Slopes::default(),
            samus: SamusPalettes {
                suits: Suits {
                    power: Row([Colour(0x0088); 16]),
                    varia: Row([Colour(0x0099); 16]),
                    gravity: Row([Colour(0x00aa); 16]),
                },
                hurt: Row([Colour(0x00ab); 16]),
                cinematic_grey: Row([Colour(0x00ac); 16]),
                beams: vec![BeamPalette {
                    beams: types::Set::default(),
                    palette: Row([Colour(0x00bb); 16]),
                }],
            },
        }
    }

    fn lit(bits: u16) -> [types::Colour; COLOURS] {
        [types::Colour::new(bits).unwrap(); COLOURS]
    }

    fn loaded(call: RoomCall, through: DoorwayId) -> Result<Palette, LoadFault> {
        loaded_with(call, through, &run_progress::Progress::default())
    }

    fn loaded_with(
        call: RoomCall,
        through: DoorwayId,
        progress: &run_progress::Progress,
    ) -> Result<Palette, LoadFault> {
        loaded_from(call, through, progress, &rooms())
    }

    fn loaded_from(
        call: RoomCall,
        through: DoorwayId,
        progress: &run_progress::Progress,
        rooms: &Rooms,
    ) -> Result<Palette, LoadFault> {
        let mut palette = Palette {
            screen: lit(0),
            goal: lit(0x0033),
        };
        load(
            Some(call),
            through,
            Run {
                progress,
                ceres: boss_ceres_ridley::Stage::Calm,
            },
            rooms,
            &mut palette,
        )
        .map(|()| palette)
    }

    /// Sprite row `row`'s colours.
    fn row(colours: &[types::Colour; COLOURS], row: usize) -> &[types::Colour] {
        &colours.as_chunks::<16>().0[8..][row]
    }

    #[test]
    fn fetching_the_contents_sets_the_sprite_rows_and_shows_her_shots_colours() {
        let palette = loaded(RoomCall::FetchContents, CHOSEN).unwrap();
        let colours = |bits| [types::Colour::new(bits).unwrap(); 16];
        assert_eq!(row(&palette.screen, 6), colours(0x00bb));
        assert_eq!(palette.screen[..224], lit(0)[..224]);
        assert_eq!(palette.screen[240..], lit(0)[240..]);
        let goal = &palette.goal;
        assert_eq!(goal[..128], lit(0x0033)[..128]);
        assert_eq!(row(goal, 0), colours(0x0044));
        assert_eq!(row(goal, 1), colours(0x0033));
        assert_eq!(row(goal, 2), colours(0x0100 | CERES_DOOR.0));
        assert_eq!(row(goal, 4), colours(0x0088));
        assert_eq!(row(goal, 5), colours(0x0055));
        assert_eq!(row(goal, 6), colours(0x00bb));
        assert_eq!(row(goal, 7)[0], colours(0x0033)[0]);
        assert_eq!(row(goal, 7)[1..], colours(0x0066)[1..]);
    }

    #[test]
    fn her_suit_is_the_gravity_suits_over_the_varias_and_beams_with_no_palette_load_nothing() {
        let mut progress = run_progress::Progress::default();
        progress.inventory.items_switched_on[types::Item::VariaSuit] = true;
        let varia = loaded_with(RoomCall::FetchContents, CHOSEN, &progress).unwrap();
        progress.inventory.items_switched_on[types::Item::GravitySuit] = true;
        let gravity = loaded_with(RoomCall::FetchContents, CHOSEN, &progress).unwrap();
        assert_eq!(row(&varia.goal, 4)[0].bits(), 0x0099);
        assert_eq!(row(&gravity.goal, 4)[0].bits(), 0x00aa);
        progress.inventory.beams_switched_on[types::Beam::Charge] = true;
        assert!(loaded_with(RoomCall::FetchContents, CHOSEN, &progress).is_ok());
        progress.inventory.beams_switched_on[types::Beam::Wave] = true;
        assert_eq!(
            loaded_with(RoomCall::FetchContents, CHOSEN, &progress),
            Err(LoadFault::Beams)
        );
    }

    #[test]
    fn fetching_the_tiles_heads_the_colours_for_the_chosen_versions_tileset() {
        let palette = loaded(RoomCall::FetchTiles, CHOSEN).unwrap();
        let (tileset, rest) = palette.goal.split_at(16);
        assert_eq!(tileset, &lit(0x0022)[..16]);
        assert_eq!(rest, &lit(0x0033)[16..]);
        assert_eq!(palette.screen, lit(0));
        let palette = loaded(RoomCall::FetchTiles, OTHER).unwrap();
        assert_eq!(palette.goal[..16], lit(0x0011)[..16]);
    }

    #[test]
    fn a_new_games_start_loads_its_default_version_with_her_row_its_own() {
        let her_row = [Colour(0x0123); 16];
        let started = super::start(
            ROOM,
            Run {
                progress: &run_progress::Progress::default(),
                ceres: boss_ceres_ridley::Stage::Calm,
            },
            &rooms(),
            &her_row,
            &mut test_random(),
        )
        .unwrap();
        let colours = |bits| [types::Colour::new(bits).unwrap(); 16];
        let goal = &started.goal;
        // Entered by no door, the default version's tileset: colours 0 to 15, then black.
        assert_eq!(goal[..16], lit(0x0011)[..16]);
        assert_eq!(goal[16..128], lit(0)[16..128]);
        assert_eq!(row(goal, 0), colours(0x0044));
        assert_eq!(row(goal, 2), colours(0x0100 | CERES_DOOR.0));
        assert_eq!(
            row(goal, 4),
            colours(0x0123),
            "her row is the start's own, not her suit's"
        );
        assert_eq!(row(goal, 5), colours(0x0055));
        assert_eq!(started.actors.doors.len(), 1);
        assert_eq!(started.room.doorways.len(), 1);
        assert_eq!(started.map.1, Area::Ceres);
        assert_eq!(
            super::start(
                RoomId(9),
                Run {
                    progress: &run_progress::Progress::default(),
                    ceres: boss_ceres_ridley::Stage::Calm,
                },
                &rooms(),
                &her_row,
                &mut test_random(),
            )
            .map(|_| ()),
            Err(LoadFault::Start)
        );
    }

    #[test]
    fn the_other_calls_leave_the_colours() {
        let palette = loaded(RoomCall::BuildRoom, CHOSEN).unwrap();
        assert_eq!(palette.goal, lit(0x0033));
    }

    #[test]
    fn a_doorway_outside_the_world_loads_nothing() {
        assert_eq!(
            loaded(RoomCall::FetchTiles, DoorwayId(9)),
            Err(LoadFault::Room(DoorwayId(9)))
        );
    }

    fn built(rooms: &Rooms) -> Result<super::Room, LoadFault> {
        let before = CameraInputs {
            screens: vec![Screen::Closed; 3],
            size: [4, 4],
            heights: [1, 2],
            lead: Lead::Near,
        };
        let progress = run_progress::Progress::default();
        let run = Run {
            progress: &progress,
            ceres: boss_ceres_ridley::Stage::Calm,
        };
        room(CHOSEN, run, rooms, &before)
    }

    #[test]
    fn building_the_room_gives_its_blocks_its_doors_by_cell_and_its_screens() {
        let built = built(&rooms()).unwrap();
        let at = |x, y| types::World::block(&built.geometry, types::Cell { x, y });
        assert_eq!(at(1, 0), types::Block::Solid);
        assert_eq!(at(0, 0), types::Block::Air);
        assert_eq!(
            at(1, 1),
            types::Block::Air,
            "a door to a room lets her through"
        );
        assert_eq!(
            at(2, 1),
            types::Block::Solid,
            "a door the room lacks doesn't"
        );
        let cells: Vec<_> = built
            .doorways
            .iter()
            .map(|door| (door.cell, door.through))
            .collect();
        assert_eq!(cells, [(types::Cell { x: 1, y: 1 }, CHOSEN)]);
        assert_eq!(
            built.camera,
            CameraInputs {
                screens: vec![Screen::Short, Screen::Closed, Screen::Closed],
                size: [1, 1],
                heights: [0x70, 0xa0],
                lead: Lead::Ahead,
            }
        );
    }

    #[test]
    fn building_the_room_turns_the_cameras_lead_back_to_ahead() {
        assert_eq!(built(&rooms()).unwrap().camera.lead, Lead::Ahead);
    }

    /// What the fixture's version places: its one door.
    fn no_actors() -> super::CeresActors {
        let door = |kind, facing| {
            boss_ceres_ridley::CeresDoor::place(
                kind,
                facing,
                placed(CeresDoorKind::ShaftDoor).position,
                boss_ceres_ridley::Stage::Calm,
            )
            .door
        };
        super::CeresActors {
            slots: super::ActorSlots {
                vents: vec![],
                doors: vec![0],
            },
            vents: Vec::new(),
            mounted_vents: Vec::new(),
            ridley: None,
            doors: vec![door(boss_ceres_ridley::DoorKind::Door, Side::Left)],
            shaft_overlay: None,
        }
    }

    /// Ridley's room: Ridley and the door of his room.
    fn ridleys_room() -> Version {
        let mut ridley = version(1);
        ridley.ceres_ridley = true;
        ridley.enemies.push(EnemyGraphics {
            species: CERES_RIDLEY,
            row: SpriteRow::new(1).unwrap(),
        });
        ridley.ceres_doors = vec![placed(CeresDoorKind::RidleyDoor)];
        ridley.setup = RoomSetup::CeresRidleyHaze;
        ridley
    }

    #[test]
    fn ridleys_room_places_him_first_and_the_door_of_his_room_unless_he_is_beaten() {
        fn run(progress: &run_progress::Progress) -> Run<'_> {
            Run {
                progress,
                ceres: boss_ceres_ridley::Stage::Calm,
            }
        }
        let mut progress = run_progress::Progress::default();
        let rooms = rooms_of(ridleys_room(), version(0));
        let actors =
            super::ceres_actors(CHOSEN, run(&progress), &rooms, &mut test_random()).unwrap();
        let placing = actors.ridley.unwrap();
        assert_eq!(placing.stage, boss_ceres_ridley::Stage::Calm);
        assert_eq!(placing.boss, boss_ceres_ridley::Boss::CeresRidley);
        assert_eq!(actors.doors.len(), 1);
        assert_eq!(
            actors.doors[0].kind,
            boss_ceres_ridley::DoorKind::RidleyDoor
        );
        assert_eq!(actors.doors[0].facing, Side::Right);
        progress.defeated[types::Area::Ceres][types::BossKind::MainBoss] = true;
        let beaten =
            super::ceres_actors(CHOSEN, run(&progress), &rooms, &mut test_random()).unwrap();
        assert_eq!(beaten.ridley, None);
        // Ridley's species, loaded where the version doesn't place him, is another actor.
        let mut unplaced = ridleys_room();
        unplaced.ceres_ridley = false;
        assert_eq!(
            super::ceres_actors(
                CHOSEN,
                run(&progress),
                &rooms_of(unplaced, version(0)),
                &mut test_random()
            ),
            Err(LoadFault::OtherActors)
        );
    }

    #[test]
    fn his_placing_sets_his_rooms_colours_and_the_door_of_his_room_takes_its_own_palette() {
        let progress = run_progress::Progress::default();
        let rooms = rooms_of(ridleys_room(), version(0));
        let goal = loaded_from(RoomCall::FetchContents, CHOSEN, &progress, &rooms)
            .unwrap()
            .goal;
        let colours = |bits| [types::Colour::new(bits).unwrap(); 16];
        assert_eq!(row(&goal, 1), colours(0x0100 | CERES_RIDLEY.0));
        // The door's own palette: his room's content, then the doors' calm set over it.
        assert_eq!(row(&goal, 2)[0], colours(0x00c0)[0]);
        assert_eq!(row(&goal, 2)[1..], colours(0x0066)[1..]);
        assert_eq!(row(&goal, 3), colours(0x00c1));
        // The palette he is drawn in, the doors', black but for its first colour.
        assert_eq!(row(&goal, 7)[0], colours(0x0033)[0]);
        assert_eq!(row(&goal, 7)[1..], colours(0)[1..]);
    }

    #[test]
    fn the_shafts_overlay_is_placed_and_sets_the_doors_colours() {
        let progress = run_progress::Progress::default();
        let run = Run {
            progress: &progress,
            ceres: boss_ceres_ridley::Stage::Calm,
        };
        let mut shaft = version(1);
        shaft.ceres_doors = vec![
            placed(CeresDoorKind::ShaftOverlay),
            placed(CeresDoorKind::Door(Side::Left)),
        ];
        let rooms = rooms_of(shaft, version(0));
        let actors = super::ceres_actors(CHOSEN, run, &rooms, &mut test_random()).unwrap();
        assert_eq!(
            actors.shaft_overlay,
            Some(placed(CeresDoorKind::ShaftOverlay).position)
        );
        assert_eq!(actors.doors.len(), 1);
        let mut overlay = version(1);
        overlay.ceres_doors = vec![placed(CeresDoorKind::ShaftOverlay)];
        let goal = loaded_from(
            RoomCall::FetchContents,
            CHOSEN,
            &progress,
            &rooms_of(overlay, version(0)),
        )
        .unwrap()
        .goal;
        assert_eq!(
            row(&goal, 7)[1..],
            [types::Colour::new(0x0066).unwrap(); 15]
        );
    }

    #[test]
    fn a_version_placing_another_actor_places_no_doors() {
        let mut other = version(1);
        other.enemies[0].species = SpeciesId(0);
        let progress = run_progress::Progress::default();
        let run = Run {
            progress: &progress,
            ceres: boss_ceres_ridley::Stage::Calm,
        };
        let rooms = rooms_of(other, version(0));
        assert_eq!(
            super::ceres_actors(CHOSEN, run, &rooms, &mut test_random()),
            Err(LoadFault::OtherActors)
        );
        assert_eq!(
            super::ceres_actors(OTHER, run, &rooms, &mut test_random()),
            Ok(no_actors())
        );
    }

    #[test]
    fn a_version_placing_unplaced_plms_or_a_short_level_builds_no_room() {
        let mut item = version(1);
        item.plms.entries.push(data::plm::PlmEntry {
            kind: data::plm::PlmKind::Item(data::plm::ItemPlm::MorphBall, 0),
            x: 0,
            y: 0,
        });
        assert_eq!(built(&rooms_of(item, version(0))), Err(LoadFault::Plm));
        let mut short = version(1);
        short.level.pop();
        assert_eq!(
            built(&rooms_of(short, version(0))),
            Err(LoadFault::Geometry)
        );
    }
}
