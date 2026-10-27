//! Blue Brinstar Face Block proximity activation and palette cycling.
//! Native hook words, actor words and little-endian hook variables stay authoritative.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::api::{self, Axis};
use crate::game::enemy_layouts::EnemyLayout8 as Face;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::abs16;

fn publish_palette(E: i32, game: &mut Game) {
    let palette = slot!(game, Face[E].base.palette_index);
    word!(game.variables_for_enemy_graphics_drawn_hook[0] = (palette >> 4) & 0xff);
}
fn within_limit(E: i32, delta: u16, game: &mut Game) -> bool {
    (abs16(delta) as i16).wrapping_sub(slot!(game, Face[E].parameter_1) as i16) < 0
}
pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.cur_enemy_index) as i32;
    slot!(game, Face[E].base.current_instruction = 0xe828);
    let hook: u16 = if word!(game.collected_items) & 4 == 0 {
        0x804c
    } else {
        0xe86e
    };
    game.ram().copy_in(0x178c, &hook.to_ne_bytes());
    game.ram().copy_in(0x178e, &0xa8u16.to_ne_bytes());
    publish_palette(E, game);
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
    slot!(game, Face[E].parameter_2, |old| (old & 1) << 15);
}
pub(crate) fn advance(game: &mut Game) {
    if word!(game.collected_items) & 4 == 0 {
        return;
    }
    game.ram().copy_in(0x178c, &0xe86eu16.to_ne_bytes());
    let E = word!(game.cur_enemy_index) as i32;
    publish_palette(E, game);
    if slot!(game, Face[E].var_A) != 0 {
        return;
    }
    let key = Key::from_legacy(word!(game.cur_enemy_index));
    let dy = api::player_delta(&Runtime::new(game), key, Axis::Y);
    if !within_limit(E, dy, game) {
        return;
    }
    let key = Key::from_legacy(word!(game.cur_enemy_index));
    let dx = api::player_delta(&Runtime::new(game), key, Axis::X);
    slot!(game, Face[E].var_B = dx);
    if within_limit(E, dx, game)
        && slot!(game, Face[E].var_B) & 0x8000 != slot!(game, Face[E].parameter_2)
    {
        let instruction = if slot!(game, Face[E].var_B) & 0x8000 == 0 {
            0xe81a
        } else {
            0xe80c
        };
        slot!(game, Face[E].base; current_instruction = instruction, instruction_timer = 1);
        slot!(game, Face[E].var_A = 1);
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
    }
}
pub(crate) fn palette(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) != 0
        || word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) != 0
    {
        return;
    }
    word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
    let mut destination = word!(game.variables_for_enemy_graphics_drawn_hook[0]);
    let mut source = 8u16.wrapping_mul(word!(game.variables_for_enemy_graphics_drawn_hook[1]));
    for _ in 0..4 {
        let color = crate::rtl::rom_table_word!(game, 0xa8e7cc, (source >> 1) as isize);
        let at = 0xc112 + usize::from(destination >> 1) * 2;
        game.ram().copy_in(at, &color.to_ne_bytes());
        source = source.wrapping_add(2);
        destination = destination.wrapping_add(2);
    }
    // The legacy phase increment reads physical byte zero, even on a big-endian host.
    word!(
        game.variables_for_enemy_graphics_drawn_hook[1] =
            (u16::from(game.ram().read_byte(0x1796)) + 1) & 7
    );
}
