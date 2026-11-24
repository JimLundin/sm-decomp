//! The boss-ceres-ridley model, built from its behaviour spec alone. Two parts of the Ceres
//! station so far: the doors, and the shaft as she arrives, itself two pieces. Each piece has
//! its own state, inputs and outputs, and its caller steps it on its own schedule.
//!
//! **The doors.** A Ceres door stands between two of the station's rooms. It slides open as
//! Samus comes near and shuts behind her once she has gone, hissing as it opens. The shaft's door
//! becomes an unseen wall once Ridley has fled, and during the escape every door keeps the
//! station shaking.
//!
//! A caller holds each [`CeresDoor`] between frames. When a room starts, [`CeresDoor::place`]
//! places it and says which colour set the doors' palette takes. Each frame the enemies run,
//! [`CeresDoor::run`] takes the door and what it reads as the frame starts ([`Inputs`]) and
//! answers with the door as its run leaves it, the sound it asks for and the shake it starts
//! ([`Run`]).
//!
//! **The shaft as she arrives.** A new game starts with her at the top of the shaft on the
//! elevator's pad. The pad holds her for a second, lowers her a pixel a frame to the shaft's
//! floor and lets her go; a cover hides the floor below until she stands on it. Before the
//! escape the shaft's beacon pulses through its colours and the platform below turns.
//!
//! The pad and the cover hold state: a caller holds the [`PadAndCover`] between frames and
//! steps it on the frame the arrival places them ([`PadAndCover::arrival`]) and on each frame
//! they run ([`PadAndCover::run`]). Each step answers with them as it leaves them, her height
//! and what is drawn of them ([`PadAndCoverStep`]).
//!
//! The lights remember nothing: on each frame the shaft's overlay runs, [`Lights::step`] takes
//! what it reads ([`LightsInputs`]) and answers with the beacon's step and the platform's look
//! ([`Lights`]).
//!
//! The tuning values of all of them are in [`Rules`], whose default holds the spec's values.

mod catalogue;
mod door;
mod figure;
mod inputs;
mod lights;
mod look;
mod pad;
mod pad_wait;
mod phase;
mod rules;
mod station;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use door::{BOX_RADIUS, CeresDoor, DoorKind, OpeningSound, Placement, Run};
pub use figure::{Figure, FigureLook};
pub use inputs::Inputs;
pub use lights::{BeaconStep, FadeSteps, Lights, LightsInputs};
pub use look::AlternatingLook;
pub use pad::{Cover, Pad, PadAndCover, PadAndCoverFigures, PadAndCoverStep, PadFigure};
pub use pad_wait::PadWait;
pub use phase::{Phase, Quarter, Wait};
pub use rules::Rules;
pub use station::{ColourSet, Shake, Stage};
