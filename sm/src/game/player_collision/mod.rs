//! Owned player collision, displacement and pose clearance.
#![forbid(unsafe_code)]
mod control;
pub(in crate::game) use sm_game::player_collision::effects;
mod geometry;
pub(crate) mod grapple;
mod inside;
mod jump;
mod motion;
pub(crate) mod operations;
mod pose;

#[cfg(sm_verify_dll)]
#[path = "../../verification/player_collision_entry_adapters.rs"]
pub(crate) mod verification;
