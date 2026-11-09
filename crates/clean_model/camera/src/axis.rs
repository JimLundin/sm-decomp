//! One axis of a frame: the camera following her there (Spec: §2), and the room keeping the view
//! in (Spec: §3): its edges, its closed and short screens, and how the view settles out of a
//! closed screen.
//!
//! The steps are written once, in `Line`. Each axis supplies only what differs: its screens and
//! where it reads them, its far bound, its rows, and whether it tests them. Every rule of the
//! room sets the camera's pixel part only; its subpixel part is the follow's.

use core::num::Wrapping;

use clean_types::{Fixed, Pair, Parts, Px, fixed};

use crate::follow::{Going, Move, before, chase, exceeds, max_by_before, min_by_exceeds, speed};
use crate::input::Frame;
use crate::room::{SCREEN, ScreenKind};
use crate::rules::Rules;

/// Notation: the pixels within a screen, a position's low byte.
const WITHIN_SCREEN: Px = Wrapping(0xFF);
/// How far past the view's near edge on the other axis its screens are read: 128 px, the view's
/// middle (Spec: ClosedScreens).
const MIDDLE: Px = Wrapping(128);
/// The room's left edge and its top edge, both position 0 (Spec: RoomEdges).
const ROOM_START: Px = Wrapping(0);
/// No rows: across, and down on a short screen, where the view stops with its top on the
/// screen's top edge (Spec: ShortScreens).
const NO_ROWS: Px = Wrapping(0);

/// The near edge of the screen holding `p`: p less p mod 256 (Spec: ClosedScreens, edge).
fn edge(p: Px) -> Px {
    p & !WITHIN_SCREEN
}

/// The near edge of the screen after the one holding `p`.
fn next_edge(p: Px) -> Px {
    edge(p) + SCREEN
}

/// Which axis a line is, with the position on the other axis whose screens it reads (Spec: §1.3,
/// Which screens it looks at).
#[derive(Clone, Copy)]
enum Axis {
    /// Across: the screens of the row holding this position down.
    Across {
        /// The position down whose row is read.
        row: Px,
    },
    /// Down: the screens of the column holding this position across.
    Down {
        /// The position across whose column is read.
        column: Px,
    },
}

/// One axis of a frame as the camera meets it: the line of screens it reads there, and the rules
/// that move the view along it.
pub(crate) struct Line<'a> {
    /// The frame's inputs.
    frame: &'a Frame<'a>,
    /// The tuning.
    rules: &'a Rules,
    /// Which axis, and the line of screens it reads.
    axis: Axis,
}

impl<'a> Line<'a> {
    /// The line across, for a camera whose y, its pixel part down before it moves down, is `y`:
    /// the screens of the row holding y + 128 (Spec: §4).
    pub(crate) fn across(frame: &'a Frame<'a>, rules: &'a Rules, y: Px) -> Self {
        Self {
            frame,
            rules,
            axis: Axis::Across { row: y + MIDDLE },
        }
    }

    /// The line down, for a camera whose x, its pixel part across after it moved across, is `x`:
    /// the screens of the column holding x + 128 (Spec: §4).
    pub(crate) fn down(frame: &'a Frame<'a>, rules: &'a Rules, x: Px) -> Self {
        Self {
            frame,
            rules,
            axis: Axis::Down { column: x + MIDDLE },
        }
    }

    /// The camera's position on this axis after the frame, from `camera`, given `her`, her
    /// position, `seen`, where it last saw her, and `aim`, its aim's pixel part, all on this axis.
    /// When she moved, it moves towards its aim and the room limits it (Spec: FollowingHer, §3);
    /// when she didn't, it settles, unless time is stopped (Spec: Settling, TimeStopped).
    pub(crate) fn follow(&self, camera: Fixed, her: Fixed, seen: Fixed, aim: Px) -> Fixed {
        let speed = speed(self.rules, her, seen);
        // The pushes' v, the speed's pixel part (Spec: ClosedScreens).
        let v = speed.pixel();
        if her.pixel() != seen.pixel() {
            return chase(camera, aim, speed)
                .map_or(camera, |m| fixed(self.limit(m, v), m.stopped.sub()));
        }
        if self.frame.time_stopped {
            camera
        } else {
            fixed(self.settle(camera.pixel(), v), camera.sub())
        }
    }

    // What each axis supplies.

    /// The kind of the screen holding `p` on this axis, in the line it reads.
    fn screen(&self, p: Px) -> ScreenKind {
        let at = match self.axis {
            Axis::Across { row } => Pair { x: p, y: row },
            Axis::Down { column } => Pair { x: column, y: p },
        };
        self.frame.room.screen(at)
    }

    /// Where a move right or down reads the next screen and its rows from, given the move.
    fn looks_from(&self, m: Move) -> Px {
        match self.axis {
            Axis::Across { .. } => m.stopped.pixel(),
            // Spec: DownwardLooksAhead. Reproduces the original's behaviour: going down, they are
            // read from where the move took the view, before the aim stopped it, so a move that
            // crosses into the next screen reads that screen's next and can back the view up past
            // its aim (C-ahead).
            Axis::Down { .. } => m.reached,
        }
    }

    /// How far below its screen's near edge the view stops where it can go no further, by the
    /// screen holding `p` (Spec: ShortScreens): across none; down, none on a short screen and
    /// `Rules::rows_not_short` on any other.
    fn rows(&self, p: Px) -> Px {
        match self.axis {
            Axis::Across { .. } => NO_ROWS,
            Axis::Down { .. } if self.screen(p).is_short() => NO_ROWS,
            Axis::Down { .. } => self.rules.rows_not_short,
        }
    }

    /// The furthest the view goes on this axis, given `rows`: the near edge of the room's last
    /// column, xr, across, and the top edge of its last row plus `rows`, yb, down (Spec:
    /// RoomEdges).
    fn far(&self, rows: Px) -> Px {
        let last = self.frame.room.last;
        match self.axis {
            Axis::Across { .. } => last.x,
            Axis::Down { .. } => last.y + rows,
        }
    }

    /// Where a move right or down that `exceeds` the far bound `far` stops, given `back`, the
    /// push back from where the move took it (Spec: RoomEdges). Across it stops on the far bound;
    /// down it backs up as from a closed screen, no higher than the bound.
    ///
    /// A view inside the room gets past it by a move past the room's last column or row (the
    /// recorded vectors Spec: RoomEdges lists); a camera another domain left left of or above the
    /// room exceeds it at once, as `FFF3` exceeds every bound, so it goes straight to the room's
    /// right edge or bottom (C-edge-right-from-left, C-edge-down-from-above,
    /// C-edge-down-from-above-short).
    const fn stop_past(&self, far: Px, back: Px) -> Px {
        match self.axis {
            Axis::Across { .. } => far,
            Axis::Down { .. } => max_by_before(far, back),
        }
    }

    /// Whether a view at `c`, its next screen closed, is pushed back towards `floor`, its
    /// screen's edge plus the rows (Spec: ClosedScreens, Settling): across always, there being no
    /// test; down only when `c` exceeds `floor`.
    const fn past_floor(&self, c: Px, floor: Px) -> bool {
        match self.axis {
            Axis::Across { .. } => true,
            Axis::Down { .. } => exceeds(c, floor),
        }
    }

    // The steps, the same on both axes.

    /// The push forward from `p`: p + v + 2, the spec's ahead (Spec: ClosedScreens).
    ///
    /// Spec: PushCarries. Reproduces the original's behaviour: the sum p + v carries once more
    /// when it passes FFFF. That shows only when the push sums from left of or above the room:
    /// from a camera another domain left there (C-push-carries), or from where a move left or up
    /// took it past the room's near edge before the aim stopped it (C-push-carries-move).
    fn ahead(&self, p: Px, v: Px) -> Px {
        let (sum, carried) = p.0.overflowing_add(v.0);
        Wrapping(sum) + Wrapping(u16::from(carried)) + self.rules.push
    }

    /// The push back from `p`: p − v − 2, the spec's back (Spec: ClosedScreens).
    ///
    /// It has no carry, as the push forward has (Spec: PushCarries): its own would take 1 more
    /// when p − v goes below 0. That needs `p` under the speed, at the room's first pixels, and
    /// every such push back lands before the edge it is held at, so the `max` that holds it
    /// keeps the edge and the pixel more would never show.
    fn back(&self, p: Px, v: Px) -> Px {
        p - v - self.rules.push
    }

    /// Whether the screen after the one holding `p` is closed.
    fn next_closed(&self, p: Px) -> bool {
        self.screen(p + SCREEN).is_closed()
    }

    /// Where a move ends once the room limits it, measured from where the aim stopped it (Spec:
    /// §3), `v` being the speed's pixel part.
    fn limit(&self, m: Move, v: Px) -> Px {
        match m.going {
            Going::Back => self.limit_back(m, v),
            Going::Forward => self.limit_forward(m, v),
        }
    }

    /// Where a move left or up ends (Spec: RoomEdges, ClosedScreens).
    fn limit_back(&self, m: Move, v: Px) -> Px {
        let c = m.stopped.pixel();
        // Before the room's near edge it stops on it, and no closed screen moves it: reached by a
        // move left or up past the room's first pixel (R1, R5, R8, R10, R11).
        if before(c, ROOM_START) {
            return ROOM_START;
        }
        // In a closed screen it is pushed forward from where the move took it, but no further
        // than the next screen's edge, the min as plain numbers: a camera that moved onto its aim
        // from 8000 away pushes to 8100, which by `before` would be kept (C-push-left-far).
        if self.screen(c).is_closed() {
            return min_by_exceeds(next_edge(c), self.ahead(m.reached, v));
        }
        c
    }

    /// Where a move right or down ends (Spec: RoomEdges, ClosedScreens). Past the far bound it
    /// stops there, and no closed screen moves it. With the next screen closed, and past the
    /// floor where the axis tests it, it is pushed back from where the move took it, but not
    /// before the floor, its screen's edge plus the rows.
    fn limit_forward(&self, m: Move, v: Px) -> Px {
        let c = m.stopped.pixel();
        let looked = self.looks_from(m);
        let rows = self.rows(looked);
        let back = self.back(m.reached, v);
        let far = self.far(rows);
        if exceeds(c, far) {
            return self.stop_past(far, back);
        }
        let floor = edge(c) + rows;
        if self.next_closed(looked) && self.past_floor(c, floor) {
            max_by_before(floor, back)
        } else {
            c
        }
    }

    /// Where the view settles on an axis she didn't move on, from `found`, its pixel part as the
    /// frame found it, `v` being the speed's pixel part (Spec: Settling).
    ///
    /// The rows are read from the screen `found` is in (Spec: ShortScreens). It is first brought
    /// into the room, between 0 and its far bound: a camera another domain left left of the room
    /// is brought to 0 (C-settle-reach-from-left, C-push-carries), one right of it or below it to
    /// its far bound (C-settle-back-right-of-room, C-settle-back-far-right,
    /// C-settle-up-from-below-short); down, the bound itself moves when the screen its top is in
    /// became short (C-settle-into-room-down). Then it eases out of a closed screen, or back from
    /// a closed next screen towards the floor, its screen's edge plus the rows.
    fn settle(&self, found: Px, v: Px) -> Px {
        let rows = self.rows(found);
        let c = min_by_exceeds(max_by_before(found, ROOM_START), self.far(rows));
        let floor = edge(c) + rows;
        if self.screen(c).is_closed() {
            self.ease_out(c, found, v)
        } else if self.next_closed(c) && self.past_floor(c, floor) {
            self.ease_back(floor, found, v)
        } else {
            c
        }
    }

    /// Where the view in a closed screen settles (Spec: Settling), from `c`, where it was
    /// brought into the room, and `found`, where the frame found it.
    ///
    /// It eases forward by the push from where it was found, `ahead(found)`. She didn't move, so
    /// the speed is the extra pixel and what she moved inside a pixel, its pixel part 1 (or 0
    /// after a creep, Spec: CreepReadsRightward), and the push is that and 2 more: 3 px a frame,
    /// or 2. An ease that reaches the next screen's edge, or exceeds it, stops on it: from a
    /// camera left of the room, `FFF0` eases to `FFF3`, which exceeds `0100`
    /// (C-settle-reach-from-left). An ease whose next screen is closed too drops to the edge of
    /// the screen it is in.
    fn ease_out(&self, c: Px, found: Px, v: Px) -> Px {
        let eased = self.ahead(found, v);
        let next = next_edge(c);
        if eased == next || exceeds(eased, next) {
            next
        } else if self.next_closed(eased) {
            edge(eased)
        } else {
            eased
        }
    }

    /// Where the view settles when the screen after its own is closed (Spec: Settling): pushed
    /// back from `found`, where the frame found it, but not before `floor`. When the push back
    /// is the later and lands in a closed screen, it goes on to that screen's next edge: from a
    /// camera right of the room, the push back can land in the column after the last, which runs
    /// on to a closed screen (C-settle-back-far-right).
    fn ease_back(&self, floor: Px, found: Px, v: Px) -> Px {
        let pushed = self.back(found, v);
        if before(floor, pushed) && self.screen(pushed).is_closed() {
            next_edge(pushed)
        } else {
            max_by_before(floor, pushed)
        }
    }
}
