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
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

#[cfg(any(feature = "frontend", sm_verify_dll))]
#[path = "verify/checkpoint.rs"]
pub mod verification_checkpoint;
#[cfg(sm_verify_dll)]
pub use actor_composed_route::checkpoint as verify_prepared_actor_checkpoint;

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

// Each instruction row declares Name [result] { effects }; [] keeps a full body.
// A shared prefix runs inside each function, before its effects and result.
// Only unsafe declarations authorize unsafe operations in their emitted bodies.
macro_rules! instruction_callbacks {
    ($visibility:vis unsafe fn $parameters:tt; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::instruction_callbacks!(@function [$visibility unsafe fn $name $parameters] $prefix $tail $body);)*
    };
    ($visibility:vis fn $parameters:tt; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::instruction_callbacks!(@function [$visibility fn $name $parameters] $prefix $tail $body);)*
    };
    ($visibility:vis unsafe fn $parameters:tt -> $result:ty; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::instruction_callbacks!(@function [$visibility unsafe fn $name $parameters -> $result] $prefix $tail $body);)*
    };
    ($visibility:vis fn $parameters:tt -> $result:ty; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::instruction_callbacks!(@function [$visibility fn $name $parameters -> $result] $prefix $tail $body);)*
    };
    ($visibility:vis unsafe fn $parameters:tt -> $result:ty; $($entries:tt)*) => {
        $crate::instruction_callbacks!($visibility unsafe fn $parameters -> $result; @prefix {}; $($entries)*);
    };
    ($visibility:vis fn $parameters:tt -> $result:ty; $($entries:tt)*) => {
        $crate::instruction_callbacks!($visibility fn $parameters -> $result; @prefix {}; $($entries)*);
    };
    (@function [$visibility:vis unsafe fn $($header:tt)*] {$($prefix:tt)*} [$tail:expr] {$($body:tt)*}) => {
        $visibility unsafe fn $($header)* { #[allow(unused_unsafe)] unsafe { $($prefix)* $($body)* $tail } }
    };
    (@function [$visibility:vis unsafe fn $($header:tt)*] {$($prefix:tt)*} [] {$($body:tt)*}) => {
        $visibility unsafe fn $($header)* { #[allow(unused_unsafe)] unsafe { $($prefix)* $($body)* } }
    };
    (@function [$($header:tt)*] {$($prefix:tt)*} [$tail:expr] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* $tail }
    };
    (@function [$($header:tt)*] {$($prefix:tt)*} [] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* }
    };
}
pub(crate) use instruction_callbacks;

// Declare forwarding wrappers with their original signatures and call order.
// Unit wrappers retain a statement; value-returning wrappers retain a tail call.
#[macro_export]
macro_rules! forward_calls {
    ($visibility:vis unsafe fn $parameters:tt -> $result:ty; $($name:ident => $call:expr;)+) => {
        $($visibility unsafe fn $name $parameters -> $result { unsafe { $call }})+
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
#[macro_use]
mod word_transfers;

#[cfg(sm_verify_dll)]
#[path = "verification/actor_population_animation_cases.rs"]
pub(crate) mod actor_population_animation_cases;
#[cfg(sm_verify_dll)]
#[path = "verification/actor_population_animation_observer.rs"]
pub(crate) mod actor_population_animation_observer;
#[cfg(sm_verify_dll)]
pub fn verify_actor_population_animation_case(id: u32) -> Option<Vec<u8>> {
    actor_population_animation_cases::capture_case(id, actor_population_animation_cases::run)
}

pub mod game {
    #[cfg(sm_verify_dll)]
    #[path = "../verification/actor_composed_route.rs"]
    pub(crate) mod actor_composed_route;
    pub(crate) mod actors;
    // Share the exact countdown prefix and expiry test across named callbacks.
    macro_rules! countdown_callbacks {
        (
            $visibility:vis unsafe fn $parameters:tt;
            before $before:tt
            when $condition:expr;
            $(
                $name:ident $effects:tt
                $(else $otherwise:tt)?
                $(after $after:tt)?;
            )+
        ) => {
            $(
                $crate::game::countdown_callbacks! {
                    @function [$visibility unsafe fn $name $parameters]
                    $before [$condition] $effects
                    $(else $otherwise)? $(after $after)?
                }
            )+
        };
        (
            @function [$($header:tt)*]
            { $($before:tt)* } [$condition:expr] $effects:block
            $(else $otherwise:block)? $(after { $($after:tt)* })?
        ) => {
            $($header)* { #[allow(unused_unsafe)] unsafe {
                $($before)*
                if $condition $effects $(else $otherwise)?
                $($($after)*)?
            }}
        };
    }
    pub(crate) use countdown_callbacks;

    pub(crate) mod callback_tables;
    // Declare address tables with the same calls and unreachable fallback.
    macro_rules! address_dispatch {
        ($($(#[$metadata:meta])* $visibility:vis unsafe fn $name:ident $parameters:tt $(-> $result:ty)?
            => $selector:expr; $(setup { $($setup:tt)* })? call $arguments:tt; default $fallback:expr;
            { $($pattern:pat => $target:tt $(::$tail:ident)*;)* })*) => {
            $(
                $(#[$metadata])*
                $visibility unsafe fn $name $parameters $(-> $result)? { unsafe {
                    $($($setup)*)?
                    match $selector {
                        $($pattern => $crate::game::address_dispatch!(@call $arguments $target $(::$tail)*),)*
                        _ => $fallback
                    }
                }}
            )*
        };
        (@call ($($argument:expr),* $(,)?) $target:path) => {
            $target($($argument),*)
        };
        (@call $arguments:tt [$($expression:tt)*]) => {
            $($expression)*
        };
    }
    pub(crate) use address_dispatch;

    // A callback's optional address feeds its dispatcher while preserving named functions.
    macro_rules! callback_dispatch {
        (
            $(#[$metadata:meta])* $visibility:vis unsafe fn $name:ident $parameters:tt $(-> $result:ty)?
            => $selector:expr; $(setup { $($setup:tt)* })? call $arguments:tt; default $fallback:expr;
            $(@body $body_mode:ident;)?
            $(
                @functions [$($header:tt)*]; $(@prefix {$($prefix:tt)*};)?
                $($([$pattern:pat])? $callback:ident $tail:tt $body:tt;)*
            )*
            $(
                @forward [$($forward_header:tt)*];
                $($([$forward_pattern:pat])? $forward_callback:ident => $forward_call:expr;)*
            )*
            $(
                @countdown [$($count_header:tt)*]; before $before:tt when $condition:expr;
                $($([$count_pattern:pat])? $count_callback:ident $effects:tt
                    $(else $otherwise:tt)? $(after $after:tt)?;)*
            )*
            $(@external {$($external_pattern:pat => $target:tt $(::$path_tail:ident)*;)*})?
        ) => {
            $(
                $crate::instruction_callbacks! {
                    $($header)*; @prefix {$($($prefix)*)?};
                    $($callback $tail $body;)*
                }
            )*
            $(
                $crate::forward_calls! {
                    $($forward_header)*; $($forward_callback => $forward_call;)*
                }
            )*
            $(
                $crate::game::countdown_callbacks! {
                    $($count_header)*; before $before when $condition;
                    $($count_callback $effects $(else $otherwise)? $(after $after)?;)*
                }
            )*
            $(#[$metadata])*
            $visibility unsafe fn $name $parameters $(-> $result)? {
                $crate::game::callback_dispatch!(@body [$($body_mode)?] {
                $($($setup)*)?
                match $selector {
                    $($($($pattern => $crate::game::address_dispatch!(@call $arguments $callback),)?)*)*
                    $($($($forward_pattern => $crate::game::address_dispatch!(@call $arguments $forward_callback),)?)*)*
                    $($($($count_pattern => $crate::game::address_dispatch!(@call $arguments $count_callback),)?)*)*
                    $($($external_pattern => $crate::game::address_dispatch!(@call $arguments $target $(::$path_tail)*),)*)?
                    _ => $fallback
                }
                })
            }
        };
        (@body [] {$($body:tt)*}) => { unsafe { $($body)* } };
        (@body [checked] {$($body:tt)*}) => { { $($body)* } };
    }
    pub(crate) use callback_dispatch;

    pub(crate) use sm_game::special_geometry;

    pub(crate) use sm_game::beam_definitions;

    pub(crate) mod beam_windows;
    pub mod constants;
    pub(crate) mod decompression;
    pub(crate) mod demo_rooms;
    pub(crate) use sm_game::dialogue;
    pub(crate) mod dialogue_ram;
    pub(crate) mod enemy_layouts;
    pub(crate) mod enemy_speeds;

    pub(crate) mod explosion_window;
    pub(crate) mod file_menu;
    pub(crate) mod game_over;
    pub(crate) mod gameplay;
    pub(crate) mod grapple_aim;
    pub(crate) mod grapple_origins;
    pub(crate) mod grapple_rope;
    pub(crate) mod grapple_swing;
    pub(crate) mod imported;
    pub(crate) mod intro_pages;
    pub(crate) mod kraid_tables;
    pub(crate) mod launch_motion;
    pub(crate) mod launch_motion_ram;
    pub(crate) mod layer_blending;
    pub(crate) mod liquid_effects;
    pub(crate) mod map_menu;
    pub mod menu_tilemap;
    pub mod menu_window;
    pub mod message_boxes;
    pub mod message_composition;
    pub(crate) mod mode7_transfers;
    pub(crate) mod mother_brain_tables;
    pub(crate) mod moving_platforms;
    pub(crate) mod prelude;
    pub(crate) use sm_game::projectile_weapon_definitions;
    pub mod scene;
    pub(crate) mod scrolling_sky;
    pub(crate) mod vram_read_queue;
    pub(crate) mod xray_background;
    pub(crate) mod zebetite_data;

    pub(crate) mod player_projectiles;
    #[path = "../verification/room_mechanisms.rs"]
    pub(crate) mod room_mechanism_checks;

    pub(crate) use sm_game::projectile_geometry;
    pub(crate) use sm_game::projectile_initialization;

    pub(crate) use sm_game::projectile_block_reactions;
    pub(crate) mod projectile_block_reactions_ram;
    pub(crate) mod projectile_presentation_ram;
    pub(crate) use sm_game::projectile_room;

    pub mod animated_tiles;
    pub mod enemy_projectiles;
    pub(crate) mod equipment_menu;
    pub(crate) mod escape_typewriter;
    pub(crate) use sm_game::external_inputs;
    pub(crate) mod gameplay_flag_host;
    pub(crate) mod gameplay_flag_runtime;
    pub(crate) use sm_game::gameplay_flag_state;
    pub use sm_game::item_pickups;
    pub(crate) mod item_pickups_runtime;
    pub(crate) use sm_game::liquid_surface;
    pub(crate) mod mechanism_displacement;
    pub mod music;
    pub(crate) mod palette_copies;
    pub(crate) mod palette_effects;
    pub(crate) mod phantoon_tables;
    pub(crate) mod player_collision;
    #[cfg(any(test, sm_verify_dll))]
    #[path = "../verification/player_collision_owned_checks.rs"]
    pub(crate) mod player_collision_owned_checks;
    pub(crate) mod player_movement;
    pub(crate) mod player_transitions;
    pub(crate) mod player_update;
    pub(crate) mod ridley_data;
    pub(crate) mod ridley_explosion_data;
    pub(crate) mod rom_vram_entries;

    pub(crate) mod boot;
    pub(crate) mod checkpoint_binding;
    pub(crate) mod controller_input;
    pub(crate) mod display_refresh;
    pub(crate) mod escape_timer;
    pub(crate) mod goofball;
    pub(crate) mod gunship;
    pub(crate) mod hud;
    pub(crate) mod maridia_turtles;
    pub(crate) mod mini_crocomire;
    pub(crate) mod options_objects;
    pub(crate) mod options_settings;
    pub(crate) mod pause_controls;
    pub mod progress_flags;
    pub(crate) mod reserve_tanks;
    pub(crate) mod rinka;
    pub(crate) mod room_backgrounds;
    pub mod room_entry;
    pub(crate) mod room_mechanisms;
    pub(crate) mod room_scrolls;
    pub mod room_state;
    pub mod samus_animation;
    pub(crate) mod samus_palette;
    pub(crate) mod samus_periodic_damage;
    pub(crate) mod sand_heap;
    pub mod save_slots;
    pub(crate) mod scanline_irq;
    pub(crate) mod session;
    pub(crate) mod session_access;
    pub(crate) mod session_checkpoint;
    pub(crate) mod session_encoding;
    pub(crate) mod session_owners;
    pub(crate) mod session_runtime;
    pub(crate) mod shooting_stars;
    pub mod sm_8b;
    pub(crate) mod sprite_drawing;
    pub(crate) mod sprite_effect_controllers;
    pub(crate) mod sprite_effect_host;
    pub(crate) mod sprite_effect_runtime;
    pub(crate) use sm_game::sprite_effect_state;
    pub(crate) mod visual_host;
    pub(crate) mod visual_runtime;
    pub(crate) use sm_game::visual_state;

    pub(crate) mod ceres_door_data;
    pub(crate) mod ceres_doors;
    pub(crate) mod ceres_elevator;
    pub(crate) mod ceres_ridley_tables;
    pub use sm_game::combat_rules;
    pub use sm_game::contact_rules;
    pub use sm_game::drop_rules;
    pub use sm_game::enemy_population;
    pub mod enemy_script;
    pub(crate) mod erratic_fireball;
    pub(crate) mod fire_geysers;
    pub(crate) mod flies;
    pub(crate) mod ground_patrol;
    pub(crate) mod hopping_blobs;
    pub(crate) mod large_snail;
    pub(crate) mod lava_jumper;
    pub(crate) mod lava_seahorse_pair;
    pub(crate) mod lavaquake;
    pub(crate) mod mother_brain_body;
    pub(crate) mod mother_brain_samus_vitals;
    pub(crate) mod mother_brain_visuals;
    pub(crate) mod norfair_rio;
    pub mod projectile_contact;
    pub(crate) mod rio;
    pub(crate) mod rippers;
    pub(crate) mod rising_falling;
    pub mod sm_a3;
    pub mod sm_a5;
    pub mod sm_a6;
    pub mod sm_a7;
    pub mod sm_a8;
    pub mod sm_a9;
    pub mod sm_aa;
    pub(crate) mod space_pirates;
    pub(crate) mod spikey_platforms;
    pub mod sprite_objects;
    pub mod waver;
}

/// Emulated SNES hardware. Also runs the original ROM as the differential
/// oracle: see `cpu_infra::RunOneFrameOfGame_Both`.
pub mod snes {
    pub mod apu;
    pub mod cart;
    pub mod cpu;
    pub mod dma;
    pub mod dsp;
    pub mod ppu;
    pub mod registers;
    pub mod snes;
    pub mod snes_other;
    pub mod spc;
}

/// Window, renderer and OpenGL output.
#[cfg(all(not(sm_verify_dll), feature = "frontend"))]
pub mod platform {
    pub mod glsl_shader;
    pub mod opengl;
    mod sdl_constants;
    mod texture_png;
}

pub mod apu_queue;
pub mod apu_ram;
pub(crate) mod audio_snapshot;
pub mod audio_upload;
#[doc(hidden)]
pub mod c_library;
/// Config file parsing.
#[cfg(all(not(sm_verify_dll), feature = "frontend"))]
pub mod config;
/// Frame runner and the per-frame comparison against the original ROM.
pub mod cpu_infra;
mod initial_state;
/// Names for raw `g_ram` offsets, recovered from snesrev's variable macros.
pub mod ram_map;
/// Game runtime library: RAM, ROM access, save states, frame loop glue.
pub mod rtl;
/// Music/sound driver reimplementation.
pub mod spc_player;
/// Debug tracing.
pub mod tracing;
/// Shared byte-compatible gameplay types.
pub mod types;
/// Small helpers (file I/O, strings).
pub mod util;

/// Runtime globals and callbacks supplied by the frontend.
pub mod host;

mod execution;
pub mod game_state;
pub use game_state::persistent_session::{
    PreparedSessionStart, SessionCheckpoint, SessionStartContract, SessionStartError,
    SuppliedSessionStart,
};
pub(crate) mod native_rom;
pub mod recording;
pub use sm_runtime::rendering;
pub mod rom;
pub mod rom_patches;
pub mod room_pack;
pub mod save_ram;
pub mod snapshot_io;

/// Execution requirements for focused verifier scenarios.
pub mod verify_probe;

#[cfg(sm_verify_dll)]
#[doc(hidden)]
pub mod verify_source;

#[doc(hidden)]
pub fn verify_player_projectiles() -> u32 {
    game::player_projectiles::checks::run()
}

#[doc(hidden)]
pub fn verify_player_projectile_case(id: u32) -> Option<Vec<u8>> {
    game::player_projectiles::checks::capture_case(id)
}

#[path = "verification/animated_tiles.rs"]
mod animated_tile_checks;

#[doc(hidden)]
pub fn verify_animated_runtime_guard() -> bool {
    game::animated_tiles::checks::guard(&mut game_state::Game::new())
}

#[doc(hidden)]
pub fn verify_animated_tile_case(id: u32) -> Option<Vec<u8>> {
    animated_tile_checks::capture_case(id)
}

#[doc(hidden)]
pub fn verify_enemy_projectile_case(id: u32) -> Option<Vec<u8>> {
    if id < game::enemy_projectiles::checks::CASES {
        game::enemy_projectiles::checks::capture_case(id)
    } else {
        game::enemy_projectiles::checks::supplemental_cases::capture_case(
            id - game::enemy_projectiles::checks::CASES,
        )
    }
}

#[path = "verification/palette_effects.rs"]
mod palette_effect_checks;

#[doc(hidden)]
pub fn verify_palette_runtime_guard() -> bool {
    game::palette_effects::checks::guard(&mut game_state::Game::new())
}

#[doc(hidden)]
pub fn verify_palette_effect_case(id: u32) -> Option<Vec<u8>> {
    palette_effect_checks::capture_case(id)
}

#[doc(hidden)]
pub fn verify_room_mechanism_case(id: u32) -> Option<Vec<u8>> {
    game::room_mechanism_checks::capture_case(id)
}

#[doc(hidden)]
pub fn verify_room_level_copy(case: u32) {
    game::room_state::check_level_copy(case)
}

pub mod audio_output;
pub(crate) mod native_audio;
pub use native_audio::SampleDelivery;
mod native_player_ram;

macro_rules! type_aliases {
    ($visibility:vis $($name:ident),+ = $target:ty;) => {
        $($visibility type $name = $target;)+
    };
}
pub(crate) use type_aliases;

#[doc(hidden)]
pub fn verify_enemy_projectile_qualified_case(id: u32) -> bool {
    std::panic::catch_unwind(|| match id {
        0..=23 => game::enemy_projectiles::checks::qualified_checks::check_case(id),
        24..=27 => game::enemy_projectiles::checks::qualified_motion_checks::check_case(id - 24),
        28 => {
            game::enemy_projectiles::reset_metadata_checks::check();
            true
        }
        29..=56 => game::enemy_projectiles::native_content_checks::check_case(id - 29),
        57..=70 => game::enemy_projectiles::checks::qualified_lavaquake_checks::check_case(id - 57),
        71..=86 => game::enemy_projectiles::checks::qualified_turret_checks::check_case(id - 71),
        _ => false,
    })
    .unwrap_or(false)
}

#[doc(hidden)]
pub fn verify_player_projectile_expanded_case(id: u32) -> Option<Vec<u8>> {
    game::player_projectiles::checks::expanded::capture(id, game::grapple_swing::react_beam)
}

#[doc(hidden)]
pub fn verify_player_projectile_presentation_reset_case(id: u32) -> Option<Vec<u8>> {
    game::player_projectiles::checks::presentation_reset::capture(id)
}

#[doc(hidden)]
pub fn verify_samus_palette_candidate(
    id: u32,
    game: &mut game_state::Game,
) -> Result<Vec<u8>, String> {
    game::visual_runtime::samus_observer::capture(id, game)
}

#[path = "verification/samus_palette_defined_candidate.rs"]
mod samus_defined_adapter;
#[path = "verification/samus_palette_defined_cases.rs"]
mod samus_defined_cases;
#[doc(hidden)]
pub fn verify_samus_palette_defined_case(id: u32) -> Option<Vec<u8>> {
    samus_defined_adapter::capture_case(id)
}
#[path = "verification/visual_runtime_checks.rs"]
mod visual_runtime_checks;
#[doc(hidden)]
pub fn verify_visual_runtime_guard() -> bool {
    visual_runtime_checks::run()
}

#[path = "verification/sprite_alias_cases.rs"]
pub(crate) mod sprite_alias_cases;
#[path = "verification/sprite_controller_cases.rs"]
pub(crate) mod sprite_controller_cases;
#[path = "verification/sprite_effect_candidate.rs"]
pub(crate) mod sprite_effect_candidate;
#[path = "verification/sprite_effect_cases.rs"]
pub(crate) mod sprite_effect_cases;
#[path = "verification/sprite_effect_observer.rs"]
pub(crate) mod sprite_effect_observer;

#[path = "verification/draygon_sprite_candidate.rs"]
pub mod draygon_sprite_candidate;
pub fn verify_sprite_effect_case(id: u32) -> Option<Vec<u8>> {
    sprite_effect_candidate::capture_case(id)
}

#[path = "verification/sprite_pool_candidate.rs"]
pub(crate) mod sprite_pool_candidate;
#[path = "verification/sprite_pool_case.rs"]
pub(crate) mod sprite_pool_case;
#[path = "verification/sprite_runtime_checks.rs"]
pub(crate) mod sprite_runtime_checks;

pub fn verify_sprite_pool_candidate(
    id: u32,
    game: &mut crate::game_state::Game,
) -> Result<Vec<u8>, String> {
    sprite_pool_candidate::capture(id, game)
}
pub fn verify_sprite_runtime_guard() -> bool {
    sprite_runtime_checks::run()
}

#[path = "verification/flags_cases.rs"]
pub(crate) mod flags_cases;

#[path = "verification/flags_candidate.rs"]
pub(crate) mod flags_candidate;

#[path = "verification/flags_save_case.rs"]
pub(crate) mod flags_save_case;

#[path = "verification/flags_save_observer.rs"]
pub(crate) mod flags_save_observer;

#[path = "verification/flags_runtime_checks.rs"]
pub(crate) mod flags_runtime_checks;

pub fn verify_gameplay_flags_case(id: u32) -> Option<Vec<u8>> {
    flags_candidate::capture_case(id)
}
pub fn verify_flags_runtime_guard() -> bool {
    flags_runtime_checks::run()
}

#[path = "verification/flags_kraid_cases.rs"]
mod flags_kraid_cases;
pub fn verify_gameplay_flags_kraid_case(id: u32) -> Option<Vec<u8>> {
    flags_kraid_cases::capture_case(id)
}

#[path = "verification/kraid_flags_case.rs"]
pub mod kraid_flags_case;
#[path = "verification/kraid_flags_observer.rs"]
pub mod kraid_flags_observer;

#[path = "verification/room_reference_cases.rs"]
pub(crate) mod room_reference_cases;
#[path = "verification/room_reference_observer.rs"]
pub(crate) mod room_reference_observer;
pub fn verify_room_reference_case(id: u32) -> Option<Vec<u8>> {
    room_reference_cases::capture_case(id, room_reference_cases::run)
}

#[cfg(sm_verify_dll)]
#[path = "verification/actor_movement_cases.rs"]
pub(crate) mod actor_movement_cases;
#[cfg(sm_verify_dll)]
#[path = "verification/actor_movement_observer.rs"]
pub(crate) mod actor_movement_observer;
#[cfg(sm_verify_dll)]
pub fn verify_actor_movement_case(id: u32) -> Option<Vec<u8>> {
    actor_movement_cases::capture_case(id, actor_movement_cases::run)
}

#[path = "verification/room_contact_start_cases.rs"]
pub(crate) mod room_contact_start_cases;
#[path = "verification/room_contact_start_observer.rs"]
pub(crate) mod room_contact_start_observer;
pub fn verify_room_contact_start_case(id: u32) -> Option<Vec<u8>> {
    room_contact_start_cases::capture_case(id, room_contact_start_cases::run)
}

#[path = "verification/room_runtime_checks.rs"]
pub(crate) mod room_runtime_checks;
#[path = "verification/room_world_cases.rs"]
pub(crate) mod room_world_cases;
#[path = "verification/room_world_fixture.rs"]
pub(crate) mod room_world_fixture;
#[path = "verification/room_world_resources.rs"]
pub(crate) mod room_world_resources;
pub fn verify_room_runtime_guard() -> bool {
    room_runtime_checks::run()
}
pub fn verify_room_world_owner(id: u32) -> Box<game_state::GameOwner> {
    room_world_cases::owner(id)
}
pub fn verify_room_world_case(id: u32, owner: &game_state::GameOwner) -> Option<Vec<u8>> {
    room_world_cases::capture(id, owner)
}
#[cfg(sm_verify_dll)]
pub use room_world_cases::verify_owned_atmosphere_prefix;
#[cfg(sm_verify_dll)]
pub use room_world_cases::verify_owned_atmosphere_settlement;
#[cfg(sm_verify_dll)]
pub use room_world_cases::{
    verify_owned_marker_prefix, verify_owned_namespace_birth_prefix,
    verify_owned_namespace_interleaved, verify_owned_resident_append, verify_owned_samus_id_prefix,
    verify_owned_samus_upload, verify_resident_append_prefix, verify_samus_ceres_position,
    verify_samus_position_offset,
};
#[cfg(sm_verify_dll)]
pub use room_world_resources::verify_room_pack_bytes;

#[path = "verification/room_background_cases.rs"]
pub(crate) mod room_background_cases;
#[path = "verification/room_background_observer.rs"]
pub(crate) mod room_background_observer;
#[path = "verification/room_background_offset_case.rs"]
pub(crate) mod room_background_offset_case;
#[path = "verification/room_background_support.rs"]
pub(crate) mod room_background_support;

pub fn verify_room_background_case(id: u32) -> Option<Vec<u8>> {
    if id == 37 {
        room_background_offset_case::capture(room_background_cases::run)
    } else {
        room_background_cases::capture_case(id, room_background_cases::run)
    }
}

#[cfg(sm_verify_dll)]
pub fn verify_actor_movement_table_case(id: u32) -> Option<Vec<u8>> {
    actor_movement_cases::capture_table_case(id, actor_movement_cases::run)
}

#[cfg(sm_verify_dll)]
#[path = "verification/actor_kraid_writer_entry.rs"]
pub(crate) mod actor_kraid_writer_entry;

#[cfg(sm_verify_dll)]
pub(crate) use game::actor_composed_route;
#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_composed_route.rs"]
pub(crate) mod player_collision_composed_route;

#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_movement_cases.rs"]
pub(crate) mod player_collision_movement_cases;

#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_pose_cases.rs"]
pub(crate) mod player_collision_pose_cases;

#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_composed_steps.rs"]
pub(crate) mod player_collision_composed_steps;

#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_walked_cases.rs"]
pub(crate) mod player_collision_walked_cases;

#[cfg(sm_verify_dll)]
#[path = "verification/player_collision_owned_cases.rs"]
pub(crate) mod player_collision_owned_cases;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_rust_observer.rs"]
pub(crate) mod player_transitions_rust_observer;

#[cfg(sm_verify_dll)]
#[path = "verification/player_transitions_animation.rs"]
pub mod player_transitions_animation;

#[cfg(sm_verify_dll)]
#[path = "verification/samus_content_binding_cases.rs"]
mod samus_content_binding_cases;
#[cfg(sm_verify_dll)]
pub use samus_content_binding_cases::{
    samus_content_prepared_lease_retains_sealed_allocation,
    samus_content_rejects_allocation_and_role_changes,
    samus_content_rejects_authored_request_changes,
    samus_content_rejects_invalid_payload_and_transfer_extents,
};

#[cfg(sm_verify_dll)]
#[path = "verification/samus_game_binding_cases.rs"]
mod samus_game_binding_cases;
#[cfg(sm_verify_dll)]
pub use samus_game_binding_cases::{
    samus_driver_body_projection_preserves_odd_lanes, samus_driver_cannon_failure_preserves_prefix,
    samus_game_adoption_uses_captured_target, samus_game_body_foreign_adoption_and_prefix,
    samus_game_body_projection_crossing_writes, samus_game_body_projection_roundtrip,
    samus_game_failed_install_preserves_binding, samus_game_native_capture_retains_allocation,
    samus_game_panicking_callback_keeps_prefix, samus_game_prepared_replacement_is_rejected,
    samus_game_reference_import_keeps_revoked_scope,
    samus_game_rejected_expectation_preserves_binding, samus_game_returning_die_load_witness,
};

#[cfg(sm_verify_dll)]
#[path = "verification/persistent_session_cases.rs"]
mod persistent_session_cases;
#[cfg(sm_verify_dll)]
pub use persistent_session_cases::{
    bank_upload_physical_prefixes, bank_upload_physical_timeout_boundaries,
    persistent_session_native_rebinding_preserves_ready_owner,
};
