//! §3 Taking another pose: where the clip of the pose she takes starts.

use clean_types::{Action, Aim, Body, LandedFrom, Turn};

use crate::clip::{Frames, Step};
use crate::inputs::{Content, Inputs, Move};
use crate::state::Playhead;

/// The actions a side of a row names.
enum Actions {
    /// Any action.
    Any,
    /// One of these.
    OneOf(&'static [Action]),
    /// Any action but these.
    AllBut(&'static [Action]),
}

/// One side of a row of `StartingClip`'s table: the poses it names. A part it leaves `None`
/// is any.
struct Poses {
    body: Option<Body>,
    actions: Actions,
    aim: Option<Aim>,
}

impl Poses {
    /// Whether `pose` is one of these.
    fn names(&self, pose: clean_types::Pose) -> bool {
        let action = match self.actions {
            Actions::Any => true,
            Actions::OneOf(actions) => actions.contains(&pose.action),
            Actions::AllBut(actions) => !actions.contains(&pose.action),
        };
        action
            && self.body.is_none_or(|body| body == pose.body)
            && self.aim.is_none_or(|aim| aim == pose.aim)
    }
}

/// Where the clip of the pose she takes starts.
#[derive(Clone, Copy)]
enum Start {
    /// Her step and the time left stay as they are.
    Kept,
    /// From this step.
    At(Step),
}

/// A row of `StartingClip`'s table: she takes a pose of `takes` from one of `from`.
struct Row {
    takes: Poses,
    from: Poses,
    starts: Start,
}

/// Every turn, which doesn't count as aimed up (`StartingClip`).
const TURNS: &[Action] = &[
    Action::Turning(Turn::Fall),
    Action::Turning(Turn::Ground),
    Action::Turning(Turn::Jump),
    Action::Turning(Turn::MoonwalkJump),
];

/// The ball's actions whose clip goes on from one to another; `Morphing` is not one.
const BALL_GOES_ON: &[Action] = &[Action::Still, Action::Rolling, Action::Falling];

/// Her run, in any body and aim.
const RUNNING: Poses = Poses {
    body: None,
    actions: Actions::OneOf(&[Action::Running]),
    aim: None,
};

/// The ball, still, rolling or falling.
const BALL: Poses = Poses {
    body: Some(Body::Ball),
    actions: Actions::OneOf(BALL_GOES_ON),
    aim: None,
};

/// The spring ball, any action.
const SPRING_BALL: Poses = Poses {
    body: Some(Body::SpringBall),
    actions: Actions::Any,
    aim: None,
};

/// Spec: `StartingClip`'s table. A pair in none of its rows starts from step 0.
const TABLE: [Row; 7] = [
    Row {
        takes: RUNNING,
        from: RUNNING,
        starts: Start::Kept,
    },
    Row {
        takes: BALL,
        from: BALL,
        starts: Start::Kept,
    },
    Row {
        takes: SPRING_BALL,
        from: SPRING_BALL,
        starts: Start::Kept,
    },
    // The spin goes on past its first step, the jump from the floor.
    Row {
        takes: Poses {
            body: None,
            actions: Actions::OneOf(&[Action::Spinning]),
            aim: None,
        },
        from: Poses {
            body: None,
            actions: Actions::OneOf(&[Action::Spinning, Action::WallJumping]),
            aim: None,
        },
        starts: Start::At(Step::SECOND),
    },
    // Her arm is already up.
    Row {
        takes: Poses {
            body: Some(Body::Upright),
            actions: Actions::OneOf(&[
                Action::Still,
                Action::Landing(LandedFrom::Jump),
                Action::Landing(LandedFrom::Spin),
            ]),
            aim: Some(Aim::Up),
        },
        from: Poses {
            body: None,
            actions: Actions::AllBut(TURNS),
            aim: Some(Aim::Up),
        },
        starts: Start::At(Step::SECOND),
    },
    Row {
        takes: Poses {
            body: None,
            actions: Actions::OneOf(&[Action::Jumping]),
            aim: Some(Aim::Up),
        },
        from: Poses {
            body: None,
            actions: Actions::OneOf(&[Action::TakingOff]),
            aim: Some(Aim::Up),
        },
        starts: Start::At(Step::SECOND),
    },
    Row {
        takes: Poses {
            body: Some(Body::Crouched),
            actions: Actions::OneOf(&[Action::Still]),
            aim: Some(Aim::Up),
        },
        from: Poses {
            body: None,
            actions: Actions::OneOf(&[Action::Crouching]),
            aim: Some(Aim::Up),
        },
        starts: Start::At(Step::SECOND),
    },
];

/// Spec: `StartingClip`. When she takes another pose, its clip starts where the table says: kept
/// as it is, or from a step with its hold plus the start's slowdown (and `HandOverSlowdown`).
#[must_use]
pub fn start(playhead: Playhead, inputs: &Inputs, content: &impl Content) -> Playhead {
    let (from, takes) = (&inputs.pose, &inputs.pose_after);
    if from == takes {
        return playhead;
    }
    let row = TABLE
        .iter()
        .find(|row| row.takes.names(takes.motion) && row.from.names(from.motion));
    let step = match row.map_or(Start::At(Step::FIRST), |row| row.starts) {
        Start::Kept => return playhead,
        Start::At(step) => step,
    };
    let hold = content.clip(takes).hold(step);
    Playhead {
        step,
        left: hold
            .plus(inputs.liquid.start_slowdown)
            .plus(hand_over_slowdown(inputs)),
    }
}

/// Spec: `HandOverSlowdown` (a quirk). The first step of a pose her clip's end handed over is
/// slowed by the clip's slowdown as well as the start's; any other pose by the start's alone.
fn hand_over_slowdown(inputs: &Inputs) -> Frames {
    if inputs.moves[Move::HandedOver] {
        inputs.liquid.slowdown
    } else {
        Frames::NONE
    }
}
