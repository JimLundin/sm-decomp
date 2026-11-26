//! Her appearance State ([`AppearanceState`]) holds all she remembers from one frame to the
//! next: her step and the time left on it, the boost stage and the laps left, her kept action,
//! the cannon selection delay, the visor delay, visor sequence position and charge-glow
//! position, and the hurt count. [`AppearanceState::INITIAL`] is the State she has as Samus is
//! initialized: a caller starts from it, or plays a frame with her initialization from any
//! State, alike.
//!
//! The caller keeps that State from one frame to the next, saving and restoring it as it is,
//! and gives it back to the next frame as [`update`] left it; nothing else carries over. The
//! current and target palettes live with their shared owner, beside it, which writes the
//! colours a frame gives her row. Before a frame, the caller may ask the read-ahead,
//! [`Ahead::of`], of that State and the figure she starts the frame in.
//!
//! On a locked elevator room drawing, [`elevator_standing_picture`] reads the current pose's
//! facing and displayed step after admitting the pose, without advancing the frame. Video
//! receives body work to publish separately from any drawing errors.
//!
//! On an ordinary room drawing, [`OrdinaryBodyDraw::body_drawn`] decides whether to submit
//! body placements, independently of the art and tile requests.

mod ahead;
mod cannon;
mod catalogue;
mod clip;
mod colours;
mod elevator;
mod flicker;
mod hurt;
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
pub use cannon::{CannonDelay, ControlEligibility, ItemControlAnswer, ItemControls, SelectedItem};
pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use clip::{
    Branch, Clip, Control, Frames, Hold, Instruction, Pace, Paces, ShownFrames, Slowdown, Step,
};
pub use colours::{
    ChargeGlowPosition, ColourEffect, ColourSequences, DryColourOutcome, DryColourReset, HurtCount,
    LiquidLevel, PaletteObservations, PaletteOpportunity, RoomColours, Unrecognized, VisorDelay,
    VisorStep,
};
pub use elevator::{
    ArtDisplacement, BodyHalf, BodyObject, BodyPiece, BodyPlacement, BodyRow, ElevatorArt,
    ElevatorBody, ElevatorDrawing, ElevatorPictureError, ElevatorProjection, ObjectSize,
    ObjectStyle, ScreenPoint, ShaftView, StandingLook, StandingPoseArt, StandingStepArt,
    elevator_standing_picture,
};
pub use flicker::OrdinaryBodyDraw;
pub use hurt::HurtOpportunity;
pub use inputs::{
    Activity, AppearanceContent, Charge, Dash, Figure, GrappleActivity, Inputs, Liquid,
    MissingClip, MoveEvent, PulseColours, SoundsWaiting, Suit,
};
pub use rules::Rules;
pub use sounds::{GraphicalContact, Sound, SoundOffer, SoundQueue};
pub use state::{Animation, AppearanceState, Laps, LapsLeft, Playhead};
pub use update::{Frame, FrameOutputs, FramePart, update};
