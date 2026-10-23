//! Checked charge-palette fields and the existing suit/visor output consumers.
use crate::game::beam_charge_palette::{self as shared, Context, Field, Input, Output};
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, words};
use crate::rom::RomImage;

fn offset(field: Field) -> usize {
    match field {
        Field::Glow => words::charged_shot_glow_timer,
        Field::Phase => words::samus_charge_palette_index,
    }
}
struct Runtime;
impl<'g> Context<&'g mut Game> for Runtime {
    fn field(&self, game: &&'g mut Game, field: Field) -> u16 {
        let at = offset(field);
        u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
    }
    fn set_field(&mut self, game: &mut &'g mut Game, field: Field, value: u16) {
        let at = offset(field);
        game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn input(&self, game: &&'g mut Game, input: Input) -> u16 {
        match input {
            Input::Hyper => word!(game.hyper_beam_flag),
            Input::GrappleInactive => u16::from(word!(game.grapple_beam_function) == 0xc4f0),
            Input::Charge => word!(game.flare_counter),
            Input::ContactAttack => u16::from(word!(game.samus_contact_damage_index) == 4),
            Input::Suit => word!(game.samus_suit_palette_index),
        }
    }
    fn output(&mut self, game: &'g mut Game, output: Output) -> (&'g mut Game, u8) {
        let result = output_with_image(game, output, |g| {
            g.loaded_rom.as_ref().expect("ROM must be loaded").clone()
        });
        (game, result)
    }
}
pub fn advance(game: &mut Game) -> u8 {
    shared::advance(game, &mut Runtime).1
}

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
