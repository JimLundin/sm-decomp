//! Audio transport, by frame or by sender service.
//!
//! The game offers sound cues to three sound queues; its sender service presents each group's
//! oldest cue on the group's request line and waits for the audio side to reply with it. The
//! audio side runs effect turns, which handle, publish and stage requests and play effects, and
//! music turns, which advance the resident song. How many turns a frame runs, and whether it runs
//! its sender service, is the caller's input: `frame::play` takes them as a `FrameCadence`.
//!
//! The whole `State` is the caller's to build, save and restore between frames; the model keeps
//! no clock of its own. Echo membership is a semantic selection of voices, not a sound output.
//! For sender-only updates, use [`offer`] on a group's queue and [`serve`] on its sender and
//! request line. The caller can save the [`SenderState`] and line ([`Request`]) independently
//! of effect and music progress.
#![expect(
    clippy::doc_markdown,
    reason = "Spec entry identifiers are plain catalogue labels throughout the model"
)]
/// Producer's admitted cue queue allowance, distinct from movement request metadata.
pub use types::audio::CueAllowance as QueueAllowance;
pub use types::audio::*;
pub mod catalogue;
mod effects;
pub mod frame;
mod music;
mod sender;
pub use music::ramp_from;
pub use sender::{Offer, offer, serve};
