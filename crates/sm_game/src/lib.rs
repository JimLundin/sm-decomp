//! Gameplay rules and state that the compiler keeps free of the emulator.
//!
//! Target package `sm_game` (docs/target-architecture). Modules move here when
//! they compile without the `sm` crate, its RAM layout, ROM access or backend;
//! `sm` re-exports them at their old paths while callers migrate.
#![forbid(unsafe_code)]

// Paths the moved compat modules were written against.
pub(crate) use progress as progress_flags;
pub(crate) use sm_data::cartridge as rom;

pub mod beam_definitions;
pub mod cartridge;
pub mod combat_rules;
pub mod constants;
pub mod contact_rules;
pub mod decompression;
pub mod demo_rooms;
pub mod dialogue;
pub mod dialogue_ram;
pub mod drop_rules;
pub mod effects;
pub mod enemy_population;
pub mod escape_timer;
pub mod external_inputs;
pub mod gameplay_flag_state;
pub mod geometry;
pub mod hud;
pub mod imported;
pub mod item_pickups;
pub mod launch_motion;
pub mod layer_blending;
pub mod liquid_effects;
pub mod liquid_surface;
pub mod menus;
pub mod message_composition;
pub mod palette_copies;
pub mod probe;
pub mod progress;
pub mod projectile_block_reactions;
pub mod projectile_geometry;
pub mod projectile_initialization;
pub mod projectile_room;
pub mod projectile_weapon_definitions;
pub mod ram;
pub mod room_entry;
pub mod save_file;
pub mod special_geometry;
pub mod sprite_drawing;
pub mod sprite_effect_state;
pub mod sprite_output;
pub mod visual_state;
pub mod word_transfers;

pub mod display {
    pub mod control;
    pub mod error;
    pub mod hdma;
    pub mod hdma_spawn;
    pub mod registers;
    pub mod sequence;
    pub mod transfer;
}

pub mod actors {
    pub mod access;
    pub mod animation;
    pub mod api;
    pub mod botwoon_death;
    pub mod codec;
    pub mod combat;
    pub mod contacts;
    pub mod content;
    pub mod dachora;
    pub mod drawing;
    pub mod effects;
    pub mod escape_animals;
    #[doc(hidden)] // Shared verification world for compat and detached checks.
    pub mod fixture;
    pub mod host;
    pub mod identity;
    pub mod kraid_fingernails;
    pub mod kraid_lint;
    pub mod kraid_sinking;
    pub mod layouts;
    pub mod media;
    pub mod mother_brain_effects;
    pub mod movement;
    pub mod norfair_lava_man;
    pub mod overlap;
    pub mod phantoon_actions;
    pub mod phantoon_motion;
    pub mod population;
    pub mod queries;
    pub mod ridley_tables;
    pub mod scene;
    pub mod scheduling;
    pub mod shaktool;
    pub mod shitroid;
    pub mod snapshot;
    pub mod staging;
    pub mod state;
    pub mod torizo;
    pub mod yapping_maw;
}
pub mod enemy_projectiles;
pub mod player_collision;
pub mod player_transitions;
pub mod player_update;

pub mod animated_tiles {
    pub mod execution;
}
pub mod checkpoint_binding;
pub mod enemy_script;
pub mod gameplay_flag_host;
pub mod palette_effects {
    pub mod execution;
    pub mod fading;
}
pub mod player_movement;
pub mod player_projectiles {
    pub mod access;
    pub mod animation;
    pub mod bomb_creation;
    pub mod bomb_terrain;
    pub mod bombs;
    pub mod cannon;
    pub mod collision;
    pub mod content;
    pub mod feedback;
    pub mod host;
    pub mod lifecycle;
    pub mod missiles;
    pub mod motion;
    pub mod placement;
    pub mod preparation;
    pub mod presentation;
    pub mod specials;
    pub mod weapons;
    // Paths the projectile modules use from their parent.
    use crate::session as state;
    use host::Host;
}
pub mod room_mechanisms {
    pub mod access;
    pub mod commands;
    pub mod content;
    pub mod creation;
    pub mod graphics;
    pub mod host;
    pub mod pre;
    pub mod presentation;
    pub mod scheduler;
    pub mod rules {
        pub mod animal_rescue;
        pub mod booster_escape;
        pub mod cannon_setup;
        pub mod contact_physics;
        pub mod crumbling_wall;
        pub mod doors;
        pub mod eye_door;
        pub mod gates;
        pub mod hands;
        pub mod item_setup;
        pub mod metroid_quota;
        pub mod mother_brain_glass;
        pub mod noob_tube;
        pub mod plants;
        pub mod scroll_setup;
        pub mod shaktool;
        pub mod sound_delay;
        pub mod stations;
        pub mod terrain_setup;
        pub mod torizo_statue;
        pub mod tourian_wall;
        pub mod treadmill;
    }
    pub mod setup;
}
pub mod grapple_rope;
pub mod samus_palette;
pub mod samus_periodic_damage;
pub mod scene;
pub mod scrolling_sky;
pub mod session;
pub mod session_access;
pub mod session_checkpoint;
pub mod session_owners;
pub mod sprite_effect_controllers;
pub mod sprite_effect_host;
pub mod sprite_objects;
#[cfg(sm_verify_dll)]
pub mod transition_trace;
pub mod visual_host;
