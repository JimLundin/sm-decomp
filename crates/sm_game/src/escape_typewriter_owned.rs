//! Escape text interpreter on canonical actor extension words.
#![forbid(unsafe_code)]
use super::{CERES, END, SET_TIMER_RESET, SET_VRAM_ADDRESS, Typewriter, ZEBES_ESCAPE_TEXT};
use crate::actors::session_slots::session_slot;
pub trait Host: crate::actors::session_slots::SlotAccess + crate::effects::SoundEffects {
    fn text_record(&mut self, cursor: u16) -> sm_data::cartridge::RomRecord;
    fn colour(&self, index: usize) -> u16;
    fn set_colour(&mut self, index: usize, value: u16);
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16);
    fn queue_typewriter_tile(&mut self, address: u16);
    fn area_index(&self) -> u16;
}
fn text_word<H: Host + ?Sized>(game: &mut H, cursor: u16) -> u16 {
    game.text_record(cursor).word(0)
}
pub fn set_up_zebes<H: Host + ?Sized>(game: &mut H) {
    // Sprite palette 1 colours Dh..Eh = BG1/2 palette 7 colours Dh..Eh.
    for index in [125, 126] {
        let colour = game.colour(index);
        game.set_colour(index + 32, colour);
    }
    session_slot!(game, Typewriter[0].text = ZEBES_ESCAPE_TEXT);
    session_slot!(game, Typewriter[0]; vram_address = 0, timer = 0, timer_reset = 0, stroke_timer = 0);
    game.set_bg2_scroll(true, 0);
    game.set_bg2_scroll(false, 0);
}
pub fn handle_text<H: Host + ?Sized>(game: &mut H, tiles: u16) -> bool {
    let timer = session_slot!(game, Typewriter[0].timer);
    if timer != 0 {
        session_slot!(game, Typewriter[0].timer = timer.wrapping_sub(1));
        return false;
    }
    let reset = session_slot!(game, Typewriter[0].timer_reset);
    session_slot!(game, Typewriter[0].timer = reset);
    let mut cursor = session_slot!(game, Typewriter[0].text);
    let code = loop {
        match text_word(game, cursor) {
            END => return true,
            SET_TIMER_RESET => {
                let reset = text_word(game, cursor.wrapping_add(2));
                session_slot!(game, Typewriter[0].timer_reset = reset);
            }
            SET_VRAM_ADDRESS => {
                let address = text_word(game, cursor.wrapping_add(2));
                session_slot!(game, Typewriter[0].vram_address = address);
            }
            code => break code,
        }
        cursor = cursor.wrapping_add(4);
    };
    session_slot!(game, Typewriter[0].text = cursor.wrapping_add(1));
    let character = match code as u8 {
        b' ' => {
            session_slot!(game, Typewriter[0].vram_address, |address| address
                .wrapping_add(1));
            return false;
        }
        b'!' => 91,
        character => u16::from(character),
    };
    session_slot!(
        game,
        Typewriter[0].tilemap_entry = tiles.wrapping_add(character).wrapping_sub(65)
    );
    let address = session_slot!(game, Typewriter[0].vram_address);
    game.queue_typewriter_tile(address);
    session_slot!(game, Typewriter[0].vram_address = address.wrapping_add(1));
    let strokes = session_slot!(game, Typewriter[0].stroke_timer).wrapping_add(1);
    session_slot!(game, Typewriter[0].stroke_timer = strokes);
    if strokes.wrapping_sub(2) & 0x8000 == 0 {
        session_slot!(game, Typewriter[0].stroke_timer = 0);
        match game.area_index() {
            CERES => game.queue_sound(1, 0x45, 3),
            _ => game.queue_sound(2, 0xd, 3),
        }
    }
    false
}
