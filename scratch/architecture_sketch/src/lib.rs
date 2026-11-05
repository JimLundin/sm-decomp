//! A compile-checked sketch of the architecture's new seams (ARCHITECTURE.md): the Tick
//! from cut to cut with a beam-sync point, NMI timing as an input, a named cost model,
//! exact and intent output from one drawing source, the `Family` context, and folding
//! typed streams only at quiescent cuts. Each module stands for the crate of the same name.

pub mod data;
pub mod game;
pub mod runtime;
pub mod verify;
