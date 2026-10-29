//! Display state the game sets directly: brightness, the fixed colour and the
//! Mode 7 transform. Compat writes each value to its register at the call
//! site's original point, so values reach the registers in the game's order.

/// Forced blank and master brightness (INIDISP).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Brightness {
    pub forced_blank: bool,
    /// 0 (black) to 15 (full).
    pub level: u8,
}

impl Brightness {
    pub const BLANK_FULL: Self = Self {
        forced_blank: true,
        level: 15,
    };
    pub const BLANK_DARK: Self = Self {
        forced_blank: true,
        level: 0,
    };
    pub const FULL: Self = Self {
        forced_blank: false,
        level: 15,
    };

    pub const fn byte(self) -> u8 {
        (self.forced_blank as u8) << 7 | self.level & 0xf
    }
}

/// Sets the selected fixed-colour channels to one intensity (COLDATA).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedColor {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    /// 0 to 31.
    pub intensity: u8,
}

impl FixedColor {
    const fn channels(red: bool, green: bool, blue: bool, intensity: u8) -> Self {
        Self {
            red,
            green,
            blue,
            intensity,
        }
    }
    /// All three channels at `intensity`.
    pub const fn all(intensity: u8) -> Self {
        Self::channels(true, true, true, intensity)
    }
    pub const fn red(intensity: u8) -> Self {
        Self::channels(true, false, false, intensity)
    }
    pub const fn green(intensity: u8) -> Self {
        Self::channels(false, true, false, intensity)
    }
    pub const fn blue(intensity: u8) -> Self {
        Self::channels(false, false, true, intensity)
    }

    pub const fn byte(self) -> u8 {
        (self.blue as u8) << 7
            | (self.green as u8) << 6
            | (self.red as u8) << 5
            | self.intensity & 0x1f
    }
}

/// Which backgrounds and sprites a screen or window mask includes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layers {
    pub bg1: bool,
    pub bg2: bool,
    pub bg3: bool,
    pub bg4: bool,
    pub sprites: bool,
}

impl Layers {
    pub const NONE: Self = Self::of(false, false, false, false, false);

    pub const fn of(bg1: bool, bg2: bool, bg3: bool, bg4: bool, sprites: bool) -> Self {
        Self {
            bg1,
            bg2,
            bg3,
            bg4,
            sprites,
        }
    }

    pub const fn byte(self) -> u8 {
        self.bg1 as u8
            | (self.bg2 as u8) << 1
            | (self.bg3 as u8) << 2
            | (self.bg4 as u8) << 3
            | (self.sprites as u8) << 4
    }
}

/// The layers drawn on the main screen (TM).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainScreen(pub Layers);
/// The layers drawn on the subscreen (TS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubScreen(pub Layers);
/// The main-screen layers the windows mask (TMW).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainWindowMask(pub Layers);
/// The subscreen layers the windows mask (TSW).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubWindowMask(pub Layers);

/// The background mode and per-layer tile size (BGMODE).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BgMode {
    /// 0 to 7.
    pub mode: u8,
    /// Mode 1 only: BG3 drawn above everything else.
    pub bg3_priority: bool,
    /// 16x16 tiles for BG1..BG4.
    pub large_tiles: [bool; 4],
}

impl BgMode {
    /// Mode 1 with BG3 on top: gameplay, menus and the HUD.
    pub const MODE1_BG3_PRIORITY: Self = Self {
        mode: 1,
        bg3_priority: true,
        large_tiles: [false; 4],
    };

    pub const fn byte(self) -> u8 {
        self.mode & 7
            | (self.bg3_priority as u8) << 3
            | (self.large_tiles[0] as u8) << 4
            | (self.large_tiles[1] as u8) << 5
            | (self.large_tiles[2] as u8) << 6
            | (self.large_tiles[3] as u8) << 7
    }
}

/// Interlace, overscan, pseudo-hires and external-sync flags (SETINI).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScreenSettings {
    pub interlace: bool,
    pub sprite_interlace: bool,
    pub overscan: bool,
    pub pseudo_hires: bool,
    pub extended_bg: bool,
    pub external_sync: bool,
}

impl ScreenSettings {
    /// Progressive 224-line output, as the game always uses.
    pub const STANDARD: Self = Self {
        interlace: false,
        sprite_interlace: false,
        overscan: false,
        pseudo_hires: false,
        extended_bg: false,
        external_sync: false,
    };

    pub const fn byte(self) -> u8 {
        self.interlace as u8
            | (self.sprite_interlace as u8) << 1
            | (self.overscan as u8) << 2
            | (self.pseudo_hires as u8) << 3
            | (self.extended_bg as u8) << 6
            | (self.external_sync as u8) << 7
    }
}

/// Mode 7 flips and what shows outside the 1024x1024 field (M7SEL).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mode7Settings {
    pub flip_x: bool,
    pub flip_y: bool,
    /// 0 wraps; 2 shows transparency; 3 repeats tile 0.
    pub outside: u8,
}

impl Mode7Settings {
    pub const WRAP: Self = Self {
        flip_x: false,
        flip_y: false,
        outside: 0,
    };

    pub const fn byte(self) -> u8 {
        self.flip_x as u8 | (self.flip_y as u8) << 1 | (self.outside & 3) << 6
    }
}

/// The Mode 7 matrix (8.8 fixed point) and centre: M7A..M7D, M7X, M7Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode7Transform {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub center_x: u16,
    pub center_y: u16,
}

impl Mode7Transform {
    /// No rotation or scaling, centred at (`center_x`, `center_y`).
    pub const fn identity(center_x: u16, center_y: u16) -> Self {
        Self {
            a: 0x100,
            b: 0,
            c: 0,
            d: 0x100,
            center_x,
            center_y,
        }
    }

    /// The six values in register order: A, B, C, D, X, Y.
    pub const fn words(self) -> [u16; 6] {
        [self.a, self.b, self.c, self.d, self.center_x, self.center_y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_match_the_original_constants() {
        assert_eq!(Brightness::BLANK_FULL.byte(), 0x8f);
        assert_eq!(Brightness::FULL.byte(), 0x0f);
        assert_eq!(Brightness::BLANK_DARK.byte(), 0x80);
        assert_eq!(BgMode::MODE1_BG3_PRIORITY.byte(), 9);
        assert_eq!(ScreenSettings::STANDARD.byte(), 0);
        assert_eq!(Mode7Settings::WRAP.byte(), 0);
        assert_eq!(Layers::of(true, false, false, false, true).byte(), 0x11);
        assert_eq!(Layers::of(false, true, false, false, false).byte(), 2);
        assert_eq!(Layers::of(false, false, true, false, false).byte(), 4);
        assert_eq!(Layers::of(true, true, false, false, true).byte(), 0x13);
        assert_eq!(Layers::of(true, true, true, false, true).byte(), 0x17);
        assert_eq!(FixedColor::all(0).byte(), 0xe0);
        assert_eq!(FixedColor::red(0).byte(), 0x20);
        assert_eq!(FixedColor::green(0).byte(), 0x40);
        assert_eq!(FixedColor::blue(0).byte(), 0x80);
        assert_eq!(
            Mode7Transform::identity(3, 4).words(),
            [256, 0, 0, 256, 3, 4]
        );
    }
}
