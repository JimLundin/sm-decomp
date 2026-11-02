//! Actual Torizo palette transfers on the prepared presentation owner.
#![forbid(unsafe_code)]
use crate::actors::remaining_owned::Core;
const COMMON_LOW: [u16; 16] = [
    0x3800, 0x3ff, 0x33b, 0x216, 0x113, 0x6b1e, 0x4a16, 0x3591, 0x20e9, 0x1580, 0x1580, 0x1580,
    0x1580, 0x1580, 0x1580, 0x1580,
];
const COMMON_HIGH: [u16; 16] = [
    0x3800, 0x2df, 0x1d7, 0xac, 0x5a73, 0x41ad, 0x2d08, 0x1863, 0x1486, 0x145, 0x145, 0x145,
    0x7fff, 0x145, 0x145, 0,
];
const BOMB_START_LOW: [u16; 16] = [
    0x3800, 0x679f, 0x5299, 0x252e, 0x14aa, 0x5efc, 0x4657, 0x35b2, 0x2d70, 0x5b7f, 0x3df8, 0x2d0e,
    0x5f5f, 0x5e1a, 0x5d35, 0xc63,
];
const BOMB_START_HIGH: [u16; 16] = [
    0x3800, 0x4aba, 0x35b2, 0x847, 3, 0x4215, 0x2970, 0x18cb, 0x1089, 0x463a, 0x28b3, 0x1809,
    0x6f7f, 0x51fd, 0x4113, 0xc63,
];
const BOMB_ACTIVE_LOW: [u16; 16] = [
    0x3800, 0x56ba, 0x41b2, 0x1447, 0x403, 0x4e15, 0x3570, 0x24cb, 0x1868, 0x6f7f, 0x51f8, 0x410e,
    0x31f, 0x1da, 0xf5, 0xc63,
];
const BOMB_ACTIVE_HIGH: [u16; 16] = [
    0x3800, 0x4215, 0x2d0d, 2, 0, 0x3970, 0x20cb, 0xc26, 0x403, 0x463a, 0x28b3, 0x1809, 0x6f7f,
    0x51fd, 0x4113, 0xc63,
];
const GOLDEN_START_LOW: [u16; 16] = [
    0x3800, 0x6ab5, 0x49b0, 0x1c45, 0xc01, 0x5613, 0x416d, 0x2cc9, 0x2066, 0x5714, 0x31cc, 0x14e3,
    0x5630, 0x3569, 0x1883, 0xc66,
];
const GOLDEN_START_HIGH: [u16; 16] = [
    0x3800, 0x5610, 0x350b, 0x800, 0, 0x416e, 0x2cc8, 0x1823, 0xc01, 0x6a31, 0x4caa, 0x2406,
    0x7f7b, 0x75f4, 0x4d10, 0xc63,
];
const GOLDEN_ACTIVE_LOW: [u16; 16] = [
    0x3800, 0x4bbe, 0x6b9, 0xa8, 0, 0x173a, 0x276, 0x1f2, 0x14d, 0x73e0, 0x4f20, 0x2a20, 0x7fe0,
    0x5aa0, 0x5920, 0x43,
];
const GOLDEN_ACTIVE_HIGH: [u16; 16] = [
    0x3800, 0x3719, 0x214, 3, 0, 0x295, 0x1d1, 0x14d, 0xa8, 0x4b40, 0x25e0, 0xe0, 0x6b40, 0x4600,
    0x4480, 0,
];

pub type Pair = [&'static [u16; 16]; 2];
const COMMON: Pair = [&COMMON_HIGH, &COMMON_LOW];
pub const BOMB_START: Pair = [&BOMB_START_HIGH, &BOMB_START_LOW];
pub const BOMB_ACTIVE: Pair = [&BOMB_ACTIVE_HIGH, &BOMB_ACTIVE_LOW];
pub const GOLDEN_START: Pair = [&GOLDEN_START_HIGH, &GOLDEN_START_LOW];
pub const GOLDEN_ACTIVE: Pair = [&GOLDEN_ACTIVE_HIGH, &GOLDEN_ACTIVE_LOW];
const WHITE: [u16; 16] = [0x7fff; 16];

pub fn copy(palettes: Pair, starts: [usize; 2], target: bool, host: &mut impl Core) {
    // The two palette writes interleave in descending color order.
    for (index, (high, low)) in palettes[0].iter().zip(palettes[1]).enumerate().rev() {
        for (start, color) in starts.into_iter().zip([*high, *low]) {
            let visual = host.session_mut().presentation.visual_mut();
            if target {
                visual.targets[start + index] = color;
            } else {
                visual.colors[start + index] = color;
            }
        }
    }
}
pub fn initialize(host: &mut impl Core) {
    copy(COMMON, [240, 176], true, host);
}
pub fn flash(host: &mut impl Core) {
    copy([&WHITE; 2], [160, 144], false, host);
}
pub fn restore(host: &mut impl Core) {
    copy(BOMB_ACTIVE, [160, 144], false, host);
}
pub fn health(health: u16, host: &mut impl Core) {
    let band = isize::from((health.to_ne_bytes()[1] & 0x78).min(0x38)) * 2;
    for index in (0u16..16).rev() {
        let color = crate::gameplay_tables::torizo_palette_colour(
            host,
            true,
            band + isize::try_from(index).expect("color index"),
        );
        host.session_mut().presentation.visual_mut().colors[usize::from(index) + 160] = color;
        let color = crate::gameplay_tables::torizo_palette_colour(
            host,
            false,
            band + isize::try_from(index).expect("color index"),
        );
        host.session_mut().presentation.visual_mut().colors[usize::from(index) + 144] = color;
    }
}

fn transition(step: u16, from: u16, to: u16, duration: u16) -> u16 {
    [0, 5, 10].into_iter().fold(0, |color, shift| {
        let from = (from >> shift) & 31;
        let to = (to >> shift) & 31;
        let component = if step == 0 {
            from
        } else if step.wrapping_sub(1) == duration {
            to
        } else {
            let remaining = duration.wrapping_sub(step).wrapping_add(1).to_le_bytes()[0];
            let distance = from.abs_diff(to) << 8;
            let delta = if remaining == 0 {
                0xffff
            } else {
                distance / u16::from(remaining)
            };
            (if to < from {
                (from << 8).wrapping_sub(delta)
            } else {
                (from << 8).wrapping_add(delta)
            }) >> 8
        };
        color | (component << shift)
    })
}
pub fn fade(mut mask: u16, host: &mut impl Core) {
    // The original fade copy is committed after the ordered color effects.
    let mut fade = host.session().presentation.visual().palette_fade;
    fade.duration = 12;
    if fade.step > fade.duration.wrapping_add(1) {
        fade.step = 0;
    } else {
        fade.cursor = 0;
        while mask != 0 {
            if mask & 1 == 0 {
                fade.cursor = fade.cursor.wrapping_add(32);
            } else {
                loop {
                    let index = usize::from(fade.cursor);
                    let current = host.remaining_color(false, index, false);
                    let target = host.remaining_color(true, index, false);
                    if current != target {
                        let color = transition(fade.step, current, target, fade.duration);
                        host.set_remaining_color(false, index, color, true);
                    }
                    fade.cursor = fade.cursor.wrapping_add(2);
                    if fade.cursor & 31 == 0 {
                        break;
                    }
                }
            }
            mask >>= 1;
        }
        fade.step = fade.step.wrapping_add(1);
    }
    host.session_mut()
        .presentation
        .visual_mut()
        .palette_fade
        .step = fade.step;
    host.session_mut()
        .presentation
        .visual_mut()
        .palette_fade
        .duration = fade.duration;
    host.session_mut()
        .presentation
        .visual_mut()
        .palette_fade
        .cursor = fade.cursor;
}
