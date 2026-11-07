//! Samus's pose assets, by her pose as the player sees it.
//!
//! Each pose has the frames it draws: how long each holds, the sprites of her top and bottom
//! halves, where they sit and the graphics they upload (docs/architecture/assets.md).

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

/// A pose's asset.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PoseAsset {
    /// The pose it shows.
    pub key: PoseKey,
    /// When it is the one shown.
    pub worn: Worn,
    /// Its frames, by animation frame number.
    pub frames: Vec<Frame>,
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

/// Samus's pose assets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SamusAssets {
    /// One asset per pose and dress that has its own sprite.
    pub poses: Vec<PoseAsset>,
    /// The holds a run uses instead of its own while she has running momentum.
    pub running: Vec<Hold>,
    /// The chest sprite.
    pub chest: Chest,
    /// The sprite lists frames name.
    pub sprites: Vec<Spritemap>,
    /// The tile uploads frames name.
    pub tiles: Vec<TileUpload>,
}

impl SamusAssets {
    /// The asset that shows `key` in `dress`, by its nearest sprite where it has none of its
    /// own; `None` for a pose no sprite shows.
    #[must_use]
    pub fn pose(&self, key: PoseKey, dress: Dress) -> Option<&PoseAsset> {
        let shown = key.shown_as();
        self.poses
            .iter()
            .find(|asset| asset.key == shown && asset.worn.holds(dress))
    }

    /// The sprite list `id` names.
    #[must_use]
    pub fn sprite(&self, id: SpriteId) -> Option<&Spritemap> {
        self.sprites.get(usize::from(id.0))
    }

    /// The upload `id` names.
    #[must_use]
    pub fn upload(&self, id: TilesId) -> Option<&TileUpload> {
        self.tiles.get(usize::from(id.0))
    }
}

#[cfg(test)]
mod tests;
