//! Fireflea lighting transitions over the current numerical and colour owners.
#![forbid(unsafe_code)]

use crate::visual_state::FirefleaLighting;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Timer,
    FlashIndex,
    CompatibilityWord,
    Darkness,
    CompatibilityLimit,
    CompatibilityShade,
}
impl Field {
    pub const ALL: [Self; 6] = [
        Self::Timer,
        Self::FlashIndex,
        Self::CompatibilityWord,
        Self::Darkness,
        Self::CompatibilityLimit,
        Self::CompatibilityShade,
    ];
    pub fn get(self, value: &FirefleaLighting) -> u16 {
        match self {
            Self::Timer => value.timer,
            Self::FlashIndex => value.flash_index,
            Self::CompatibilityWord => value.compatibility_word,
            Self::Darkness => value.darkness,
            Self::CompatibilityLimit => value.compatibility_limit,
            Self::CompatibilityShade => value.compatibility_shade,
        }
    }
    pub fn get_mut(self, value: &mut FirefleaLighting) -> &mut u16 {
        match self {
            Self::Timer => &mut value.timer,
            Self::FlashIndex => &mut value.flash_index,
            Self::CompatibilityWord => &mut value.compatibility_word,
            Self::Darkness => &mut value.darkness,
            Self::CompatibilityLimit => &mut value.compatibility_limit,
            Self::CompatibilityShade => &mut value.compatibility_shade,
        }
    }
}
pub trait Access {
    fn lighting(&self) -> FirefleaLighting;
    fn publish_lighting(&mut self, value: FirefleaLighting, fields: &[Field]);
}

const FLASH: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
const DARKNESS: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];

pub fn darken(lighting: &mut FirefleaLighting) {
    if lighting.darkness.wrapping_sub(12) & 0x8000 != 0 {
        lighting.darkness = lighting.darkness.wrapping_add(2);
    }
}

/// These two publications precede creation of the lighting's HDMA object.
pub fn begin(lighting: &mut FirefleaLighting) {
    lighting.timer = 6;
    lighting.flash_index = 0;
}

/// The HDMA object and first table word have been published before this point.
pub fn finish_initialization(lighting: &mut FirefleaLighting) {
    lighting.compatibility_word = 0;
    lighting.darkness = 0;
    lighting.compatibility_limit = 24;
    lighting.compatibility_shade = FLASH[0];
}

/// Retain the full blending word and exact colour-channel publication order.
/// Table reads remain after timer/index publications, including their failure prefix.
pub fn advance(
    lighting: &mut FirefleaLighting,
    blending: &mut u16,
    fixed_color: &mut [u8; 3],
    frozen: bool,
) {
    *blending = (*blending & 0xff00) | 0xc;
    if frozen {
        return;
    }
    lighting.timer = lighting.timer.wrapping_sub(1);
    if lighting.timer == 0 {
        lighting.timer = 6;
        lighting.flash_index = if lighting.darkness.wrapping_sub(10) & 0x8000 != 0 {
            let next = lighting.flash_index.wrapping_add(1);
            if next >= 12 { 0 } else { next }
        } else {
            6
        };
    }
    let shade = (DARKNESS[usize::from(lighting.darkness >> 1)]
        .wrapping_add(FLASH[usize::from(lighting.flash_index)])
        >> 8)
        .to_le_bytes()[0];
    fixed_color[1] = shade | 0x80;
    fixed_color[2] = shade | 0x40;
    fixed_color[0] = shade | 0x20;
}

/// Current source-interval transitions preserve each publication before a later
/// table failure. The copied record is a transition draft, never another owner.
pub fn darken_current(host: &mut (impl Access + ?Sized)) {
    let mut value = host.lighting();
    if value.darkness.wrapping_sub(12) & 0x8000 != 0 {
        darken(&mut value);
        host.publish_lighting(value, &[Field::Darkness]);
    }
}
pub fn begin_current(host: &mut (impl Access + ?Sized)) {
    let mut value = host.lighting();
    begin(&mut value);
    host.publish_lighting(value, &[Field::Timer, Field::FlashIndex]);
}
pub fn finish_current(host: &mut (impl Access + ?Sized)) {
    let mut value = host.lighting();
    finish_initialization(&mut value);
    host.publish_lighting(
        value,
        &[
            Field::CompatibilityWord,
            Field::Darkness,
            Field::CompatibilityLimit,
            Field::CompatibilityShade,
        ],
    );
}
pub fn advance_after_blending<H>(host: &mut H, frozen: bool)
where
    H: Access + crate::display::preparation::Access + ?Sized,
{
    use crate::display::preparation::Field as Color;
    if frozen {
        return;
    }
    let mut value = host.lighting();
    value.timer = value.timer.wrapping_sub(1);
    host.publish_lighting(value, &[Field::Timer]);
    if value.timer == 0 {
        value.timer = 6;
        host.publish_lighting(value, &[Field::Timer]);
        value.flash_index = if value.darkness.wrapping_sub(10) & 0x8000 != 0 {
            let next = value.flash_index.wrapping_add(1);
            if next >= 12 { 0 } else { next }
        } else {
            6
        };
        host.publish_lighting(value, &[Field::FlashIndex]);
    }
    let shade = (DARKNESS[usize::from(value.darkness >> 1)]
        .wrapping_add(FLASH[usize::from(value.flash_index)])
        >> 8)
        .to_le_bytes()[0];
    host.store_display_control_byte(
        Color::FixedColor1,
        shade | 0x80,
        crate::display::preparation::Store::Named,
    );
    host.store_display_control_byte(
        Color::FixedColor2,
        shade | 0x40,
        crate::display::preparation::Store::Named,
    );
    host.store_display_control_byte(
        Color::FixedColor0,
        shade | 0x20,
        crate::display::preparation::Store::Named,
    );
}
