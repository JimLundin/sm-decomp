//! The escape typewriter, which types the Ceres and Zebes escape text one character at
//! a time into the BG2 tilemap (bank $A6:C23F-$A6:C4FF). Its state is kept in Mother
//! Brain's (or Ceres Ridley's) slot.
#![forbid(unsafe_code)]
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::SlotAccess;
use crate::cartridge::NativeDiagnostics;
use crate::display::sequence::scroll::BgScroll;
use crate::display::transfer::{Source, TilemapQueue, TilemapUpload, queue_tilemap};
use crate::effects::SoundEffects;
use crate::player_projectiles::content;
use crate::ram::CompatibilityRam;
use crate::session_slot;

/// The typewriter's view of slot 0.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Typewriter {
    pub base: EnemyBase,
    pad_0fa8: [u16; 8],
    pad_7800: [u8; 26696 + 64 + 1984],
    pad_8000: [u16; 26],
    /// $7E:8034: the tilemap entry being typed, the queued transfer's source.
    pub tilemap_entry: u16,
    /// $7E:8036: the instruction list pointer, into the text in bank $A6.
    pub text: u16,
    /// $7E:8038: the VRAM tilemap address of the next character.
    pub vram_address: u16,
    /// $7E:803A: the instruction timer, frames until the next character.
    pub timer: u16,
    /// $7E:803C: the instruction timer's reset value.
    pub timer_reset: u16,
    /// $7E:803E: the stroke timer, which sounds every second character.
    pub stroke_timer: u16,
}
same_words!(Typewriter: EnemyLayout72 {
    tilemap_entry = var_3a, text = var_3b, vram_address = var_3c, timer = var_3d,
    timer_reset = var_3e, stroke_timer = var_3f,
});

/// What the typewriter needs of its host: the palette colours, the area and the VRAM
/// tilemap queue, besides the slot words, the native text and sounds.
pub trait Host:
    SlotAccess + SoundEffects + NativeDiagnostics + CompatibilityRam + TilemapQueue
{
    /// Palette colour `index`.
    fn colour(&self, index: usize) -> u16;
    fn set_colour(&mut self, index: usize, value: u16);
    /// The current area index ($079F).
    fn area_index(&self) -> u16;
}

/// Typewriter text - Zebes escape timer ($A6:C49C).
const ZEBES_ESCAPE_TEXT: u16 = 0xc49c;
/// Instructions, by their first word; any other word types its low byte.
const END: u16 = 0;
const SET_TIMER_RESET: u16 = 1;
const SET_VRAM_ADDRESS: u16 = 13;
const CERES: u16 = 6;

/// The text word at `cursor` in bank $A6.
fn text_word(game: &mut (impl Host + ?Sized), cursor: u16) -> u16 {
    let address = 0xa6_0000 | u32::from(cursor);
    if address & 0x8000 == 0 {
        game.native_rom_diagnostic(address);
    }
    content::captured(game.native_image(), address).word(0)
}

/// Set up Zebes escape typewriter ($A6:C23F).
pub fn set_up_zebes<H: Host + ?Sized>(game: &mut H) {
    // Sprite palette 1 colours Dh..Eh = BG1/2 palette 7 colours Dh..Eh.
    for index in [125, 126] {
        let colour = game.colour(index);
        game.set_colour(index + 32, colour);
    }
    session_slot!(game, Typewriter[0].text = ZEBES_ESCAPE_TEXT);
    session_slot!(game, Typewriter[0]; vram_address = 0, timer = 0, timer_reset = 0, stroke_timer = 0);
    crate::session_cohort::with_projected(game, |_, scroll: &mut BgScroll| scroll.bg2 = [0, 0]);
}

/// Handle typewriter text ($A6:C2B1), drawing 'A' as `tiles`; true once the text ends.
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
    let source = Source {
        address: 0x8034,
        bank: 0x7e,
    };
    let upload = TilemapUpload {
        bytes: 2,
        source,
        column: address & 0x8000 != 0,
        vram_address: address,
    };
    queue_tilemap(game, upload);
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
