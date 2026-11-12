//! The camera model, built from its behaviour spec alone: the screen's view following Samus.
//!
//! A follow camera with a lead. Each frame, on each axis, the view moves towards its aim (her
//! position less the lead across and the aim height down) as fast as she moved plus a pixel,
//! then the room limits it: its edges and its closed screens. On an axis she didn't move on it
//! settles out of a closed screen instead, unless time is stopped. A frame whose aim is the
//! view's own pixel neither moves nor settles.
//!
//! A caller builds a `Room` from its run of screens, each the shared `types::ScreenKind`,
//! and its size across and down, places a
//! `Camera` with `Camera::new`, and steps it once a frame with `Camera::step`, reading
//! `Camera::position`. `Rules::REFERENCE` is the original's tuning; `CATALOGUE` names every
//! behaviour.
#![expect(
    clippy::doc_markdown,
    reason = "the docs name a spec entry in plain words, as Spec: Name, and keep backticks for \
              Rust items"
)]
#![expect(
    clippy::redundant_pub_crate,
    reason = "an item only the crate's other modules use is marked pub(crate), so the crate's \
              public interface is only what lib.rs exports"
)]

mod axis;
mod catalogue;
mod follow;
mod input;
mod room;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use input::{Frame, LeadSetting, Samus};
pub use room::{Heights, Room, RoomError};
pub use rules::{Leads, Rules};

use axis::{Axis, Line};
use follow::aim;
use types::{Fixed, Pair, Parts, Position};

/// The camera: where the view is and where it last saw her, all a later frame needs (Spec: §1).
///
/// A frame supplies the rest (Spec: §1.3, `Frame`): her motion's outputs (her position, the y a
/// placement put her at, if any, her facing, whether she goes backwards and whether she rises),
/// the room, the lead setting and whether time is stopped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Camera {
    /// The view's top-left corner in the room.
    position: Pair<Fixed>,
    /// Where it last saw her (Spec: Remembering).
    seen: Position,
}

impl Camera {
    /// A camera with its view at `position`, having last seen her at `her`, her position as the
    /// view is placed. Placing the view is the caller's: a door's passage places it in
    /// the next room (Spec: §5).
    #[must_use]
    pub const fn new(position: Pair<Fixed>, her: Position) -> Self {
        Self {
            position,
            seen: her,
        }
    }

    /// The view's top-left corner in the room: where the room is drawn on the screen.
    #[must_use]
    pub const fn position(&self) -> Pair<Fixed> {
        self.position
    }

    /// Where it last saw her, which its next move is measured from (Spec: Remembering).
    #[must_use]
    pub const fn seen(&self) -> Position {
        self.seen
    }

    /// The camera after one frame of following her.
    #[must_use]
    pub fn step(self, rules: &Rules, frame: &Frame) -> Self {
        let her = frame.samus.position;
        // Spec: Remembering. Before it follows, a placement replaces the pixel part of where it
        // last saw her down with the placed y's, its subpixel part kept.
        let seen = Pair {
            y: frame
                .samus
                .placed
                .map_or(self.seen.y, |placed| placed.with_sub(self.seen.y.sub())),
            ..self.seen
        };
        let aim = aim(rules, frame);
        // Spec: §4. Across first, in the row holding the point 128 px below its top edge before it
        // moves down; then down, in the column holding the point 128 px right of its left edge
        // after it moved across.
        let x = Line::new(frame, rules, Axis::Across, seen, self.position).follow(aim.x);
        let across = Pair { x, ..self.position };
        let y = Line::new(frame, rules, Axis::Down, seen, across).follow(aim.y);
        // Spec: Remembering. After both axes, where it last saw her becomes her position.
        Self {
            position: Pair { x, y },
            seen: her,
        }
    }
}
