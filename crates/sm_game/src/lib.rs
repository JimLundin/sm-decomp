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
pub mod dialogue;
pub mod drop_rules;
pub mod effects;
pub mod enemy_population;
pub mod external_inputs;
pub mod gameplay_flag_state;
pub mod geometry;
pub mod imported;
pub mod item_pickups;
pub mod launch_motion;
pub mod liquid_surface;
pub mod message_composition;
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
pub mod sprite_effect_state;
pub mod visual_state;
pub mod word_transfers;

pub mod display {
    pub mod control;
    pub mod error;
    pub mod hdma;
    pub mod transfer;
}

pub mod actors {
    pub mod access;
    pub mod botwoon_death;
    pub mod codec;
    pub mod content;
    pub mod effects;
    #[doc(hidden)] // Shared verification world for compat and detached checks.
    pub mod fixture;
    pub mod identity;
    pub mod media;
    pub mod overlap;
    pub mod scene;
    pub mod snapshot;
    pub mod state;
}
pub mod enemy_projectiles {
    pub mod access;
    pub mod attachments_spore_maw;
    pub mod attachments_waffle_botwoon;
    pub mod collision;
    pub mod contact;
    pub mod content;
    pub mod coordinates;
    pub mod host;
    pub mod presentation;
    pub mod tables;
}
pub mod player_collision {
    pub mod effects;
    pub mod state;
}
pub mod player_transitions {
    pub mod content;
    pub mod demo;
    pub mod effects;
    pub mod requests;
    pub mod state;
}
pub mod player_update {
    pub mod content;
    pub mod effects;
    pub mod minimap;
    pub mod state;
}

pub mod animated_tiles {
    pub mod execution;
}
pub mod checkpoint_binding;
pub mod enemy_script;
pub mod gameplay_flag_host;
pub mod palette_effects {
    pub mod execution;
}
pub mod player_movement {
    pub mod content;
    pub mod effects;
}
pub mod player_projectiles {
    pub mod access;
    pub mod content;
    pub mod host;
    pub mod placement;
}
pub mod room_mechanisms {
    pub mod access;
    pub mod content;
    pub mod creation;
    pub mod graphics;
    pub mod host;
    pub mod presentation;
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
pub mod samus_periodic_damage;
pub mod scene;
pub mod session;
pub mod session_access;
pub mod session_checkpoint;
pub mod session_owners;
pub mod sprite_effect_controllers;
pub mod sprite_effect_host;
pub mod sprite_objects;
pub mod visual_host;
