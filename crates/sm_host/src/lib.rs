//! Super Metroid in Rust.
//!
//! Mechanically transpiled from snesrev/sm (rev 578f90b) with c2rust 0.22.1 on
//! 2026-10-17; see `tools/transpile/README.md`. The C source is no longer part
//! of this project. Everything here is now maintained Rust; the plan for making
//! it idiomatic without changing behaviour is in `docs/PLAN.md`.
//!
//! Gameplay and runtime state is owned by `game_state::Game` and passed
//! explicitly through Rust calls; C ABI remains at external boundaries.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]

// Run the body before testing the guard; body locals remain in scope.
#[macro_export]
macro_rules! do_until {
    ($guard:expr; $($body:tt)*) => {
        loop {
            $($body)*
            if $guard {
                break;
            }
        }
    };
}

// Access byte lanes through the original caller-side raw pointer.
#[macro_export]
macro_rules! byte {
    ($place:expr; $lane:expr) => {
        *(&raw mut $place as *mut u8).offset($lane)
    };
    (@pointer $pointer:expr; $lane:expr) => {
        *($pointer as *mut u8).offset($lane)
    };
}

// Emit place updates in caller scope, preserving the original capture and stores.
#[macro_export]
macro_rules! update_place {
    (reference $destination:ident $([$old:ident])? = $place:expr => $value:expr;) => {
        let ref mut $destination = $place;
        $(let $old = *$destination;)?
        *$destination = $value;
    };
    (previous $old:ident $(: $type:ty)? = $place:expr => $value:expr;) => {
        let $old $(: $type)? = $place;
        $place = $value;
    };
    (previous mut $old:ident $(: $type:ty)? = $place:expr => $value:expr;) => {
        let mut $old $(: $type)? = $place;
        $place = $value;
    };

    // RAM words may start at odd byte addresses. Capture once without a reference;
    // keep old-value loads and RHS temporaries in caller order through the store.
    (unaligned_word $destination:ident $([$old:ident])? = $pointer:expr => $value:expr;) => {
        let $destination: *mut u16 = $pointer;
        $(let $old = u16::from_le($destination.read_unaligned());)?
        $crate::rtl::write_word($value, $destination);
    };
}

// Declare forwarding wrappers with their original signatures and call order.
// Unit wrappers retain a statement; value-returning wrappers retain a tail call.
#[macro_export]
macro_rules! forward_calls {
    ($visibility:vis unsafe fn $parameters:tt -> $result:ty; $($name:ident => $call:expr;)+) => {
        $($visibility unsafe fn $name $parameters -> $result { #[allow(unused_unsafe)] unsafe { $call }})+
    };
    ($visibility:vis unsafe fn $parameters:tt; $($name:ident => $call:expr;)+) => {
        $($visibility unsafe fn $name $parameters { #[allow(unused_unsafe)] unsafe { $call; }})+
    };
    ($visibility:vis fn $parameters:tt -> $result:ty; $($name:ident => $call:expr;)+) => {
        $($visibility fn $name $parameters -> $result { $call })+
    };
    ($visibility:vis fn $parameters:tt; $($name:ident => $call:expr;)+) => {
        $($visibility fn $name $parameters { $call; })+
    };
}

// Share explicit record metadata and consecutive field types without changing layout.
macro_rules! typed_struct {
    (initial; $attributes:tt; $($visibility:vis struct $name:ident {
        $($field_visibility:vis $($field:ident),+ : $field_type:ty;)*
    })+) => {
        $($crate::typed_struct!(@attributes $attributes $visibility struct $name {
            $($field_visibility $($field),+: $field_type;)*
        });
        #[forbid(unsafe_code)]
        impl $crate::initial_state::InitialState for $name {
            const INITIAL: Self = Self {
                $($($field: <$field_type as $crate::initial_state::InitialState>::INITIAL,)*)*
            };
        })+
    };

    ($(#[$metadata:meta])* $visibility:vis struct $name:ident $fields:tt) => {
        $crate::typed_struct!(@attributes [$(#[$metadata])*] $visibility struct $name $fields);
    };
    ($attributes:tt; $($visibility:vis $kind:ident $name:ident $fields:tt)+) => {
        $($crate::typed_struct!(@attributes $attributes $visibility $kind $name $fields);)+
    };
    (@attributes [$(#[$metadata:meta])*] $visibility:vis $kind:ident $name:ident {
        $($field_visibility:vis $($field:ident),+ : $field_type:ty;)*
    }) => {
        $(#[$metadata])*
        $visibility $kind $name {
            $($($field_visibility $field: $field_type,)*)*
        }
    };
}
pub(crate) use typed_struct;

// Preserve each original assertion scope and the caller's C failure binding.
#[macro_export]
macro_rules! native_assertion {
    ($label:lifetime: $condition:expr; $($diagnostic:tt)*) => {
        $label: {
            if $condition {
            } else {
                $crate::native_assertion!(@failure $($diagnostic)*);
            }
        }
    };
    ($label:lifetime; $($diagnostic:tt)*) => {
        $label: {
            $crate::native_assertion!(@failure $($diagnostic)*);
        }
    };
    (@failure $assertion:literal, $file:literal, $line:expr, $function:literal $(,)?) => {
        $crate::c_library::__assert_fail(
            $assertion as *const u8 as *const ::core::ffi::c_char,
            $file as *const u8 as *const ::core::ffi::c_char,
            $line,
            $function as *const u8 as *const ::core::ffi::c_char,
        )
    };
    (@failure $assertion:literal, $file:literal, $line:expr, pointer $function:expr $(,)?) => {
        $crate::c_library::__assert_fail(
            $assertion as *const u8 as *const ::core::ffi::c_char,
            $file as *const u8 as *const ::core::ffi::c_char,
            $line,
            $function,
        )
    };
}

// Emit ordered field stores without hoisting the receiver or values.
macro_rules! set_fields {
    ($receiver:tt; $($field:ident = $value:expr),+ $(,)?) => {
        $($crate::set_fields!(@store $receiver, $field, $value);)+
    };

    ($receiver:tt, $value:expr; $($field:ident),+ $(,)?) => {
        $($crate::set_fields!(@store $receiver, $field, $value);)+
    };
    (@store [$($receiver:tt)*], $field:ident, $value:expr) => {
        $($receiver)*.$field = $value
    };
}
pub(crate) use set_fields;

/// Gameplay, one module per original ROM bank (`sm_80` = bank $80, ...).
// Access macros of the moved projectile, enemy-projectile and room mechanism modules.
#[macro_use]
extern crate sm_game;
#[path = "../../../sm/src/word_transfers.rs"]
mod word_transfers;


pub mod game {
    pub(crate) mod actors;

    pub(crate) mod cinematic_lease;
    pub use sm_game::constants;
    pub(crate) mod ending_screens;
    pub(crate) mod enemy_speeds;

    pub(crate) mod gameplay;
    pub(crate) mod menu_host;
    pub mod message_composition;
    pub(crate) mod mode7_transfers;
    pub(crate) mod prelude;
    pub mod scene;
    pub(crate) mod visual_checks;
    pub(crate) mod vram_read_queue;
    pub(crate) mod xray_background;

    pub(crate) mod player_projectiles;

    pub(crate) mod projectile_block_reactions_ram;

    pub mod animated_tiles;
    pub mod enemy_projectiles;
    pub(crate) mod gameplay_flag_runtime;
    pub use sm_game::item_pickups;
    pub(crate) mod item_pickups_runtime;
    pub(crate) use sm_game::liquid_surface;
    pub(crate) mod mechanism_displacement;
    pub mod music;
    pub(crate) mod palette_effects;
    pub(crate) use sm_game::player_collision;
    pub(crate) use sm_game::player_movement;
    pub(crate) use sm_game::player_transitions;
    pub(crate) use sm_game::player_update;

    pub(crate) mod boot;
    pub(crate) mod display_commands;
    pub(crate) mod display_refresh;
    pub(crate) mod display_transfers;
    pub(crate) mod hosts;
    pub use sm_game::progress as progress_flags;
    pub(crate) mod room_backgrounds;
    pub mod room_entry;
    pub(crate) mod room_mechanisms;
    pub(crate) mod room_scrolls;
    pub mod room_state;
    pub mod samus_animation;
    pub(crate) use sm_game::samus_palette;
    pub(crate) mod scanline_irq;
    pub(crate) use sm_game::session;
    pub(crate) mod session_checkpoint;
    pub(crate) use sm_game::session_encoding;
    pub(crate) mod session_runtime;
    pub(crate) use sm_game::sprite_effect_host;
    pub(crate) mod sprite_effect_runtime;
    #[cfg(test)]
    mod transition_representation_tests;
    pub(crate) use sm_game::sprite_effect_state;
    pub(crate) use sm_game::visual_host;
    pub(crate) mod visual_runtime;
    pub(crate) use sm_game::visual_state;

    pub use sm_game::combat_rules;
    pub use sm_game::contact_rules;
    pub use sm_game::drop_rules;
    pub use sm_game::enemy_population;
    pub use sm_game::enemy_script;
    pub(crate) mod mother_brain_body;
    pub mod projectile_contact;
    pub mod sm_a9;
    pub(crate) mod space_pirates;
    pub use sm_game::sprite_objects;
}

/// Emulated SNES hardware. Also runs the original ROM as the differential
/// oracle: see `cpu_infra::run_one_frame_of_game_both`.
pub mod snes {
    #[path = "../../../crates/sm_host/src/stubs/apu.rs"]
    pub mod apu;
    #[path = "../../../crates/sm_host/src/stubs/cpu.rs"]
    pub mod cpu;
    pub mod cart;
    pub mod dma;
    pub mod ppu;
    pub mod registers;
    pub mod snes;
    pub mod snes_other;
}
#[path = "../../../sm/src/apu_queue.rs"]
pub mod apu_queue;
#[path = "../../../sm/src/apu_ram.rs"]
pub mod apu_ram;
#[path = "../../../sm/src/audio_snapshot.rs"]
pub mod audio_snapshot;
#[path = "../../../sm/src/audio_upload.rs"]
pub mod audio_upload;
#[path = "../../../sm/src/c_library.rs"]
pub mod c_library;
pub mod cpu_infra;
#[path = "../../../sm/src/initial_state.rs"]
pub mod initial_state;
#[path = "../../../sm/src/ram_map.rs"]
pub mod ram_map;
#[path = "../../../sm/src/ram_words.rs"]
pub mod ram_words;
#[path = "../../../sm/src/rtl.rs"]
pub mod rtl;
#[path = "../../../sm/src/spc_player.rs"]
pub mod spc_player;
#[path = "../../../sm/src/types.rs"]
pub mod types;
#[path = "../../../sm/src/util.rs"]
pub mod util;
#[path = "../../../sm/src/host.rs"]
pub mod host;
#[path = "../../../sm/src/execution.rs"]
pub mod execution;
#[path = "../../../sm/src/game_state.rs"]
pub mod game_state;
#[path = "../../../sm/src/projection.rs"]
pub mod projection;
#[path = "../../../sm/src/native_rom.rs"]
pub mod native_rom;
#[path = "../../../sm/src/oracle_patch_policy.rs"]
mod oracle_patch_policy;
#[path = "../../../sm/src/oracle_mode.rs"]
pub mod oracle_mode;
#[path = "../../../sm/src/recording.rs"]
pub mod recording;
#[path = "../../../sm/src/rom.rs"]
pub mod rom;
#[path = "../../../sm/src/snapshot_io.rs"]
pub mod snapshot_io;
#[path = "../../../sm/src/verify_probe.rs"]
pub mod verify_probe;
#[path = "../../../sm/src/audio_output.rs"]
pub mod audio_output;
#[path = "../../../sm/src/native_audio.rs"]
pub mod native_audio;
#[path = "../../../sm/src/verification/oracle_host.rs"]
pub(crate) mod oracle_host;
#[path = "../../../sm/src/verification/ram_names.rs"]
pub mod ram_names;
pub use sm_runtime::rendering;
pub use game_state::persistent_session::{
    PreparedSessionStart, SessionCheckpoint, SessionStartContract, SessionStartError,
    SuppliedSessionStart,
};
pub use native_audio::SampleDelivery;
macro_rules! type_aliases {
    ($visibility:vis $($name:ident),+ = $target:ty;) => {
        $($visibility type $name = $target;)+
    };
}
pub(crate) use type_aliases;
#[path = "stubs/rom_patches.rs"]
pub mod rom_patches;
#[path = "../../../sm/src/lock_masks.rs"]
mod lock_masks;
#[path = "../../../sm/src/verify/checkpoint.rs"]
pub mod verification_checkpoint;
#[path = "../../../sm/src/verification/observation/machine.rs"]
mod machine;
pub mod replay;
