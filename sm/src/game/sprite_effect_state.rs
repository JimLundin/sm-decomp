//! Sprite-effect declarations only. Active effects and compatibility tails are distinct.
#![forbid(unsafe_code)]
use crate::game::session::{Point, Position};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpriteEffect {
    pub instruction: u16,
    pub timer: u16,
    pub appearance: u16,
    pub x: Position,
    pub y: Position,
    pub disable_flags: u16,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpriteEffects {
    pub active: [SpriteEffect; 32],
    /// Legacy live byte index; normal completed update leaves FFFE.
    pub selected: u16,
}
/// Words 32..63 of each 64-word compatibility column; never playable slots.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpriteCompatibilityTails {
    pub instruction: [u16; 32],
    pub timer: [u16; 32],
    pub appearance: [u16; 32],
    pub x: [u16; 32],
    pub x_fraction: [u16; 32],
    pub y: [u16; 32],
    pub y_fraction: [u16; 32],
    pub disable_flags: [u16; 32],
}
/// Active index 0..31, created by the implementation inside this module.
/// No generation counter: stale/reused-slot behavior is retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteSlot(pub(crate) u8);
/// Compatibility caller's unmodified even/odd byte handle, including FFFF.
/// Conversion shifts once; broad indices must not be clamped to active slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacySpriteHandle(pub u16);
/// Already shifted word index. Full synthetic u16 range can exceed WRAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacySpriteWordIndex(pub u16);
/// Private migration content identity; not a prepared Map Rando asset identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacySpriteDefinition(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacySpriteProgram(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacySpriteMap(pub u16);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteSpawn {
    pub position: Point,
    pub definition: LegacySpriteDefinition,
    pub appearance: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnResult {
    Allocated(SpriteSlot),
    Full,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteDraw {
    pub map: LegacySpriteMap,
    pub position: Point,
    pub palette: u16,
    pub base_tile: u16,
}
