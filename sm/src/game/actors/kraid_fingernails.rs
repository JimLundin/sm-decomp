//! Kraid's fingernail projectiles: shared initialization from the parent body.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::constants::enemy_properties::kEnemyProps_Invisible;
use crate::game::enemy_layouts::KraidLayout as Kraid;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::slot;

/// The fingernail instruction list at $A7:8B0A; its first entry's spritemap
/// word (offset 2) seeds the part before the list runs.
const INSTRUCTIONS: u16 = 0x8b0a;

pub(crate) fn KraidFingernailInit(k: u16, game: &mut Game) {
    let palette_index = slot!(game, Kraid[0].base.palette_index);
    let e = i32::from(k);
    slot!(game, Kraid[e].base.palette_index = palette_index);
    slot!(game, Kraid[e].kraid_var_B = 40);
    slot!(
        game,
        Kraid[e].base.properties =
            slot!(game, Kraid[e].base.properties) | kEnemyProps_Invisible as u16
    );
    slot!(game, Kraid[e].base; instruction_timer = 0x7fff, current_instruction = INSTRUCTIONS);
    let spritemap = native_record(0xa7, INSTRUCTIONS, game).word(2);
    slot!(game, Kraid[e].base.spritemap_pointer = spritemap);
    slot!(game, Kraid[e].kraid_next = 0xbd60);
    slot!(game, Kraid[e].kraid_var_A = 0xb92d);
    slot!(game, Kraid[e].kraid_var_F = 64);
}
