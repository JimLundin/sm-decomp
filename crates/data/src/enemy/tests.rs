use super::*;
use crate::rom::Rom;

/// A file offset in a `LoROM` image.
fn file(bank: u8, offset: u16) -> usize {
    (usize::from(bank & 0x7f) << 15) | usize::from(offset & 0x7fff)
}

fn put(image: &mut [u8], at: usize, bytes: &[u8]) {
    image[at..][..bytes.len()].copy_from_slice(bytes);
}

fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|word| word.to_le_bytes()).collect()
}

#[test]
fn a_population_decodes_to_its_terminator_and_quota() {
    let mut image = vec![0; 0x11_0000];
    let at = file(0xa1, 0x9000);
    put(
        &mut image,
        at,
        &words(&[0xe6bf, 1, 2, 3, 4, 5, 6, 7, 0xd73f, 8, 9, 10, 11, 12, 13, 14, 0xffff]),
    );
    image[file(0xa1, 0x9022)] = 3;
    let decoded = population(Lorom::new(&image), RomAddress::new(0xa1, 0x9000)).unwrap();
    assert_eq!(decoded.kill_quota, 3);
    assert_eq!(decoded.placements.len(), 2);
    assert_eq!(
        decoded.placements[1],
        Placement {
            species: SpeciesKey::ELEVATOR,
            x: 8,
            y: 9,
            init: 10,
            properties: 11,
            extra_properties: 12,
            parameter_1: 13,
            parameter_2: 14,
        }
    );
}

#[test]
fn a_species_header_decodes_each_field_at_its_offset() {
    let mut image = vec![0; 0x11_0000];
    // Each word holds its own offset, so a field read from the wrong place shows.
    let header: Vec<u16> = (0..32_u16).map(|index| 0x8000 | index.wrapping_mul(2)).collect();
    let at = file(0xa0, 0xe6bf);
    put(&mut image, at, &words(&header));
    put(&mut image, file(0xa0, 0xe6cb), &[0xa8, 0x05]);
    put(&mut image, file(0xa0, 0xe6f5), &[0x00, 0xb1, 0x9a]);
    image[file(0xa0, 0xe6f8)] = 0x02;
    let decoded = species(Lorom::new(&image), SpeciesKey::MORPH_BALL_EYE).unwrap();
    assert_eq!(decoded.tile_bytes, 0x8000);
    assert_eq!(decoded.palette, RomAddress::new(0xa8, 0x8002));
    assert_eq!(decoded.health, 0x8004);
    assert_eq!(decoded.damage, 0x8006);
    assert_eq!((decoded.radius_x, decoded.radius_y), (0x8008, 0x800a));
    assert_eq!((decoded.ai_bank, decoded.hurt_flash), (0xa8, 0x05));
    assert_eq!(decoded.cry, 0x800e);
    assert_eq!(decoded.boss, 0x8010);
    assert_eq!(decoded.routines.init, Routine(RomAddress::new(0xa8, 0x8012)));
    assert_eq!(decoded.parts, 0x8014);
    assert_eq!(decoded.routines.main, Routine(RomAddress::new(0xa8, 0x8018)));
    assert_eq!(decoded.routines.frozen, Routine(RomAddress::new(0xa8, 0x801e)));
    assert_eq!(decoded.death_animation, 0x8022);
    assert_eq!(decoded.routines.power_bomb, Routine(RomAddress::new(0xa8, 0x8028)));
    assert_eq!(decoded.routines.touch, Routine(RomAddress::new(0xa8, 0x8030)));
    assert_eq!(decoded.routines.shot, Routine(RomAddress::new(0xa8, 0x8032)));
    assert_eq!(decoded.tiles, RomAddress::new(0x9a, 0xb100));
    assert_eq!(decoded.layer, 0x02);
    assert_eq!(
        (decoded.drops, decoded.vulnerabilities, decoded.name),
        (0x803a, 0x803c, 0x803e)
    );
}

#[test]
fn the_species_map_gives_dense_ids_in_load_order() {
    let map = SpeciesMap::slice_zero();
    assert_eq!(
        map.get(SpeciesKey::ELEVATOR),
        Some((SpeciesId(1), Behaviour::Elevator))
    );
    assert_eq!(map.get(SpeciesKey(0x1234)), None);
    let repeated = vec![
        (SpeciesKey::ELEVATOR, Behaviour::Elevator),
        (SpeciesKey::ELEVATOR, Behaviour::Elevator),
    ];
    assert_eq!(SpeciesMap::new(repeated), None);
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_morph_ball_rooms_enemies_decode_as_the_bank_logs_say() {
    let Some(cartridge) = rom() else {
        return;
    };
    let rom = cartridge.lorom();
    let decoded = population(rom, RomAddress::new(0xa1, 0x93ac)).unwrap();
    let species_order: Vec<SpeciesKey> = decoded
        .placements
        .iter()
        .map(|placement| placement.species)
        .collect();
    assert_eq!(
        species_order,
        [
            SpeciesKey::MORPH_BALL_EYE,
            SpeciesKey::MORPH_BALL_EYE,
            SpeciesKey::ELEVATOR,
            SpeciesKey::BLUE_BRINSTAR_FACE_BLOCK,
            SpeciesKey::BLUE_BRINSTAR_FACE_BLOCK,
            SpeciesKey::BLUE_BRINSTAR_FACE_BLOCK,
            SpeciesKey::BLUE_BRINSTAR_FACE_BLOCK,
        ]
    );
    assert_eq!(decoded.kill_quota, 0);
    assert_eq!((decoded.placements[2].x, decoded.placements[2].y), (0x0580, 0x02c2));
    assert_eq!(decoded.placements[0].parameter_2, 0x8000);

    let table = SpeciesTable::load(rom, SpeciesMap::slice_zero()).unwrap();
    let eye = table.get(SpeciesId(0)).unwrap();
    assert_eq!((eye.ai_bank, eye.parts, eye.health), (0xa8, 2, 0x14));
    assert_eq!((eye.tiles, eye.layer), (RomAddress::new(0xb1, 0x9a00), 5));
    let elevator = table.get(SpeciesId(1)).unwrap();
    assert_eq!((elevator.ai_bank, elevator.radius_x, elevator.radius_y), (0xa3, 0x10, 0x08));
    assert_eq!(elevator.layer, 5);
    let face = table.get(SpeciesId(2)).unwrap();
    assert_eq!((face.ai_bank, face.palette), (0xa8, RomAddress::new(0xa8, 0xe7ac)));
}
