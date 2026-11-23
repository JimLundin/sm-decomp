//! Her appearance State ([`AppearanceState`]) holds all she remembers from one frame to the
//! next: her step and the time left on it, the boost stage and the laps left, her kept action,
//! the cannon selection delay, and the visor delay, visor sequence position and charge-glow
//! position. [`AppearanceState::INITIAL`] is the State she has as Samus is initialized: a
//! caller starts from it, or plays a frame with her initialization from any State, alike.
//!
//! The caller keeps that State from one frame to the next, saving and restoring it as it is,
//! and gives it back to the next frame as [`update`] left it; nothing else carries over. The
//! current and target palettes are not in it: the caller keeps them with their shared owner.
//! Before a frame, the caller may ask the read-ahead, [`Ahead::of`], of that State.

mod ahead;
mod cannon;
mod catalogue;
mod clip;
mod colours;
mod inputs;
mod moves;
mod play;
mod rules;
mod run;
mod sounds;
mod starting;
mod state;
mod update;

pub use ahead::Ahead;
pub use cannon::{CannonDelay, CannonEvent, ControlEligibility, ItemControlAnswer, SelectedItem};
pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, Control, Frames, Hold, Instruction, Pace, Paces, Slowdown, Step};
pub use colours::{
    ChargeGlowPosition, ColourEffect, ColourSequences, DryColourReset, GrappleActivity,
    LiquidLevel, PaletteObservations, PaletteOpportunity, RoomColours, Suit, VisorDelay, VisorStep,
};
pub use inputs::{
    Activity, Charge, Content, Dash, Figure, Inputs, Liquid, MoveEvent, SoundsWaiting,
};
pub use rules::Rules;
pub use sounds::{GraphicalContact, SoundQueue, SoundRequest};
pub use state::{AppearanceState, Laps, LapsLeft, Playhead, State};
pub use update::{Frame, FrameOutputs, FramePart, update};
