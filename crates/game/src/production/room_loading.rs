//! The room's loading: what the passage's calls load of the next room, from our own room and
//! tileset formats.
//!
//! The passage says when each call's work is done; this says what it loads. The room that
//! loads is the one the doorway leads to, in the version progress chooses as she comes in
//! through it.

use data::assets::tilesets::Tilesets;
use data::assets::world::{self, World};
use data::id::DoorwayId;
use data::room::Area;
use run_progress::{EnteredRoom, Progress, RoomStates, Version};
use world_doors::{Palette, RoomCall};

/// The rooms and tilesets a passage loads from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rooms {
    /// The rooms and the doorways between them.
    pub world: World,
    /// The tilesets their versions draw with.
    pub tilesets: Tilesets,
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
    /// A colour of the tileset has more than 15 bits.
    Colour,
}

/// What the room's loading leaves of the screen, once the work `call` names is done.
///
/// Fetching the tiles sets the colours the fade in heads for to the tileset's, from the
/// screen's first colour on; the colours past the tileset's keep the goal they had.
///
/// # Errors
/// A doorway, room or tileset the content lacks leaves the colours as they were.
pub fn load(
    call: Option<RoomCall>,
    through: DoorwayId,
    progress: &Progress,
    rooms: &Rooms,
    palette: &mut Palette,
) -> Result<(), LoadFault> {
    match call {
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
            | RoomCall::FetchContents
            | RoomCall::AnimateTiles
            | RoomCall::PlayMusic
            | RoomCall::ReadLiquid,
        ) => Ok(()),
    }
}

/// The version of the room `through` leads to that loads: the first whose condition holds for
/// `progress` as she comes in through it, or the default.
fn version<'a>(
    rooms: &'a Rooms,
    through: DoorwayId,
    progress: &Progress,
) -> Result<&'a world::Version, LoadFault> {
    let room = rooms
        .world
        .doorway(through)
        .and_then(|doorway| rooms.world.room(doorway.room))
        .ok_or(LoadFault::Room(through))?;
    let conditions: Vec<_> = room
        .conditional
        .iter()
        .map(|entry| condition(entry.condition))
        .collect();
    let entered = EnteredRoom {
        area: area(room.place.area)?,
        entered_by: Some(types::Entrance(through.0)),
    };
    match (RoomStates {
        conditional: &conditions,
    })
    .choose(progress, entered)
    {
        Version::Conditional(place) => room.version(place).ok_or(LoadFault::Room(through)),
        Version::Default => Ok(&room.default),
    }
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
const fn area(area: Area) -> Result<types::Area, LoadFault> {
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
    use super::{LoadFault, Rooms, load};
    use data::assets::art::{Colour, Palette as Row, PaletteSet};
    use data::assets::tilesets::{Tileset, Tilesets};
    use data::assets::world::{
        Arrival, Conditional, Door, Doorway, Heading, ItemLighting, MapPlace, Pair, Room,
        ScreenCoordinate, Version, World,
    };
    use data::id::{DoorwayId, RoomId, TilesetId};
    use data::room::{Area, Screens, Scrollers};
    use world_doors::{COLOURS, Palette, RoomCall};

    const ROOM: RoomId = RoomId(1);
    /// The doorway the room's conditional version is entered by, and another into it.
    const CHOSEN: DoorwayId = DoorwayId(1);
    const OTHER: DoorwayId = DoorwayId(2);

    fn version(tileset: u16) -> Version {
        Version {
            tileset: TilesetId(tileset),
            back: data::assets::world::Back::Level,
            plms: data::plm::PlmPopulation::default(),
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
            scrollers: Scrollers { up: 0, down: 0 },
            item_lighting: ItemLighting::Lit,
            doors: vec![Door::To(CHOSEN), Door::To(OTHER)],
            conditional: vec![Conditional {
                condition: data::assets::world::Condition::EnteredBy(CHOSEN),
                version: version(1),
            }],
            default: version(0),
        };
        Rooms {
            world: World::new(vec![room], vec![doorway(CHOSEN), doorway(OTHER)]).unwrap(),
            tilesets: Tilesets {
                sets: vec![tileset(0, 0x0011), tileset(1, 0x0022)],
                ..Tilesets::default()
            },
        }
    }

    fn lit(bits: u16) -> [types::Colour; COLOURS] {
        [types::Colour::new(bits).unwrap(); COLOURS]
    }

    fn loaded(call: RoomCall, through: DoorwayId) -> Result<Palette, LoadFault> {
        let mut palette = Palette {
            screen: lit(0),
            goal: lit(0x0033),
        };
        load(
            Some(call),
            through,
            &run_progress::Progress::default(),
            &rooms(),
            &mut palette,
        )
        .map(|()| palette)
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
}
