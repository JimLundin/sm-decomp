use super::*;
use data::assets::tilesets::{TILESETS as REGISTERED_TILESETS, name as tileset_name};
use data::assets::world::doorways::{DOORWAYS as REGISTERED_DOORWAYS, name as doorway_name};
use data::assets::world::rooms::{ROOMS, name};
use data::assets::world::{BossKind, Event, Set};
use data::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    Some(Rom::new(std::fs::read(path).ok()?).unwrap())
}

#[test]
fn every_registered_room_has_one_place() {
    for entry in ROOMS {
        assert_eq!(key(entry.id).and_then(id), Some(entry.id), "{}", entry.name);
    }
    for (place, &(_, pointer)) in PLACES.iter().enumerate() {
        assert!(
            PLACES
                .iter()
                .take(place)
                .all(|&(_, other)| other != pointer)
        );
    }
}

#[test]
fn every_registered_doorway_and_tileset_has_one_place() {
    for entry in REGISTERED_DOORWAYS {
        let places: Vec<_> = DOORWAYS.iter().filter(|&&(id, _)| id == entry.id).collect();
        assert_eq!(places.len(), 1, "{}", entry.name);
        assert_eq!(
            doorway(RomAddress::new(DOOR_BANK, places[0].1)),
            Some(entry.id)
        );
    }
    assert_eq!(DOORWAYS.len(), REGISTERED_DOORWAYS.len());
    for entry in REGISTERED_TILESETS {
        let index = tileset_index(entry.id);
        assert!(index.is_some(), "{}", entry.name);
        assert_eq!(
            TILESETS
                .iter()
                .filter(|&&(_, known)| Some(known) == index)
                .count(),
            1
        );
    }
    assert_eq!(TILESETS.len(), REGISTERED_TILESETS.len());
}

#[test]
fn an_unregistered_room_is_refused() {
    let unregistered = RoomId(10);
    assert_eq!(
        import(Lorom::new(&[]), &[unregistered]),
        Err(WorldError::Unregistered(unregistered))
    );
}

/// The rooms the engine loads or will: slice 0's and Ceres's, whose doors lead only to
/// registered rooms.
const LOADED: [RoomId; 7] = [
    MORPH_BALL_ROOM,
    CERES_ELEVATOR_ROOM,
    FALLING_TILE_ROOM,
    MAGNET_STAIRS_ROOM,
    DEAD_SCIENTIST_ROOM,
    ESCAPE_ROOM_58,
    CERES_RIDLEY_ROOM,
];

#[test]
fn a_door_names_the_room_it_leads_to_by_id() {
    let Some(cartridge) = rom() else {
        return;
    };
    let world = import(cartridge.lorom(), &[MORPH_BALL_ROOM, CERES_RIDLEY_ROOM]).unwrap();
    let leads_to = |id| {
        let mut names: Vec<_> = world
            .room(id)
            .unwrap()
            .doors
            .iter()
            .map(|&door| {
                world
                    .leads_to(door)
                    .map_or("nowhere", |to| name(to).unwrap())
            })
            .collect();
        names.sort_unstable();
        names
    };
    assert_eq!(
        leads_to(MORPH_BALL_ROOM),
        [
            "Blue Brinstar Elevator Room",
            "Construction Zone",
            "Green Hill Zone",
            "nowhere"
        ]
    );
    assert_eq!(leads_to(CERES_RIDLEY_ROOM), ["58 Escape Room"]);
}

#[test]
fn a_door_to_an_unregistered_room_is_refused() {
    let Some(cartridge) = rom() else {
        return;
    };
    assert!(matches!(
        import(cartridge.lorom(), &[GREEN_HILL_ZONE]),
        Err(WorldError::UnregisteredDestination {
            room: GREEN_HILL_ZONE,
            ..
        })
    ));
}

#[test]
fn a_condition_names_progress_terms_and_refuses_what_has_no_id() {
    let room = CERES_RIDLEY_ROOM;
    let mut main = Set::default();
    main[BossKind::MainBoss] = true;
    assert_eq!(
        condition(room, room::Condition::MainAreaBossDead),
        Ok(world::Condition::Defeated(main))
    );
    assert_eq!(
        condition(room, room::Condition::BossDead(BossBits(1))),
        Ok(world::Condition::Defeated(main))
    );
    assert_eq!(
        condition(room, room::Condition::Event(Event::PlanetAwake)),
        Ok(world::Condition::Happened(Event::PlanetAwake))
    );
    assert_eq!(
        condition(room, room::Condition::BossDead(BossBits(8))),
        Err(WorldError::UnknownBoss(room))
    );
    assert_eq!(
        condition(room, room::Condition::Door(RomAddress::new(0x83, 0x8000))),
        Err(WorldError::UnregisteredDoorway(room))
    );
    let through = RomAddress::new(DOOR_BANK, DOORWAYS[0].1);
    assert_eq!(
        condition(room, room::Condition::Door(through)),
        Ok(world::Condition::EnteredBy(DOORWAYS[0].0))
    );
}

#[test]
fn each_room_holds_what_the_decoder_reads() {
    let Some(cartridge) = rom() else {
        return;
    };
    let ids = LOADED;
    let world = import(cartridge.lorom(), &ids).unwrap();
    for id in ids {
        let decoded = room::decode(cartridge.lorom(), key(id).unwrap()).unwrap();
        let room = world.room(id).unwrap();
        assert_eq!(room.id, id);
        assert_eq!(room.place.area, decoded.area);
        assert_eq!([room.place.x, room.place.y], {
            let place = decoded.map_position;
            [place.x, place.y]
        });
        assert_eq!(room.size, decoded.size);
        assert_eq!(room.scrollers, decoded.scrollers);
        assert_eq!(room.doors.len(), decoded.doors.doors.len());
        for (&ours, &theirs) in room.doors.iter().zip(&decoded.doors.doors) {
            match (ours, theirs) {
                (world::Door::To(through), room::Door::ToRoom(at)) => {
                    let into = world.leads_to(ours).unwrap();
                    assert_eq!(key(into), Some(at));
                    // The registry names a doorway by the rooms it joins, so a doorway given
                    // another door's place reads as the wrong pair.
                    assert_eq!(
                        doorway_name(through),
                        Some(format!("{} to {}", name(id).unwrap(), name(into).unwrap()).as_str())
                    );
                }
                (world::Door::Nowhere, room::Door::Nowhere) => {}
                pair => panic!("{pair:?}"),
            }
        }
        assert_eq!(room.conditional.len(), decoded.conditional.len());
        let places = version_places(cartridge.lorom(), id).unwrap();
        assert_eq!(places.len(), room.conditional.len() + 1);
        for ((place, (index, state)), held) in decoded.states().enumerate().zip(&places) {
            assert_eq!(usize::from(index.0), place);
            let version = room.version(place).unwrap();
            assert_eq!(&version.plms, &state.plms);
            assert_eq!(
                tileset_index(version.tileset),
                Some(TilesetIndex(state.tileset))
            );
            assert_eq!(version.back == Back::Level, state.layers.layer2.is_some());
            assert_eq!(held.state, state.at);
            assert_eq!(held.scroll_lists, state.scroll_lists);
        }
        if room.place.area == data::room::Area::Ceres {
            // Ceres's rooms change for the escape once Ridley is defeated, their first
            // condition, and draw from the escape's tilesets then.
            let escape = |version: &Version| {
                tileset_name(version.tileset).is_some_and(|name| name.ends_with(", escape"))
            };
            assert!(escape(&room.conditional[0].version), "{id:?}");
            assert!(!escape(&room.default), "{id:?}");
        }
        for (ours, theirs) in room.conditional.iter().zip(&decoded.conditional) {
            assert_eq!(Ok(ours.condition), condition(id, theirs.condition));
        }
    }
    assert_eq!(
        import(cartridge.lorom(), &[MORPH_BALL_ROOM, MORPH_BALL_ROOM]),
        Err(WorldError::Inconsistent(Inconsistent::RepeatedRoom(
            MORPH_BALL_ROOM
        )))
    );
}
