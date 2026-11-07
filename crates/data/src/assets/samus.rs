//! Samus's sprite sheet and animation clips, a clip for each pose as the player sees it.
//!
//! A clip's frames each hold for a time and draw sprites of the sheet: her top and bottom
//! halves, placed from her position, and the graphics they upload (docs/architecture/assets.md).

pub mod import;
pub mod key;

use crate::spritemap::{SpriteAttributes, Spritemap};
use key::{Dress, PoseKey, Worn};

/// A sprite list in [`SamusAssets`]'s pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteId(pub u16);

/// A tile upload in [`SamusAssets`]'s pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilesId(pub u16);

/// How long a frame holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hold {
    /// Shown this many frames.
    Shows(u8),
    /// A step of the animation's control, which motion runs; the byte is what the timer is
    /// set from if the animation stays on it.
    Control(u8),
}

impl Hold {
    /// The byte the animation timer is set from.
    #[must_use]
    pub const fn timer_byte(self) -> u8 {
        match self {
            Self::Shows(frames) => frames,
            Self::Control(byte) => byte,
        }
    }
}

/// The graphics a frame's half uploads before it is drawn: two parts, each to its own place.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileUpload {
    /// The first part's bytes.
    pub first: Vec<u8>,
    /// The second part's bytes.
    pub second: Vec<u8>,
}

/// What a frame uploads: its top half's graphics and, when it has them, its bottom half's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameTiles {
    /// The top half's.
    pub top: TilesId,
    /// The bottom half's.
    pub bottom: Option<TilesId>,
}

/// One frame of a pose's animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Frame {
    /// How long it holds.
    pub hold: Hold,
    /// Her top half's sprites, drawn first; `None` draws nothing.
    pub top: Option<SpriteId>,
    /// Her bottom half's sprites, when the frame draws that half.
    pub bottom: Option<SpriteId>,
    /// Whether the chest sprite is drawn before the bottom half.
    pub chest: bool,
    /// Pixels the sprites' origin sits below her position (negative is above).
    pub origin_y: i16,
    /// The graphics it uploads; `None` uploads nothing.
    pub tiles: Option<FrameTiles>,
}

/// A frame of a clip, by its place in the clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameNumber(pub u16);

/// What a clip does after its last shown frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClipEnd {
    /// Plays on from this frame.
    Loop {
        /// The frame it plays on from; the first is a whole loop.
        from: FrameNumber,
    },
    /// Hands over to another pose, which her motion picks.
    HandOver,
}

/// A pose's animation clip, shown in a dress.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clip {
    /// The pose it shows.
    pub key: PoseKey,
    /// The dress variant it is.
    pub worn: Worn,
    /// Its frames, in order; the animation's frame number indexes them.
    pub frames: Vec<Frame>,
    /// What it does after its last shown frame.
    pub end: ClipEnd,
}

/// The single sprite her first suit's facing-the-screen pose adds before her bottom half.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chest {
    /// Pixels left of her position.
    pub left: u16,
    /// Pixels above her position.
    pub up: u16,
    /// Its tile and attributes.
    pub attributes: SpriteAttributes,
}

/// Her sprite sheet: every sprite list and every graphics upload, once, for the clips to
/// name.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpriteSheet {
    /// The sprite lists, each sprite with its offset, size, tile, palette, priority and flips.
    pub sprites: Vec<Spritemap>,
    /// The graphics uploads.
    pub uploads: Vec<TileUpload>,
}

impl SpriteSheet {
    /// The sprite list `id` names.
    #[must_use]
    pub fn sprite(&self, id: SpriteId) -> Option<&Spritemap> {
        self.sprites.get(usize::from(id.0))
    }

    /// The upload `id` names.
    #[must_use]
    pub fn upload(&self, id: TilesId) -> Option<&TileUpload> {
        self.uploads.get(usize::from(id.0))
    }
}

/// Samus's sprite sheet and animation clips.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SamusAssets {
    /// The sheet the clips draw from.
    pub sheet: SpriteSheet,
    /// One clip per pose and dress that has its own sprites.
    pub clips: Vec<Clip>,
    /// The holds a run uses instead of its clip's while she has running momentum.
    pub running: Vec<Hold>,
    /// The chest sprite.
    pub chest: Chest,
}

impl SamusAssets {
    /// The clip that shows `key` in `dress`, by its nearest pose's where it has none of its
    /// own; `None` for a pose no sprite shows.
    #[must_use]
    pub fn clip(&self, key: PoseKey, dress: Dress) -> Option<&Clip> {
        let shown = key.shown_as();
        self.clips
            .iter()
            .find(|clip| clip.key == shown && clip.worn.holds(dress))
    }
}

#[cfg(test)]
mod tests;
