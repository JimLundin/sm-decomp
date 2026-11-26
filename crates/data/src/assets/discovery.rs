//! The egg, the baby and the pieces they release in the discovery scene.

use crate::spritemap::Spritemap;
use core::num::NonZeroU16;
pub use types::Pair;
pub use types::discovery::{Coord, Offset, RoomPoint};

/// A picture in the scene's drawing collection, with its duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Index of its sprite map.
    pub picture: u16,
    /// Finished frames displaying this picture.
    pub hold: NonZeroU16,
}

/// A nonempty sequence of pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    /// The first frame.
    pub first: Frame,
    /// The following frames.
    pub rest: Vec<Frame>,
}

/// A particle's picture and signed motion per finished frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flight {
    /// Index of its sprite map.
    pub picture: u16,
    /// Horizontal displacement per step.
    pub drift: Offset,
    /// Vertical displacements in flight order.
    pub path: Vec<Offset>,
}

/// The imported pictures, animation and motion of the discovery encounter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scene {
    /// Sprite maps in scene-local identity order.
    pub drawings: Vec<Spritemap>,
    /// Shared anchor of egg and baby.
    pub home: RoomPoint,
    /// Picture while waiting.
    pub closed: u16,
    /// Finite opening animation.
    pub hatch: Clip,
    /// Hatch age of the scatter and release markers.
    pub release_at: u16,
    /// Picture after hatching.
    pub spent: u16,
    /// Repeating baby animation.
    pub pulse: Clip,
    /// Placed shell fragments in foreground order.
    pub fragments: Vec<(RoomPoint, Flight)>,
    /// Falling droplets in foreground order.
    pub drops: Vec<Flight>,
    /// Finite animation after impact.
    pub splash: Clip,
    /// Sprite palette of the scene.
    pub palette: u8,
    /// Shell sound in the second sound library.
    pub hatch_sound: u8,
    /// Baby sound in the third sound library.
    pub cry: u8,
}
