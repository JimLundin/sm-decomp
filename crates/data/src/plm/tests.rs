use super::*;
use crate::rom::{Lorom, Rom};

/// The file offset of `$8F:offset`.
fn room_bank(offset: u16) -> usize {
    usize::from(offset & 0x7fff) | 0x78000
}

fn image(population: &[u8], lists: &[(u16, &[u8])]) -> Vec<u8> {
    let mut image = vec![0; 0x80000];
    image[room_bank(0x8000)..][..population.len()].copy_from_slice(population);
    for (at, bytes) in lists {
        image[room_bank(*at)..][..bytes.len()].copy_from_slice(bytes);
    }
    image
}

const AT: RomAddress = RomAddress::new(0x8f, 0x8000);

#[test]
fn each_entry_decodes_to_its_kind_and_a_shared_list_once() {
    let population = [
        0x03, 0xb7, 31, 43, 0x00, 0x90, // scroll, list $9000
        0x47, 0xb6, 8, 35, 0x00, 0x80, // up
        0x3b, 0xb6, 88, 33, 0x00, 0x80, // right
        0x03, 0xb7, 87, 10, 0x00, 0x90, // scroll, the same list
        0x23, 0xef, 69, 41, 0x1a, 0x00, // morph ball
        0x00, 0x00,
    ];
    let rom = image(&population, &[(0x9000, &[18, 2, 13, 0, 0x80])]);
    let decoded = population_at(&rom);
    let first = ScrollChangesId(0);
    let kinds: Vec<PlmKind> = decoded.entries.iter().map(|entry| entry.kind).collect();
    assert_eq!(
        kinds,
        [
            PlmKind::Scroll(first),
            PlmKind::Extension(Extension::Up),
            PlmKind::Extension(Extension::Right),
            PlmKind::Scroll(first),
            PlmKind::Item(ItemPlm::MorphBall, 0x1a),
        ]
    );
    assert_eq!((decoded.entries[0].x, decoded.entries[0].y), (31, 43));
    assert_eq!(decoded.scroll_changes.len(), 1);
    assert_eq!(
        decoded.changes(first),
        Some(
            [
                ScrollChange {
                    screen: 18,
                    area: ScrollArea::Green
                },
                ScrollChange {
                    screen: 13,
                    area: ScrollArea::Red
                },
            ]
            .as_slice()
        )
    );
    assert_eq!(
        decoded.changes_at(RomAddress::new(0x8f, 0x9000)),
        Some(first)
    );
    // The kinds give the original's IDs back.
    let ids: Vec<u16> = kinds.iter().map(|kind| kind.id()).collect();
    assert_eq!(ids, [0xb703, 0xb647, 0xb63b, 0xb703, 0xef23]);
}

fn population_at(image: &[u8]) -> PlmPopulation {
    population(Lorom::new(image), AT).unwrap()
}

#[test]
fn an_unmodelled_plm_keeps_its_id_and_argument_and_an_unknown_area_is_an_error() {
    let rom = image(&[0x48, 0xc8, 1, 2, 0x34, 0x12, 0, 0], &[]);
    let decoded = population_at(&rom);
    assert_eq!(
        decoded.entries[0].kind,
        PlmKind::Unmodelled {
            id: 0xc848,
            argument: 0x1234
        }
    );
    assert_eq!(decoded.entries[0].kind.id(), 0xc848);
    let rom = image(
        &[0x03, 0xb7, 0, 0, 0x00, 0x90, 0, 0],
        &[(0x9000, &[13, 3, 0x80])],
    );
    assert_eq!(
        population(Lorom::new(&rom), AT),
        Err(PlmError::ScrollArea {
            at: RomAddress::new(0x8f, 0x9001),
            value: 3
        })
    );
}

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_morph_ball_rooms_plms_decode_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let decoded = population(cartridge.lorom(), RomAddress::new(0x8f, 0x867e)).unwrap();
    // Five scroll PLMs, eight upwards and three rightwards extensions, and the Morph Ball.
    assert_eq!(decoded.entries.len(), 17);
    let count = |wanted: fn(PlmKind) -> bool| {
        decoded
            .entries
            .iter()
            .filter(|entry| wanted(entry.kind))
            .count()
    };
    assert_eq!(count(|kind| matches!(kind, PlmKind::Scroll(_))), 5);
    assert_eq!(count(|kind| kind == PlmKind::Extension(Extension::Up)), 8);
    assert_eq!(
        count(|kind| kind == PlmKind::Extension(Extension::Right)),
        3
    );
    let last = decoded.entries.last().unwrap();
    assert_eq!(last.kind, PlmKind::Item(ItemPlm::MorphBall, 0x1a));
    assert_eq!((last.x, last.y), (69, 41));
    // Four lists: $9F05, $9F08, $9F0B (two PLMs) and $9F0E, each one change.
    let lists: Vec<(u16, Vec<ScrollChange>)> = decoded
        .scroll_changes
        .iter()
        .map(|(at, changes)| (at.offset, changes.clone()))
        .collect();
    let change = |screen, area| vec![ScrollChange { screen, area }];
    assert_eq!(
        lists,
        [
            (0x9f05, change(18, ScrollArea::Blue)),
            (0x9f08, change(18, ScrollArea::Red)),
            (0x9f0b, change(13, ScrollArea::Green)),
            (0x9f0e, change(13, ScrollArea::Red)),
        ]
    );
}
