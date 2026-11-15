//! The camera model, built from its behaviour spec alone: the screen's view following Samus.
//!
//! A follow camera with a lead (Spec: FollowingHer, LeadingHer, AimingHeight). On each axis
//! where Samus's pixel part changed, the view moves towards its aim (her position less the lead
//! across and the aim height down) as fast as she moved plus a pixel. If that aim is the view's
//! own pixel, the view stays unchanged, subpixel included, without settling. When her pixel
//! part did not change on an axis, that axis settles instead, unless time is stopped
//! (Spec: Settling, TimeStopped).
//!
//! Following pushes a view entering a closed screen towards its edge, backing off against her
//! move by 2 px per frame in the reference profile, or 1 when the move carried a subpixel.
//! A settling view may partly show a closed screen while easing out (Spec: ClosedScreens).
//! Room-edge corrections apply in the direction of following; settling first brings the view
//! into the room. Subsequent screen corrections can take it outside again, as ScreensRunOn
//! permits. A view another domain left outside returns according to the directional rules:
//! at once (sometimes to the far edge), or a few pixels per frame (Spec: RoomEdges).
//!
//! A caller builds a `Room` from its run of screens, each the shared `types::ScreenKind`,
//! and its size across and down, places a
//! `Camera` with `Camera::initialise` for fresh Samus or `Camera::restore` for retained state,
//! and steps it once an ordinary following frame with `Camera::step`, reading
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
use core::num::Wrapping;
use follow::{aim, distance};
use types::{Fixed, Pair, Parts, Position, VerticalFraction, fixed};

/// The view's top-left corner, distinct from the remembered centre (Spec: RoomEdges).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ViewPosition(pub Pair<Fixed>);

/// Where the camera last saw her, not the view's position (Spec: Remembering).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct RememberedPosition(pub Position);

/// Persistent camera state: the view, the remembered centre, and the next shot's retained
/// height fraction (Spec: Remembering, RememberedHeightFraction).
///
/// It can be checkpointed with serde; all three quantities are saved, and restored without fresh initialization.
///
/// A frame supplies the rest (Spec: §1.3, `Frame`): her motion's outputs (her position, the y a
/// placement put her at, if any, her facing, whether she goes backwards and whether she rises),
/// the room, the lead setting and whether time is stopped.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    /// The view's top-left corner in the room.
    position: Pair<Fixed>,
    /// Where it last saw her (Spec: Remembering).
    seen: Position,
    /// The receipt weapons read, independent of view motion (Spec: RememberedHeightFraction).
    vertical_fraction: VerticalFraction,
}

impl Camera {
    /// Restore all persistent state at any allowed values (Spec: RememberedHeightFraction,
    /// Remembering). Restoring or placing positions does not initialize the receipt.
    #[must_use]
    pub const fn restore(
        view: ViewPosition,
        remembered: RememberedPosition,
        vertical_fraction: VerticalFraction,
    ) -> Self {
        Self {
            position: view.0,
            seen: remembered.0,
            vertical_fraction,
        }
    }

    /// Fresh Samus initialization explicitly starts the receipt at zero
    /// (Spec: RememberedHeightFraction). This is not ordinary view placement.
    #[must_use]
    pub const fn initialise(view: ViewPosition, remembered: RememberedPosition) -> Self {
        Self::restore(view, remembered, VerticalFraction::ZERO)
    }

    /// Place the view without ordinary following, retaining the remembered centre and receipt
    /// (Spec: RememberedHeightFraction). Other domains decide their own view movement.
    #[must_use]
    pub const fn with_view(mut self, view: ViewPosition) -> Self {
        self.position = view.0;
        self
    }

    /// Replace the remembered centre without initializing the receipt (Spec: Remembering,
    /// RememberedHeightFraction).
    #[must_use]
    pub const fn with_remembered(mut self, remembered: RememberedPosition) -> Self {
        self.seen = remembered.0;
        self
    }

    /// The typed receipt available to the next shot; reading it does not clear it
    /// (Spec: RememberedHeightFraction). A frame omitting `step` retains it.
    #[must_use]
    pub const fn vertical_fraction(&self) -> VerticalFraction {
        self.vertical_fraction
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
            y: frame.samus.placed.map_or(self.seen.y, |placed| {
                fixed(Wrapping(placed.0), self.seen.y.sub())
            }),
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
            // Equal pixels use her minus remembered, even for an upward creep. Measure before
            // remembering her current centre, without the extra following speed; wrapping borrow
            // leaves the fractional receipt unchanged by room limits or stopped time.
            vertical_fraction: VerticalFraction(distance(her.y, seen.y).sub()),
        }
    }
}
