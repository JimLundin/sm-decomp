//! The escape typewriter, which types the Ceres and Zebes escape text one character at
//! a time into the BG2 tilemap (bank $A6:C23F-$A6:C4FF). Its state is kept in Mother
//! Brain's (or Ceres Ridley's) slot.
#![forbid(unsafe_code)]
use crate::actors::slot_view::{SlotState, slot_view};
use crate::cartridge::NativeDiagnostics;
use crate::display::transfer::{Source, TilemapQueue, TilemapUpload, queue_tilemap};
use crate::display_scroll::{self as scroll, Word::Bg2H, Word::Bg2V};
use crate::effects::SoundEffects;
use crate::player_projectiles::content;
use crate::ram::CompatibilityRam;

slot_view! {
    /// The typewriter's view of slot 0: the tilemap entry being typed (the
    /// queued transfer's source, $7E:8034), the instruction list pointer into
    /// the text in bank $A6, the VRAM tilemap address of the next character,
    /// the instruction timer (frames until the next character) and the stroke
    /// timer, which sounds every second character. The instruction timer's
    /// reset value, slot 0's `var_3e`, is Botwoon's death word in the Session:
    /// [`timer_reset`] and [`set_timer_reset`].
    Typewriter, TypewriterEdit {
        record: [],
        extended: [
            tilemap_entry = var_3a, text = var_3b, vram_address = var_3c, timer = var_3d,
            stroke_timer = var_3f,
        ],
    }
}

/// The instruction timer's reset value ($7E:803C), slot 0's `var_3e`.
pub fn timer_reset(game: &(impl SlotState + ?Sized)) -> u16 {
    game.session().botwoon_death.tail_ready
}

/// Store the instruction timer's reset value ($7E:803C); like the record's
/// stores, it refreshes the host's observations.
pub fn set_timer_reset(game: &mut (impl SlotState + ?Sized), reset: u16) {
    game.session_mut().botwoon_death.tail_ready = reset;
    game.record_stored();
}

/// What the typewriter needs of its host: the palette colours, the area and the VRAM
/// tilemap queue, besides the slot words, the native text and sounds.
pub trait Host:
    SlotState + SoundEffects + NativeDiagnostics + CompatibilityRam + TilemapQueue
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
    *TypewriterEdit::of(game, 0).text() = ZEBES_ESCAPE_TEXT;
    *TypewriterEdit::of(game, 0).vram_address() = 0;
    *TypewriterEdit::of(game, 0).timer() = 0;
    set_timer_reset(game, 0);
    *TypewriterEdit::of(game, 0).stroke_timer() = 0;
    scroll::write(game, Bg2H, 0);
    scroll::write(game, Bg2V, 0);
}

/// Handle typewriter text ($A6:C2B1), drawing 'A' as `tiles`; true once the text ends.
pub fn handle_text<H: Host + ?Sized>(game: &mut H, tiles: u16) -> bool {
    let timer = Typewriter::of(game, 0).timer();
    if timer != 0 {
        *TypewriterEdit::of(game, 0).timer() = timer.wrapping_sub(1);
        return false;
    }
    let reset = timer_reset(game);
    *TypewriterEdit::of(game, 0).timer() = reset;
    let mut cursor = Typewriter::of(game, 0).text();
    let code = loop {
        match text_word(game, cursor) {
            END => return true,
            SET_TIMER_RESET => {
                let reset = text_word(game, cursor.wrapping_add(2));
                set_timer_reset(game, reset);
            }
            SET_VRAM_ADDRESS => {
                let address = text_word(game, cursor.wrapping_add(2));
                *TypewriterEdit::of(game, 0).vram_address() = address;
            }
            code => break code,
        }
        cursor = cursor.wrapping_add(4);
    };
    *TypewriterEdit::of(game, 0).text() = cursor.wrapping_add(1);
    let character = match code as u8 {
        b' ' => {
            let address = Typewriter::of(game, 0).vram_address();
            *TypewriterEdit::of(game, 0).vram_address() = address.wrapping_add(1);
            return false;
        }
        b'!' => 91,
        character => u16::from(character),
    };
    *TypewriterEdit::of(game, 0).tilemap_entry() = tiles.wrapping_add(character).wrapping_sub(65);
    let address = Typewriter::of(game, 0).vram_address();
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
    *TypewriterEdit::of(game, 0).vram_address() = address.wrapping_add(1);
    let strokes = Typewriter::of(game, 0).stroke_timer().wrapping_add(1);
    *TypewriterEdit::of(game, 0).stroke_timer() = strokes;
    if strokes.wrapping_sub(2) & 0x8000 == 0 {
        *TypewriterEdit::of(game, 0).stroke_timer() = 0;
        match game.area_index() {
            CERES => game.queue_sound(1, 0x45, 3),
            _ => game.queue_sound(2, 0xd, 3),
        }
    }
    false
}
