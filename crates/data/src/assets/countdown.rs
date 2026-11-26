//! The escape timer's word and digits in drawing order.
use crate::spritemap::Spritemap;

/// The timer uses a fixed word followed by six decimal digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Figures {
    /// The frame and letters around the digits.
    pub frame: Spritemap,
    /// Figures for zero through nine.
    pub digits: [Spritemap; 10],
}
