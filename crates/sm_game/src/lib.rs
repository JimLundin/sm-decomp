//! Gameplay rules and state that the compiler keeps free of the emulator.
//!
//! Target package `sm_game` (docs/target-architecture). Modules move here when
//! they compile without the `sm` crate, its RAM layout, ROM access or backend;
//! `sm` re-exports them at their old paths while callers migrate.
#![forbid(unsafe_code)]
// Original table names (kEproj..., word_86DCB9) keep their bank-log provenance.
#![allow(non_upper_case_globals, non_snake_case)]

pub mod beam_definitions;
pub mod combat_rules;
pub mod contact_rules;
pub mod drop_rules;
pub mod enemy_population;
pub mod gameplay_flag_state;
pub mod geometry;
pub mod item_pickups;
pub mod liquid_surface;
pub mod progress;
pub mod projectile_block_reactions;
pub mod projectile_geometry;
pub mod projectile_initialization;
pub mod projectile_room;
pub mod projectile_weapon_definitions;
pub mod room_entry;
pub mod special_geometry;
pub mod sprite_effect_state;
pub mod visual_state;

pub mod actors {
    pub mod botwoon_death;
    pub mod overlap;
    pub mod state;
}
pub mod enemy_projectiles {
    pub mod tables;
}
pub mod player_collision {
    pub mod state;
}
pub mod player_transitions {
    pub mod demo;
}
pub mod player_update {
    pub mod state;
}
