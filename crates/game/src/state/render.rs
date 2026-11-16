//! Pure render coordinates independent of mutable room gameplay.

/// A layer or background corner in 16-pixel blocks.
#[derive(canonical::Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockPoint {
    /// Column.
    pub x: u16,
    /// Row.
    pub y: u16,
}
