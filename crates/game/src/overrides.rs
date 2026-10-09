//! Hand-written replacements for routines that don't recompile cleanly
//! (raster-timed HUD/IRQ code, stack-manipulating dispatchers, the NMI wait loop).
//!
//! Rules:
//! - Mark the function `override = true` in config/functions.toml so recomp skips it.
//! - Overrides run under the same frame-by-frame diff as generated code.
//! - Cite the snesrev/sm function you based it on, so behaviour can be compared.

use runtime::TableEntry;

pub static TABLE: &[TableEntry] = &[
    // (0x80_8338, 0b11, wait_for_nmi),
];

// Example shape — the NMI wait loop becomes a yield to the frame scheduler:
//
// fn wait_for_nmi(m: &mut runtime::Machine) {
//     m.bus.tick(FRAME_REMAINDER); // run PPU/APU to vblank, run NMI handler
//     m.pull16(); // RTS
// }
