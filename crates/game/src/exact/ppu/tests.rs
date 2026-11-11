use super::{restore_after_message_box, set_up_message_box};
use crate::state::video::{Colour, FixedColour, Layer, Scroll, TilemapBase, Video};

#[test]
fn setting_up_writes_the_text_colours_and_the_box_playfield() {
    let mut video = Video::default();
    video.committed.colour_math.fixed = FixedColour {
        red: 6,
        green: 2,
        blue: 1,
    };
    video.committed.scroll[2] = Scroll { x: 5, y: 9 };
    video.playfield.colour_math.layers.0 = 0xB3;
    video.batch.cgram[0x19] = Colour(0x1234);
    set_up_message_box(&mut video);
    assert_eq!(
        video.memory.cgram[0x19..0x1B],
        [Colour(0x0BB1), Colour(0x001F)]
    );
    // The palette buffer stays for the next requested NMI.
    assert_eq!(video.batch.cgram[0x19], Colour(0x1234));
    assert_eq!(video.playfield.bg3_tilemap, TilemapBase(0x58));
    assert_eq!(video.playfield.main_screen.bits, 0x17);
    assert!(!video.playfield.main_screen.contains(Layer::Bg4));
    assert_eq!(video.playfield.colour_math.layers.0, 0);
    assert_eq!(video.committed.colour_math.fixed, FixedColour::default());
    assert_eq!(video.committed.scroll[2], Scroll::default());
}

#[test]
fn restoring_takes_the_mirrors_and_the_palette_buffer_back() {
    let mut video = Video::default();
    video.playfield.bg3_tilemap = TilemapBase(0x5A);
    video.batch.registers.main_screen.insert(Layer::Bg1);
    video.batch.playfield_colour_math.layers.0 = 0xB3;
    video.batch.cgram[0x19] = Colour(0x524A);
    video.batch.cgram[0x1B] = Colour(0x7FFF);
    set_up_message_box(&mut video);
    restore_after_message_box(&mut video);
    assert_eq!(video.playfield.bg3_tilemap, TilemapBase(0x5A));
    assert_eq!(video.playfield.main_screen.bits, 0x01);
    assert_eq!(video.playfield.colour_math.layers.0, 0xB3);
    assert_eq!(
        video.memory.cgram[0x19..0x1C],
        [Colour(0x524A), Colour(0), Colour(0)]
    );
}
