//! The camera model, built from its behaviour spec alone: the screen's view following Samus.
//!
//! A follow camera with a lead. Each frame, on each axis, the view moves towards its aim (her
//! position less the lead across and the aim height down) as fast as she moved plus a pixel,
//! then the room limits it: its edges and its closed screens. On an axis she didn't move on it
//! settles out of a closed screen instead, unless time is stopped. `Camera::step` is one frame;
//! `CATALOGUE` names every behaviour.

mod catalogue;
mod follow;
mod input;
mod limits;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use input::{Frame, Heights, LeadSetting, Room, Samus, ScreenKind};
pub use rules::{Leads, Rules};

use clean_types::{Fixed, Heading, Pair, Parts, Px, fixed};
use follow::{aim, chase, speed};
use limits::Bounds;

/// The camera's state (§1): all a later frame needs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Camera {
    /// The camera position, the view's top-left corner in the room: the step's output.
    pub position: Pair<Fixed>,
    /// `Remembering`: where it last saw her.
    pub seen: Pair<Fixed>,
}

impl Camera {
    /// One frame: the camera after it follows her, which reports its position (§1).
    #[must_use]
    pub fn step(self, rules: &Rules, frame: &Frame) -> Self {
        let samus = &frame.samus;
        // Remembering: before it follows, the pixel part of where it last saw her down becomes
        // `camera_from`'s, its subpixel part kept.
        let seen = Pair {
            y: samus.camera_from.with_sub(self.seen.y.sub()),
            ..self.seen
        };
        let turn = Turn {
            rules,
            frame,
            seen,
            aim: aim(rules, frame),
        };
        // §4: across first, from the row its middle is in; then down, from the column the move
        // across took it to.
        let x = turn.axis(Heading::Right, self.position);
        let y = turn.axis(Heading::Down, Pair { x, ..self.position });
        // Remembering: after both axes, where it last saw her becomes her position.
        Self {
            position: Pair { x, y },
            seen: samus.position,
        }
    }
}

/// One frame's readings, shared by its two axes.
struct Turn<'a> {
    rules: &'a Rules,
    frame: &'a Frame<'a>,
    /// Where it last saw her, after the placement.
    seen: Pair<Fixed>,
    /// The aim, across and down.
    aim: Pair<Px>,
}

impl Turn<'_> {
    /// The camera's position on one axis, by its forward heading, after the frame.
    fn axis(&self, heading: Heading, camera: Pair<Fixed>) -> Fixed {
        let (c, across) = camera.along(heading);
        let (her, _) = self.frame.samus.position.along(heading);
        let (seen, _) = self.seen.along(heading);
        let (aim, _) = self.aim.along(heading);
        let speed = speed(her, seen, self.rules.extra);
        let bounds = Bounds {
            rules: self.rules,
            room: self.frame.room,
            heading,
            across: across.pixel(),
            v: speed.pixel(),
        };
        // FollowingHer: she moved on the axis when her pixel part isn't where it last saw her.
        if her.pixel() != seen.pixel() {
            return chase(c, aim, speed).map_or(c, |m| fixed(bounds.limit(m), m.to.sub()));
        }
        // TimeStopped: settling is skipped. Settling sets the pixel part only.
        if self.frame.time_stopped {
            c
        } else {
            fixed(bounds.settle(c.pixel()), c.sub())
        }
    }
}
