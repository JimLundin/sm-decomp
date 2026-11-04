//! Names for work RAM ($7E:0000-$7F:FFFF) offsets.
//!
//! Extracted from snesrev/sm `variables.h` and `variables_extra.h` (rev 578f90b).
//! c2rust expanded those macros, so the transpiled code reads e.g.
//! `*(g_ram.offset(0x998) as *mut uint16)` where the C said `game_state`.
//! The verifier names RAM mismatches from the same table
//! (`crate::ram_names::RAM_VARS`).

pub(crate) use crate::ram_words::{checked_word, slot, word};
pub(crate) use sm_game::ram::{word_arrays, words};
