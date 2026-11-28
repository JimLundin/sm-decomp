#![cfg(not(doctest))]
//! A room's quake: the shake under way, how starts and stops replace it, how each update counts
//! it down, and how far it moves the room's layers and actors.
//!
//! A frame runs in this order: the starts and stops other parts of the game make
//! ([`start_or_stop`]), Drawing's query for the actors' projectiles ([`drawn_offset`]), then the
//! room's update ([`step`]).

mod catalogue;

use core::num;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE, Check, Kind, Switch};
use types::Pair;
pub use types::quake::{
    QuakeDirection, QuakeDuration, QuakeKind, QuakePhase, QuakeReach, QuakeSize,
    RoomQuakeState as State, Shake,
};

/// Spec: `ShakenLayers`. A shake runs through 4 frames of remaining, two each way.
const SWING_CYCLE: u16 = 4;
/// Spec: `ShakenLayers`. The offset is `+size` while remaining mod 4 is below 2, `−size` after.
const FRAMES_EACH_WAY: u16 = 2;

/// Spec: `ShakenActors`. How many frames an actor asked to shake goes on shaking: its next 2.
pub const ACTOR_SHAKE_FRAMES: u16 = 2;

/// An offset in whole signed pixels, x and y, −3 to 3.
///
/// On a layer it is added to where the view sits (+1 on x shows the picture 1 px further left);
/// on a projectile it is added to where it is drawn (+1 on x draws it 1 px further right).
pub type Offset = Pair<i16>;

/// No offset: the layer or projectile stays where its owner put it.
pub const NO_OFFSET: Offset = Pair { x: 0, y: 0 };

/// What the scheduler gives the room's quake this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opportunity {
    /// An ordinary update: the frames of play, and of play's ending at the station's elevator.
    Ordinary,
    /// The scheduler skips the quake this frame: a door's passage.
    Absent,
    /// An update was asked for without an opportunity the scheduler established.
    Unestablished,
}

/// Whether time runs this frame. The reserve tanks' refill, her death and the x-ray scope hold
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Time {
    /// Time runs.
    Running,
    /// Time is stopped.
    Held,
}

/// Spec: `ShakeStarts`. What another part of the game asks of the quake that makes the place shake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShakeOrder {
    /// Start a shake of this kind for this many frames; 0 frames is a stop.
    Start {
        /// The kind of shake.
        kind: QuakeKind,
        /// How many frames it lasts.
        frames: QuakeDuration,
    },
    /// End the shake under way.
    Stop,
}

/// What an ordinary update publishes for this frame; none of it is State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effects {
    /// The front layer's (the room's blocks') offset.
    pub front: Offset,
    /// The back layer's (the room's background's) offset.
    pub back: Offset,
    /// Whether every actor the room runs this frame is asked to shake for its next
    /// [`ACTOR_SHAKE_FRAMES`] frames.
    pub actors_asked: bool,
}

/// What a requested update did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub enum Outcome {
    /// The update ran and published these effects.
    Updated(Effects),
    /// There was no opportunity: nothing changed and nothing was published.
    NoOpportunity,
    /// The update had no established opportunity: refused before changing anything.
    Unsupported,
}

/// Spec: `ShakeStarts`. A start replaces whatever shake is under way, keeping the phase; a stop,
/// or a start of 0 frames, leaves the room steady. It applies in play and in a passage alike.
pub const fn start_or_stop(state: &mut State, order: ShakeOrder) {
    state.shake = match order {
        ShakeOrder::Start { kind, frames } => match num::NonZeroU16::new(frames.0) {
            Some(remaining) => Some(Shake { kind, remaining }),
            None => None,
        },
        ShakeOrder::Stop => None,
    };
}

/// Spec: `ShakenActors`. Drawing's query: the offset added to each actor's projectile.
///
/// It is the layers' offset while a shake that reaches the actors runs, else none.
#[must_use]
pub fn drawn_offset(state: State, time: Time) -> Offset {
    felt_shake(state, time).map_or(NO_OFFSET, |shake| {
        reached(shake.kind.reach).actors.offset(sway(shake))
    })
}

/// Spec: `QuietQuakePhase`, `ShakeCountsDown`. The room's update.
///
/// It publishes this frame's effects from the shake as the update begins, advances the phase,
/// and, while time runs, spends one of the shake's frames. Without an opportunity nothing
/// changes.
pub fn step(state: &mut State, opportunity: Opportunity, time: Time) -> Outcome {
    match opportunity {
        Opportunity::Ordinary => Outcome::Updated(update(state, time)),
        Opportunity::Absent => Outcome::NoOpportunity,
        Opportunity::Unestablished => Outcome::Unsupported,
    }
}

/// Spec: `ShakeCountsDown`. One ordinary update.
fn update(state: &mut State, time: Time) -> Effects {
    let effects = effects(felt_shake(*state, time));
    state.phase = state.phase.advanced();
    if let (Time::Running, Some(shake)) = (time, state.shake) {
        state.shake = counted_down(shake);
    }
    effects
}

/// Spec: `ShakeCountsDown`. The shake with one frame spent; none once its last is spent.
const fn counted_down(shake: Shake) -> Option<Shake> {
    match num::NonZeroU16::new(shake.remaining.get().saturating_sub(1)) {
        Some(remaining) => Some(Shake {
            kind: shake.kind,
            remaining,
        }),
        None => None,
    }
}

/// Spec: `ShakeCountsDown`, `ShakenActors`. The shake the room feels this frame: the one under
/// way while time runs, none while time is held.
const fn felt_shake(state: State, time: Time) -> Option<Shake> {
    match time {
        Time::Running => state.shake,
        Time::Held => None,
    }
}

/// Spec: `ShakenLayers`, `ShakenActors`. What a felt shake publishes: each part it reaches moved
/// by its sway, the rest still.
fn effects(felt: Option<Shake>) -> Effects {
    let Some(shake) = felt else {
        return Effects {
            front: NO_OFFSET,
            back: NO_OFFSET,
            actors_asked: false,
        };
    };
    let reached = reached(shake.kind.reach);
    let sway = sway(shake);
    Effects {
        front: reached.front.offset(sway),
        back: reached.back.offset(sway),
        actors_asked: matches!(reached.actors, Part::Shaken),
    }
}

/// Spec: `ShakenLayers`, `ShakenActors`. Whether a shake moves a part of the room.
#[derive(Clone, Copy)]
enum Part {
    /// Moved by the shake's sway.
    Shaken,
    /// Left where it is.
    Still,
}

impl Part {
    /// The part's offset under a shake's sway.
    const fn offset(self, sway: Offset) -> Offset {
        match self {
            Self::Shaken => sway,
            Self::Still => NO_OFFSET,
        }
    }
}

/// Spec: `ShakenLayers`, `ShakenActors`. The parts of the room a reach moves.
struct Reached {
    /// The front layer.
    front: Part,
    /// The back layer.
    back: Part,
    /// The actors: asked to shake, and their projectiles drawn moved.
    actors: Part,
}

/// Spec: `ShakenLayers`, `ShakenActors`. Which parts each reach moves.
const fn reached(reach: QuakeReach) -> Reached {
    match reach {
        QuakeReach::Whole => Reached {
            front: Part::Shaken,
            back: Part::Shaken,
            actors: Part::Shaken,
        },
        QuakeReach::Scenery => Reached {
            front: Part::Shaken,
            back: Part::Shaken,
            actors: Part::Still,
        },
        QuakeReach::Front => Reached {
            front: Part::Shaken,
            back: Part::Still,
            actors: Part::Still,
        },
        QuakeReach::Back => Reached {
            front: Part::Still,
            back: Part::Shaken,
            actors: Part::Shaken,
        },
    }
}

/// Spec: `ShakenLayers`. Whether a direction moves an axis.
#[derive(Clone, Copy)]
enum Axis {
    /// Moved by the shake's size.
    Moved,
    /// Left at 0.
    Unmoved,
}

/// Spec: `ShakenLayers`. Which axes each direction moves.
const fn axes(direction: QuakeDirection) -> Pair<Axis> {
    match direction {
        QuakeDirection::Diagonal => Pair {
            x: Axis::Moved,
            y: Axis::Moved,
        },
        QuakeDirection::Sideways => Pair {
            x: Axis::Moved,
            y: Axis::Unmoved,
        },
        QuakeDirection::UpAndDown => Pair {
            x: Axis::Unmoved,
            y: Axis::Moved,
        },
    }
}

impl Axis {
    /// The axis's share of a signed step.
    const fn offset(self, step: i16) -> i16 {
        match self {
            Self::Moved => step,
            Self::Unmoved => 0,
        }
    }
}

/// Spec: `ShakenLayers`. The offset a shake gives what it moves this frame: its size along each
/// axis its direction moves, `+size` while its remaining mod 4 is 0 or 1 and `−size` while it is
/// 2 or 3, so it jumps back and forth two frames each way.
fn sway(shake: Shake) -> Offset {
    let size = i16::from(shake.kind.size.pixels());
    let step = if shake.remaining.get() % SWING_CYCLE < FRAMES_EACH_WAY {
        size
    } else {
        size.wrapping_neg()
    };
    let axes = axes(shake.kind.direction);
    Pair {
        x: axes.x.offset(step),
        y: axes.y.offset(step),
    }
}
