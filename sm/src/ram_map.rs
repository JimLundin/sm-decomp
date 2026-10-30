//! Names for work RAM ($7E:0000-$7F:FFFF) offsets.
//!
//! Extracted from snesrev/sm `variables.h` and `variables_extra.h` (rev 578f90b).
//! c2rust expanded those macros, so the transpiled code reads e.g.
//! `*(g_ram.offset(0x998) as *mut uint16)` where the C said `game_state`.
//! The verifier uses this table to name RAM mismatches. Use these names when
//! replacing raw gameplay accesses (phase 3 of docs/PLAN.md).

pub use sm_game::ram::{RAM_VARS, RamVar, Shape};
pub(crate) use sm_game::ram::{checked_word, word, word_arrays, words};

pub(crate) use sm_game::ram::slot;
