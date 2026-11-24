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
            Some(CeresDoorKind::Later),
            Some(CeresDoorKind::ShaftDoor),
            Some(CeresDoorKind::Later),
            Some(CeresDoorKind::Later),
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
