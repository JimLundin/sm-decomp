use super::*;
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
            .map(|door| match door {
                world::Door::To(to) => name(*to).unwrap(),
                world::Door::Nowhere => "nowhere",
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
                (world::Door::To(to), room::Door::ToRoom(at)) => assert_eq!(key(to), Some(at)),
                (world::Door::Nowhere, room::Door::Nowhere) => {}
                pair => panic!("{pair:?}"),
            }
        }
        assert_eq!(room.conditional.len(), decoded.conditional.len());
        for (place, (index, state)) in decoded.states().enumerate() {
            assert_eq!(usize::from(index.0), place);
            assert_eq!(
                room.version(place).map(|version| &version.plms),
                Some(&state.plms)
            );
        }
        for (ours, theirs) in room.conditional.iter().zip(&decoded.conditional) {
            assert_eq!(Ok(ours.condition), condition(id, theirs.condition));
        }
    }
    assert_eq!(
        import(cartridge.lorom(), &[MORPH_BALL_ROOM, MORPH_BALL_ROOM]),
        Err(WorldError::Repeated(MORPH_BALL_ROOM))
    );
}
