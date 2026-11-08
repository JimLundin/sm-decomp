//! The camera model, built from its behaviour spec alone: the screen's view following Samus.
//!
//! A follow camera with a lead. Each frame, on each axis, the view moves towards its aim (her
//! position less the lead across and the aim height down) as fast as she moved plus a pixel,
//! then the room limits it: its edges and its closed screens. On an axis she didn't move on it
//! settles out of a closed screen instead, unless time is stopped.
//!
//! A caller builds a `Room`, places a `Camera` with `Camera::new`, and steps it once a frame with
//! `Camera::step`, reading `Camera::position`. `Rules::REFERENCE` is the original's tuning;
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

mod catalogue;
mod follow;
mod input;
mod limits;
mod room;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use input::{Frame, LeadSetting, Samus};
pub use room::{Heights, Room, RoomError, ScreenKind};
pub use rules::{Leads, Rules};

use clean_types::{Fixed, Pair, Parts, Position, Px, fixed};
use follow::{aim, chase, speed};
use limits::{Across, Down, Line, Push};

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
        let samus = &frame.samus;
        // Spec: Remembering. Before it follows, the pixel part of where it last saw her down
        // becomes camera_from's, its subpixel part kept.
        let seen = Pair {
            y: samus.camera_from.with_sub(self.seen.y.sub()),
            ..self.seen
        };
        let following = Following {
            rules,
            frame,
            seen,
            aim: aim(rules, frame),
        };
        // Spec: §4. Across first, in the row its middle was in; then down, in the column the
        // move across took it to.
        let x = following.across(self.position);
        let y = following.down(Pair { x, ..self.position });
        // Spec: Remembering. After both axes, where it last saw her becomes her position.
        Self {
            position: Pair { x, y },
            seen: samus.position,
        }
    }
}

/// What one frame's following reads on both axes.
struct Following<'a> {
    /// The tuning.
    rules: &'a Rules,
    /// The frame's inputs.
    frame: &'a Frame<'a>,
    /// Where it last saw her, after the placement down.
    seen: Position,
    /// The point it aims at.
    aim: Pair<Px>,
}

impl Following<'_> {
    /// The camera's position across after the frame.
    fn across(&self, camera: Pair<Fixed>) -> Fixed {
        let axis = OnAxis {
            camera: camera.x,
            her: self.frame.samus.position.x,
            seen: self.seen.x,
            aim: self.aim.x,
            speed: speed(self.frame.samus.position.x, self.seen.x, self.rules.extra),
        };
        let line = Across::new(self.frame.room, camera.y.pixel(), axis.push(self.rules));
        axis.follow(&line, self.frame.time_stopped)
    }

    /// The camera's position down after the frame, from where it is after moving across.
    fn down(&self, camera: Pair<Fixed>) -> Fixed {
        let axis = OnAxis {
            camera: camera.y,
            her: self.frame.samus.position.y,
            seen: self.seen.y,
            aim: self.aim.y,
            speed: speed(self.frame.samus.position.y, self.seen.y, self.rules.extra),
        };
        let push = axis.push(self.rules);
        let line = Down::new(self.frame.room, self.rules, camera.x.pixel(), push);
        axis.follow(&line, self.frame.time_stopped)
    }
}

/// One axis of a frame: the camera, her, where it last saw her, its aim and its speed there.
#[derive(Clone, Copy)]
struct OnAxis {
    /// The camera's position on the axis.
    camera: Fixed,
    /// Her position on the axis.
    her: Fixed,
    /// Where it last saw her on the axis.
    seen: Fixed,
    /// Its aim's pixel part on the axis.
    aim: Px,
    /// Its speed on the axis (Spec: FollowingHer).
    speed: Fixed,
}

impl OnAxis {
    /// Whether she moved on the axis: her pixel part is not where it last saw her (Spec:
    /// FollowingHer).
    fn she_moved(self) -> bool {
        self.her.pixel() != self.seen.pixel()
    }

    /// The push on the axis, from its speed (Spec: ClosedScreens).
    fn push(self, rules: &Rules) -> Push {
        Push::new(self.speed, rules)
    }

    /// The camera's position on the axis after the frame. When she moved, it moves towards its
    /// aim and the room limits it (Spec: FollowingHer, §3); when she didn't, it settles, unless
    /// time is stopped (Spec: Settling, TimeStopped). The room's rules set its pixel part only.
    fn follow(self, line: &impl Line, time_stopped: bool) -> Fixed {
        if self.she_moved() {
            return chase(self.camera, self.aim, self.speed)
                .map_or(self.camera, |m| fixed(line.limit(m), m.stopped.sub()));
        }
        if time_stopped {
            self.camera
        } else {
            fixed(line.settle(self.camera.pixel()), self.camera.sub())
        }
    }
}
