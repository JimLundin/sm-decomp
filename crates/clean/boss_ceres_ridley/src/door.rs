//! A Ceres door: how a room places it, and its run each frame the enemies run. The doors
//! between rooms, the shaft's door, the door of Ridley's room and the getaway's walls are all
//! Ceres doors, each of its kind.

use core::num::Wrapping;

use types::{Hitbox, Pair, Px, Radius, Side};

use crate::figure::{Figure, figures};
use crate::inputs::Inputs;
use crate::phase::{Phase, Quarter, Wait};
use crate::rules::Rules;
use crate::station::{DoorColours, DoorPalette, ShakeLayers, ShakeStart, Stage};

/// `Solid`: a door's box reaches 8 px to the left and right of its position and 32 px up and
/// down: 16 px wide and 64 px tall. The box is the doors' content and never changes; only
/// whether it is solid does.
pub const BOX_RADIUS: Radius = Pair {
    x: Wrapping(8),
    y: Wrapping(32),
};

/// `RidleyRoomStart`: the pause, in frames.
const PAUSE_FRAMES: u16 = 2;
/// `RidleyRoomStart`: the door of Ridley's room stands open behind her for 2 frames.
const PAUSE: Wait = Wait::new(PAUSE_FRAMES).expect("the pause is 2 frames, not 0");
/// `LockedIn`: the lock's look, in frames.
const LOCK_LOOK_FRAMES: u16 = 2;
/// `LockedIn`: the locked door looks every 2 frames whether Ridley is beaten.
const LOCK_LOOK: Wait = Wait::new(LOCK_LOOK_FRAMES).expect("the lock's look is 2 frames, not 0");

/// A door's kind. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorKind {
    /// A door between two rooms.
    Door,
    /// One of the two walls Ridley's getaway puts up at the sides of his room: shown during
    /// the getaway only, and never in her way.
    GetawayWall,
    /// The door of Ridley's room, facing right: it slams shut behind her as the room starts and
    /// stays locked until Ridley is beaten; then it is a `Door`.
    RidleyDoor,
    /// The shaft's door: a left-facing door until Ridley's getaway, a wall from then on.
    ShaftDoor,
}

impl DoorKind {
    /// The phase a placed door of this kind steps into at its first run.
    ///
    /// `RoomStart`: a door starts hidden, to look for her once. `ShaftWall`: the shaft's door
    /// does too, unless Ridley has fled, when it is the wall. `RidleyRoomStart`: the door of
    /// Ridley's room lingers open, whatever the stage, whether he is beaten and wherever she
    /// is. `GetawayWalls`: a getaway wall stands or is unseen by the stage.
    const fn first_phase(self, stage: Stage, rules: &Rules) -> Phase {
        match self {
            Self::ShaftDoor if stage.ridley_has_fled() => Phase::Wall,
            Self::Door | Self::ShaftDoor => Phase::Starting { wait: rules.poll },
            Self::RidleyDoor => Phase::Lingering { wait: PAUSE },
            Self::GetawayWall => getaway_wall(stage),
        }
    }

    /// `EscapeShakes`: what the shakes a door of this kind starts move: the back layer for the
    /// door of Ridley's room, the whole view for any other door; a getaway wall starts none.
    const fn shake_layers(self) -> Option<ShakeLayers> {
        match self {
            Self::Door | Self::ShaftDoor => Some(ShakeLayers::Whole),
            Self::RidleyDoor => Some(ShakeLayers::Back),
            Self::GetawayWall => None,
        }
    }

    /// `TheDoorsColours`: the palette the doors' colours go to when a door of this kind is
    /// placed: the door of Ridley's room placed while calm takes them in its own palette, since
    /// while Ridley is in the room his colours hold the doors'; any other, the doors' palette.
    const fn palette(self, stage: Stage) -> DoorPalette {
        if matches!((self, stage), (Self::RidleyDoor, Stage::Calm)) {
            DoorPalette::Own
        } else {
            DoorPalette::Doors
        }
    }
}

/// `GetawayWalls`: a getaway wall stands during the getaway and is unseen at any other stage. It
/// looks for nobody, and steps at every run.
const fn getaway_wall(stage: Stage) -> Phase {
    match stage {
        Stage::Getaway => Phase::Standing,
        Stage::Calm | Stage::Escape => Phase::Unseen,
    }
}

/// `LockedIn`: the locked door, when it looks: while Ridley lives it stays locked, wherever she
/// is; once he is beaten it is a closed door like any other.
const fn lock(ridley_beaten: bool, rules: &Rules) -> Phase {
    if ridley_beaten {
        Phase::Closed { wait: rules.poll }
    } else {
        Phase::Locked { wait: LOCK_LOOK }
    }
}

/// A door of the Ceres station: the state a caller holds from one frame to the next. Its kind,
/// facing and position never change; its phase carries the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresDoor {
    /// Whether it becomes a wall, slams shut or stands in for the room's sides.
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
    /// The colour set the door's palette takes, and which palette that is.
    pub colours: DoorColours,
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
    pub shake: Option<ShakeStart>,
}

impl CeresDoor {
    /// `RoomStart`, `TheDoorsColours`: a room's start places the door from the room's content,
    /// `Placed`, so its first run starts it; the door's palette takes the stage's colour set.
    /// (Ridley's getaway places its walls the same way.)
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
            colours: DoorColours {
                set: stage.door_colours(),
                palette: kind.palette(stage),
            },
        }
    }

    /// One run of the door, on a frame the enemies run (on any other frame nothing of it
    /// changes).
    ///
    /// `Waiting`: while its wait is over 1 the run only counts it off; when it runs out the
    /// door steps. `Opening`: the run that steps into the opening's first figure asks for the
    /// opening sound. `EscapeShakes`: every run may start a shake, whether it steps or not, of
    /// what its kind's shakes move.
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
            shake: self.kind.shake_layers().and_then(|layers| {
                inputs
                    .escape_shake(rules)
                    .map(|shake| ShakeStart { shake, layers })
            }),
        }
    }

    /// The phase the door steps into when its wait runs out, with its new wait.
    fn step(self, inputs: &Inputs, rules: &Rules) -> Phase {
        let near = inputs.she_is_near(self.position, rules.reach);
        match (self.phase, near) {
            // `RoomStart`, `ShaftWall`, `RidleyRoomStart`, `GetawayWalls`: a placed door's first
            // run starts it by its kind.
            (Phase::Placed, _) => self.kind.first_phase(inputs.stage, rules),
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
            // `SlamShut`: the lingering door slams shut, silently, near her or not.
            (Phase::Lingering { .. }, _) => Phase::Slamming {
                quarter: Quarter::First,
                wait: rules.slam,
            },
            // `SlamShut`: a slam goes on, and once done the door is shut, for one run.
            (Phase::Slamming { quarter, .. }, _) => {
                quarter
                    .next()
                    .map_or(Phase::Slammed, |quarter| Phase::Slamming {
                        quarter,
                        wait: rules.slam,
                    })
            }
            // `LockedIn`: the shut door locks, whether Ridley is beaten or not.
            (Phase::Slammed, _) => Phase::Locked { wait: LOCK_LOOK },
            // `LockedIn`: the locked door looks whether Ridley is beaten.
            (Phase::Locked { .. }, _) => lock(inputs.ridley_beaten, rules),
            // `GetawayWalls`: a getaway wall stands or is unseen by the stage, at every run.
            (Phase::Standing | Phase::Unseen, _) => getaway_wall(inputs.stage),
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
