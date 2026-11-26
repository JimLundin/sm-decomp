//! Imported sprite art for Ridley's ordinary fight.

use crate::spritemap::{ExtendedMap, Spritemap};

/// Body maps in one facing direction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyLooks {
    /// Mouth closed.
    pub rest: Vec<ExtendedMap>,
    /// Mouth half open.
    pub open: Vec<ExtendedMap>,
    /// Mouth wide.
    pub wide: Vec<ExtendedMap>,
    /// Legs half extended.
    pub takeoff1: Vec<ExtendedMap>,
    /// Legs extended.
    pub takeoff2: Vec<ExtendedMap>,
}

/// Ordinary fight art, independent of the flyby.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Art {
    /// The three baby looks in animation order.
    pub baby: [Spritemap; 3],
    /// The ordinary wing maps by beat, left then right.
    pub wings: [[Spritemap; 10]; 2],
    /// The three tail segment looks, from the base toward the tip.
    pub links: [Spritemap; 3],
    /// The tip maps around one rotation.
    pub tip: [Spritemap; 16],
    /// The body seen from the left.
    pub left: BodyLooks,
    /// The body seen from the right.
    pub right: BodyLooks,
    /// The body seen from the front.
    pub front: Vec<ExtendedMap>,
    /// The door drawn with Ridley.
    pub attached_door: Spritemap,
}
