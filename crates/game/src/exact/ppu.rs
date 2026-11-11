//! The PPU set-up the message box writes itself while the NMI lags: its text colours straight
//! to CGRAM, the playfield it shows over, a black backdrop and BG3's scroll, and the way back.
//!
//! The box's BG3 Y scroll HDMA channel is not here: it goes with the line program's HDMA
//! channels (E3), and `message_box` still points the channel itself.

use crate::state::video::{
    ColourWrite, Layer, Layers, Playfield, PlayfieldColourMath, Scroll, TilemapBase, Video,
};

/// The text colours the box writes to CGRAM, from BG3 palette 6's colour 1: the Y button's
/// green and the save arrow's red.
const TEXT_COLOURS_START: usize = 0x19;
const TEXT_COLOURS: [u16; 2] = [0x0BB1, 0x001F];
/// Where in CGRAM the text colours are.
const TEXT_COLOUR_RANGE: std::ops::Range<usize> =
    TEXT_COLOURS_START..TEXT_COLOURS_START + TEXT_COLOURS.len();

/// The playfield the box shows over: its BG3 tilemap at the overlay's, 32x32, and BG1, BG2,
/// BG3 and sprites on the main screen, with no colour math.
const BOX_BG3_TILEMAP: TilemapBase = TilemapBase(0x58);
const BOX_MAIN_SCREEN: [Layer; 4] = [Layer::Bg1, Layer::Bg2, Layer::Bg3, Layer::Objects];

/// The three COLDATA writes that make the sub screen's backdrop black.
const BLACK_BACKDROP: [ColourWrite; 3] = [ColourWrite(0x20), ColourWrite(0x40), ColourWrite(0x80)];

/// BG3's index among the backgrounds.
const BG3: usize = 2;

/// Set the PPU up for the box: BG3 palette 6's text colours, the playfield it shows over
/// (keeping BG3's tilemap base for the way back), a black sub screen backdrop and BG3's
/// scroll at 0.
pub fn set_up_message_box(video: &mut Video) {
    video.message_box.bg3_tilemap = video.playfield.bg3_tilemap;
    if let Some(colours) = video.memory.cgram.get_mut(TEXT_COLOUR_RANGE) {
        for (colour, &value) in colours.iter_mut().zip(&TEXT_COLOURS) {
            colour.0 = value;
        }
    }
    let mut main_screen = Layers::default();
    for layer in BOX_MAIN_SCREEN {
        main_screen.insert(layer);
    }
    video.playfield = Playfield {
        main_screen,
        bg3_tilemap: BOX_BG3_TILEMAP,
        colour_math: PlayfieldColourMath::default(),
    };
    let committed = &mut video.committed;
    committed.colour_math.fixed = BLACK_BACKDROP
        .iter()
        .fold(committed.colour_math.fixed, |colour, &write| {
            colour.with(write)
        });
    // Each scroll register takes two writes of 0: the low then the high byte.
    if let Some(scroll) = committed.scroll.get_mut(BG3) {
        *scroll = Scroll::default();
    }
}

/// Put the PPU back after the box: BG3's tilemap base it kept, the main screen and colour math
/// from the mirrors the NMI commits, and the text colours from the palette buffer.
// open: HDMA's enable comes back from what the box saved when it turned HDMA off, which joins
// with the HDMA channels (E3).
pub fn restore_after_message_box(video: &mut Video) {
    video.playfield.bg3_tilemap = video.message_box.bg3_tilemap;
    video.playfield.main_screen = video.batch.registers.main_screen;
    video.playfield.colour_math = video.batch.playfield_colour_math;
    if let (Some(colours), Some(saved)) = (
        video.memory.cgram.get_mut(TEXT_COLOUR_RANGE),
        video.batch.cgram.get(TEXT_COLOUR_RANGE),
    ) {
        colours.copy_from_slice(saved);
    }
}

#[cfg(test)]
mod tests;
