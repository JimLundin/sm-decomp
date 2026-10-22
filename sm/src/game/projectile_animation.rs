//! Player projectile impact and bomb-explosion entry points.
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

#[forbid(unsafe_code)]
fn table(address: u32, index: isize, game: &Game) -> u16 {
    crate::rtl::rom_table_word!(game, address, index)
}

#[forbid(unsafe_code)]
pub fn start_impact(k: u16, game: &mut Game) {
    crate::game::projectile_lifecycle_ram::impact(k, game);
}
#[forbid(unsafe_code)]
pub fn start_bomb_explosion(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    word!(game.projectile_bomb_instruction_ptr[slot] = table(0x938683, 0, game));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}
