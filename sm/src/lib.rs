//! Super Metroid in Rust.
//!
//! Mechanically transpiled from snesrev/sm (rev 578f90b) with c2rust 0.22.1 on
//! 2026-10-17; see `tools/transpile/README.md`. The C source is no longer part
//! of this project. Everything here is now maintained Rust; the plan for making
//! it idiomatic without changing behaviour is in `docs/PLAN.md`.
//!
//! Gameplay and runtime state is owned by `game_state::Game` and passed
//! explicitly through Rust calls; C ABI remains at external boundaries.
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

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
        $crate::ram_map::write_word($value, $destination);
    };
}

// Each instruction row declares Name [result] { effects }; [] keeps a full body.
// A shared prefix runs inside each function, before its effects and result.
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
    (@function [$($header:tt)*] {$($prefix:tt)*} [$tail:expr] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* $tail }
    };
    (@function [$($header:tt)*] {$($prefix:tt)*} [] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* }
    };
}
pub(crate) use instruction_callbacks;
pub(crate) use instruction_callbacks as prefixed_callbacks;

// Declare forwarding wrappers with their original signatures and call order.
// Unit wrappers retain a statement; value-returning wrappers retain a tail call.
#[macro_export]
macro_rules! forward_calls {
    ($visibility:vis unsafe fn $parameters:tt -> $result:ty; $($name:ident => $call:expr;)+) => {
        $($visibility unsafe fn $name $parameters -> $result { $call })+
    };
    ($visibility:vis unsafe fn $parameters:tt; $($name:ident => $call:expr;)+) => {
        $($visibility unsafe fn $name $parameters { $call; })+
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
        __assert_fail(
            $assertion as *const u8 as *const c_char,
            $file as *const u8 as *const c_char,
            $line,
            $function as *const u8 as *const c_char,
        )
    };
    (@failure $assertion:literal, $file:literal, $line:expr, pointer $function:expr $(,)?) => {
        __assert_fail(
            $assertion as *const u8 as *const c_char,
            $file as *const u8 as *const c_char,
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

pub mod game {
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
            $($header)* {
                $($before)*
                if $condition $effects $(else $otherwise)?
                $($($after)*)?
            }
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
                $visibility unsafe fn $name $parameters $(-> $result)? {
                    $($($setup)*)?
                    match $selector {
                        $($pattern => $crate::game::address_dispatch!(@call $arguments $target $(::$tail)*),)*
                        _ => $fallback
                    }
                }
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
                $($($setup)*)?
                match $selector {
                    $($($($pattern => $crate::game::address_dispatch!(@call $arguments $callback),)?)*)*
                    $($($($forward_pattern => $crate::game::address_dispatch!(@call $arguments $forward_callback),)?)*)*
                    $($($($count_pattern => $crate::game::address_dispatch!(@call $arguments $count_callback),)?)*)*
                    $($($external_pattern => $crate::game::address_dispatch!(@call $arguments $target $(::$path_tail)*),)*)?
                    _ => $fallback
                }
            }
        };
    }
    pub(crate) use callback_dispatch;

    // Keep both field assignments in their original statement/evaluation order.
    macro_rules! enemy_instruction {
        ([$($enemy:tt)*]; instruction = $instruction:expr, timer = $timer:expr) => {
            $($enemy)*.current_instruction = $instruction;
            $($enemy)*.instruction_timer = $timer;
        };
        ([$($enemy:tt)*]; timer = $timer:expr, instruction = $instruction:expr) => {
            $($enemy)*.instruction_timer = $timer;
            $($enemy)*.current_instruction = $instruction;
        };
    }

    pub(crate) mod beam_firing;
    pub(crate) mod beam_windows;
    pub mod constants;
    pub(crate) mod decompression;
    pub(crate) mod enemy_layouts;
    pub(crate) mod enemy_ram;
    pub(crate) mod explosion_window;
    pub(crate) mod intro_pages;
    pub mod message_boxes;
    pub(crate) mod dialogue;
    pub(crate) mod dialogue_ram;
    pub(crate) mod moving_platforms;
    pub(crate) mod minimap;
    pub(crate) mod xray_background;
    pub(crate) mod vram_read_queue;
    pub(crate) mod mode7_transfers;
    pub(crate) mod layer_blending;
    pub(crate) mod liquid_effects;
    pub(crate) mod scrolling_sky;
    pub(crate) mod grapple_swing;
    pub(crate) mod gameplay;
    pub(crate) mod prelude;
    pub(crate) mod launch_motion;
    pub(crate) mod launch_motion_ram;
    pub(crate) mod projectile_flight;
    pub(crate) mod room_backgrounds;
    pub(crate) mod room_scrolls;
    pub mod samus_animation;
    pub(crate) mod sprite_drawing;
    pub(crate) mod samus_pose;
    pub mod progress_flags;
    pub mod music;
    pub mod room_state;
    pub mod sm_80;
    pub mod sm_81;
    pub mod sm_82;
    pub mod plm_spawning;
    pub mod sm_84;
    pub mod item_pickups;
    pub mod item_pickups_ram;
    pub mod sm_86;
    pub mod eproj_script;
    pub mod sm_87;
    pub mod sm_88;
    pub mod sm_89;
    pub mod sm_8b;
    pub mod sm_8d;
    pub mod sm_8f;
    pub mod sm_90;
    pub mod sm_91;
    pub mod projectile_animation;
    pub mod sm_94;
    pub mod sm_9b;
    pub mod sm_a0;
    pub mod enemy_population;
    pub mod enemy_population_ram;
    pub mod sm_a2;
    pub mod sm_a3;
    pub mod sm_a4;
    pub mod sm_a5;
    pub mod sm_a6;
    pub mod sm_a7;
    pub mod sm_a8;
    pub mod sm_a9;
    pub mod sm_aa;
    pub(crate) mod mother_brain_visuals;
    pub mod sm_b2;
    pub mod sm_b3;
    pub mod sprite_objects;
    pub mod scene_drawing;
    pub mod enemy_scheduling;
    pub mod enemy_script;
    pub mod enemy_animation;
    pub mod combat_rules;
    pub mod enemy_combat;
    pub mod enemy_contact;
    pub mod contact_rules;
    pub mod projectile_contact;
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
    pub mod input;
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
mod initial_state;
pub mod audio_upload;
pub(crate) mod audio_snapshot;
#[doc(hidden)]
pub mod c_library;
/// Config file parsing.
#[cfg(all(not(sm_verify_dll), feature = "frontend"))]
pub mod config;
/// Frame runner and the per-frame comparison against the original ROM.
pub mod cpu_infra;
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

pub mod game_state;
pub mod rom;
pub mod rom_patches;
pub mod save_ram;
pub mod snapshot_io;
pub mod recording;

/// Execution requirements for focused verifier scenarios.
pub mod verify_probe;

#[doc(hidden)]
pub mod verify_source;

pub(crate) mod native_audio;
mod native_player_ram;

macro_rules! type_aliases {
    ($visibility:vis $($name:ident),+ = $target:ty;) => {
        $($visibility type $name = $target;)+
    };
}
pub(crate) use type_aliases;
