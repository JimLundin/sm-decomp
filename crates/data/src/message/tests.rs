use super::*;

/// The file offset of `$85:offset`.
fn at(offset: u16) -> usize {
    0x28000 + usize::from(offset & 0x7fff)
}

fn put(image: &mut [u8], offset: u16, words: &[u16]) {
    for (index, word) in words.iter().enumerate() {
        image[at(offset).wrapping_add(index.wrapping_mul(2))..][..2]
            .copy_from_slice(&word.to_le_bytes());
    }
}

/// A table whose first message is small with two tilemap words, the second a shoot-button
/// box with one, and the rest small boxes with none.
fn image() -> Vec<u8> {
    let mut image = vec![0; 0x30000];
    put(&mut image, 0x8000, &[0x1111; BORDER_WORDS]);
    put(&mut image, 0x8040, &[0x2222; BORDER_WORDS]);
    put(&mut image, 0x8426, &[10, 11, 12, 13, 14, 15, 16, 17]);
    put(&mut image, 0x8749, &[0, 0x12a]);
    let mut definitions = vec![
        SMALL_SETUP,
        SMALL_DRAW,
        0x9000,
        SHOOT_SETUP,
        LARGE_DRAW,
        0x9004,
    ];
    for _ in 2..DEFINITION_COUNT {
        definitions.extend([SMALL_SETUP, SMALL_DRAW, 0x9006]);
    }
    put(&mut image, 0x869b, &definitions);
    put(&mut image, 0x9000, &[0xaaaa, 0xbbbb, 0xcccc]);
    image
}

#[test]
fn a_messages_tilemap_runs_to_the_next_ones() {
    let image = image();
    let boxes = MessageBoxes::load(Lorom::new(&image)).expect("tables load");
    let first = boxes.message(1).expect("message 1");
    assert_eq!(first.tilemap, [0xaaaa, 0xbbbb]);
    assert_eq!(first.setup, Setup::Small);
    assert_eq!(first.borders, BorderSize::Small);
    let second = boxes.message(2).expect("message 2");
    assert_eq!(second.tilemap, [0xcccc]);
    assert_eq!(second.setup, Setup::ShootButton);
    assert_eq!(second.borders, BorderSize::Large);
    assert_eq!(second.button, 0x12a);
}

#[test]
fn the_last_definition_and_a_backwards_one_have_no_words() {
    let image = image();
    let boxes = MessageBoxes::load(Lorom::new(&image)).expect("tables load");
    assert_eq!(
        boxes
            .message(u16::try_from(DEFINITION_COUNT).expect("fits"))
            .map(|m| m.tilemap.len()),
        Some(0)
    );
    assert_eq!(boxes.message(0), None);
    assert_eq!(boxes.message(0x1e), None);
}

#[test]
fn rows_start_where_the_setup_puts_them() {
    assert_eq!(Setup::Small.rows(), 0x1c0);
    assert_eq!(Setup::Large.rows(), 0x1a0);
    assert_eq!(Setup::ShootButton.rows(), 0x1a0);
    assert_eq!(Setup::RunButton.rows(), 0x1a0);
}

#[test]
fn borders_and_letters_come_from_their_tables() {
    let image = image();
    let boxes = MessageBoxes::load(Lorom::new(&image)).expect("tables load");
    assert_eq!(boxes.border(BorderSize::Large), [0x1111; BORDER_WORDS]);
    assert_eq!(boxes.border(BorderSize::Small), [0x2222; BORDER_WORDS]);
    assert_eq!(boxes.letter(Letter::A), Some(10));
    assert_eq!(boxes.letter(Letter::Select), Some(14));
    assert_eq!(boxes.letter(Letter::Blank), Some(17));
}

#[test]
fn an_unknown_routine_is_an_error() {
    let mut image = image();
    put(&mut image, 0x869b, &[0x8000]);
    assert_eq!(
        MessageBoxes::load(Lorom::new(&image)),
        Err(MessageError::Routine(0x8000))
    );
}

#[test]
fn a_table_past_the_image_is_an_error() {
    let image = vec![0; 0x10000];
    assert!(matches!(
        MessageBoxes::load(Lorom::new(&image)),
        Err(MessageError::Address(_))
    ));
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<crate::rom::Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    crate::rom::Rom::new(std::fs::read(path).ok()?).ok()
}

#[test]
fn the_tables_decode_as_the_bank_log_says() {
    let Some(rom) = rom() else {
        return;
    };
    let boxes = MessageBoxes::load(rom.lorom()).expect("tables load");
    // Morphing ball: a small box over a 40h-byte tilemap, with the small box's borders.
    let morph = boxes.message(9).expect("morphing ball");
    assert_eq!(
        (morph.setup, morph.borders, morph.tilemap.len()),
        (Setup::Small, BorderSize::Small, 0x20)
    );
    // Missile: a large box with the shoot button, drawn at $012A.
    let missile = boxes.message(2).expect("missile");
    assert_eq!(
        (missile.setup, missile.borders, missile.button),
        (Setup::ShootButton, BorderSize::Large, 0x12a)
    );
    // The borders' words, the box's edge tiles.
    assert_eq!(boxes.border(BorderSize::Small)[6], 0x284e);
    assert_eq!(boxes.border(BorderSize::Large)[3], 0x284e);
}
