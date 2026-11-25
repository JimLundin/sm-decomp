//! The gunship's flight to Ceres, built from its behaviour spec alone.
//!
//! The screen is dark while the flight is made ready and its music loads; the gunship rushes in
//! toward the camera past streaking stars; with a white flash the view turns to the gunship from
//! behind, flying away among drifting sprites; the station comes into view and its label is typed
//! letter by letter; the picture fades to black and the flight is done.
//!
//! [`step`] runs one frame: it reads the flight's [`State`], the scene's content and whether a
//! track waits in audio's queue, and reports what the renderer, the sprite drawing and the tile
//! writing show that frame, with the music requests it hands to audio.

mod catalogue;
mod frame;
mod plane;
mod rules;
mod sprites;
mod stages;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use frame::{FrameReport, step};
pub use plane::PlaneTransform;
pub use rules::Rules;
pub use sprites::SpriteView;
pub use stages::MusicQueue;
pub use types::flight::State;
