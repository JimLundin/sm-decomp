//! The same behaviour as `plain.rs`, under the K5 and K8 lint set.
//!
//! The maths lives in one `Fixed` type whose operators state their overflow once.

/// A position or speed in pixels and 1/65536ths of a pixel. Positions wrap at 16 bits of
/// pixels, as the original's do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(u32);

impl Fixed {
    #[must_use]
    pub const fn new(pixels: u16, subpixels: u16) -> Self {
        let [high, low] = pixels.to_be_bytes();
        let [sub_high, sub_low] = subpixels.to_be_bytes();
        Self(u32::from_be_bytes([high, low, sub_high, sub_low]))
    }
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
    #[must_use]
    pub const fn pixels(self) -> u16 {
        let [high, low, _, _] = self.0.to_be_bytes();
        u16::from_be_bytes([high, low])
    }
    #[must_use]
    pub const fn with_pixels(self, pixels: u16) -> Self {
        let [_, _, sub_high, sub_low] = self.0.to_be_bytes();
        let [high, low] = pixels.to_be_bytes();
        Self(u32::from_be_bytes([high, low, sub_high, sub_low]))
    }
    #[must_use]
    pub const fn wrapping_add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
    #[must_use]
    pub const fn wrapping_sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

pub const WALK_ACCEL: Fixed = Fixed::new(0, 0x3000);
pub const RUN_ACCEL: Fixed = Fixed::new(0, 0x4000);
pub const WALK_MAX: Fixed = Fixed::new(2, 0);
pub const RUN_MAX: Fixed = Fixed::new(3, 0);
pub const DECEL: Fixed = Fixed::new(0, 0x2000);
pub const GRAVITY: Fixed = Fixed::new(0, 0x1C00);
pub const TERMINAL: Fixed = Fixed::new(5, 0);
pub const JUMP: Fixed = Fixed::new(4, 0xE000);
pub const HALF_HEIGHT: u16 = 0x14;

/// A column inside a 16-pixel block, valid by construction.
#[derive(Clone, Copy, Debug)]
pub struct Column(u8);
impl Column {
    #[must_use]
    pub const fn of(x: u16) -> Self {
        let [_, low] = x.to_be_bytes();
        Self(low & 0xF)
    }
    #[must_use]
    pub const fn mirrored(self) -> Self {
        Self(self.0 ^ 0xF)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slope {
    Steep,
    Gentle,
    GentleLow,
    Flat,
}
impl Slope {
    const HEIGHTS: [[u8; 16]; 4] = [
        [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
        [15, 15, 14, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8],
        [7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0],
        [8; 16],
    ];
    fn height(self, column: Column) -> Option<u8> {
        let [steep, gentle, gentle_low, flat] = &Self::HEIGHTS;
        let row = match self {
            Self::Steep => steep,
            Self::Gentle => gentle,
            Self::GentleLow => gentle_low,
            Self::Flat => flat,
        };
        row.get(usize::from(column.0)).copied()
    }
}
impl TryFrom<u8> for Slope {
    type Error = u8;
    fn try_from(shape: u8) -> Result<Self, u8> {
        match shape {
            0 => Ok(Self::Steep),
            1 => Ok(Self::Gentle),
            2 => Ok(Self::GentleLow),
            3 => Ok(Self::Flat),
            other => Err(other),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Vertical {
    #[default]
    Grounded,
    Rising(Fixed),
    Falling(Fixed),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Samus {
    pub x: Fixed,
    pub y: Fixed,
    pub x_speed: Fixed,
    pub facing: Facing,
    pub vertical: Vertical,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Held {
    pub direction: Option<Facing>,
    pub run: bool,
    pub jump: bool,
}

pub struct Level {
    pub width: u16,
    pub blocks: Vec<u16>,
    pub bts: Vec<u8>,
}
impl Level {
    fn slope_at(&self, column: u16, row: u16) -> Option<(Slope, bool)> {
        if column >= self.width {
            return None;
        }
        let index = usize::from(row).checked_mul(usize::from(self.width))?.checked_add(usize::from(column))?;
        let [kind, _] = self.blocks.get(index)?.to_be_bytes();
        if kind >> 4 != 1 {
            return None;
        }
        let bts = *self.bts.get(index)?;
        Slope::try_from(bts & 0x1F).ok().map(|slope| (slope, bts & 0x40 != 0))
    }
}

pub fn step(samus: &mut Samus, level: &Level, held: Held) {
    if let Some(want) = held.direction {
        if want != samus.facing {
            samus.facing = want;
            samus.x_speed = Fixed::default();
        }
        let (accel, max) = if held.run { (RUN_ACCEL, RUN_MAX) } else { (WALK_ACCEL, WALK_MAX) };
        samus.x_speed = samus.x_speed.wrapping_add(accel).min(max);
    } else {
        samus.x_speed = samus.x_speed.saturating_sub(DECEL);
    }
    samus.x = match samus.facing {
        Facing::Left => samus.x.wrapping_sub(samus.x_speed),
        Facing::Right => samus.x.wrapping_add(samus.x_speed),
    };

    if held.jump && samus.vertical == Vertical::Grounded {
        samus.vertical = Vertical::Rising(JUMP);
    }
    samus.vertical = match samus.vertical {
        Vertical::Grounded => Vertical::Grounded,
        Vertical::Rising(speed) if speed <= GRAVITY => Vertical::Falling(Fixed::default()),
        Vertical::Rising(speed) => {
            let speed = speed.wrapping_sub(GRAVITY);
            samus.y = samus.y.wrapping_sub(speed);
            Vertical::Rising(speed)
        }
        Vertical::Falling(speed) => {
            let speed = speed.wrapping_add(GRAVITY).min(TERMINAL);
            samus.y = samus.y.wrapping_add(speed);
            Vertical::Falling(speed)
        }
    };
    land_on_slope(samus, level);
}

fn land_on_slope(samus: &mut Samus, level: &Level) {
    if matches!(samus.vertical, Vertical::Rising(_)) {
        return;
    }
    let x = samus.x.pixels();
    let bottom = samus.y.pixels().wrapping_add(HALF_HEIGHT);
    let Some((slope, mirrored)) = level.slope_at(x >> 4, bottom >> 4) else {
        return;
    };
    let column = if mirrored { Column::of(x).mirrored() } else { Column::of(x) };
    let Some(height) = slope.height(column) else {
        return;
    };
    let floor = (bottom & !0xF) | u16::from(height);
    if bottom >= floor {
        samus.y = samus.y.with_pixels(samus.y.pixels().wrapping_sub(bottom.wrapping_sub(floor)));
        samus.vertical = Vertical::Grounded;
    }
}
