//! The screen brightness fade ($0723 frames per step, $0725 countdown), owned
//! by `Visuals::screen_fade`, read and stored through the host's projected
//! bytes with the same bytes and order as the named words they replace.
#![forbid(unsafe_code)]
use crate::projection;
use crate::ram::CompatibilityRam;
pub use crate::visual_state::ScreenFade;

const DELAY: usize = 0x723;
const COUNTER: usize = 0x725;

pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> ScreenFade {
    ScreenFade {
        delay: projection::word(h, DELAY),
        counter: projection::word(h, COUNTER),
    }
}

/// Store the delay, then the counter.
pub fn set_both<H: CompatibilityRam + ?Sized>(h: &mut H, delay: u16, counter: u16) {
    projection::set_word(h, DELAY, delay);
    set_counter(h, counter);
}

pub fn set_counter<H: CompatibilityRam + ?Sized>(h: &mut H, counter: u16) {
    projection::set_word(h, COUNTER, counter);
}

/// Store only the delay's (or with `counter`, the counter's) low byte, as the
/// 8-bit stores do.
pub fn set_low<H: CompatibilityRam + ?Sized>(h: &mut H, counter: bool, value: u8) {
    projection::set_byte(h, if counter { COUNTER } else { DELAY }, value);
}
