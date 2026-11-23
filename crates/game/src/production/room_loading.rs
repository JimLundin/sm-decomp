//! The room's loading: what the passage's calls load of the next room, from our own room,
//! tileset, sprite and species formats.
//!
//! The passage says when each call's work is done; this says what it loads. The room that
//! loads is the one the doorway leads to, in the version progress chooses as she comes in
//! through it.

use boss_ceres_ridley::{ColourSet, Stage};
use data::assets::actors::{self, Actors, SpriteRow};
use data::assets::art::{Colour, ROW_COLOURS};
use data::assets::samus::palettes::SamusPalettes;
use data::assets::tilesets::Tilesets;
use data::assets::world::{self, World};
use data::id::DoorwayId;
use data::room::Area;
use run_progress::{EnteredRoom, Progress, RoomStates, Version};
use types::Item;
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
}

/// What a room's loading reads of the run.
#[derive(Clone, Copy, Debug)]
pub struct Run<'a> {
    /// Her progress, which chooses the next room's version, her suit and her beams.
    pub progress: &'a Progress,
    /// The Ceres station's stage, whose colour set the doors a room places take.
    pub ceres: Stage,
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
            contents(version(rooms, through, progress)?, run, rooms, palette)
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
    for loaded in &version.enemies {
        let own = rooms
            .actors
            .species(loaded.species)
            .and_then(|species| species.palette)
            .ok_or(LoadFault::Species(loaded.species))?;
        set(&mut goal, loaded.row, 0, &own.0)?;
    }
    if version
        .enemies
        .iter()
        .any(|loaded| loaded.species == actors::CERES_DOOR)
    {
        let doors = &rooms.actors.ceres_doors;
        let shades = match run.ceres.door_colours() {
            ColourSet::Calm => &doors.calm,
            ColourSet::Escape => &doors.escape,
        };
        set(&mut goal, SpriteRow::CERES_DOORS, 1, shades)?;
    }
    let suits = &rooms.samus.suits;
    let on = &inventory.items_switched_on;
    let suit = if on[Item::GravitySuit] {
        &suits.gravity
    } else if on[Item::VariaSuit] {
        &suits.varia
    } else {
        &suits.power
    };
    set(&mut goal, SpriteRow::SUIT, 0, &suit.0)?;
    palette.screen = screen;
    palette.goal = goal;
    Ok(())
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
    use super::{LoadFault, Rooms, Run, load};
    use data::assets::actors::{
        Actors, CERES_DOOR, CeresDoorColours, EnemyGraphics, Species, SpriteRow,
    };
    use data::assets::art::{Colour, Palette as Row, PaletteSet};
    use data::assets::samus::palettes::{BeamPalette, SamusPalettes, Suits};
    use data::assets::tilesets::{Tileset, Tilesets};
    use data::assets::world::{
        Arrival, Conditional, Door, Doorway, Heading, ItemLighting, MapPlace, Pair, Room,
        ScreenCoordinate, Version, World,
    };
    use data::id::{DoorwayId, RoomId, SpeciesId, TilesetId};
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
            enemies: vec![EnemyGraphics {
                species: CERES_DOOR,
                row: SpriteRow::new(2).unwrap(),
            }],
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
            actors: Actors {
                species: (0..=CERES_DOOR.0)
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
            },
            samus: SamusPalettes {
                suits: Suits {
                    power: Row([Colour(0x0088); 16]),
                    varia: Row([Colour(0x0099); 16]),
                    gravity: Row([Colour(0x00aa); 16]),
                },
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
            &rooms(),
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
