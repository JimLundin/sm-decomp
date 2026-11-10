//! One axis of a frame: the camera following her there (Spec: §2), and the room keeping the view
//! in (Spec: §3): its edges, its closed and short screens, and how the view settles out of a
//! closed screen.
//!
//! The steps are written once, in `Line`. Each axis gives its values once, in `Line::new`, and
//! says which coordinate of a screen its positions are, in `Line::screen`; each of the three
//! decisions where the axes differ is one `match` on `Axis`. Every rule of the room sets the
//! camera's pixel part only; its subpixel part is the follow's.

use core::num::Wrapping;

use clean_types::{Fixed, Pair, Parts, Position, Px, ScreenKind, fixed};

use crate::follow::{Going, Move, before, chase, exceeds, max_by_before, min_by_exceeds, speed};
use crate::input::Frame;
use crate::room::SCREEN;
use crate::rules::Rules;

/// Notation: the pixels within a screen, a position's low byte.
const WITHIN_SCREEN: Px = Wrapping(0xFF);
/// How far past the view's near edge on the other axis lies the point its screens are read by:
/// 128 px below its top edge across, 128 px right of its left edge down (Spec: §1.3, Which
/// screens it looks at; ClosedScreens).
const READ_POINT: Px = Wrapping(128);
/// The room's left edge and its top edge, both position 0 (Spec: RoomEdges).
const ROOM_START: Px = Wrapping(0);
/// No margin: across, and down on a short screen, where the view stops with its top on the
/// screen's top edge (Spec: ShortScreens).
const NO_MARGIN: Px = Wrapping(0);

/// The near edge of the screen holding `p`: p less p mod 256 (Spec: ClosedScreens, edge).
fn edge(p: Px) -> Px {
    p & !WITHIN_SCREEN
}

/// The near edge of the screen after the one holding `p`.
fn next_edge(p: Px) -> Px {
    edge(p) + SCREEN
}

/// Which axis a line is. The axes differ in six things (Spec: §1.3, §3):
///
/// - the point on the other axis whose screens it reads: across, 128 px below the view's top
///   edge, so the row holding y + 128; down, 128 px right of its left edge, so the column holding
///   x + 128;
/// - its far end: across, the near edge of the room's last column (xr); down, the top edge of
///   its last row, to which the margin is added where it bounds the view (yb);
/// - its margin: across none; down, `Rules::margin` on any screen but a short one;
/// - where a move right or down reads its screens from: across, where the aim stopped it; down,
///   where the move took it (Spec: DownwardLooksAhead);
/// - how a move past the far bound stops: across, on the bound; down, backing up as from a
///   closed screen (Spec: RoomEdges);
/// - whether the floor, its screen's edge plus the margin, is tested: down only (Spec:
///   ClosedScreens, Settling).
///
/// The first three are values, given in `Line::new` (`Line::screen` then places the read point
/// on the other axis); the last three are decisions, each written once, in `Line::looks_from`,
/// `Line::stop_past` and `Line::past_floor`.
#[derive(Clone, Copy)]
pub(crate) enum Axis {
    /// x, rightward.
    Across,
    /// y, downward.
    Down,
}

/// One axis of a frame as the camera meets it: the view and her on it, the line of screens it
/// reads there, and the rules that move the view along it.
pub(crate) struct Line<'a> {
    /// The frame's inputs.
    frame: &'a Frame<'a>,
    /// The tuning.
    rules: &'a Rules,
    /// Which axis.
    axis: Axis,
    /// The camera's position on this axis as the frame found it.
    camera: Fixed,
    /// The point on the other axis, 128 px past the view's near edge there, whose row (across) or
    /// column (down) holds the screens this line reads.
    read_point: Px,
    /// The near edge of the room's last column across (xr), of its last row down. Where it bounds
    /// the view, in `limit_forward` and `settle`, the margin of the screen read is added to it,
    /// so down the bound is yb (Spec: RoomEdges).
    far: Px,
    /// The margin: how far under its screen's top edge the view's top may stop where it can go no
    /// lower; across there is none. Down it is this on any screen but a short one, which has none
    /// (Spec: ShortScreens).
    margin: Px,
    /// Whether she moved on this axis: her pixel part is not where it last saw her (Spec:
    /// FollowingHer).
    moved: bool,
    /// The speed the camera moves at (Spec: FollowingHer).
    speed: Fixed,
    /// The speed's whole-pixel part, the spec's v, which the pushes add (Spec: ClosedScreens).
    whole: Px,
}

impl<'a> Line<'a> {
    /// The line on `axis`, given `seen`, where it last saw her, and `view`, the camera's position
    /// as the frame stands when the line is made: across, before it moves down; down, after it
    /// moved across (Spec: §4).
    pub(crate) fn new(
        frame: &'a Frame<'a>,
        rules: &'a Rules,
        axis: Axis,
        seen: Position,
        view: Pair<Fixed>,
    ) -> Self {
        let her = frame.samus.position;
        let last = frame.room.last;
        let (camera, other, her, seen, far, margin) = match axis {
            Axis::Across => (view.x, view.y, her.x, seen.x, last.x, NO_MARGIN),
            Axis::Down => (view.y, view.x, her.y, seen.y, last.y, rules.margin),
        };
        let speed = speed(rules, her, seen);
        Self {
            frame,
            rules,
            axis,
            camera,
            read_point: other.pixel() + READ_POINT,
            far,
            margin,
            moved: her.pixel() != seen.pixel(),
            speed,
            whole: speed.pixel(),
        }
    }

    /// The camera's position on this axis after the frame, given `aim`, its aim's pixel part.
    /// When she moved, it moves towards its aim and the room limits it (Spec: FollowingHer, §3);
    /// when she didn't, it settles, unless time is stopped (Spec: Settling, TimeStopped).
    pub(crate) fn follow(&self, aim: Px) -> Fixed {
        let camera = self.camera;
        if self.moved {
            return chase(camera, aim, self.speed)
                .map_or(camera, |m| fixed(self.limit(m), m.stopped.sub()));
        }
        if self.frame.time_stopped {
            camera
        } else {
            fixed(self.settle(), camera.sub())
        }
    }

    /// The kind of the screen holding `p` on this axis, in the row or column the read point is in.
    fn screen(&self, p: Px) -> ScreenKind {
        let at = match self.axis {
            Axis::Across => Pair {
                x: p,
                y: self.read_point,
            },
            Axis::Down => Pair {
                x: self.read_point,
                y: p,
            },
        };
        self.frame.room.screen(at)
    }

    // Where the axes decide differently.

    /// Where a move right or down reads the next screen and its margin from, given the move.
    fn looks_from(&self, m: Move) -> Px {
        match self.axis {
            Axis::Across => m.stopped.pixel(),
            // Spec: DownwardLooksAhead. Reproduces the original's behaviour: going down, they are
            // read from where the move took the view, before the aim stopped it, so a move that
            // crosses into the next screen reads that screen's next and can back the view up past
            // its aim (C-ahead).
            Axis::Down => m.reached,
        }
    }

    /// Where a move right or down that exceeds `bound`, the far end plus the margin (xr across,
    /// yb down), stops, given `back`, the push back from where the move took it (Spec:
    /// RoomEdges).
    ///
    /// A view inside the room gets past it by a move past the room's last column or row
    /// (C-right-onto-edge, C-down-to-bottom-31); a camera another domain left left of or above the
    /// room exceeds it at once, as `FFF3` exceeds every bound (C-edge-right-from-left,
    /// C-edge-down-from-above, C-edge-down-from-above-short).
    const fn stop_past(&self, bound: Px, back: Px) -> Px {
        match self.axis {
            // Across it stops on the room's right edge.
            Axis::Across => bound,
            // Down it backs up as from a closed screen, the max by `before`: from inside or above
            // the room the push back is before the bound, so it stops on the bound; from a camera
            // left far enough below the room it is after it, so it backs up 2 px, or 1 when the
            // move carried a subpixel (C-edge-down-from-below).
            Axis::Down => max_by_before(bound, back),
        }
    }

    /// Whether a view at `c`, its next screen closed, is pushed back towards `floor`, its
    /// screen's edge plus the margin (Spec: ClosedScreens, Settling).
    const fn past_floor(&self, c: Px, floor: Px) -> bool {
        match self.axis {
            // Across there is no such test: the margin is none, and the view is pushed back.
            Axis::Across => true,
            // Down only a top past the floor is: one within the margin of its screen's edge stays
            // where it is (C-down-within-margin, C-left-edge-keeps-subpixel).
            Axis::Down => exceeds(c, floor),
        }
    }

    // The steps, the same on both axes.

    /// The margin at the screen holding `p`: none on a short screen, the line's margin on any
    /// other (Spec: ShortScreens).
    fn margin_at(&self, p: Px) -> Px {
        if self.screen(p) == ScreenKind::Short {
            NO_MARGIN
        } else {
            self.margin
        }
    }

    /// The push forward from `p`: p + v + 2, the spec's ahead (Spec: ClosedScreens).
    ///
    /// Spec: PushCarries. Reproduces the original's behaviour: the sum p + v carries once more
    /// when it passes FFFF. That shows only when the push sums from left of or above the room:
    /// from a camera another domain left there (C-push-carries), or from where a move left or up
    /// took it past the room's near edge before the aim stopped it (C-push-carries-move).
    fn push_ahead(&self, p: Px) -> Px {
        let (sum, carried) = p.0.overflowing_add(self.whole.0);
        Wrapping(sum) + Wrapping(u16::from(carried)) + self.rules.push
    }

    /// The push back from `p`: p − v − 2, the spec's back (Spec: ClosedScreens).
    ///
    /// It has no carry, as the push forward has (Spec: PushCarries): its own would take 1 more
    /// when p − v goes below 0. That needs `p` under the speed, at the room's first pixels, and
    /// every such push back lands before the edge it is held at, so the `max` that holds it
    /// keeps the edge and the pixel more would never show.
    fn push_back(&self, p: Px) -> Px {
        p - self.whole - self.rules.push
    }

    /// Whether the screen after the one holding `p` is closed.
    fn next_closed(&self, p: Px) -> bool {
        self.screen(p + SCREEN) == ScreenKind::Closed
    }

    /// Where a move ends once the room limits it, measured from where the aim stopped it (Spec:
    /// §3).
    fn limit(&self, m: Move) -> Px {
        match m.going {
            Going::Back => self.limit_back(m),
            Going::Forward => self.limit_forward(m),
        }
    }

    /// Where a move left or up ends (Spec: RoomEdges, ClosedScreens).
    fn limit_back(&self, m: Move) -> Px {
        let c = m.stopped.pixel();
        // Before the room's near edge it stops on it, and no closed screen moves it: reached by a
        // move left or up past the room's first pixel (C-aim-above-top, C-left-past-edge,
        // C-rise-into-corner, C-aim-above-top-right-edge, C-left-edge-keeps-subpixel).
        if before(c, ROOM_START) {
            return ROOM_START;
        }
        // In a closed screen it is pushed forward from where the move took it, but no further
        // than the next screen's edge, the min as plain numbers: a camera that moved onto its aim
        // from 8000 away pushes to 8100, which by `before` would be kept (C-push-left-far).
        if self.screen(c) == ScreenKind::Closed {
            return min_by_exceeds(next_edge(c), self.push_ahead(m.reached));
        }
        c
    }

    /// Where a move right or down ends (Spec: RoomEdges, ClosedScreens). Past the far bound it
    /// stops as the axis decides, and no closed screen moves it. With the next screen closed, and
    /// past the floor where the axis tests it, it is pushed back from where the move took it, but
    /// not before the floor, its screen's edge plus the margin.
    fn limit_forward(&self, m: Move) -> Px {
        let c = m.stopped.pixel();
        let looked = self.looks_from(m);
        let margin = self.margin_at(looked);
        let back = self.push_back(m.reached);
        let bound = self.far + margin;
        if exceeds(c, bound) {
            return self.stop_past(bound, back);
        }
        let floor = edge(c) + margin;
        if self.next_closed(looked) && self.past_floor(c, floor) {
            max_by_before(floor, back)
        } else {
            c
        }
    }

    /// Where the view settles on an axis she didn't move on, from its pixel part as the frame
    /// found it (Spec: Settling).
    ///
    /// The margin is read from the screen it was found in (Spec: ShortScreens). It is first
    /// brought into the room, between 0 and its far bound: a camera another domain left left of
    /// the room is brought to 0 (C-settle-reach-from-left, C-push-carries), one right of it or
    /// below it to its far bound (C-settle-back-right-of-room, C-settle-back-far-right,
    /// C-settle-up-from-below-short); down, the bound itself moves when the screen its top is in
    /// became short (C-settle-into-room-down). Then it eases out of a closed screen, or back from
    /// a closed next screen towards the floor, its screen's edge plus the margin.
    fn settle(&self) -> Px {
        let found = self.camera.pixel();
        let margin = self.margin_at(found);
        let c = min_by_exceeds(max_by_before(found, ROOM_START), self.far + margin);
        let floor = edge(c) + margin;
        if self.screen(c) == ScreenKind::Closed {
            self.ease_out(c, found)
        } else if self.next_closed(c) && self.past_floor(c, floor) {
            self.ease_back(floor, found)
        } else {
            c
        }
    }

    /// Where the view in a closed screen settles (Spec: Settling), from `c`, where it was
    /// brought into the room, and `found`, where the frame found it.
    ///
    /// It eases forward by the push from where it was found. She didn't move, so the speed is the
    /// extra pixel and what she moved inside a pixel, its pixel part 1 (or 0 after a creep, Spec:
    /// CreepReadsRightward), and the push is that and 2 more: 3 px a frame, or 2. An ease that
    /// the next screen's edge doesn't exceed, one that is the edge or exceeds it, stops on it:
    /// from a camera left of the room, `FFF0` eases to `FFF3`, which exceeds `0100`
    /// (C-settle-reach-from-left).
    /// An ease whose next screen is closed too drops to the edge of the screen it is in.
    fn ease_out(&self, c: Px, found: Px) -> Px {
        let eased = self.push_ahead(found);
        let next = next_edge(c);
        if !exceeds(next, eased) {
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
    fn ease_back(&self, floor: Px, found: Px) -> Px {
        let pushed = self.push_back(found);
        if before(floor, pushed) && self.screen(pushed) == ScreenKind::Closed {
            next_edge(pushed)
        } else {
            max_by_before(floor, pushed)
        }
    }
}
