//! Taking another pose: where the clip of the pose she takes starts.

use types::{Action, Aim, Body, LandedFrom, Pose};

use crate::clip::{Clip, Slowdown, Step};
use crate::inputs::{Inputs, MoveEvent};
use crate::state::Playhead;

/// The actions a side of a row names.
enum Actions {
    /// Any action.
    Any,
    /// One of these.
    OneOf(&'static [Action]),
    /// Any action but a turn: a turn aimed up doesn't shoot, so it doesn't count as aimed up.
    AnyButATurn,
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
    fn names(&self, pose: Pose) -> bool {
        let action = match self.actions {
            Actions::Any => true,
            Actions::OneOf(actions) => actions.contains(&pose.action),
            Actions::AnyButATurn => !matches!(pose.action, Action::Turning(_)),
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

/// The ball's actions whose clip goes on from one to another.
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
            actions: Actions::AnyButATurn,
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

/// Spec: `StartingClip`. When she takes another pose and the table doesn't keep her clip going,
/// `clip_after`, the clip of that pose, starts from the step the table says, with its hold plus
/// the start slowdown (and `HandOverSlowdown`'s step slowdown). Otherwise `played`, the playhead
/// her clip at the frame's start left as it played, stays as it is.
#[must_use]
pub fn start(played: Playhead, inputs: &Inputs, clip_after: &Clip) -> Playhead {
    let (from, takes) = (&inputs.figure, &inputs.figure_after);
    if from == takes {
        return played;
    }
    let row = TABLE
        .iter()
        .find(|row| row.takes.names(takes.pose) && row.from.names(from.pose));
    let Start::At(step) = row.map_or(Start::At(Step::FIRST), |row| row.starts) else {
        return played;
    };
    let slowdowns = [inputs.liquid.start_slowdown, hand_over_slowdown(inputs)];
    Playhead::on(step, clip_after.hold(step), &slowdowns)
}

/// Spec: `HandOverSlowdown` (a quirk). The first step of a pose her clip's end handed over is
/// slowed by the step slowdown as well as the start slowdown; any other pose by the start
/// slowdown alone.
fn hand_over_slowdown(inputs: &Inputs) -> Slowdown {
    if inputs.moves[MoveEvent::HandedOver] {
        inputs.liquid.step_slowdown
    } else {
        Slowdown::NONE
    }
}
