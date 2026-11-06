//! The camera follows Samus.
//!
//! Each frame layer 1 moves toward where it wants to be at the speed Samus moved, the scroll
//! areas hold it out of red screens, and the BG scroll mirrors follow it (bank `$90` log, the
//! main scrolling routine `$90:94EC`; bank `$80` log, the scroll zones `$80:A528` to
//! `$80:A936` and the BG scrolls `$80:A3AB`).
//!
//! The original moves the camera only by whole screens' worth of scroll area: a red screen
//! is one the camera may not show, so moving into one pushes it back to the screen's edge 2
//! pixels a frame faster than Samus moves, and a blue screen at the room's bottom hides the
//! two rows the HUD would otherwise cover.

use crate::state::room::{Camera, Layer2Scroll, ScrollArea};
use crate::state::samus::{Acceleration, Facing, Fixed, MovementType, Samus, Speed, YDirection};
use crate::state::video::{Registers, Scroll};

/// The room's scroll areas, as the scrolling reads them.
#[derive(Clone, Copy, Debug)]
pub struct ScrollAreas<'a> {
    /// One per screen, row-major.
    pub areas: &'a [ScrollArea],
    /// Screens across: the row length.
    pub width: u16,
    /// Screens down.
    pub height: u16,
}

impl ScrollAreas<'_> {
    /// The area at `index` into the row-major table.
    ///
    /// The original reads past the room's table when the camera's far edge is beyond the
    /// room's last screen (the notes at `$80:A528` and `$80:A731`); what it finds there is
    /// whatever the buffer held, and the engine reads red, the area that stops the camera
    /// (docs/LEGACY.md).
    fn at(self, index: u16) -> ScrollArea {
        self.areas
            .get(usize::from(index))
            .copied()
            .unwrap_or(ScrollArea::Red)
    }

    /// The index of the screen `column` across and `row` down, as the hardware multiplier
    /// gives it: the row times the width's low byte.
    fn index(self, column: u16, row: u8) -> u16 {
        let [_, width] = self.width.to_be_bytes();
        u16::from(row)
            .wrapping_mul(u16::from(width))
            .wrapping_add(column)
    }

    /// The area of the screen `column` across and `row` down.
    fn area(self, column: u16, row: u8) -> ScrollArea {
        self.at(self.index(column, row))
    }

    /// The camera's furthest position along an axis `screens` long: the last screen's left or
    /// top edge, as the original's swapped bytes of the count less one give it.
    const fn last(screens: u16) -> u16 {
        screens.wrapping_sub(1).swap_bytes()
    }
}

/// The screen-sized half a position is in: its high byte.
const fn screen(position: u16) -> u8 {
    position.to_be_bytes()[0]
}

/// The screen the playfield's middle is in, along the other axis.
const fn middle_screen(position: u16) -> u8 {
    screen(position.wrapping_add(0x80))
}

/// The left or top edge of the screen `position` is in.
const fn screen_start(position: u16) -> u16 {
    position & 0xFF00
}

/// The left or top edge of the next screen.
const fn next_screen(position: u16) -> u16 {
    screen_start(position).wrapping_add(0x100)
}

/// Whether a word is negative, read as signed.
const fn negative(word: u16) -> bool {
    word & 0x8000 != 0
}

/// Whether `a` is less than `b` as the original's compare-and-branch-on-minus tests it: the
/// sign of their difference.
const fn before(a: u16, b: u16) -> bool {
    negative(a.wrapping_sub(b))
}

/// `start` moved forward by `speed` pixels and 2 more. The second add carries the first's
/// overflow, as the original's two `ADC`s do.
fn ahead(start: u16, speed: u16) -> u16 {
    let (sum, carry) = start.overflowing_add(speed);
    sum.wrapping_add(2).wrapping_add(u16::from(carry))
}

/// `start` moved back by `speed` pixels and 2 more. The second subtract borrows the first's
/// underflow, as the original's two `SBC`s do.
fn behind(start: u16, speed: u16) -> u16 {
    let (difference, borrow) = start.overflowing_sub(speed);
    difference.wrapping_sub(2).wrapping_sub(u16::from(borrow))
}

/// The camera's speed on an axis: how far Samus moved since the last frame, plus one pixel.
///
/// The direction is the whole pixels' (`$90:96C0`, `$90:96FF`): a move left within a pixel
/// counts as rightward, and the speed is then less than one pixel.
const fn speed(now: Fixed, previous: Fixed) -> Speed {
    let (now_whole, now) = (now.pixels, Speed::new(now.pixels, now.subpixels).0);
    let previous_whole = previous.pixels;
    let previous = Speed::new(previous.pixels, previous.subpixels).0;
    let moved = if before(now_whole, previous_whole) {
        previous.wrapping_sub(now)
    } else {
        now.wrapping_sub(previous)
    };
    Speed(moved.wrapping_add(ONE_PIXEL.0))
}

/// The pixel the camera's speed adds to Samus's move (`$90:9EAD`).
const ONE_PIXEL: Speed = Speed::new(1, 0);

/// `position` moved by `speed`, forward or back, carrying through the subpixels.
const fn moved(position: Fixed, speed: Speed, forward: bool) -> Fixed {
    let start = Speed::new(position.pixels, position.subpixels).0;
    let end = Speed(if forward {
        start.wrapping_add(speed.0)
    } else {
        start.wrapping_sub(speed.0)
    });
    Fixed {
        pixels: end.pixels(),
        subpixels: end.subpixels(),
    }
}

/// The main scrolling routine (`$90:94EC`): layer 1 follows Samus across, then down, and the
/// frame's position becomes the one the next frame's speed is measured from.
// open: slow grapple scrolling (`$90:94F7`) joins with the grapple beam, the scrolling
// finished hook (`$90:9563`, Spore Spawn's floor) with that room, and time frozen (which the
// scroll zones test, `$80:A52C`) with the reserve tanks and the X-ray scope.
pub fn follow(camera: &mut Camera, samus: &mut Samus, areas: ScrollAreas<'_>) {
    let speed_x = speed(samus.position.x, samus.previous_position.x);
    horizontal(camera, samus, areas, speed_x);
    let speed_y = speed(samus.position.y, samus.previous_position.y);
    vertical(camera, samus, areas, speed_y);
    samus.previous_position = samus.position;
}

/// Handle horizontal scrolling (`$90:95A0`).
fn horizontal(camera: &mut Camera, samus: &Samus, areas: ScrollAreas<'_>, speed: Speed) {
    if samus.position.x.pixels == samus.previous_position.x.pixels {
        settle_x(camera, areas, speed);
        return;
    }
    // open: a knockback (`$0A52`) also aims the camera backwards; it joins with damage.
    let backwards = samus.movement.kind() == MovementType::Moonwalking
        || samus
            .movement
            .horizontal()
            .is_some_and(|motion| motion.acceleration == Acceleration::TurningAround);
    let (facing_right, facing_left) = camera.distance.lead();
    let lead = if (samus.facing == Facing::Left) == backwards {
        facing_right
    } else {
        facing_left
    };
    let ideal = samus.position.x.pixels.wrapping_sub(lead);
    if ideal == camera.x.pixels {
        return;
    }
    let rightward = !before(ideal, camera.x.pixels);
    camera.x = moved(camera.x, speed, rightward);
    if rightward {
        scroll_right(camera, areas, ideal, speed.pixels());
    } else {
        scroll_left(camera, areas, ideal, speed.pixels());
    }
}

/// Handle vertical scrolling (`$90:964F`).
fn vertical(camera: &mut Camera, samus: &Samus, areas: ScrollAreas<'_>, speed: Speed) {
    if samus.position.y.pixels == samus.previous_position.y.pixels {
        settle_y(camera, areas, speed);
        return;
    }
    let rising = samus
        .movement
        .vertical()
        .is_some_and(|motion| motion.direction == YDirection::Up);
    let scroller = if rising {
        camera.scrollers.down
    } else {
        camera.scrollers.up
    };
    let ideal = samus.position.y.pixels.wrapping_sub(scroller);
    if ideal == camera.y.pixels {
        return;
    }
    let downward = !before(ideal, camera.y.pixels);
    camera.y = moved(camera.y, speed, downward);
    if downward {
        scroll_down(camera, areas, ideal, speed.pixels());
    } else {
        scroll_up(camera, areas, ideal, speed.pixels());
    }
}

/// Horizontal autoscrolling (`$80:A528`): Samus hasn't moved across, so the camera, inside
/// the room, leaves a red screen toward the open one beside it.
fn settle_x(camera: &mut Camera, areas: ScrollAreas<'_>, speed: Speed) {
    let start = camera.x.pixels;
    let x = if negative(start) { 0 } else { start }.min(ScrollAreas::last(areas.width));
    let row = middle_screen(camera.y.pixels);
    let left = areas.index(u16::from(screen(x)), row);
    let speed = speed.pixels();
    camera.x.pixels = if areas.at(left) == ScrollArea::Red {
        let edge = next_screen(x);
        let proposed = ahead(start, speed);
        if proposed >= edge {
            edge
        } else if areas.area(u16::from(screen(proposed).wrapping_add(1)), row) == ScrollArea::Red {
            screen_start(proposed)
        } else {
            proposed
        }
    } else if areas.at(left.wrapping_add(1)) == ScrollArea::Red {
        let edge = screen_start(x);
        let proposed = behind(start, speed);
        if before(proposed, edge) {
            edge
        } else if areas.area(u16::from(screen(proposed)), row) == ScrollArea::Red {
            next_screen(proposed)
        } else {
            proposed
        }
    } else {
        x
    };
}

/// Scrolling right (`$80:A641`): no further than the ideal position or the room's last
/// screen, and back out of a red screen on the right.
fn scroll_right(camera: &mut Camera, areas: ScrollAreas<'_>, ideal: u16, speed: u16) {
    let start = camera.x.pixels;
    if before(ideal, camera.x.pixels) {
        camera.x = Fixed {
            pixels: ideal,
            subpixels: 0,
        };
    }
    let x = camera.x.pixels;
    let last = ScrollAreas::last(areas.width);
    if last < x {
        camera.x.pixels = last;
        return;
    }
    let right = areas
        .index(u16::from(screen(x)), middle_screen(camera.y.pixels))
        .wrapping_add(1);
    if areas.at(right) != ScrollArea::Red {
        return;
    }
    let edge = screen_start(x);
    let proposed = behind(start, speed);
    camera.x.pixels = if before(proposed, edge) {
        edge
    } else {
        proposed
    };
}

/// Scrolling left (`$80:A6BB`): no further than the ideal position or the room's left edge,
/// and back out of a red screen on the left.
fn scroll_left(camera: &mut Camera, areas: ScrollAreas<'_>, ideal: u16, speed: u16) {
    let start = camera.x.pixels;
    if before(camera.x.pixels, ideal) {
        camera.x = Fixed {
            pixels: ideal,
            subpixels: 0,
        };
    }
    let x = camera.x.pixels;
    if negative(x) {
        camera.x.pixels = 0;
        return;
    }
    if areas.area(u16::from(screen(x)), middle_screen(camera.y.pixels)) != ScrollArea::Red {
        return;
    }
    let edge = next_screen(x);
    let proposed = ahead(start, speed);
    camera.x.pixels = proposed.min(edge);
}

/// How far past the last screen's top the camera may go at the room's bottom: the two rows
/// the HUD covers, shown unless the screen above is blue.
fn bottom_rows(areas: ScrollAreas<'_>, index: u16) -> u16 {
    if areas.at(index) == ScrollArea::Blue {
        0
    } else {
        0x1F
    }
}

/// Vertical autoscrolling (`$80:A731`): Samus hasn't moved down or up, so the camera, inside
/// the room, leaves a red screen toward the open one above or below it.
fn settle_y(camera: &mut Camera, areas: ScrollAreas<'_>, speed: Speed) {
    camera.y.pixels = settled_y(camera, areas, speed.pixels());
}

/// Where vertical autoscrolling leaves the camera's top edge.
fn settled_y(camera: &Camera, areas: ScrollAreas<'_>, speed: u16) -> u16 {
    let column = u16::from(middle_screen(camera.x.pixels));
    let rows = bottom_rows(areas, areas.index(column, screen(camera.y.pixels)));
    let start = camera.y.pixels;
    let floor = ScrollAreas::last(areas.height).wrapping_add(rows);
    let y = if negative(start) { 0 } else { start }.min(floor);
    let upper = areas.index(column, screen(y));
    if areas.at(upper) == ScrollArea::Red {
        let edge = next_screen(y);
        let proposed = ahead(start, speed);
        if proposed >= edge {
            edge
        } else if areas.area(column, screen(proposed).wrapping_add(1)) == ScrollArea::Red {
            screen_start(proposed)
        } else {
            proposed
        }
    } else if areas.at(upper.wrapping_add(areas.width)) == ScrollArea::Red {
        let edge = screen_start(y).wrapping_add(rows);
        if edge >= y {
            return y;
        }
        let proposed = behind(start, speed);
        if before(proposed, edge) {
            edge
        } else if areas.area(column, screen(proposed)) == ScrollArea::Red {
            next_screen(proposed)
        } else {
            proposed
        }
    } else {
        y
    }
}

/// Scrolling down (`$80:A893`): no further than the ideal position or the room's bottom, and
/// back out of a red screen below.
fn scroll_down(camera: &mut Camera, areas: ScrollAreas<'_>, ideal: u16, speed: u16) {
    let start = camera.y.pixels;
    let upper = areas.index(
        u16::from(middle_screen(camera.x.pixels)),
        screen(camera.y.pixels),
    );
    let rows = bottom_rows(areas, upper);
    if before(ideal, camera.y.pixels) {
        camera.y = Fixed {
            pixels: ideal,
            subpixels: 0,
        };
    }
    let y = camera.y.pixels;
    let bottom = ScrollAreas::last(areas.height).wrapping_add(rows);
    let edge = if bottom < y {
        bottom
    } else {
        if areas.at(upper.wrapping_add(areas.width)) != ScrollArea::Red {
            return;
        }
        let edge = screen_start(y).wrapping_add(rows);
        if edge >= y {
            return;
        }
        edge
    };
    let proposed = behind(start, speed);
    camera.y.pixels = if before(proposed, edge) {
        edge
    } else {
        proposed
    };
}

/// Scrolling up (`$80:A936`): no further than the ideal position or the room's top, and back
/// out of a red screen above.
fn scroll_up(camera: &mut Camera, areas: ScrollAreas<'_>, ideal: u16, speed: u16) {
    let start = camera.y.pixels;
    if before(camera.y.pixels, ideal) {
        camera.y = Fixed {
            pixels: ideal,
            subpixels: 0,
        };
    }
    let y = camera.y.pixels;
    if negative(y) {
        camera.y.pixels = 0;
        return;
    }
    if areas.area(u16::from(middle_screen(camera.x.pixels)), screen(y)) != ScrollArea::Red {
        return;
    }
    let edge = next_screen(y);
    let proposed = ahead(start, speed);
    camera.y.pixels = proposed.min(edge);
}

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
