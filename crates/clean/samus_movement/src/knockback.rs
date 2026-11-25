//! §12 Thrown back by a hit: what she remembers of a knockback (its push time left, its side
//! and the push under way), and which poses take a push.
//!
//! A hit (`types::Hit`) gives its push time and side (`PushTime`); with push time left and no
//! push under way, a pose that admits one starts it (`PushStarts`); the push throws her up or
//! drives her down (`PushMoves`, `moves`), and the first frame she starts with no push time left
//! ends it (`PushEnds`). Her motion itself stays in her `Run` and `Vertical`.

use types::{Action, Aim, Body, Facing, Gun, Hit, Legs, Pose, PushFrames, Side};

/// `PushStarts`, `PushMoves`, `PushEnds`: a push under way, by the way it throws her. With no
/// push under way there is none (`Knockback::push`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Push {
    /// Thrown upward and clear: she moves by her vertical speed, a rise the pull slows.
    Up,
    /// Driven down and clear along the ground: the ground pull moves her, her vertical speed
    /// kept.
    Down,
}

/// §12 what she remembers of a knockback: the push time left, the side the hit pushes her to,
/// and the push under way.
///
/// State a caller gives and reads back. A frame's hit gives her a fresh push length and side
/// whether or not a push is under way; what is left of it drops by one at every frame's end.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Knockback {
    /// `PushTime`: the push time left, 0 to 45 frames.
    pub time_left: PushFrames,
    /// `PushMoves`: the way the hit pushes her; in `Recoiling` her horizontal speed points so.
    pub side: Side,
    /// The push under way, if one is.
    pub push: Option<Push>,
}

/// `PushStarts`: what her pose at the frame's start does when a push is due.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recoil {
    /// Upright or crouched: the push starts, and she takes `Recoiling` in place of the pose
    /// chosen.
    Recoils,
    /// The ball, the spring ball included: the push starts, and she keeps her pose.
    KeepsHerPose,
    /// A turn, a change of shape, or a push's own poses: no push starts, and the time runs on.
    Waits,
}

impl Knockback {
    /// No push time left and no push under way. `side` is the last hit's, read only once a hit
    /// gives push time again.
    #[must_use]
    pub const fn spent(side: Side) -> Self {
        Self {
            time_left: PushFrames::NONE,
            side,
            push: None,
        }
    }

    /// `PushTime`: the frame's hit, if one comes, gives its push time and its side.
    pub(crate) const fn take(&mut self, hit: Option<Hit>) {
        if let Some(hit) = hit {
            self.time_left = hit.push;
            self.side = hit.side;
        }
    }

    /// `PushStarts`: push time left and no push under way: a push is due.
    pub(crate) fn due(self) -> bool {
        self.push.is_none() && self.time_left != PushFrames::NONE
    }

    /// `PushEnds`: a push under way with no push time left: starting a frame so, the push ends.
    pub(crate) fn ends(self) -> bool {
        self.push.is_some() && self.time_left == PushFrames::NONE
    }

    /// `PushTime`: at every frame's end, after everything else, the time goes down by 1, to no
    /// lower than 0, whether a push is under way or not.
    pub(crate) const fn count_down(&mut self) {
        self.time_left = self.time_left.less_one();
    }
}

impl Recoil {
    /// `PushStarts`: what `pose`, hers at the frame's start, does with a push due. A turn, a
    /// change of shape, `Recoiling` and `Leaping` wait; else upright or crouched she recoils, and
    /// the ball keeps its pose.
    #[must_use]
    pub const fn of(pose: Pose) -> Self {
        match pose.action {
            Action::Turning(_)
            | Action::Crouching
            | Action::Morphing
            | Action::StandingUp
            | Action::Unmorphing
            | Action::Recoiling
            | Action::Leaping => Self::Waits,
            Action::AgainstWall
            | Action::Falling
            | Action::Jumping
            | Action::Landing(_)
            | Action::Moonwalking
            | Action::Rolling
            | Action::Running
            | Action::Spinning
            | Action::Still
            | Action::TakingOff
            | Action::WallJumping => match pose.body {
                Body::Upright | Body::Crouched => Self::Recoils,
                Body::Ball | Body::SpringBall => Self::KeepsHerPose,
            },
        }
    }

    /// `PushStarts`: the push's kind: driven down where she holds forward (her facing's side,
    /// held this frame: `forward_held`) and isn't the ball, else thrown up.
    #[must_use]
    pub fn push(self, forward_held: bool) -> Push {
        if self == Self::Recoils && forward_held {
            Push::Down
        } else {
            Push::Up
        }
    }
}

/// `PushStarts`: the pose a push throws her into: upright `Recoiling`, her facing kept, aim
/// `Forward`.
#[must_use]
pub const fn recoiling(facing: Facing) -> Pose {
    Pose {
        body: Body::Upright,
        action: Action::Recoiling,
        facing,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}
