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

// Share consecutive field types without changing declaration order or layout.
macro_rules! typed_struct {
    ($(#[$metadata:meta])* $visibility:vis struct $name:ident {
        $($field_visibility:vis $($field:ident),+ : $field_type:ty;)*
    }) => {
        $(#[$metadata])*
        $visibility struct $name {
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

// Emit repeated literal field stores without hoisting the receiver or value.
macro_rules! set_fields {
    ($receiver:tt, $value:expr; $($field:ident),+ $(,)?) => {
        $($crate::set_fields!(@store $receiver, $field, $value);)+
    };
    (@store [$($receiver:tt)*], $field:ident, $value:expr) => {
        $($receiver)*.$field = $value
    };
}
pub(crate) use set_fields;

/// Gameplay, one module per original ROM bank (`sm_80` = bank $80, ...).
pub mod game {
    pub(crate) mod callback_tables;
    // Declare address tables with the same calls and unreachable fallback.
    macro_rules! address_dispatch {
        ($($(#[$metadata:meta])* $visibility:vis unsafe fn $name:ident $parameters:tt $(-> $result:ty)?
            => $selector:expr; call $arguments:tt; default $fallback:expr;
            { $($pattern:pat => $target:tt;)* })*) => {
            $(
                $(#[$metadata])*
                $visibility unsafe fn $name $parameters $(-> $result)? {
                    match $selector {
                        $($pattern => $crate::game::address_dispatch!(@call $arguments $target),)*
                        _ => $fallback
                    }
                }
            )*
        };
        (@call ($($argument:expr),* $(,)?) $target:ident) => {
            $target($($argument),*)
        };
        (@call $arguments:tt [$($expression:tt)*]) => {
            $($expression)*
        };
    }
    pub(crate) use address_dispatch;

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
    pub(crate) use enemy_instruction;

    pub mod constants;
    pub(crate) mod enemy_layouts;
    pub(crate) mod enemy_ram;
    pub mod sm_80;
    pub mod sm_81;
    pub mod sm_82;
    pub mod sm_84;
    pub mod sm_85;
    pub mod sm_86;
    pub mod sm_87;
    pub mod sm_88;
    pub mod sm_89;
    pub mod sm_8b;
    pub mod sm_8d;
    pub mod sm_8f;
    pub mod sm_90;
    pub mod sm_91;
    pub mod sm_92;
    pub mod sm_93;
    pub mod sm_94;
    pub mod sm_9b;
    pub mod sm_a0;
    pub mod sm_a2;
    pub mod sm_a3;
    pub mod sm_a4;
    pub mod sm_a5;
    pub mod sm_a6;
    pub mod sm_a7;
    pub mod sm_a8;
    pub mod sm_a9;
    pub mod sm_aa;
    pub mod sm_ad;
    pub mod sm_b2;
    pub mod sm_b3;
    pub mod sm_b4;
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
}

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

/// Execution requirements for focused verifier scenarios.
pub mod verify_probe;

pub(crate) mod native_audio;
