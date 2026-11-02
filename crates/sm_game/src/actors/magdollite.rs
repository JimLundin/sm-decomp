//! Magdollite callbacks run through the shared typed implementation.
//! The retained Host supplies the original imported table observations.
pub trait Host {
    /// Colours 9-12 of the glow palette whose colour 0 is word `source`.
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4];
    /// Arm step `index`'s field at `address` (growth, instruction list or y offset).
    fn magdollite_arm(&self, address: u32, index: isize) -> u16;
    /// The original diagnostic for an undefined lava height.
    fn warn_lava_height(&mut self, value: i32);
}

pub use crate::actors::remaining_owned::magdollite::{
    SPAWN_DATA_BYTES, cycle_palette, damage, finish_arm_growth, grow_arm, initialize, instruction,
    main_ai, reaction, run_state, shrink_arm, state,
};
