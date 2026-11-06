//! The register file a line is composed with, decoded once per change into typed settings.

use data::video::PpuRegisters;

/// Whether bit `mask` of `value` is set.
const fn bit(value: u8, mask: u8) -> bool {
    value & mask != 0
}

/// The register file as the line program changes it, with the write latches the paired
/// registers share.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegisterFile {
    pub registers: PpuRegisters,
    /// The byte the last scroll write left, the low byte of the next one.
    scroll_latch: u8,
    /// The byte the last horizontal scroll write left, which supplies the low three bits.
    horizontal_latch: u8,
    /// The byte the last Mode 7 write left.
    affine_latch: u8,
}

impl RegisterFile {
    /// The file at a frame's start, its latches cleared.
    pub fn new(registers: PpuRegisters) -> Self {
        Self {
            registers,
            ..Self::default()
        }
    }

    /// Apply one write to the port `port` ($21xx's low byte), as the PPU latches it.
    /// Ports the picture doesn't read are ignored.
    pub fn write(&mut self, port: u8, value: u8) {
        let registers = &mut self.registers;
        let high = u16::from(value) << 8;
        match port {
            0x00 => registers.inidisp = value,
            0x01 => registers.obsel = value,
            0x05 => registers.bgmode = value,
            0x06 => registers.mosaic = value,
            0x07..=0x0a => {
                if let Some(slot) = registers
                    .bg_sc
                    .get_mut(usize::from(port.wrapping_sub(0x07)))
                {
                    *slot = value;
                }
            }
            0x0b => registers.bg12nba = value,
            0x0c => registers.bg34nba = value,
            0x0d..=0x14 => {
                let offset = port.wrapping_sub(0x0d);
                let horizontal = offset & 1 == 0;
                if offset < 2 {
                    let word = (high | u16::from(self.affine_latch)) & 0x1fff;
                    self.affine_latch = value;
                    if let Some(slot) = registers.m7_scroll.get_mut(usize::from(offset)) {
                        *slot = word;
                    }
                }
                let scroll = if horizontal {
                    let low = (self.scroll_latch & 0xf8) | (self.horizontal_latch & 7);
                    self.horizontal_latch = value;
                    high | u16::from(low)
                } else {
                    high | u16::from(self.scroll_latch)
                };
                self.scroll_latch = value;
                let layer = registers.bg_scroll.get_mut(usize::from(offset >> 1));
                if let Some(slot) = layer.and_then(|pair| pair.get_mut(usize::from(!horizontal))) {
                    *slot = scroll & 0x3ff;
                }
            }
            0x1a => registers.m7sel = value,
            0x1b..=0x20 => {
                let word = i16::from_le_bytes([self.affine_latch, value]);
                self.affine_latch = value;
                let index = usize::from(port.wrapping_sub(0x1b));
                // M7X and M7Y keep 13 bits.
                let word = if index >= 4 { word & 0x1fff } else { word };
                if let Some(slot) = registers.m7.get_mut(index) {
                    *slot = word;
                }
            }
            0x23..=0x25 => {
                if let Some(slot) = registers
                    .window_select
                    .get_mut(usize::from(port.wrapping_sub(0x23)))
                {
                    *slot = value;
                }
            }
            0x26..=0x29 => {
                if let Some(slot) = registers
                    .window_positions
                    .get_mut(usize::from(port.wrapping_sub(0x26)))
                {
                    *slot = value;
                }
            }
            0x2a => registers.wbglog = value,
            0x2b => registers.wobjlog = value,
            0x2c..=0x2f => {
                if let Some(slot) = registers
                    .screens
                    .get_mut(usize::from(port.wrapping_sub(0x2c)))
                {
                    *slot = value;
                }
            }
            0x30 => registers.cgwsel = value,
            0x31 => registers.cgadsub = value,
            0x32 => {
                // Each set select bit takes the component: blue, green, red.
                let [red, green, blue] = &mut registers.fixed_color;
                for (select, component) in [(0x80, blue), (0x40, green), (0x20, red)] {
                    if bit(value, select) {
                        *component = value & 0x1f;
                    }
                }
            }
            0x33 => registers.setini = value,
            _ => {}
        }
    }
}

/// A background mode with the variant bits that change its layer order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Zero,
    /// BGMODE bit 3 puts BG3's high-priority tiles in front of everything.
    One {
        bg3_front: bool,
    },
    Two,
    Three,
    Four,
    Five,
    Six,
    /// SETINI's EXTBG makes BG2 the Mode 7 map's low seven bits, bit 7 its priority.
    Seven {
        extended: bool,
    },
}

impl Mode {
    const fn new(bgmode: u8, setini: u8) -> Self {
        match bgmode & 7 {
            0 => Self::Zero,
            1 => Self::One {
                bg3_front: bit(bgmode, 8),
            },
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            5 => Self::Five,
            6 => Self::Six,
            _ => Self::Seven {
                extended: bit(setini, 0x40),
            },
        }
    }

    /// Bits per pixel of each background, `None` where the mode has no such layer.
    pub const fn depths(self) -> [Option<Depth>; 4] {
        use Depth::{Eight, Four, Seven, Two};
        match self {
            Self::Zero => [Some(Two), Some(Two), Some(Two), Some(Two)],
            Self::One { .. } => [Some(Four), Some(Four), Some(Two), None],
            Self::Two => [Some(Four), Some(Four), None, None],
            Self::Three => [Some(Eight), Some(Four), None, None],
            Self::Four => [Some(Eight), Some(Two), None, None],
            Self::Five => [Some(Four), Some(Two), None, None],
            Self::Six => [Some(Four), None, None, None],
            Self::Seven { extended: false } => [Some(Eight), None, None, None],
            Self::Seven { extended: true } => [Some(Eight), Some(Seven), None, None],
        }
    }

    /// Whether the mode samples BG3's offset-per-tile map.
    pub const fn offset_per_tile(self) -> bool {
        matches!(self, Self::Two | Self::Four | Self::Six)
    }

    /// Whether the mode draws 512 dots a line.
    pub const fn hires(self) -> bool {
        matches!(self, Self::Five | Self::Six)
    }
}

/// A background's bits per pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Two,
    Four,
    /// Mode 7 EXTBG's BG2.
    Seven,
    Eight,
}

impl Depth {
    /// The 2-bit plane pairs a tile row has.
    pub const fn plane_pairs(self) -> u16 {
        match self {
            Self::Two => 1,
            Self::Four => 2,
            Self::Seven | Self::Eight => 4,
        }
    }
}

/// Where a window-gated effect applies (CGWSEL's clip and prevent fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Nowhere,
    OutsideWindow,
    InsideWindow,
    Everywhere,
}

impl Region {
    const fn new(field: u8) -> Self {
        match field & 3 {
            0 => Self::Nowhere,
            1 => Self::OutsideWindow,
            2 => Self::InsideWindow,
            _ => Self::Everywhere,
        }
    }

    pub const fn contains(self, window: bool) -> bool {
        match self {
            Self::Nowhere => false,
            Self::OutsideWindow => !window,
            Self::InsideWindow => window,
            Self::Everywhere => true,
        }
    }
}

/// How a layer's two windows combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowLogic {
    Or,
    And,
    Xor,
    Xnor,
}

/// One window's use by a layer: off, or on with its sense.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowUse {
    Off,
    Inside,
    Outside,
}

impl WindowUse {
    const fn new(nibble: u8) -> Self {
        match (bit(nibble, 2), bit(nibble, 1)) {
            (false, _) => Self::Off,
            (true, false) => Self::Inside,
            (true, true) => Self::Outside,
        }
    }
}

/// A layer's window mask: background 1-4, sprites, or the colour window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerWindows {
    pub first: WindowUse,
    pub second: WindowUse,
    pub logic: WindowLogic,
}

/// The two windows' spans and each layer's mask over them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windows {
    pub first: (u8, u8),
    pub second: (u8, u8),
    pub layers: [LayerWindows; 6],
}

impl Windows {
    /// Whether layer `layer`'s mask covers dot `x`.
    pub fn contains(&self, layer: Plane, x: u8) -> bool {
        let Some(mask) = self.layers.get(layer.index()) else {
            return false;
        };
        let test = |use_: WindowUse, (left, right): (u8, u8)| match use_ {
            WindowUse::Off => None,
            WindowUse::Inside => Some((left..=right).contains(&x)),
            WindowUse::Outside => Some(!(left..=right).contains(&x)),
        };
        match (test(mask.first, self.first), test(mask.second, self.second)) {
            (None, None) => false,
            (Some(one), None) | (None, Some(one)) => one,
            (Some(one), Some(two)) => match mask.logic {
                WindowLogic::Or => one || two,
                WindowLogic::And => one && two,
                WindowLogic::Xor => one != two,
                WindowLogic::Xnor => one == two,
            },
        }
    }
}

/// A plane of the picture: the four backgrounds, sprites, the colour window's slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plane {
    Background(Bg),
    Sprites,
    Colour,
}

impl Plane {
    /// The plane's bit in the screen designations and its slot in the window masks.
    pub const fn index(self) -> usize {
        match self {
            Self::Background(bg) => bg.index(),
            Self::Sprites => 4,
            Self::Colour => 5,
        }
    }
}

/// One of the four backgrounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bg {
    One,
    Two,
    Three,
    Four,
}

impl Bg {
    pub const ALL: [Self; 4] = [Self::One, Self::Two, Self::Three, Self::Four];

    pub const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }
}

/// A background's maps, tiles and scroll.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Background {
    pub map: u16,
    pub size: MapSize,
    pub tiles: u16,
    pub horizontal: u16,
    pub vertical: u16,
    pub big_tiles: bool,
    pub mosaic: bool,
}

/// Whether a background's map is two screens wide and two high.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapSize {
    pub wider: bool,
    pub higher: bool,
}

/// What colour math adds to or subtracts from the main screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    Fixed,
    Subscreen,
}

/// What the Mode 7 map shows past its 1024-dot square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outside {
    /// The map repeats.
    Wrap,
    Transparent,
    /// Tile 0's dots.
    TileZero,
}

/// A mirror on each axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flip {
    pub horizontal: bool,
    pub vertical: bool,
}

/// The planes a screen shows and the planes its window masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Screen {
    pub enabled: u8,
    pub windowed: u8,
}

impl Screen {
    /// Whether the screen shows `plane` at all.
    pub const fn enables(self, plane: Plane) -> bool {
        bit(self.enabled, 1 << plane.index())
    }

    /// Whether the screen masks `plane` with its window.
    pub const fn windows(self, plane: Plane) -> bool {
        bit(self.windowed, 1 << plane.index())
    }
}

/// Colour math as CGWSEL and CGADSUB set it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColourMath {
    pub clip: Region,
    pub prevent: Region,
    pub operand: Operand,
    pub direct_colour: bool,
    pub subtract: bool,
    pub half: bool,
    /// Per layer slot: backgrounds 1-4, sprites from palettes 4-7, the backdrop.
    pub enabled: [bool; 6],
    /// Red, green, blue.
    pub fixed: [u8; 3],
}

/// The Mode 7 transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affine {
    /// A, B, C, D.
    pub matrix: [i16; 4],
    /// The centre X and Y, 13-bit signed.
    pub centre: [i32; 2],
    /// M7HOFS, M7VOFS, 13-bit signed.
    pub scroll: [i32; 2],
    pub outside: Outside,
    pub flip: Flip,
}

/// Sprite sizes and tile bases (OBSEL).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteSetup {
    /// The small and large size, in dots.
    pub sizes: [u8; 2],
    /// The tile base words of name tables 0 and 1.
    pub bases: [u16; 2],
    pub interlace: bool,
}

/// Everything one line's composition reads, decoded from the register file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineSetup {
    pub forced_blank: bool,
    pub brightness: u8,
    pub mode: Mode,
    pub backgrounds: [Background; 4],
    pub mosaic_size: u8,
    pub windows: Windows,
    pub main: Screen,
    pub sub: Screen,
    pub math: ColourMath,
    pub affine: Affine,
    pub sprites: SpriteSetup,
    pub interlace: bool,
    pub pseudo_hires: bool,
}

/// A 13-bit signed register value.
fn signed13(word: u16) -> i32 {
    i32::from(word.cast_signed().wrapping_shl(3) >> 3)
}

/// The window spans and each layer's mask.
fn windows(r: &PpuRegisters) -> Windows {
    let selects = r.window_select;
    let nibbles = selects
        .iter()
        .flat_map(|&select| [select & 15, select >> 4]);
    let logic = u16::from(r.wbglog) | (u16::from(r.wobjlog) << 8);
    let mut layers = [LayerWindows {
        first: WindowUse::Off,
        second: WindowUse::Off,
        logic: WindowLogic::Or,
    }; 6];
    for (slot, (layer, nibble)) in layers.iter_mut().zip(nibbles).enumerate() {
        let shift = u32::try_from(slot.wrapping_mul(2)).unwrap_or_default();
        layer.first = WindowUse::new(nibble);
        layer.second = WindowUse::new(nibble >> 2);
        layer.logic = match (logic >> shift) & 3 {
            0 => WindowLogic::Or,
            1 => WindowLogic::And,
            2 => WindowLogic::Xor,
            _ => WindowLogic::Xnor,
        };
    }
    let [w1l, w1r, w2l, w2r] = r.window_positions;
    Windows {
        first: (w1l, w1r),
        second: (w2l, w2r),
        layers,
    }
}

impl From<&RegisterFile> for LineSetup {
    fn from(file: &RegisterFile) -> Self {
        let r = &file.registers;
        let mode = Mode::new(r.bgmode, r.setini);
        let names = [
            r.bg12nba & 15,
            r.bg12nba >> 4,
            r.bg34nba & 15,
            r.bg34nba >> 4,
        ];
        let backgrounds = core::array::from_fn(|index| {
            let sc = r.bg_sc.get(index).copied().unwrap_or_default();
            let [horizontal, vertical] = r.bg_scroll.get(index).copied().unwrap_or_default();
            let shift = u32::try_from(index).unwrap_or_default();
            Background {
                map: u16::from(sc & 0xfc) << 8,
                size: MapSize {
                    wider: bit(sc, 1),
                    higher: bit(sc, 2),
                },
                tiles: u16::from(names.get(index).copied().unwrap_or_default()) << 12,
                horizontal,
                vertical,
                big_tiles: bit(r.bgmode, 0x10_u8.wrapping_shl(shift)),
                mosaic: bit(r.mosaic, 1_u8.wrapping_shl(shift)),
            }
        });
        let [tm, ts, tmw, tsw] = r.screens;
        let [m7a, m7b, m7c, m7d, m7x, m7y] = r.m7;
        let [m7h, m7v] = r.m7_scroll;
        let sizes = match r.obsel >> 5 {
            0 => [8, 16],
            1 => [8, 32],
            2 => [8, 64],
            4 => [16, 64],
            5 => [32, 64],
            // 3, and the two undocumented sizes 6 and 7 as LakeSnes draws them.
            _ => [16, 32],
        };
        let first_base = u16::from(r.obsel & 7) << 13;
        let gap = u16::from((r.obsel & 0x18).wrapping_add(8)) << 9;
        Self {
            forced_blank: bit(r.inidisp, 0x80),
            brightness: r.inidisp & 15,
            mode,
            backgrounds,
            mosaic_size: (r.mosaic >> 4).wrapping_add(1),
            windows: windows(r),
            main: Screen {
                enabled: tm,
                windowed: tmw,
            },
            sub: Screen {
                enabled: ts,
                windowed: tsw,
            },
            math: ColourMath {
                clip: Region::new(r.cgwsel >> 6),
                prevent: Region::new(r.cgwsel >> 4),
                operand: if bit(r.cgwsel, 2) {
                    Operand::Subscreen
                } else {
                    Operand::Fixed
                },
                direct_colour: bit(r.cgwsel, 1),
                subtract: bit(r.cgadsub, 0x80),
                half: bit(r.cgadsub, 0x40),
                enabled: core::array::from_fn(|slot| {
                    bit(
                        r.cgadsub,
                        1_u8.wrapping_shl(u32::try_from(slot).unwrap_or_default()),
                    )
                }),
                fixed: r.fixed_color,
            },
            affine: Affine {
                matrix: [m7a, m7b, m7c, m7d],
                centre: [signed13(m7x.cast_unsigned()), signed13(m7y.cast_unsigned())],
                scroll: [signed13(m7h), signed13(m7v)],
                outside: match (bit(r.m7sel, 0x80), bit(r.m7sel, 0x40)) {
                    (false, _) => Outside::Wrap,
                    (true, false) => Outside::Transparent,
                    (true, true) => Outside::TileZero,
                },
                flip: Flip {
                    horizontal: bit(r.m7sel, 1),
                    vertical: bit(r.m7sel, 2),
                },
            },
            sprites: SpriteSetup {
                sizes,
                bases: [first_base, first_base.wrapping_add(gap)],
                interlace: bit(r.setini, 2),
            },
            interlace: bit(r.setini, 1),
            pseudo_hires: bit(r.setini, 8),
        }
    }
}
