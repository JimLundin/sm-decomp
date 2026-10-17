//! Super Metroid in Rust.
//!
//! Mechanically transpiled from snesrev/sm (rev 578f90b) with c2rust 0.22.1 on
//! 2026-10-17; see `tools/transpile/README.md`. The C source is no longer part
//! of this project. Everything here is now maintained Rust; the plan for making
//! it idiomatic without changing behaviour is in `docs/PLAN.md`.
//!
//! Modules still link to each other by symbol name (`extern "C"` blocks plus
//! `#[no_mangle]`), exactly as the C translation units did. Replacing those
//! with ordinary `use` paths is part of phase 2 of the plan.
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(extern_types)]

/// Gameplay, one module per original ROM bank (`sm_80` = bank $80, ...).
pub mod game {
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
    pub mod snes;
    pub mod snes_other;
    pub mod spc;
}

/// Window, renderer and OpenGL output.
#[cfg(not(sm_verify_dll))]
pub mod platform {
    pub mod gl_core;
    pub mod glsl_shader;
    pub mod opengl;
}

/// Config file parsing.
#[cfg(not(sm_verify_dll))]
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
