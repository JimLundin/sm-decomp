use super::*;
use data::assets::world::rooms::ROOMS;
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
        assert!(key(entry.id).is_some(), "{}", entry.name);
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
    let unregistered = RoomId(7);
    assert_eq!(
        import(Lorom::new(&[]), &[unregistered]),
        Err(WorldError::Unregistered(unregistered))
    );
}

#[test]
fn each_room_holds_what_the_decoder_reads() {
    let Some(cartridge) = rom() else {
        return;
    };
    let ids = ROOMS.map(|entry| entry.id);
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
        assert_eq!(room.doors, decoded.doors.doors);
        assert_eq!(room.conditional.len(), decoded.conditional.len());
        for (place, (index, state)) in decoded.states().enumerate() {
            assert_eq!(usize::from(index.0), place);
            assert_eq!(
                room.version(place).map(|version| &version.plms),
                Some(&state.plms)
            );
        }
        for (ours, theirs) in room.conditional.iter().zip(&decoded.conditional) {
            assert_eq!(ours.condition, theirs.condition);
        }
    }
    assert_eq!(
        import(cartridge.lorom(), &[MORPH_BALL_ROOM, MORPH_BALL_ROOM]),
        Err(WorldError::Repeated(MORPH_BALL_ROOM))
    );
}
