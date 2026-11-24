//! The world in our own format (docs/architecture/assets.md §3.1).
//!
//! The rooms by our [`RoomId`], each with its place on the map, its size, its scrollers, its
//! doors and its versions, the conditional ones in the order they are checked, then the
//! default; and the doorways their doors lead through, by our [`DoorwayId`].
//!
//! `import::world` builds it from the ROM; the engine reads only this. It holds what the
//! engine reads of a room today: a version's level, scroll areas, PLMs, tileset, layer 2, enemy
//! graphics, Ceres doors and setup, and the room's doors and the rule each doorway runs.
//! A record names other content by our ids, never by where the original keeps it.

pub mod doorways;
pub mod rooms;

pub use types::{BossKind, Event, Fixed, Heading, Item, Pair, Px, Set, Side};

use crate::assets::actors::EnemyGraphics;
use crate::id::{DoorwayId, RoomId, TilesetId};
use crate::level::{Block, ScrollArea};
use crate::plm::PlmPopulation;
use crate::room::{Area, Screens, Scrollers};

/// Where a door of a room's door list leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Door {
    /// Through this doorway: a door block of this door lets her through, and her passage to
    /// the doorway's room starts.
    To(DoorwayId),
    /// Nowhere: an elevator's doorway, whose door block is solid to her.
    Nowhere,
}

/// A doorway: one way into a room. Leaving a room through a door is arriving through the
/// door's doorway (D15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Doorway {
    /// Our id.
    pub id: DoorwayId,
    /// The room it leads into.
    pub room: RoomId,
    /// Where the view arrives and how the passage carries her into the room.
    pub arrival: Arrival,
    /// What passing through it runs beyond the passage.
    pub rule: DoorRule,
}

/// What passing through a doorway runs beyond the passage, as the room's contents are fetched.
/// Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorRule {
    /// Into Ceres's elevator room from the hall below it: the screen goes back to its usual
    /// mode, out of the rotating view.
    FromCeresElevatorShaft,
    /// Nothing.
    Nothing,
    /// Into Ceres's elevator room: the screen's rotating view, started level.
    ToCeresElevatorShaft,
}

/// What a version of a room runs as its contents are fetched, beyond placing them. Listed
/// alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomSetup {
    /// The station's haze: a backdrop colour blended over the screen.
    CeresHaze,
    /// The haze, with the elevator's door shut as solid blocks: the escape's elevator room.
    CeresHazeDoorShut,
    /// The haze, with Ridley's room's own tile bases and the floor's screen mode.
    CeresRidleyHaze,
    /// Nothing.
    Nothing,
}

/// A coordinate on a room's grid of 256-pixel screens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenCoordinate(pub u8);

impl ScreenCoordinate {
    /// The screen's leading edge, in pixels.
    #[must_use]
    pub fn pixels(self) -> Px {
        core::num::Wrapping(u16::from(self.0) << 8)
    }
}

/// A doorway's arrival, before the passage adjusts the view for the way she leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrival {
    /// The way she leaves by: the way the view and she move.
    pub way: Heading,
    /// The view's place in the next room, aligned to its grid of screens.
    pub view: Pair<ScreenCoordinate>,
    /// Her speed in pixels and subpixels per frame, or the passage's own speed when absent.
    pub speed: Option<Fixed>,
}

/// What chooses a conditional version.
///
/// It says what progress's own condition (`run_progress::Condition`) says, in the same terms,
/// a doorway where progress has its entrance. They stay two types because neither crate can
/// name the other's: progress is a clean-room model, which depends only on `types`, and `data`
/// may not depend on a model. Conversion belongs to content admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// She came in through this doorway.
    EnteredBy(DoorwayId),
    /// The event has happened.
    Happened(Event),
    /// A boss of any of these kinds is defeated in the room's own area.
    Defeated(Set<BossKind>),
    /// The item is hers.
    Has(Item),
    /// The Morph Ball is hers and her missile capacity isn't 0.
    MorphBallAndMissiles,
    /// Her power bomb capacity isn't 0.
    PowerBombs,
}

/// Where a room sits on its area's map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapPlace {
    /// The area.
    pub area: Area,
    /// The room's left column, in map cells.
    pub x: u8,
    /// The room's top row, in map cells.
    pub y: u8,
}

/// What layer 2, behind the level's front, shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Back {
    /// A tile layer of the level's own.
    Level,
    /// The version's background, not part of the level.
    Background,
}

/// One version of a room: what loads when its condition chooses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    /// Its level's front layer, a block each, row-major from the top left: as many as the
    /// room's screens hold.
    pub level: Vec<Block>,
    /// The camera's scroll areas, a screen each, row-major. A table's run on past the room's
    /// screens with the bytes the original keeps after them; preset areas cover the room's
    /// screens alone, and the run past them stays as the last room left it.
    pub scrolls: Vec<ScrollArea>,
    /// The tileset its blocks draw from.
    pub tileset: TilesetId,
    /// What layer 2 shows.
    pub back: Back,
    /// The PLMs it places, and the scroll changes they make.
    pub plms: PlmPopulation,
    /// The species it loads, in order, each with its sprite palette row.
    pub enemies: Vec<EnemyGraphics>,
    /// The station's door actors it places, in the room's order of its enemies.
    pub ceres_doors: Vec<CeresDoorPlacement>,
    /// It places Ceres Ridley, who stands at his own place whatever his placement says, as
    /// the room's first enemy, before its door actors.
    pub ceres_ridley: bool,
    /// What it runs as its contents are fetched, beyond placing them.
    pub setup: RoomSetup,
}

/// A Ceres door actor as a room's start places it: what it is and where.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresDoorPlacement {
    /// What it is.
    pub kind: CeresDoorKind,
    /// The centre of its box in the room.
    pub position: Pair<Px>,
}

/// What a Ceres door actor is. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CeresDoorKind {
    /// A door between two rooms, opening the way it faces.
    Door(Side),
    /// One of the two walls Ridley's getaway puts up, at that side of his room.
    GetawayWall(Side),
    /// The door of Ridley's room, facing right.
    RidleyDoor,
    /// The shaft's door, facing left.
    ShaftDoor,
    /// The shaft's overlay, which shows the shaft's beacon and its platform.
    ShaftOverlay,
}

/// A version a condition chooses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conditional {
    /// The condition.
    pub condition: Condition,
    /// The version it chooses.
    pub version: Version,
}

/// A room's item lighting policy during passages.
///
/// The passage reads both the source and destination room's policy.
/// A dark policy also hides the front layer during the slide (world-doors `FinishedFrames`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemLighting {
    /// Item colours may stay lit.
    Lit,
    /// Item colours fade with the room.
    Dark,
}

/// A room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Our id.
    pub id: RoomId,
    /// Where it sits on the map.
    pub place: MapPlace,
    /// Its size.
    pub size: Screens,
    /// Its vertical scrollers.
    pub scrollers: Scrollers,
    /// Its item-colour policy during passages.
    pub item_lighting: ItemLighting,
    /// Its doors, by the place its door blocks name.
    pub doors: Vec<Door>,
    /// The conditional versions, in the order they are checked.
    pub conditional: Vec<Conditional>,
    /// The version that loads when no condition holds.
    pub default: Version,
}

impl Room {
    /// The version at `place` in the room's list, the default after the conditional ones.
    #[must_use]
    pub fn version(&self, place: usize) -> Option<&Version> {
        self.conditional
            .get(place)
            .map(|entry| &entry.version)
            .or_else(|| (place == self.conditional.len()).then_some(&self.default))
    }
}

/// The rooms and the doorways their doors lead through, each once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    rooms: Vec<Room>,
    doorways: Vec<Doorway>,
}

/// Why rooms and doorways don't make a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inconsistent {
    /// An earlier room shares this room's id.
    RepeatedRoom(RoomId),
    /// An earlier doorway shares this doorway's id.
    RepeatedDoorway(DoorwayId),
    /// A door of this room leads through a doorway the world doesn't hold.
    MissingDoorway(RoomId),
}

/// The key of the first of `items` that an earlier one shares its key with.
fn repeated<T, K: PartialEq>(items: &[T], key: impl Fn(&T) -> K) -> Option<K> {
    items.iter().enumerate().find_map(|(place, item)| {
        let own = key(item);
        items
            .iter()
            .take(place)
            .any(|earlier| key(earlier) == own)
            .then_some(own)
    })
}

impl World {
    /// The world of `rooms`, whose doors lead through `doorways`.
    ///
    /// # Errors
    ///
    /// [`Inconsistent`] when an id repeats, or a door leads through a doorway not among
    /// `doorways`.
    pub fn new(rooms: Vec<Room>, doorways: Vec<Doorway>) -> Result<Self, Inconsistent> {
        if let Some(id) = repeated(&rooms, |room| room.id) {
            return Err(Inconsistent::RepeatedRoom(id));
        }
        if let Some(id) = repeated(&doorways, |doorway| doorway.id) {
            return Err(Inconsistent::RepeatedDoorway(id));
        }
        let held = |id: DoorwayId| doorways.iter().any(|doorway| doorway.id == id);
        let missing = rooms.iter().find(|room| {
            room.doors
                .iter()
                .any(|&door| matches!(door, Door::To(id) if !held(id)))
        });
        if let Some(room) = missing {
            return Err(Inconsistent::MissingDoorway(room.id));
        }
        Ok(Self { rooms, doorways })
    }

    /// Doorway `id`, if the world holds it.
    #[must_use]
    pub fn doorway(&self, id: DoorwayId) -> Option<&Doorway> {
        self.doorways.iter().find(|doorway| doorway.id == id)
    }

    /// The room `door` leads into, if it leads anywhere.
    #[must_use]
    pub fn leads_to(&self, door: Door) -> Option<RoomId> {
        match door {
            Door::To(id) => self.doorway(id).map(|doorway| doorway.room),
            Door::Nowhere => None,
        }
    }

    /// Room `id`, if the world holds it.
    #[must_use]
    pub fn room(&self, id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }

    /// Each room, in the order it was imported.
    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(id: RoomId) -> Room {
        let version = || Version {
            level: Vec::new(),
            scrolls: Vec::new(),
            tileset: TilesetId(0),
            back: Back::Level,
            plms: PlmPopulation {
                entries: Vec::new(),
                scroll_changes: Vec::new(),
            },
            enemies: Vec::new(),
            ceres_doors: Vec::new(),
            ceres_ridley: false,
            setup: RoomSetup::Nothing,
        };
        Room {
            id,
            place: MapPlace {
                area: Area::Ceres,
                x: 1,
                y: 2,
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
            doors: vec![Door::Nowhere, Door::To(doorways::ELEVATOR_TO_FALLING_TILE)],
            conditional: vec![Conditional {
                condition: Condition::PowerBombs,
                version: version(),
            }],
            default: version(),
        }
    }

    #[test]
    fn a_room_lists_its_conditional_versions_then_its_default() {
        let room = room(rooms::CERES_RIDLEY_ROOM);
        let is = |place: usize, version: &Version| {
            room.version(place)
                .is_some_and(|found| core::ptr::eq(found, version))
        };
        assert!(is(0, &room.conditional[0].version));
        assert!(is(1, &room.default));
        assert_eq!(room.version(2), None);
    }

    const INTO_FALLING_TILE: Doorway = Doorway {
        id: doorways::ELEVATOR_TO_FALLING_TILE,
        room: rooms::FALLING_TILE_ROOM,
        arrival: Arrival {
            way: Heading::Right,
            view: Pair {
                x: ScreenCoordinate(2),
                y: ScreenCoordinate(3),
            },
            speed: None,
        },
        rule: DoorRule::Nothing,
    };

    #[test]
    fn destination_lighting_is_independent_of_source_and_import_order() {
        for source_policy in [ItemLighting::Lit, ItemLighting::Dark] {
            for destination_policy in [ItemLighting::Lit, ItemLighting::Dark] {
                let mut source = room(rooms::CERES_ELEVATOR_ROOM);
                source.item_lighting = source_policy;
                let mut destination = room(rooms::FALLING_TILE_ROOM);
                destination.item_lighting = destination_policy;
                for records in [
                    vec![source.clone(), destination.clone()],
                    vec![destination.clone(), source.clone()],
                ] {
                    let world = World::new(records, vec![INTO_FALLING_TILE]).unwrap();
                    let leaving = world.room(source.id).unwrap();
                    let arriving = world
                        .room(world.leads_to(leaving.doors[1]).unwrap())
                        .unwrap();
                    assert_eq!(leaving.item_lighting, source_policy);
                    assert_eq!(arriving.item_lighting, destination_policy);
                }
            }
        }
    }

    #[test]
    fn a_world_holds_each_room_once_by_its_id() {
        let ceres = [rooms::CERES_ELEVATOR_ROOM, rooms::FALLING_TILE_ROOM];
        let world = World::new(ceres.map(room).to_vec(), vec![INTO_FALLING_TILE]).unwrap();
        assert_eq!(
            world.room(rooms::FALLING_TILE_ROOM).map(|room| room.id),
            Some(rooms::FALLING_TILE_ROOM)
        );
        assert_eq!(world.room(rooms::MORPH_BALL_ROOM), None);
        assert_eq!(
            World::new(
                vec![room(rooms::MORPH_BALL_ROOM), room(rooms::MORPH_BALL_ROOM)],
                vec![INTO_FALLING_TILE]
            ),
            Err(Inconsistent::RepeatedRoom(rooms::MORPH_BALL_ROOM))
        );
    }

    #[test]
    fn a_door_leads_into_its_doorways_room_and_a_world_holds_every_doorway_once() {
        let world =
            World::new(vec![room(rooms::MORPH_BALL_ROOM)], vec![INTO_FALLING_TILE]).unwrap();
        let doors = &world.room(rooms::MORPH_BALL_ROOM).unwrap().doors;
        assert_eq!(
            doors
                .iter()
                .map(|&door| world.leads_to(door))
                .collect::<Vec<_>>(),
            [None, Some(rooms::FALLING_TILE_ROOM)]
        );
        assert_eq!(
            World::new(vec![room(rooms::MORPH_BALL_ROOM)], Vec::new()),
            Err(Inconsistent::MissingDoorway(rooms::MORPH_BALL_ROOM))
        );
        assert_eq!(
            World::new(Vec::new(), vec![INTO_FALLING_TILE, INTO_FALLING_TILE]),
            Err(Inconsistent::RepeatedDoorway(INTO_FALLING_TILE.id))
        );
    }
}
