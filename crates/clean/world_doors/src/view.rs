//! Where the passage puts the view and her: the moves along the way she leaves, the squaring,
//! the set-off, and where she is placed, lands and steps clear.

use core::num::Wrapping;
use core::ops::RangeInclusive;

use types::{Fixed, Heading, PIXEL, Pair, Parts, Position, Px, ZERO, fixed, px};

use crate::passage::Arrival;

/// A move of `amount` the way she leaves: rightward and downward add, leftward and upward take
/// away, on the way's axis only.
pub fn toward(way: Heading, amount: Fixed) -> Pair<Fixed> {
    let signed = match way {
        Heading::Down | Heading::Right => amount,
        Heading::Left | Heading::Up => -amount,
    };
    Pair::from_along(way, signed, ZERO)
}

/// A position moved by `by`, each axis's subpixel carrying into its pixel part.
pub fn plus(position: Position, by: Pair<Fixed>) -> Position {
    Pair {
        x: position.x + by.x,
        y: position.y + by.y,
    }
}

/// A position's pixel parts.
pub fn pixels(position: Position) -> Pair<Px> {
    Pair {
        x: position.x.pixel(),
        y: position.y.pixel(),
    }
}

/// The position of these pixel parts, keeping the subpixel parts of `kept`.
pub fn at_pixels(pixels: Pair<Px>, kept: Position) -> Position {
    Pair {
        x: fixed(pixels.x, kept.x.sub()),
        y: fixed(pixels.y, kept.y.sub()),
    }
}

/// A pixel's place on its screen, 0 to 255: a screen is 256 px square, so this is the pixel's
/// low byte.
const fn on_its_screen(pixel: Px) -> u8 {
    let [low, _] = pixel.0.to_le_bytes();
    low
}

/// `Squaring`: a place on its screen at or past the middle creeps on to the next screen's edge.
const SCREEN_MIDDLE: u8 = 128;

/// `Squaring`: the view's creep of one pixel across the way she leaves, toward the nearest edge of
/// a screen (y for a way left or right, x for a way up or down); none once it is on an edge.
pub fn squared(camera: Position, way: Heading) -> Option<Position> {
    let (along, across) = camera.along(way);
    let creep = match on_its_screen(across.pixel()) {
        0 => return None,
        place if place >= SCREEN_MIDDLE => PIXEL,
        _ => -PIXEL,
    };
    Some(Pair::from_along(way, along, across + creep))
}

/// `SettingOff`: how far back from its arrival the view sets off, against the way she leaves,
/// and how much lower the arrival is taken from then on.
struct SetOff {
    /// The distance back, in px.
    back: Px,
    /// How much lower the arrival's y is taken.
    lowered: Px,
}

/// `SettingOff`: the set-off for each way she leaves. A way `Up` sets off 255 px back and takes
/// its arrival 32 px lower, so its slide ends on the lowered arrival.
const fn set_off(way: Heading) -> SetOff {
    match way {
        Heading::Down => SetOff {
            back: Wrapping(224),
            lowered: Wrapping(0),
        },
        Heading::Left | Heading::Right => SetOff {
            back: Wrapping(256),
            lowered: Wrapping(0),
        },
        Heading::Up => SetOff {
            back: Wrapping(255),
            lowered: Wrapping(32),
        },
    }
}

/// `SettingOff`: the view put back from its arrival against the way she leaves, keeping the
/// camera's subpixel parts; the arrival is taken lower as the way's set-off says.
pub fn setting_off(arrival: &mut Arrival, camera: Position) -> Position {
    let SetOff { back, lowered } = set_off(arrival.way);
    let view = plus(
        at_pixels(arrival.view, camera),
        toward(arrival.way, -px(back)),
    );
    arrival.view.y += lowered;
    view
}

/// `WhereSheLands`: placed in the next room, each pixel part of hers is the camera's plus her old
/// one's place on its screen, so she keeps where she was on the screen; her subpixels stay.
pub fn place(her: Position, camera: Position) -> Position {
    let on_screen = |her: Fixed, camera: Fixed| {
        fixed(
            camera.pixel() + Wrapping(u16::from(on_its_screen(her.pixel()))),
            her.sub(),
        )
    };
    Pair {
        x: on_screen(her.x, camera.x),
        y: on_screen(her.y, camera.y),
    }
}

/// `WhereSheLands`: where her x settles after the slide.
enum Settle {
    /// It stays.
    Stays,
    /// On the left edge of its 8 px column.
    LeftEdge,
    /// On the right edge of its 8 px column.
    RightEdge,
}

/// `WhereSheLands`: where she settles for each way she leaves: on the far edge of her 8 px
/// column, the way she went, or where she is for a way up or down.
const fn settle(way: Heading) -> Settle {
    match way {
        Heading::Down | Heading::Up => Settle::Stays,
        Heading::Left => Settle::LeftEdge,
        Heading::Right => Settle::RightEdge,
    }
}

/// `WhereSheLands`: the last pixel of an 8 px column.
const COLUMN_8_LAST: Px = Wrapping(7);

/// `WhereSheLands`: her x's pixel part settled for the way she left by.
pub fn settled(her: Position, way: Heading) -> Position {
    let x = her.x.pixel();
    let x = match settle(way) {
        Settle::Stays => x,
        Settle::LeftEdge => x & !COLUMN_8_LAST,
        Settle::RightEdge => x | COLUMN_8_LAST,
    };
    Pair {
        x: fixed(x, her.x.sub()),
        y: her.y,
    }
}

/// `WhereSheLands`: a screen's second 16 px column or row.
const SECOND_CELL: RangeInclusive<u8> = 16..=31;
/// `WhereSheLands`: a screen's second-last 16 px column.
const SECOND_LAST_CELL: RangeInclusive<u8> = 224..=239;
/// `WhereSheLands`: the last pixel of a 16 px column or row.
const CELL_LAST: Px = Wrapping(15);
/// `WhereSheLands`: from a cell's edge to its middle.
const HALF_CELL: Px = Wrapping(8);

/// `WhereSheLands`: she steps clear of a screen's second column into the middle of the third,
/// of its second-last column into the middle of the third-last, and of its second row into the
/// middle of the third. A screen's second-last row is never stepped from.
pub fn stepped_clear(her: Position) -> Position {
    let Pair { x, y } = pixels(her);
    let x = if SECOND_CELL.contains(&on_its_screen(x)) {
        (x | CELL_LAST) + HALF_CELL
    } else if SECOND_LAST_CELL.contains(&on_its_screen(x)) {
        (x & !CELL_LAST) - HALF_CELL
    } else {
        x
    };
    let y = if SECOND_CELL.contains(&on_its_screen(y)) {
        (y | CELL_LAST) + HALF_CELL
    } else {
        y
    };
    at_pixels(Pair { x, y }, her)
}
