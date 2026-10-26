//! Shared liquid-surface precedence and SNES sign-bit comparisons.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiquidAtHeight {
    None,
    Water,
    LavaAcid,
}
pub(crate) fn at_height(water_y: u16, lava_y: u16, options: u16, y: u16) -> LiquidAtHeight {
    if water_y & 0x8000 == 0 {
        if water_y.wrapping_sub(y) & 0x8000 != 0 && options & 4 == 0 {
            return LiquidAtHeight::Water;
        }
    } else if lava_y & 0x8000 == 0 && lava_y.wrapping_sub(y) & 0x8000 != 0 {
        return LiquidAtHeight::LavaAcid;
    }
    LiquidAtHeight::None
}
