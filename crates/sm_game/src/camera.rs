//! The room camera: layer 1 and 2 positions, their block coordinates and the
//! background offsets, owned by `Session::room`.
//!
//! # Design
//!
//! **Cohort.** The camera is the words at $08F7..$091B (layer block
//! coordinates, their previous values, the BG block coordinates, layer 1's
//! fractional and whole position and layer 2's position), $091D..$0925 (the
//! BG1 offsets and BG2 scrolls) and the ideal layer 1 position at $0B0A and
//! $0B0E. The layer 2 scroll policy bytes at $091B/$091C already belong to
//! the room entry and stay there. Every word is the little-endian value of
//! its two RAM bytes; arrays are `[x, y]`, so the original's axis offset of
//! two bytes is an array index here.
//!
//! **Encoding boundary.** `session_encoding` projects every camera byte, so
//! byte, odd-offset and native-order accesses keep their exact RAM
//! semantics while the Session holds the camera. The room checkpoint carries
//! the cohort.
//!
//! **Writers.** Room scrolling (camera following, the scroll triggers, the
//! door camera steps and the viewport fill) runs on the owned-Session view
//! and reads and writes these fields directly. Layer 1's whole position is
//! also the external camera observation (`ExternalInputs::camera_x/_y`), so
//! its stores go through [`set_layer1`] (and BG1's x offset, another
//! observation, through [`set_bg1_offset`]), which publish through the view's
//! byte path and refreshes that observation in the original order.
//! The remaining writers are compatibility code that still names the RAM
//! words: bank code (cinematics, boss shakes, the demo loader, Ceres) and
//! native CPU or DMA writes. They write the byte image outside a lease, or
//! inside a lease's foreign call, which exports first and imports after, so
//! no typed copy competes with them. Each moves onto these fields when its domain does; the projection
//! can shrink once no reader or writer names the RAM words.
#![forbid(unsafe_code)]
use crate::runtime_access::SessionRam;

/// The camera's words; see the module documentation for the RAM layout.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Camera {
    /// Layer 1's and 2's block coordinates ($08F7, $08FB).
    pub layer1_block: [u16; 2],
    pub layer2_block: [u16; 2],
    /// Their values when the scrolling tiles were last queued ($08FF, $0903).
    pub previous_layer1_block: [u16; 2],
    pub previous_layer2_block: [u16; 2],
    /// The BG tilemap block coordinates ($0907, $090B).
    pub bg1_block: [u16; 2],
    pub bg2_block: [u16; 2],
    /// Layer 1's position fractions and whole positions ($090F..$0917),
    /// interleaved in RAM as x fraction, x, y fraction, y.
    pub layer1_fraction: [u16; 2],
    pub layer1: [u16; 2],
    /// Layer 2's position ($0917).
    pub layer2: [u16; 2],
    /// BG1's offsets and BG2's scrolls from the layer positions ($091D, $0921).
    pub bg1_offset: [u16; 2],
    pub bg2_scroll: [u16; 2],
    /// The layer 1 position the camera follows toward ($0B0A, $0B0E).
    pub ideal_layer1: [u16; 2],
}

/// Store layer 1's whole position on `axis` (0 = x, 1 = y). The store is
/// also the camera observation, so it publishes through the view's bytes.
pub fn set_layer1(h: &mut (impl SessionRam + ?Sized), axis: usize, value: u16) {
    h.write_word_le(value, LAYER1 + 4 * axis);
}

/// Store BG1's offset on `axis`. The x offset is also an observation.
pub fn set_bg1_offset(h: &mut (impl SessionRam + ?Sized), axis: usize, value: u16) {
    h.write_word_le(value, BG1_OFFSET + 2 * axis);
}

/// RAM addresses of layer 1's whole x position and BG1's x offset.
pub const LAYER1: usize = 0x911;
pub const BG1_OFFSET: usize = 0x91d;
