//! The samus-appearance model, built from its behaviour spec alone: her animation's timing, the
//! sounds of her stride, pose selection, passages and drawing, when her cannon may follow the
//! selected item, and her colour sequences during ordinary dry movement.
//!
//! A frame is played whole, by [`update`]: the [`AppearanceState`] the caller kept, the frame's
//! parts ([`Frame`]: its item controls, stride, animation [`Inputs`], pose selection, palette
//! opportunity, passage entry and drawing, each with its owners' inputs), her clips and the
//! run's paces ([`Content`]) and the tuning ([`Rules`]; `Rules::VANILLA` is the game) go in. The
//! appearance State to keep for the next frame and the frame's outputs ([`FrameOutputs`]) come
//! out: the end and turned far, the graphical contact, whether the cannon may follow, whether
//! the ordinary colour reset applied and the frame's sounds as one ordered list. Her step, the
//! boost stage and her colour sequences are read from the State the frame leaves.
//!
//! The end her clip reaches and whether her spin has turned far follow from the frame's start
//! alone, so samus-movement, which reads both before its step, reads them with [`Ahead::of`]
//! first; the frame's outputs give the same reading.
//!
//! The appearance State is all her appearance remembers: the four animation quantities
//! ([`State`]: her step and the time left on it, the boost stage and the laps left), the cannon
//! selection delay ([`CannonDelay`]) and her colour sequences ([`ColourSequences`]). A caller may
//! set each to any value the spec allows, saves and restores it between frames with serde, and
//! gives it back as `update` left it. Her clips are data: each [`Clip`] is a list of steps, each
//! one shown for a [`Hold`] or a [`Control`].

mod ahead;
mod cannon;
mod catalogue;
mod clip;
mod colours;
mod inputs;
mod moves;
mod opportunity;
mod play;
mod rules;
mod run;
mod starting;
mod state;
mod update;

pub use ahead::Ahead;
pub use cannon::{CannonDelay, CannonEvent, ControlEligibility, ItemControlAnswer, SelectedItem};
pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, Control, Frames, Hold, Instruction, Pace, Paces, Slowdown, Step};
pub use colours::{
    ChargeGlowPosition, ColourEffect, ColourSequences, DryColourFacts, DryColourReset,
    GrappleActivity, LiquidLevel, PaletteOpportunity, RoomColours, ScrewAttack, Suit, VisorDelay,
    VisorStep,
};
pub use inputs::{Content, Dash, Figure, Inputs, Liquid, MoveEvent, SoundsWaiting};
pub use opportunity::{
    Charge, CinematicActivity, ContactSuppressor, GraphicalContact, NormalDrawing, PassageEntry,
    Selection, SoundQueue, SoundRequest, Stride,
};
pub use rules::Rules;
pub use state::{AppearanceState, Laps, LapsLeft, Playhead, State};
pub use update::{Frame, FrameOutputs, FrameSound, update};
