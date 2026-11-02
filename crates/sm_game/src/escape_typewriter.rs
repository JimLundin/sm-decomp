//! The escape typewriter, which types the Ceres and Zebes escape text one character at
//! a time into the BG2 tilemap (bank $A6:C23F-$A6:C4FF). Its state is kept in Mother
//! Brain's (or Ceres Ridley's) slot.
#![forbid(unsafe_code)]
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::SlotAccess;
use crate::cartridge::NativeDiagnostics;
use crate::display::sequence::scroll::{self, Word::Bg2H, Word::Bg2V};
use crate::display::transfer::{Source, TilemapQueue, TilemapUpload, queue_tilemap};
use crate::effects::SoundEffects;
use crate::ram::CompatibilityRam;

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

/// Set up Zebes escape typewriter ($A6:C23F).
pub fn set_up_zebes<H: Host + ?Sized>(game: &mut H) {
    owned::set_up_zebes(&mut Compatibility(game))
}

/// Handle typewriter text ($A6:C2B1), drawing 'A' as `tiles`; true once the text ends.
pub fn handle_text<H: Host + ?Sized>(game: &mut H, tiles: u16) -> bool {
    owned::handle_text(&mut Compatibility(game), tiles)
}

#[path = "escape_typewriter_owned.rs"]
pub mod owned;
struct Compatibility<'a, H: Host + ?Sized>(&'a mut H);
impl<H: Host + ?Sized> SlotAccess for Compatibility<'_, H> {
    fn slot_word(&self, at: usize) -> u16 {
        self.0.slot_word(at)
    }
    fn set_slot_word(&mut self, at: usize, value: u16) {
        self.0.set_slot_word(at, value);
    }
    fn slot_byte(&self, at: usize) -> u8 {
        self.0.slot_byte(at)
    }
    fn set_slot_byte(&mut self, at: usize, value: u8) {
        self.0.set_slot_byte(at, value);
    }
}
impl<H: Host + ?Sized> SoundEffects for Compatibility<'_, H> {
    fn queue_sound(&mut self, bank: usize, sound: u8, max: u8) {
        self.0.queue_sound(bank, sound, max);
    }
}
impl<H: Host + ?Sized> owned::Host for Compatibility<'_, H> {
    fn text_record(&mut self, cursor: u16) -> sm_data::cartridge::RomRecord {
        crate::cartridge::native_record(0xa6, cursor, self.0)
    }
    fn colour(&self, index: usize) -> u16 {
        self.0.colour(index)
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        self.0.set_colour(index, value);
    }
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16) {
        scroll::write(self.0, if horizontal { Bg2H } else { Bg2V }, value);
    }
    fn area_index(&self) -> u16 {
        self.0.area_index()
    }
    fn queue_typewriter_tile(&mut self, address: u16) {
        queue_tilemap(
            self.0,
            TilemapUpload {
                bytes: 2,
                source: Source {
                    address: 0x8034,
                    bank: 0x7e,
                },
                column: address & 0x8000 != 0,
                vram_address: address,
            },
        );
    }
}
