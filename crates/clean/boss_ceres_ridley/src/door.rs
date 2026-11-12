//! A Ceres door: how a room places it, and its run each frame the enemies run.

use core::num::Wrapping;

use types::{Hitbox, Pair, Px, Radius, Side};

use crate::figure::{Figure, figures};
use crate::inputs::Inputs;
use crate::phase::{Phase, Quarter};
use crate::rules::Rules;
use crate::station::{ColourSet, Shake, Stage};

/// `Solid`: a door's box reaches 8 px to the left and right of its position and 32 px up and
/// down: 16 px wide and 64 px tall. The box is the doors' content and never changes; only
/// whether it is solid does.
pub const BOX_RADIUS: Radius = Pair {
    x: Wrapping(8),
    y: Wrapping(32),
};

/// A door's kind. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorKind {
    /// A door between two rooms.
    Door,
    /// The shaft's door: a left-facing door until Ridley's getaway, a wall from then on.
    ShaftDoor,
}

impl DoorKind {
    /// `ShaftWall`, `RoomStart`: whether a door placed at this stage becomes the wall at its
    /// first run: the shaft's door, once Ridley has fled.
    #[must_use]
    pub const fn becomes_wall(self, stage: Stage) -> bool {
        matches!(self, Self::ShaftDoor) && stage.ridley_has_fled()
    }
}

/// A door of the Ceres station: the state a caller holds from one frame to the next. Its kind,
/// facing and position never change; its phase carries the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresDoor {
    /// Whether it becomes a wall.
    pub kind: DoorKind,
    /// The way it opens, and the side its figures are drawn for.
    pub facing: Side,
    /// The centre of its box, in whole pixels.
    pub position: Pair<Px>,
    /// Its phase, with its wait.
    pub phase: Phase,
}

/// What a room's start answers when it places a door.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    /// The door, `Placed`.
    pub door: CeresDoor,
    /// The colour set the doors' palette takes.
    pub colours: ColourSet,
}

/// The doors' opening sound, asked for with a queue limit of 6 (`Opening`). What the queue does
/// with a request it can't hold is audio's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpeningSound;

impl OpeningSound {
    /// `Opening`: the queue limit the sound is asked for with.
    pub const QUEUE_LIMIT: u8 = 6;
}

/// What one run of a door answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// The door as its run leaves it.
    pub door: CeresDoor,
    /// The opening sound, if the run asks for it.
    pub sound: Option<OpeningSound>,
    /// The shake the run starts, if any.
    pub shake: Option<Shake>,
}

impl CeresDoor {
    /// `RoomStart`, `TheDoorsColours`: a room's start places the door from the room's content,
    /// `Placed`, so its first run starts it; the doors' palette takes the stage's colour set.
    #[must_use]
    pub const fn place(
        kind: DoorKind,
        facing: Side,
        position: Pair<Px>,
        stage: Stage,
    ) -> Placement {
        Placement {
            door: Self {
                kind,
                facing,
                position,
                phase: Phase::Placed,
            },
            colours: stage.door_colours(),
        }
    }

    /// One run of the door, on a frame the enemies run (on any other frame nothing of it
    /// changes).
    ///
    /// `Waiting`: while its wait is over 1 the run only counts it off; when it runs out the
    /// door steps. `Opening`: the run that steps into the opening's first figure asks for the
    /// opening sound. `EscapeShakes`: every run may start a shake, whether it steps or not.
    #[must_use]
    pub fn run(self, inputs: &Inputs, rules: &Rules) -> Run {
        let counted = self.phase.counted_down();
        let phase = counted.unwrap_or_else(|| self.step(inputs, rules));
        let opening_starts = matches!(
            phase,
            Phase::Opening {
                quarter: Quarter::First,
                ..
            }
        );
        Run {
            door: Self { phase, ..self },
            sound: (counted.is_none() && opening_starts).then_some(OpeningSound),
            shake: inputs.escape_shake(rules),
        }
    }

    /// The phase the door steps into when its wait runs out, with its new wait.
    fn step(self, inputs: &Inputs, rules: &Rules) -> Phase {
        let near = inputs.she_is_near(self.position, rules.reach);
        match (self.phase, near) {
            // `ShaftWall`, `RoomStart`: the shaft's door placed once Ridley has fled is a wall.
            (Phase::Placed, _) if self.kind.becomes_wall(inputs.stage) => Phase::Wall,
            // `RoomStart`: any other door starts hidden, to look for her once.
            (Phase::Placed, _) => Phase::Starting { wait: rules.poll },
            // `RoomStart`, `Polling`: a starting or open door that sees her stands open, silently.
            (Phase::Starting { .. } | Phase::Open { .. }, true) => Phase::Open { wait: rules.poll },
            // `RoomStart`, `Polling`: a starting or closed door that doesn't see her is closed.
            (Phase::Starting { .. } | Phase::Closed { .. }, false) => {
                Phase::Closed { wait: rules.poll }
            }
            // `Opening`: a closed door that sees her starts to open.
            (Phase::Closed { .. }, true) => Phase::Opening {
                quarter: Quarter::First,
                wait: rules.hold,
            },
            // `Closing`: an open door that no longer sees her starts to close.
            (Phase::Open { .. }, false) => Phase::Closing {
                quarter: Quarter::First,
                wait: rules.hold,
            },
            // `Opening`: an opening goes on, near her or not, and once done the door is open.
            (Phase::Opening { quarter, .. }, _) => {
                quarter
                    .next()
                    .map_or(Phase::Open { wait: rules.poll }, |quarter| Phase::Opening {
                        quarter,
                        wait: rules.hold,
                    })
            }
            // `Closing`: a closing goes on, near her or not, and once done the door is closed.
            (Phase::Closing { quarter, .. }, _) => {
                quarter
                    .next()
                    .map_or(Phase::Closed { wait: rules.poll }, |quarter| {
                        Phase::Closing {
                            quarter,
                            wait: rules.hold,
                        }
                    })
            }
            // `ShaftWall`: the wall stays a wall.
            (Phase::Wall, _) => Phase::Wall,
        }
    }

    /// The figure the door holds, from its phase and facing; none while it is `Placed`.
    #[must_use]
    pub const fn figure(self) -> Option<Figure> {
        figures(self.phase).for_door(self.facing)
    }

    /// `Solid`: its box, which the framework's solid contact meets while its phase is solid.
    #[must_use]
    pub const fn hitbox(self) -> Hitbox {
        Hitbox {
            centre: self.position,
            radius: BOX_RADIUS,
        }
    }
}
