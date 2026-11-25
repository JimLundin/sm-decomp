use super::*;
use data::assets::actors::SPECIES;
use data::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    Some(Rom::new(std::fs::read(path).ok()?).unwrap())
}

/// An invented image, large enough to hold the graphics sets' bank, with `bytes` at `at`.
fn image(at: RomAddress, bytes: &[u8]) -> Vec<u8> {
    let mut image = vec![0; 0x20_0000];
    let offset = Lorom::new(&image).file_offset(at).unwrap();
    image[offset..][..bytes.len()].copy_from_slice(bytes);
    image
}

/// The graphics sets' bank.
const SETS: u8 = 0xb4;

#[test]
fn every_registered_species_has_one_place_in_id_order() {
    let ids: Vec<_> = PLACES.iter().map(|&(id, _)| id).collect();
    let registered: Vec<_> = SPECIES.iter().map(|&(id, _)| id).collect();
    assert_eq!(ids, registered);
    for (place, &(_, pointer)) in PLACES.iter().enumerate() {
        assert!(PLACES[..place].iter().all(|&(_, other)| other != pointer));
    }
}

#[test]
fn a_graphics_set_names_each_species_with_the_low_byte_of_its_palette_word_as_its_row() {
    let at = RomAddress::new(SETS, 0x9000);
    let set = [0x3f, 0xe2, 0x02, 0xd0, 0x3f, 0xe1, 0x01, 0x00, 0xff, 0xff];
    let image = image(at, &set);
    let row = |row| SpriteRow::new(row).unwrap();
    assert_eq!(
        graphics_set(Lorom::new(&image), at),
        Ok(vec![
            EnemyGraphics {
                species: CERES_DOOR,
                row: row(2),
            },
            EnemyGraphics {
                species: CERES_RIDLEY,
                row: row(1),
            },
        ])
    );
}

#[test]
fn a_graphics_set_with_an_unknown_species_or_a_row_past_the_eight_is_refused() {
    let at = RomAddress::new(SETS, 0x9000);
    let unknown = image(at, &[0x34, 0x12, 0x00, 0x00, 0xff, 0xff]);
    assert_eq!(
        graphics_set(Lorom::new(&unknown), at),
        Err(ActorsError::Unregistered(RomAddress::new(
            SPECIES_BANK,
            0x1234
        )))
    );
    let past = image(at, &[0x3f, 0xe2, 0x08, 0x00, 0xff, 0xff]);
    assert_eq!(
        graphics_set(Lorom::new(&past), at),
        Err(ActorsError::Row(RomAddress::new(SPECIES_BANK, 0xe23f)))
    );
}

#[test]
fn a_palette_row_is_sixteen_little_endian_words() {
    let mut bytes = [0; ROW_BYTES];
    bytes[2] = 0x1f;
    bytes[ROW_BYTES - 1] = 0x7c;
    let palette = palette_of(&bytes);
    assert_eq!(palette.0[1], Colour(0x001f));
    assert_eq!(palette.0[ROW_COLOURS - 1], Colour(0x7c00));
}

#[test]
fn every_registered_species_and_the_shared_palettes_import_from_the_rom() {
    let Some(rom) = rom() else { return };
    let actors = import(rom.lorom()).unwrap();
    let elevator = actors.species(data::assets::actors::ELEVATOR).unwrap();
    assert_eq!(elevator.palette, None);
    // Ceres Ridley's eye fades in over 64 steps, his body over 16 sets.
    assert_eq!(actors.ceres_ridley.eye_steps.len(), 64);
    assert_eq!(actors.ceres_ridley.body_sets.len(), 16);
    // His tail's hit is worth 15, apart from his species' contact damage.
    assert_eq!(actors.ceres_tail_worth, 15);
    for &(id, name) in &SPECIES {
        assert_eq!(
            actors.species(id).map(|species| species.id),
            Some(id),
            "{name}"
        );
    }
}

#[test]
fn a_ceres_door_actor_is_named_by_its_first_parameter_and_no_other_value_imports() {
    let named: Vec<_> = (0..=7).map(ceres_door_kind).collect();
    assert_eq!(
        named,
        [
            Some(CeresDoorKind::Door(Side::Right)),
            Some(CeresDoorKind::Door(Side::Left)),
            Some(CeresDoorKind::ShaftOverlay),
            Some(CeresDoorKind::RidleyDoor),
            Some(CeresDoorKind::ShaftDoor),
            Some(CeresDoorKind::GetawayWall(Side::Left)),
            Some(CeresDoorKind::GetawayWall(Side::Right)),
            None,
        ]
    );
    assert_eq!(ceres_door_kind(0xffff), None);
}

#[test]
fn the_beacon_takes_the_first_six_colours_of_each_sixteen_byte_row() {
    // Invented words: row r's colour c is r × 100h + c, so a wrong stride or count shows.
    let words = (0..8u16).flat_map(|row| (0..8u16).map(move |colour| row * 0x100 + colour));
    let bytes: Vec<u8> = words.flat_map(u16::to_le_bytes).collect();
    assert_eq!(bytes.len(), 8 * 16);
    let image = image(BEACON, &bytes);
    let sets = beacon(Lorom::new(&image)).unwrap().sets;
    for (row, set) in (0..).zip(sets) {
        let expected: Vec<_> = (0..6).map(|colour| Colour(row * 0x100 + colour)).collect();
        assert_eq!(set.to_vec(), expected, "row {row}");
    }
}

#[test]
fn ridleys_eye_steps_hold_the_sets_they_name_and_his_body_its_sixteen_sets() {
    // Invented words: set s's colour c is s × 100h + c, so a wrong stride or count shows.
    let sets = |colours: u16| -> Vec<u8> {
        (0..16u16)
            .flat_map(|set| (0..colours).map(move |colour| set * 0x100 + colour))
            .flat_map(u16::to_le_bytes)
            .collect()
    };
    let mut image = image(EYE_STEPS, &[2, 0, 2, END_OF_STEPS]);
    for (at, bytes) in [(EYE_SETS, sets(3)), (BODY_SETS, sets(11))] {
        let offset = Lorom::new(&image).file_offset(at).unwrap();
        image[offset..][..bytes.len()].copy_from_slice(&bytes);
    }
    let set = |set: u16, colours: u16| -> Vec<_> {
        (0..colours)
            .map(|colour| Colour(set * 0x100 + colour))
            .collect()
    };
    let ridley = ceres_ridley_colours(Lorom::new(&image)).unwrap();
    let steps: Vec<_> = ridley.eye_steps.iter().map(|step| step.to_vec()).collect();
    assert_eq!(steps, [set(2, 3), set(0, 3), set(2, 3)]);
    assert_eq!(ridley.body_sets.len(), 16);
    assert_eq!(ridley.body_sets[15].to_vec(), set(15, 11));

    // A step naming a set past the sixteen, or no step at all, is refused.
    for steps in [[16, END_OF_STEPS], [END_OF_STEPS, 0]] {
        let offset = Lorom::new(&image).file_offset(EYE_STEPS).unwrap();
        image[offset..][..2].copy_from_slice(&steps);
        assert_eq!(
            ceres_ridley_colours(Lorom::new(&image)),
            Err(ActorsError::EyeSteps)
        );
    }
    // Steps naming sets with no end byte before the eye's sets are refused, though the
    // sets' bytes would read as more steps and an end byte comes later.
    let offset = Lorom::new(&image).file_offset(EYE_STEPS).unwrap();
    let space = usize::from(EYE_SETS.offset - EYE_STEPS.offset);
    image[offset..][..space].fill(1);
    let after = Lorom::new(&image).file_offset(BODY_SETS).unwrap() + 2 * 11 * 16;
    image[after] = END_OF_STEPS;
    assert_eq!(
        ceres_ridley_colours(Lorom::new(&image)),
        Err(ActorsError::EyeSteps)
    );
}

#[test]
fn ridleys_tail_worth_is_his_set_ups_load_and_other_code_there_is_refused() {
    // An invented worth, loaded and stored where his tail's hit reads it.
    let mut code = vec![LOAD, 0x34, 0x12];
    code.extend_from_slice(&TAIL_WORTH_STORE);
    let rom = image(TAIL_WORTH_LOAD, &code);
    assert_eq!(ceres_tail_worth(Lorom::new(&rom)), Ok(0x1234));

    // A load stored elsewhere, or no load at all, is not his tail's worth.
    for at in 3..7 {
        let mut other = code.clone();
        other[at] ^= 1;
        let rom = image(TAIL_WORTH_LOAD, &other);
        assert_eq!(
            ceres_tail_worth(Lorom::new(&rom)),
            Err(ActorsError::TailWorth)
        );
    }
    let mut other = code.clone();
    other[0] = 0xad;
    let rom = image(TAIL_WORTH_LOAD, &other);
    assert_eq!(
        ceres_tail_worth(Lorom::new(&rom)),
        Err(ActorsError::TailWorth)
    );

    // A worth up to the most whole energy is his tail's; one past it is refused.
    for (low, high, worth) in [
        (0xff, 0x7f, Ok(0x7fff)),
        (0x00, 0x80, Err(ActorsError::TailWorth)),
    ] {
        let mut other = code.clone();
        other[1] = low;
        other[2] = high;
        let rom = image(TAIL_WORTH_LOAD, &other);
        assert_eq!(ceres_tail_worth(Lorom::new(&rom)), worth);
    }
}
