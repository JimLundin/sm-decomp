//! Checked charge-palette fields and the existing suit/visor output consumers.
use crate::game::beam_charge_palette::Output;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, words};
use crate::rom::RomImage;

/// Table images are acquired at each original read. The generic native copy
/// deliberately resolves its own current bank-9B source through Game.
pub fn output_with_image(
    game: &mut Game,
    output: Output,
    mut image: impl FnMut(&Game) -> RomImage,
) -> u8 {
    use crate::game::sm_91::{CopyToSamusSuitPalette, HandleVisorPalette};
    let palette = match output {
        Output::Hyper(phase) => {
            image(game).table_word(crate::rtl::lorom_offset(0x91d829) as usize, phase as isize)
        }
        Output::Charge {
            contact_attack,
            suit,
            phase,
        } => {
            let base = image(game).table_word(
                crate::rtl::lorom_offset(if contact_attack { 0x91d7ff } else { 0x91d7d5 }) as usize,
                (suit >> 1) as isize,
            );
            let address = 0x910000 | u32::from(base.wrapping_add(phase));
            if address & 0x8000 == 0 {
                unsafe {
                    crate::rtl::RomPtr(address, game);
                }
            }
            image(game).table_word(crate::rtl::lorom_offset(address) as usize, 0)
        }
        Output::Highlight => {
            for i in 0..15 {
                let at = 0xc19e - 2 * i;
                game.g_ram[at..at + 2].copy_from_slice(&1023u16.to_le_bytes());
            }
            return 0;
        }
        Output::Visor => return unsafe { HandleVisorPalette(game) },
    };
    unsafe {
        CopyToSamusSuitPalette(palette, game);
    }
    0
}
