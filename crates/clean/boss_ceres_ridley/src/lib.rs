//! The boss-ceres-ridley model, part 1: the Ceres doors, built from its behaviour spec alone.
//!
//! A Ceres door stands between two of the station's rooms. It slides open as Samus comes near and
//! shuts behind her once she has gone, hissing as it opens. The shaft's door becomes an unseen
//! wall once Ridley has fled, and during the escape every door keeps the station shaking.
//!
//! A caller holds each [`CeresDoor`] between frames. When a room starts, [`CeresDoor::place`]
//! places it and says which colour set the doors' palette takes. Each frame the enemies run,
//! [`CeresDoor::run`] takes the door and what it reads as the frame starts ([`Inputs`]) and
//! answers with the door as its run leaves it, the sound it asks for and the shake it starts
//! ([`Run`]). The tuning values are in [`Rules`], whose default holds the spec's values.

mod catalogue;
mod door;
mod figure;
mod inputs;
mod phase;
mod rules;
mod station;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use door::{BOX_RADIUS, CeresDoor, DoorKind, OpeningSound, Placement, Run};
pub use figure::{Figure, FigureLook};
pub use inputs::Inputs;
pub use phase::{Phase, Quarter, Wait};
pub use rules::Rules;
pub use station::{ColourSet, Shake, Stage};
