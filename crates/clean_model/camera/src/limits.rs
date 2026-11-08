//! Area limits (§3): keeping the view in the room and out of its closed screens.
//!
//! `ClosedScreens`, `RoomEdges`, `ShortScreens`, `Settling`, `DownwardLooksAhead`,
//! `ScreensRunOn`, `BelowTheRoom`, `PushCarries`; `TimeStopped` is the step's (the crate root).
//! Every rule here sets the camera's pixel part only.

use core::iter::repeat;
use core::num::Wrapping;

use clean_types::{Heading, Pair, Parts, Px, below};

use crate::follow::{Going, Move};
use crate::input::{Room, ScreenKind};
use crate::rules::Rules;

/// Notation: a screen is 256 px square.
const SCREEN: Px = Wrapping(256);
/// Notation: a position's pixel within its screen, the low byte of its pixel part (its high byte
/// is the screen's number).
const WITHIN: Px = Wrapping(0xFF);
/// `ClosedScreens`: the view's middle, 128 px from its near edge (the view is 256 px across).
const MIDDLE: Px = Wrapping(128);
/// `RoomEdges`: the room's left and top edges.
const ORIGIN: Px = Wrapping(0);

/// `ClosedScreens`: `edge(p) = p − p mod 256`, the near edge of the screen holding `p`.
fn edge(p: Px) -> Px {
    p & !WITHIN
}

/// `FollowingHer`: the later of two positions, compared by `before`.
const fn latest(a: Px, b: Px) -> Px {
    if below(a.0, b.0) { b } else { a }
}

/// `FollowingHer`: the earlier of two positions, compared by `before`.
const fn earliest(a: Px, b: Px) -> Px {
    if below(a.0, b.0) { a } else { b }
}

/// `PushCarries`: the one more a push's sum carries.
fn carried(carry: bool) -> Px {
    Wrapping(u16::from(carry))
}

impl Room {
    /// How many screens the room is across and down.
    fn size(&self) -> Pair<usize> {
        Pair {
            x: self.screens.first().map_or(0, Vec::len),
            y: self.screens.len(),
        }
    }

    /// `ClosedScreens`: the kind of the screen holding a point.
    ///
    /// `ScreensRunOn`: the screens run on, row after row, so the one after a row's last column
    /// is the first of the row below. `BelowTheRoom`: below the room's last row come its `below`
    /// rows; a screen they don't hold is `Closed`.
    fn screen(&self, at: Pair<Px>) -> ScreenKind {
        let w = self.size().x;
        let ([row, _], [column, _]) = (at.y.0.to_be_bytes(), at.x.0.to_be_bytes());
        let index = usize::from(row)
            .wrapping_mul(w)
            .wrapping_add(usize::from(column));
        self.screens
            .iter()
            .chain(&self.below)
            .flat_map(|row| {
                row.iter()
                    .copied()
                    .chain(repeat(ScreenKind::Closed))
                    .take(w)
            })
            .nth(index)
            .unwrap_or(ScreenKind::Closed)
    }

    /// `RoomEdges`: the near edge of the room's last screen on an axis, `(n − 1) × 256`.
    fn last(&self, heading: Heading) -> Px {
        let (n, _) = self.size().along(heading);
        Wrapping(u16::try_from(n).unwrap_or(u16::MAX)) * SCREEN - SCREEN
    }
}

/// §3: the room's limits on one axis of one frame.
pub struct Bounds<'a> {
    /// The tuning.
    pub rules: &'a Rules,
    /// The room.
    pub room: &'a Room,
    /// The axis, by its forward heading: `Right` across, `Down` down.
    pub heading: Heading,
    /// The camera's pixel part on the other axis: the screens read are in the row (across) or
    /// column (down) holding its middle.
    pub across: Px,
    /// `v`: the speed's pixel part.
    pub v: Px,
}

impl Bounds<'_> {
    /// `ClosedScreens`: the screen holding `p` on this axis.
    fn screen(&self, p: Px) -> ScreenKind {
        self.room
            .screen(Pair::from_along(self.heading, p, self.across + MIDDLE))
    }

    /// `ClosedScreens`: whether the screen after the one holding `p` is `Closed`.
    fn next_closed(&self, p: Px) -> bool {
        self.screen(p + SCREEN) == ScreenKind::Closed
    }

    /// `ShortScreens`: `rows`, by the screen holding `p`: 0 across, and down 0 on a short screen
    /// and 31 on any other.
    fn rows(&self, p: Px) -> Px {
        if self.heading.vertical() && self.screen(p) != ScreenKind::Short {
            self.rules.short_rows
        } else {
            ORIGIN
        }
    }

    /// `RoomEdges`: the furthest right, `xr`, or down, `yb`, with the `rows` it reads.
    fn far(&self, rows: Px) -> Px {
        self.room.last(self.heading) + rows
    }

    /// `ClosedScreens`: the push forward, `ahead(p) = p + v + 2`; `PushCarries`.
    fn ahead(&self, p: Px) -> Px {
        let (sum, carry) = p.0.overflowing_add(self.v.0);
        Wrapping(sum) + self.rules.push + carried(carry)
    }

    /// `ClosedScreens`: the push back, `back(p) = p − v − 2`; `PushCarries`.
    fn back(&self, p: Px) -> Px {
        let (difference, borrow) = p.0.overflowing_sub(self.v.0);
        Wrapping(difference) - self.rules.push - carried(borrow)
    }

    /// §3: the room's limits on a move, measured from where the aim stopped it.
    pub fn limit(&self, m: Move) -> Px {
        let c = m.to.pixel();
        match m.going {
            // RoomEdges: left or up before 0, it stops on 0.
            Going::Back if below(c.0, ORIGIN.0) => ORIGIN,
            // ClosedScreens: going left or up, the camera's screen closed.
            Going::Back if self.screen(c) == ScreenKind::Closed => {
                earliest(edge(c) + SCREEN, self.ahead(m.reach))
            }
            Going::Back => c,
            Going::Forward => self.forward(m),
        }
    }

    /// §3, going right or down, from `c` where the aim stopped it and `cm` where the move took it.
    fn forward(&self, m: Move) -> Px {
        let (c, cm) = (m.to.pixel(), m.reach);
        let down = self.heading.vertical();
        // DownwardLooksAhead: going down, the next screen and `rows` are read from `cm`'s screen.
        let look = if down { cm } else { c };
        let rows = self.rows(look);
        let far = self.far(rows);
        let floor = edge(c) + rows;
        // RoomEdges: across it stops on `xr`; down it backs up as from a closed screen.
        let stop = if down { self.back(cm) } else { far };
        if below(far.0, c.0) {
            latest(far, stop)
        } else if self.next_closed(look) && below(floor.0, c.0) {
            // ClosedScreens: going right or down, the next screen closed.
            latest(floor, self.back(cm))
        } else {
            c
        }
    }

    /// `Settling`: on an axis she didn't move on, from `c0`, the camera's pixel part as the frame
    /// found it: brought into the room, it eases out of a closed screen it partly shows.
    pub fn settle(&self, c0: Px) -> Px {
        let a = self.ahead(c0);
        // ShortScreens: settling, `rows` is read from the screen its top is in before it settles.
        let rows = self.rows(c0);
        let c = earliest(latest(c0, ORIGIN), self.far(rows));
        let end = edge(c) + SCREEN;
        let floor = edge(c) + rows;
        match self.screen(c) {
            ScreenKind::Closed if !below(a.0, end.0) => end,
            ScreenKind::Closed if self.next_closed(a) => edge(a),
            ScreenKind::Closed => a,
            ScreenKind::Open | ScreenKind::Short if self.next_closed(c) && below(floor.0, c.0) => {
                latest(floor, self.back(c0))
            }
            ScreenKind::Open | ScreenKind::Short => c,
        }
    }
}
