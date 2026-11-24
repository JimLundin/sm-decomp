//! Production owns complete clean model values and publishes their typed outputs.
//! Exact encoding and immutable imported assets are separate from gameplay decisions.
#![deny(clippy::float_arithmetic)]

pub mod commit;
pub mod enemy_interactions_bridge;
pub mod exact;
pub mod intent;
pub mod production;
pub mod samus_appearance_bridge;
pub mod state;
pub mod weapons_projectiles_bridge;
// proof: a comment in an engine source
