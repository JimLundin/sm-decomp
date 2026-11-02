//! Actual lazy prepared table acquisitions; immutable loaded allocation is retained.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use sm_data::mother_brain::*;
pub fn fake_death_explosion(game: &PreparedActorState<'_>, index: isize) -> ExplosionPosition {
    ExplosionPosition {
        x: game.loaded_table_word(FAKE_DEATH_EXPLOSION_ADDRESS, 2 * index),
        y: game.loaded_table_word(FAKE_DEATH_EXPLOSION_ADDRESS, 2 * index + 1),
    }
}
pub fn brain_attack(game: &PreparedActorState<'_>, index: isize) -> BrainAttack {
    BrainAttack {
        instruction_list: game.loaded_table_word(BRAIN_ATTACK_ADDRESS, index),
    }
}
pub fn brain_palette_set(game: &PreparedActorState<'_>, index: isize) -> BrainPaletteSet {
    BrainPaletteSet {
        colors: game.loaded_table_word(BRAIN_PALETTE_SET_ADDRESS, index),
    }
}
pub fn walk_forwards(game: &PreparedActorState<'_>, index: isize) -> Walk {
    Walk {
        instruction_list: game.loaded_table_word(WALK_FORWARDS_ADDRESS, index),
    }
}
pub fn walk_backwards(game: &PreparedActorState<'_>, index: isize) -> Walk {
    Walk {
        instruction_list: game.loaded_table_word(WALK_BACKWARDS_ADDRESS, index),
    }
}
pub fn falling_tube(game: &PreparedActorState<'_>, index: isize) -> FallingTube {
    FallingTube {
        function: game.loaded_table_word(FALLING_TUBE_ADDRESS, index),
    }
}
pub fn rainbow_beam_palette(game: &PreparedActorState<'_>, index: isize) -> RainbowBeamPalette {
    RainbowBeamPalette {
        colors: game.loaded_table_word(RAINBOW_BEAM_PALETTE_ADDRESS, index),
    }
}
