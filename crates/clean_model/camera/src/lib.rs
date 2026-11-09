//! The camera model, built from its behaviour spec alone: the screen's view following Samus.
//!
//! A follow camera with a lead. Each frame, on each axis, the view moves towards its aim (her
//! position less the lead across and the aim height down) as fast as she moved plus a pixel,
//! then the room limits it: its edges and its closed screens. On an axis she didn't move on it
//! settles out of a closed screen instead, unless time is stopped.
//!
//! A caller builds a `Room` from its run of screens, its width and its height, places a `Camera`
//! with `Camera::new`, and steps it once a frame with `Camera::step`, reading `Camera::position`. `Rules::REFERENCE` is the original's tuning;
//! `CATALOGUE` names every behaviour.
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
pub use room::{Heights, Room, RoomError, ScreenKind};
pub use rules::{Leads, Rules};

use axis::Line;
use clean_types::{Fixed, Pair, Parts, Position};
use follow::aim;

/// The camera: where the view is and where it last saw her, all a later frame needs (Spec: §1).
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
        // Spec: Remembering. Before it follows, the pixel part of where it last saw her down
        // becomes camera_from's, its subpixel part kept.
        let seen = Pair {
            y: frame.samus.camera_from.with_sub(self.seen.y.sub()),
            ..self.seen
        };
        let aim = aim(rules, frame);
        let Pair { x, y } = self.position;
        // Spec: §4. Across first, in the row its middle was in; then down, in the column the
        // move across took it to.
        let x = Line::across(frame, rules, y.pixel()).follow(x, her.x, seen.x, aim.x);
        let y = Line::down(frame, rules, x.pixel()).follow(y, her.y, seen.y, aim.y);
        // Spec: Remembering. After both axes, where it last saw her becomes her position.
        Self {
            position: Pair { x, y },
            seen: her,
        }
    }
}
