//! Keeping the view in the room (Spec: §3): its edges, its closed and short screens, and how the
//! view settles out of a closed screen, with one type for each axis, `Across` and `Down`.
//!
//! Every rule here sets the camera's pixel part only; its subpixel part is the follow's.

use core::num::Wrapping;

use clean_types::{Fixed, Parts, Px};

use crate::follow::{Going, Move, before, exceeds, later, smaller};
use crate::room::{Room, SCREEN, ScreenKind};
use crate::rules::Rules;

/// Notation: the pixels within a screen, a position's low byte.
const WITHIN_SCREEN: Px = Wrapping(0xFF);
/// How far past the view's near edge on the other axis the screens are read: 128 px (Spec:
/// ClosedScreens). Across, they are read in the row holding the view's top plus this; down, in
/// the column holding its left edge plus this.
const MIDDLE: Px = Wrapping(128);
/// The room's left edge and its top edge, both position 0 (Spec: RoomEdges).
const ROOM_START: Px = Wrapping(0);
/// The rows on a short screen: none, so the view stops with its top on the screen's top edge
/// (Spec: ShortScreens).
const NO_ROWS: Px = Wrapping(0);

/// The near edge of the screen holding `p`: p less p mod 256 (Spec: ClosedScreens, edge).
fn edge(p: Px) -> Px {
    p & !WITHIN_SCREEN
}

/// The near edge of the screen after the one holding `p`.
fn next_edge(p: Px) -> Px {
    edge(p) + SCREEN
}

/// The push that moves the view out of a closed screen: the speed's pixel part and 2 px more
/// (Spec: ClosedScreens).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Push {
    /// The speed's pixel part (the spec's v).
    speed: Px,
    /// How far it pushes beyond the speed.
    beyond: Px,
}

impl Push {
    /// The push of a frame whose speed on the axis is `speed`.
    pub(crate) fn new(speed: Fixed, rules: &Rules) -> Self {
        Self {
            speed: speed.pixel(),
            beyond: rules.push,
        }
    }

    /// The push forward from `p`: p + v + 2 (the spec's ahead), its sum carrying once more.
    fn ahead(self, p: Px) -> Px {
        push_carries(p, self.speed) + self.beyond
    }

    /// The push back from `p`: p − v − 2 (the spec's back).
    fn back(self, p: Px) -> Px {
        p - self.speed - self.beyond
    }
}

/// Spec: PushCarries. The sum p + v of a push forward, and 1 more when it passes FFFF.
///
/// Reproduces the original's behaviour: a push's sum carries once more. It shows only when the
/// push sums from left of or above the room.
fn push_carries(p: Px, v: Px) -> Px {
    let (sum, carried) = p.0.overflowing_add(v.0);
    Wrapping(sum) + Wrapping(u16::from(carried))
}

/// Spec: DownwardLooksAhead. Where a move down reads the next screen and its rows from: the
/// position the move took the view to, before the aim stopped it.
///
/// Reproduces the original's behaviour: a move down that crosses into the next screen reads that
/// screen's next, and can back the view up past its aim.
const fn downward_looks_ahead(m: Move) -> Px {
    m.reached
}

/// One axis of the room as the camera meets it: the screens it reads there, the push it moves by,
/// and the rules that keep it in the room.
pub(crate) trait Line {
    /// The kind of the screen holding position `p` on this axis.
    fn screen(&self, p: Px) -> ScreenKind;

    /// This frame's push on this axis.
    fn push(&self) -> Push;

    /// Where a move right or down ends, from where the aim stopped it (Spec: RoomEdges,
    /// ClosedScreens).
    fn limit_forward(&self, m: Move) -> Px;

    /// Where the view settles on an axis she didn't move on, from `found`, its pixel part as the
    /// frame found it (Spec: Settling).
    fn settle(&self, found: Px) -> Px;

    /// Where a move ends once the room limits it, measured from where the aim stopped it (Spec:
    /// §3).
    fn limit(&self, m: Move) -> Px {
        match m.going {
            Going::Back => self.limit_back(m),
            Going::Forward => self.limit_forward(m),
        }
    }

    /// Whether the screen after the one holding `p` is closed.
    fn next_closed(&self, p: Px) -> bool {
        self.screen(p + SCREEN).is_closed()
    }

    /// Where a move left or up ends. Before the room's edge, it stops on it, and no closed
    /// screen moves it (Spec: RoomEdges). In a closed screen, it is pushed forward from where
    /// the move took it, but no further than the next screen's edge, the smaller of the two as
    /// plain numbers (Spec: ClosedScreens).
    fn limit_back(&self, m: Move) -> Px {
        let stopped = m.stopped.pixel();
        if before(stopped, ROOM_START) {
            return ROOM_START;
        }
        if self.screen(stopped).is_closed() {
            return smaller(next_edge(stopped), self.push().ahead(m.reached));
        }
        stopped
    }

    /// Where the view settles when it is in a closed screen (Spec: Settling), from `clamped`,
    /// where it was brought into the room, and `found`, where the frame found it. It eases
    /// forward from where it was found; an ease that reaches the next screen's edge stops on
    /// it, and one whose next screen is closed too drops to the edge of the screen it is in.
    fn ease_out(&self, clamped: Px, found: Px) -> Px {
        let eased = self.push().ahead(found);
        let next = next_edge(clamped);
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
    /// is the later and lands in a closed screen, it goes on to that screen's next edge.
    fn ease_back(&self, floor: Px, found: Px) -> Px {
        let pushed = self.push().back(found);
        if before(floor, pushed) && self.screen(pushed).is_closed() {
            next_edge(pushed)
        } else {
            later(floor, pushed)
        }
    }
}

/// The room across, one frame: the screens are read in the row holding the view's middle.
pub(crate) struct Across<'a> {
    /// The room.
    room: &'a Room,
    /// The position down whose row the screens are read in.
    row: Px,
    /// This frame's push across.
    push: Push,
}

impl<'a> Across<'a> {
    /// The room across, for a camera whose pixel part down is `camera_y` before it moves down.
    pub(crate) fn new(room: &'a Room, camera_y: Px, push: Push) -> Self {
        Self {
            room,
            row: camera_y + MIDDLE,
            push,
        }
    }
}

impl Line for Across<'_> {
    fn screen(&self, x: Px) -> ScreenKind {
        self.room.screen(x, self.row)
    }

    fn push(&self) -> Push {
        self.push
    }

    /// Past the room's right edge, as plain numbers, it stops on it, and no closed screen moves
    /// it (Spec: RoomEdges). With the next screen closed, it is pushed back from where the move
    /// took it, but not before its own screen's edge (Spec: ClosedScreens).
    fn limit_forward(&self, m: Move) -> Px {
        let stopped = m.stopped.pixel();
        let right_edge = self.room.right_edge();
        if exceeds(stopped, right_edge) {
            right_edge
        } else if self.next_closed(stopped) {
            later(edge(stopped), self.push.back(m.reached))
        } else {
            stopped
        }
    }

    /// First brought into the room, between 0 and its right edge; then it eases out of a closed
    /// screen, or back from a closed next screen to its own screen's edge (Spec: Settling).
    fn settle(&self, found: Px) -> Px {
        let clamped = smaller(later(found, ROOM_START), self.room.right_edge());
        match self.screen(clamped) {
            ScreenKind::Closed => self.ease_out(clamped, found),
            ScreenKind::Open | ScreenKind::Short if self.next_closed(clamped) => {
                self.ease_back(edge(clamped), found)
            }
            ScreenKind::Open | ScreenKind::Short => clamped,
        }
    }
}

/// The room down, one frame: the screens are read in the column holding the view's middle.
pub(crate) struct Down<'a> {
    /// The room.
    room: &'a Room,
    /// The position across whose column the screens are read in.
    column: Px,
    /// This frame's push down.
    push: Push,
    /// The rows a screen other than a short one gives (Spec: ShortScreens).
    short_rows: Px,
}

impl<'a> Down<'a> {
    /// The room down, for a camera whose pixel part across is `camera_x` after it moved across.
    pub(crate) fn new(room: &'a Room, rules: &Rules, camera_x: Px, push: Push) -> Self {
        Self {
            room,
            column: camera_x + MIDDLE,
            push,
            short_rows: rules.short_rows,
        }
    }

    /// How far below its screen's top edge the view stops where it can go no lower, by the
    /// screen holding `p`: none on a short screen, the rule's short rows on any other (Spec:
    /// ShortScreens).
    fn rows(&self, p: Px) -> Px {
        if self.screen(p).is_short() {
            NO_ROWS
        } else {
            self.short_rows
        }
    }

    /// The furthest down the view goes, the last row's top edge plus `rows` (Spec: RoomEdges,
    /// yb).
    fn bottom(&self, rows: Px) -> Px {
        self.room.last_row_edge() + rows
    }
}

impl Line for Down<'_> {
    fn screen(&self, y: Px) -> ScreenKind {
        self.room.screen(self.column, y)
    }

    fn push(&self) -> Push {
        self.push
    }

    /// The next screen and the rows are read from where the move took it (Spec:
    /// DownwardLooksAhead). Past the room's bottom, as plain numbers, it is pushed back from
    /// there, but not above the bottom, and no closed screen moves it (Spec: RoomEdges). With the
    /// next screen closed and its top more than the rows below its screen's edge, it is pushed
    /// back the same way, but not above those rows (Spec: ClosedScreens).
    fn limit_forward(&self, m: Move) -> Px {
        let stopped = m.stopped.pixel();
        let looked = downward_looks_ahead(m);
        let rows = self.rows(looked);
        let bottom = self.bottom(rows);
        let floor = edge(stopped) + rows;
        let pushed = self.push.back(m.reached);
        if exceeds(stopped, bottom) {
            later(bottom, pushed)
        } else if self.next_closed(looked) && exceeds(stopped, floor) {
            later(floor, pushed)
        } else {
            stopped
        }
    }

    /// The rows are read from the screen its top was in as the frame found it (Spec:
    /// ShortScreens). First brought into the room, between 0 and its bottom; then it eases out
    /// of a closed screen, or, with the next screen closed and its top more than the rows below
    /// its screen's edge, back towards those rows (Spec: Settling).
    fn settle(&self, found: Px) -> Px {
        let rows = self.rows(found);
        let clamped = smaller(later(found, ROOM_START), self.bottom(rows));
        let floor = edge(clamped) + rows;
        match self.screen(clamped) {
            ScreenKind::Closed => self.ease_out(clamped, found),
            ScreenKind::Open | ScreenKind::Short
                if self.next_closed(clamped) && exceeds(clamped, floor) =>
            {
                self.ease_back(floor, found)
            }
            ScreenKind::Open | ScreenKind::Short => clamped,
        }
    }
}
