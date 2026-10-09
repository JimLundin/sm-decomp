//! 65816 → Rust static recompiler.
//!
//! Pipeline: `rom` (LoROM mapping) → `decode` (one instruction, width-aware)
//! → `cfg` (blocks reachable from a function entry, tracking M/X through REP/SEP)
//! → `emit` (Rust source calling `runtime::ops` / `runtime::ea`).
//!
//! Inputs come from `config/functions.toml` (entries + width variants, seeded
//! from snesrev/sm names and the trace/miss logs) and `config/jump_tables.toml`.

pub mod cfg;
pub mod config;
pub mod decode;
pub mod emit;
pub mod opcodes;
pub mod rom;
