use clean_types::{Flip, Look, Picture};

use super::*;

/// An image as big as the HUD's tables' banks reach, every byte 0.
fn blank_image() -> Vec<u8> {
    vec![0; 0x9_0000]
}

/// Writes `words` at `address`.
fn put(image: &mut [u8], address: RomAddress, words: &[u16]) {
    let offset = Lorom::new(image).file_offset(address).unwrap();
    for (bytes, word) in image[offset..].as_chunks_mut::<2>().0.iter_mut().zip(words) {
        *bytes = word.to_le_bytes();
    }
}

/// The look of an invented word: picture `number`, flipped across when `across`, in colours
/// the importer drops.
fn word(number: u16, across: bool) -> u16 {
    number | 0x3C00 | if across { 0x4000 } else { 0 }
}

fn look(number: u16, across: bool) -> Look {
    Look {
        picture: Picture::new(number).unwrap(),
        flip: if across { Flip::Across } else { Flip::None },
    }
}

/// `count` invented words from `first`, every other one flipped across.
fn run(first: u16, count: u16) -> Vec<u16> {
    (0..count)
        .map(|k| word(first.wrapping_add(k), k % 2 == 1))
        .collect()
}

#[test]
fn each_picture_is_read_from_its_table() {
    let mut image = blank_image();
    put(&mut image, BLANK, &[word(0x101, false)]);
    put(&mut image, DIGITS, &run(0x110, 10));
    put(&mut image, FULL_TANK, &[word(0x121, false)]);
    put(&mut image, EMPTY_TANK, &[word(0x122, true)]);
    put(&mut image, RESERVE_SIGN, &run(0x130, 6));
    put(&mut image, MISSILES_ICON, &run(0x140, 6));
    put(&mut image, SUPER_MISSILES_ICON, &run(0x150, 4));
    put(&mut image, POWER_BOMBS_ICON, &run(0x160, 4));
    put(&mut image, GRAPPLE_BEAM_ICON, &run(0x170, 4));
    put(&mut image, X_RAY_SCOPE_ICON, &run(0x180, 4));
    put(&mut image, MAP_BLANK, &[word(0x191, false)]);
    put(&mut image, DIAGONAL, &[word(0x192, false)]);
    let pictures = HudPictures::load(Lorom::new(&image)).unwrap();
    let pair = |first: u16| [look(first, false), look(first.wrapping_add(1), true)];
    let two_by_two = |first: u16| [pair(first), pair(first.wrapping_add(2))];
    assert_eq!(
        pictures,
        HudPictures {
            blank: Picture::new(0x101).unwrap(),
            digits: core::array::from_fn(|d| {
                let d = u16::try_from(d).unwrap();
                look(0x110_u16.wrapping_add(d), d % 2 == 1)
            }),
            full_tank: look(0x121, false),
            empty_tank: look(0x122, true),
            reserve_sign: [pair(0x130), pair(0x132), pair(0x134)],
            icons: ItemIcons {
                missiles: [
                    [look(0x140, false), look(0x141, true), look(0x142, false)],
                    [look(0x143, true), look(0x144, false), look(0x145, true)],
                ],
                super_missiles: two_by_two(0x150),
                power_bombs: two_by_two(0x160),
                grapple_beam: two_by_two(0x170),
                x_ray_scope: two_by_two(0x180),
            },
            map_blank: look(0x191, false),
            diagonal: Picture::new(0x192).unwrap(),
        }
    );
}

#[test]
fn a_word_keeps_its_flips_and_drops_its_colours() {
    let mut image = blank_image();
    put(&mut image, MAP_BLANK, &[0xDD5A]);
    let pictures = HudPictures::load(Lorom::new(&image)).unwrap();
    assert_eq!(
        pictures.map_blank,
        Look {
            picture: Picture::new(0x15A).unwrap(),
            flip: Flip::Both,
        }
    );
}

#[test]
fn a_short_image_is_refused() {
    assert!(HudPictures::load(Lorom::new(&[0; 0x100])).is_err());
}
