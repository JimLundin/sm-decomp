//! The BG scrolls follow the camera, and the blocks they bring into view.
//!
//! Each frame, once the camera has followed Samus (`camera_world`), BG1 scrolls with layer 1 and
//! layer 2 follows layer 1 at the room's rates (bank `$80` log, the BG scrolls `$80:A3AB`).
//! A layer that has moved into another block column or row then exposes the line coming into
//! view (`$80:A3DF`): moving right the column 16 blocks on, moving left the one at the left
//! edge; moving down the row 15 blocks down, moving up the one below the top. Layer 2 exposes
//! lines only on an axis where BG2 shows its level data.

use crate::intent::{BlockLine, LayerExposure, LevelLayer};
use crate::state::room::{BlockPoint, Camera, Layer2Scroll};
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
// open: time frozen (`$80:A3AB`'s first test) with its users.
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

/// A layer position in blocks, signed as the original's shift sign-extends it from bit 11.
const fn layer_block(position: u16) -> u16 {
    (position.cast_signed() >> 4).cast_unsigned()
}

/// A BG scroll's top-left corner in blocks.
const fn bg_blocks(scroll: Scroll) -> BlockPoint {
    BlockPoint {
        x: scroll.x >> 4,
        y: scroll.y >> 4,
    }
}

/// Whether `now` is before `then` on the 16-bit ring of positions: a layer at block 0 moving
/// left reaches block `FFFFh` and is before it.
const fn before(now: u16, then: u16) -> bool {
    now.wrapping_sub(then).cast_signed() < 0
}

/// Where one layer's lines are measured from: the layer's blocks now, its background's, and the
/// blocks it last exposed lines from.
#[derive(Clone, Copy, Debug)]
struct Moved {
    layer: BlockPoint,
    background: BlockPoint,
    exposed: BlockPoint,
}

impl Moved {
    /// The lines `layer` exposes on the axes it streams, column then row; the layer's blocks
    /// are where it has now exposed lines from.
    fn exposures(
        self,
        layer: LevelLayer,
        (across, down): (bool, bool),
        exposures: &mut Vec<LayerExposure>,
    ) -> BlockPoint {
        let Self {
            layer: now,
            background,
            exposed,
        } = self;
        let mut from = exposed;
        let mut expose = |line, ahead: BlockPoint| {
            exposures.push(LayerExposure {
                layer,
                line,
                level: BlockPoint {
                    x: now.x.wrapping_add(ahead.x),
                    y: now.y.wrapping_add(ahead.y),
                },
                background: BlockPoint {
                    x: background.x.wrapping_add(ahead.x),
                    y: background.y.wrapping_add(ahead.y),
                },
            });
        };
        if across && now.x != exposed.x {
            from.x = now.x;
            let x = if before(now.x, exposed.x) { 0 } else { 16 };
            expose(BlockLine::Column, BlockPoint { x, y: 0 });
        }
        if down && now.y != exposed.y {
            from.y = now.y;
            let y = if before(now.y, exposed.y) { 1 } else { 15 };
            expose(BlockLine::Row, BlockPoint { x: 0, y });
        }
        from
    }
}

/// The lines of blocks the layers have moved into view since they last exposed lines (`$80:A3DF`):
/// layer 1's column and row, then layer 2's. Each layer remembers where it is.
///
/// The original makes BG1's column, BG2's, BG1's row, then BG2's; each line goes to its own
/// buffers, so the order between the layers changes no byte.
// open: the original builds no column while the mode 7 flag (`$0783`) is set, a room the
// slice's rooms don't load.
pub fn exposures(camera: &mut Camera, mirrors: &Registers) -> Vec<LayerExposure> {
    let [bg1, bg2, ..] = mirrors.scroll;
    let layer1 = Moved {
        layer: BlockPoint {
            x: layer_block(camera.x.pixels),
            y: layer_block(camera.y.pixels),
        },
        background: bg_blocks(bg1),
        exposed: camera.drawn.layer1,
    };
    let layer2 = Moved {
        layer: BlockPoint {
            x: layer_block(camera.layer2.x),
            y: layer_block(camera.layer2.y),
        },
        background: bg_blocks(bg2),
        exposed: camera.drawn.layer2,
    };
    let streams = (
        camera.layer2_scroll.x.shows_level(),
        camera.layer2_scroll.y.shows_level(),
    );
    let mut exposures = Vec::new();
    camera.drawn.layer1 = layer1.exposures(LevelLayer::One, (true, true), &mut exposures);
    camera.drawn.layer2 = layer2.exposures(LevelLayer::Two, streams, &mut exposures);
    exposures
}

#[cfg(test)]
mod tests;
