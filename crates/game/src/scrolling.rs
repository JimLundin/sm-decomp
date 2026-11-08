//! The BG scrolls follow the camera.
//!
//! Each frame, once the camera has followed Samus (`camera_world`), BG1 scrolls with layer 1 and
//! layer 2 follows layer 1 at the room's rates (bank `$80` log, the BG scrolls `$80:A3AB`).

use crate::state::room::{Camera, Layer2Scroll};
use crate::state::video::{Registers, Scroll};

/// Layer 2's position on an axis from layer 1's, or `None` while it stays fixed (`$80:A2F9`,
/// `$80:A33A`): a scaled rate multiplies the low and high bytes separately, as the hardware
/// multiplier does.
fn layer2(layer1: u16, scroll: Layer2Scroll) -> Option<u16> {
    match scroll {
        Layer2Scroll::WithLayer1 => Some(layer1),
        Layer2Scroll::Fixed => None,
        Layer2Scroll::Scaled { rate, .. } => {
            let [high, low] = layer1.to_be_bytes();
            let rate = u16::from(rate);
            let low_part = u16::from(low).wrapping_mul(rate) >> 8;
            Some(low_part.wrapping_add(u16::from(high).wrapping_mul(rate)))
        }
    }
}

/// The BG scrolls from the camera (`$80:A3AB`): BG1 from layer 1, layer 2 from layer 1 at
/// its rates, and BG2 from layer 2 unless it stays fixed.
// open: the BG graphics update when scrolling (`$80:A3DF`), the layer and BG block
// coordinates and the level data columns and rows they upload, joins with the level
// uploads; time frozen (`$80:A3AB`'s first test) with its users.
pub fn scroll_backgrounds(camera: &mut Camera, mirrors: &mut Registers) {
    let [bg1, bg2, ..] = &mut mirrors.scroll;
    *bg1 = Scroll {
        x: camera.x.pixels.wrapping_add(camera.bg1_offset.x),
        y: camera.y.pixels.wrapping_add(camera.bg1_offset.y),
    };
    if let Some(x) = layer2(camera.x.pixels, camera.layer2_scroll.x) {
        camera.layer2.x = x;
        bg2.x = x.wrapping_add(camera.bg2_offset.x);
    }
    if let Some(y) = layer2(camera.y.pixels, camera.layer2_scroll.y) {
        camera.layer2.y = y;
        bg2.y = y.wrapping_add(camera.bg2_offset.y);
    }
}

#[cfg(test)]
mod tests;
