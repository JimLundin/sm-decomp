//! HDMA objects as Session-owned state ($18B0–$195B): step 2 of the liquid
//! and HDMA domain described in `crate::liquid`.
//!
//! The block is the enable flag, the selected object and fourteen six-word
//! arrays, one per field of the six objects. It is owned whole and
//! contiguously: an index past the sixth object reads or writes the next
//! array, as in the original, so every aliased word is covered by the same
//! owner.
//!
//! Lane:
//! 1. This type, its `session_encoding` entries and the persistent
//!    `SessionCheckpoint` fields. No writer changes; compat writers stay on
//!    the RAM bridge.
//! 2. The interpreter (`visual_runtime`) under one lease per frame.
//! 3. The callback families (liquid, sky, x-ray, explosion windows, Mother
//!    Brain, Crocomire, Ceres, haze, menus) one at a time.
//!
//! Every word keeps its full stored `u16`.

/// Objects the original runs at once.
pub const OBJECTS: usize = 6;

/// One HDMA object's words, at `$18B4 + 12 * field + 2 * object`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HdmaObject {
    /// $18B4: the HDMA channel mask; zero for a free object.
    pub channels: u16,
    /// $18C0: the channel's register slot.
    pub bank_slot: u16,
    /// $18CC: the instruction list cursor.
    pub instruction_list: u16,
    /// $18D8: the HDMA table pointer.
    pub table: u16,
    /// $18E4.
    pub instruction_timer: u16,
    /// $18F0: the pre-instruction run each frame.
    pub pre_instruction: u16,
    /// $18FC: the pre-instruction's bank.
    pub pre_instruction_bank: u16,
    /// $1908.
    pub timer: u16,
    /// $1914–$1950: words whose meaning depends on the object.
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub e: u16,
    pub f: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HdmaObjects {
    /// $18B0: objects run only while this is set.
    pub enabled: u16,
    /// $18B2: the object being run, as a byte offset.
    pub selected: u16,
    pub objects: [HdmaObject; OBJECTS],
}
