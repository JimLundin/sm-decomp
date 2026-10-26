//! Owned player collision, displacement and pose clearance.
#![forbid(unsafe_code)]
mod control;
pub(in crate::game) mod effects;
mod geometry;
pub(crate) mod grapple;
mod inside;
mod jump;
mod motion;
pub(crate) mod operations;
mod pose;
pub(in crate::game) mod state;

#[cfg(sm_verify_dll)]
#[path = "../../verification/player_collision_entry_adapters.rs"]
pub(crate) mod verification;
