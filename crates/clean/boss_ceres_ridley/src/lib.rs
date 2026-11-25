//! The boss-ceres-ridley model, built from its behaviour spec alone. The parts of the Ceres
//! station so far: the doors, the shaft as she arrives, Ridley's room as she enters, Ridley's
//! waking, and his tail at rest and its hit. Each
//! piece has its own state, inputs and outputs, and its caller steps it on its own schedule.
//!
//! **The doors.** A Ceres door stands between two of the station's rooms. It slides open as
//! Samus comes near and shuts behind her once she has gone, hissing as it opens. The shaft's door
//! becomes an unseen wall once Ridley has fled, and during the escape every door keeps the
//! station shaking.
//!
//! A caller holds each [`CeresDoor`] between frames. When a room starts, [`CeresDoor::place`]
//! places it and says which colour set its palette takes, and which palette. Each frame the
//! enemies run,
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
//! **Ridley's room as she enters.** Its door is a Ceres door of its own kind: it stands open
//! behind her a moment, slams shut and stays locked, drawn by Ridley, until he is beaten; then
//! it is an ordinary door. The two walls his getaway puts up are Ceres doors too, shown during
//! the getaway only. Ridley waits in the dark with the baby, which now and then cries and
//! glows, until he wakes.
//!
//! Ridley holds state: a caller holds the [`CeresRidley`] between frames and steps it on the
//! frame the room places him ([`CeresRidley::place`]) and on each frame he runs
//! ([`CeresRidley::run`]), with what he reads. The placing answers with what
//! he hands the rest of the game ([`RidleyPlacing`]); a run, with the baby's cry and colours
//! ([`RidleyRun`]).
//!
//! **Ceres Ridley wakes.** His waiting over, his eye lights up out of the dark, then his body
//! fades in; with its last colours he turns tangible and the fight's music starts. A moment
//! later he roars, shuts his jaws and sits a while before he takes off. The same
//! [`CeresRidley::run`] carries him through it, reading his content ([`RidleyContent`]): the
//! run answers with his eye's and body's colours, the fight's music and the roar's sound too.
//!
//! **His tail at rest and its hit.** From the moment he starts moving until he takes off, his
//! tail hangs still behind him ([`CeresRidley::tail`]), its curl changing now and then at
//! random. Once he is tangible its tip hits her when her body comes within its reach and her
//! protection has run out: the run reads her ([`Her`]) and answers with the frame's hit, its
//! side and its push time ([`types::TailImpact`]), which her energy and her motion read as it
//! stands.
//!
//! The tuning values of all of them are in [`Rules`], whose default holds the spec's values.

mod baby;
mod catalogue;
mod door;
mod figure;
mod inputs;
mod lights;
mod look;
mod pad;
mod pad_wait;
mod phase;
mod ridley;
mod ridley_phase;
mod ridley_wait;
mod roar;
mod rules;
mod shown;
mod station;
mod tail;
mod waking;

pub use baby::{
    Baby, BabyColours, BabyCry, BabyFigure, BabyLook, BabyRoutine, BabyRun, FidgetFigure,
    GlowFigure,
};
pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use door::{BOX_RADIUS, CeresDoor, DoorKind, OpeningSound, Placement, Run};
pub use figure::{Figure, FigureLook};
pub use inputs::Inputs;
pub use lights::{BeaconStep, FadeSteps, Lights, LightsInputs};
pub use look::AlternatingLook;
pub use pad::{Cover, Pad, PadAndCover, PadAndCoverFigures, PadAndCoverStep, PadFigure};
pub use pad_wait::PadWait;
pub use phase::{Phase, Quarter, Wait};
pub use ridley::{
    Boss, CeresRidley, Drawn, GoalColours, MusicStop, RidleyDrawing, RidleyPlacing, RidleyRun,
    RoomColours, ShakeStop,
};
pub use ridley_phase::{FightMusic, RidleyPhase, RoarSound};
pub use ridley_wait::{RidleyWait, WakeWait};
pub use roar::{RidleyLook, RoarFigure, TimedRoarFigure};
pub use rules::Rules;
pub use shown::{FigureLength, ShownFigure};
pub use station::{ColourSet, DoorColours, DoorPalette, Shake, ShakeLayers, ShakeStart, Stage};
pub use tail::{Her, TailCurl};
pub use waking::{BodyColours, EyeColours, FadeSets, ReadyWait, RidleyContent, RoarWait, RunsLeft};
