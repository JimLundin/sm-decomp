//! Verification tooling for the Super Metroid engine (docs/SCENARIO_HARNESS.md).
//!
//! Target package `sm_tools`: the scenario harness's observation model, the
//! engine's typed projection, the lockstep comparison and its report. It reads
//! the target packages' typed state and never the emulator crate; the oracle's
//! projection stays with the oracle in `sm`.
#![forbid(unsafe_code)]

pub mod engine;
pub mod free_run;
pub mod lockstep;
pub mod observation;
pub mod trace;
