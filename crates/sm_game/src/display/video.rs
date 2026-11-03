//! Typed video memory uploads. A producer brackets its register writes with
//! `producing` and `produced`, giving the uploads those writes make as typed
//! values, so a host whose video memory takes typed calls can apply them in
//! place of the register writes. A producer that no longer names registers
//! hands the host a `Command` instead, which the host applies at once.

use super::control::{Brightness, FixedColor, Layers};
use super::preparation::{Access, Field, Store};
use super::transfer::Source;

/// One upload. Objects and palettes come from work RAM, `source` being the
/// offset there; tiles come from anywhere on the CPU bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upload {
    /// OAM from the OAMADD word `address`: the low table, then the high table.
    Objects {
        address: u16,
        source: usize,
        bytes: u16,
    },
    /// CGRAM from palette entry `index`.
    Palette {
        index: u8,
        source: usize,
        bytes: u16,
    },
    /// VRAM from word `address`, its bytes alternating low and high and the
    /// address stepping by `increment` words after each high byte. A `bytes`
    /// of zero transfers 64 KiB, as the DMA does.
    Tiles {
        address: u16,
        increment: u16,
        source: Source,
        bytes: u16,
    },
}

/// One of window 1's edges, the B-bus target of a window HDMA channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEdge {
    Left,
    Right,
}

/// An indirect HDMA channel that sets one window 1 edge each line: `table`
/// lists line counts and pointers into `indirect_bank`. Setting it up clears
/// the channel's transfer cursors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHdma {
    /// The HDMA channel, 0 to 7.
    pub channel: u8,
    pub edge: WindowEdge,
    pub table: Source,
    pub indirect_bank: u8,
}

/// One of the four background layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    Bg1,
    Bg2,
    Bg3,
    Bg4,
}

impl Background {
    pub const ALL: [Self; 4] = [Self::Bg1, Self::Bg2, Self::Bg3, Self::Bg4];

    /// 0 for BG1 to 3 for BG4.
    pub const fn index(self) -> u8 {
        self as u8
    }
}

/// A scroll direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// One background's horizontal or vertical scroll offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollOffset {
    pub background: Background,
    pub axis: Axis,
}

/// A Mode 7 matrix or centre parameter, in register order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode7Parameter {
    A,
    B,
    C,
    D,
    CenterX,
    CenterY,
}

impl Mode7Parameter {
    pub const ALL: [Self; 6] = [
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::CenterX,
        Self::CenterY,
    ];
}

/// A screen, window or colour-math control the game sets directly. Each value
/// is the byte the original stored, also kept in its display mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    MainScreen(Layers),
    SubScreen(Layers),
    MainWindowMask(Layers),
    SubWindowMask(Layers),
    /// BG1 and BG2's window enable and inversion bits.
    WindowsBg1Bg2(u8),
    /// BG3 and BG4's window enable and inversion bits.
    WindowsBg3Bg4(u8),
    /// The sprites' and the colour window's enable and inversion bits.
    WindowsSpritesColor(u8),
    /// Where colour math clips and applies, and its subscreen source.
    ColorSelection(u8),
    /// Colour math's operation and the layers it applies to.
    ColorMath(u8),
    /// The HDMA channels that run, one bit each.
    HdmaChannels(u8),
    /// Forced blank and master brightness.
    Brightness(Brightness),
    /// The sprites' sizes and tile base.
    ObjectLayout(u8),
    /// The background mode, BG3 priority and per-layer tile size.
    BackgroundMode(u8),
    /// The mosaic size and the layers it applies to.
    Mosaic(u8),
    /// A background's tilemap address and size.
    Tilemap(Background, u8),
    /// BG1 and BG2's tile bases.
    CharactersBg1Bg2(u8),
    /// BG3 and BG4's tile bases.
    CharactersBg3Bg4(u8),
    /// The VRAM port's address step and remapping.
    VramStep(u8),
    /// Mode 7's flips and what shows outside its field.
    Mode7Selection(u8),
    /// One byte into a Mode 7 parameter's two-write latch.
    Mode7Latch(Mode7Parameter, u8),
    /// Window 1's left edge.
    Window1Left(u8),
    /// Window 1's right edge.
    Window1Right(u8),
    /// Window 2's left edge.
    Window2Left(u8),
    /// Window 2's right edge.
    Window2Right(u8),
    /// How the two windows combine for each background.
    WindowLogicBackgrounds(u8),
    /// How the two windows combine for the sprites and the colour window.
    WindowLogicSpritesColor(u8),
    /// Interlace, overscan, pseudo-hires and external-sync flags.
    ScreenSettings(u8),
}

impl Control {
    pub const fn byte(self) -> u8 {
        match self {
            Self::MainScreen(layers)
            | Self::SubScreen(layers)
            | Self::MainWindowMask(layers)
            | Self::SubWindowMask(layers) => layers.byte(),
            Self::Brightness(brightness) => brightness.byte(),
            Self::WindowsBg1Bg2(bits)
            | Self::WindowsBg3Bg4(bits)
            | Self::WindowsSpritesColor(bits)
            | Self::ColorSelection(bits)
            | Self::ColorMath(bits)
            | Self::HdmaChannels(bits)
            | Self::ObjectLayout(bits)
            | Self::BackgroundMode(bits)
            | Self::Mosaic(bits)
            | Self::Tilemap(_, bits)
            | Self::CharactersBg1Bg2(bits)
            | Self::CharactersBg3Bg4(bits)
            | Self::VramStep(bits)
            | Self::Mode7Selection(bits)
            | Self::Mode7Latch(_, bits)
            | Self::Window1Left(bits)
            | Self::Window1Right(bits)
            | Self::Window2Left(bits)
            | Self::Window2Right(bits)
            | Self::WindowLogicBackgrounds(bits)
            | Self::WindowLogicSpritesColor(bits)
            | Self::ScreenSettings(bits) => bits,
        }
    }
}

/// Apply `control` and store its byte in the display's `mirror` field (a
/// named store, as the original's RAM store), mirror first when `mirror_first`.
pub fn apply_mirrored(
    host: &mut (impl Video + Access + ?Sized),
    producer: &'static str,
    control: Control,
    mirror: Field,
    mirror_first: bool,
) {
    let store = |host: &mut _| {
        Access::store_display_control_byte(host, mirror, control.byte(), Store::Named)
    };
    if mirror_first {
        store(host);
        host.apply(producer, Command::Control(control));
    } else {
        host.apply(producer, Command::Control(control));
        store(host);
    }
}

/// Where a control's byte is also kept: nowhere, or its display mirror field,
/// stored before or after the control is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    None,
    Before(Field),
    After(Field),
}

/// Apply each control in order, storing its byte in its mirror as named.
pub fn apply_controls(
    host: &mut (impl Video + Access + ?Sized),
    producer: &'static str,
    controls: &[(Control, Mirror)],
) {
    for &(control, mirror) in controls {
        match mirror {
            Mirror::None => host.apply(producer, Command::Control(control)),
            Mirror::Before(field) => apply_mirrored(host, producer, control, field, true),
            Mirror::After(field) => apply_mirrored(host, producer, control, field, false),
        }
    }
}

/// A display change a producer makes now, at its original point. The host
/// applies it before returning; the emulated backend's encoder performs the
/// original's register writes for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Upload to VRAM now as `Upload::Tiles`, setting the VRAM port before
    /// the transfer channel.
    TilesNow {
        address: u16,
        increment: u16,
        source: Source,
        bytes: u16,
    },
    /// The fixed colour's selected channels.
    FixedColor(FixedColor),
    WindowHdma(WindowHdma),
    Control(Control),
    /// Upload to VRAM now as `Upload::Tiles` with a one-word step, setting
    /// the VRAM port byte by byte (address low, address high, step) before
    /// the transfer channel, as the options screen's tile upload does.
    TilesNowBytePort {
        address: u16,
        source: Source,
        bytes: u16,
    },
    /// Upload a work RAM tilemap to VRAM now as `Upload::Tiles` with a
    /// one-word step, the message boxes' way: the channel first, each of its
    /// registers written as a word, clearing its indirect and table cursors,
    /// then the VRAM port's step.
    TilemapNow {
        address: u16,
        source: Source,
        bytes: u16,
    },
    /// Read `bytes` of VRAM from word `address` back to work RAM at
    /// `destination` now, after the port's discarded prefetch read, with the
    /// channel set up as for `TilemapNow`.
    ReadTilemapNow {
        address: u16,
        destination: Source,
        bytes: u16,
    },
    /// Two CGRAM colours from palette entry `index`, through the port.
    Colors {
        index: u8,
        colors: [u16; 2],
    },
    /// A scroll offset through its two-write latch: low byte, then high.
    Scroll {
        scroll: ScrollOffset,
        offset: u16,
    },
    /// As `Scroll`, after the original's discarded read of the write-only
    /// scroll register ($85:8190).
    ScrollAfterRead {
        scroll: ScrollOffset,
        offset: u16,
    },
    /// A 16-bit store to a Mode 7 parameter: its low byte to the parameter's
    /// latch and its high byte to the next register up, as a word store does.
    Mode7Word(Mode7Parameter, u16),
    /// HDMA channel `channel`'s B-bus target becomes `scroll`.
    HdmaScrollTarget {
        channel: u8,
        scroll: ScrollOffset,
    },
    /// An indirect two-write HDMA channel to a scroll offset: `table` lists
    /// line counts and pointers into its own bank. The original also starts
    /// the indirect address at the table's and clears the channel's cursors.
    ScrollHdma {
        channel: u8,
        scroll: ScrollOffset,
        table: Source,
    },
    /// Start no DMA and stop every HDMA channel: one 16-bit store of zero
    /// that clears both enables.
    StopDma,
}

/// A host that takes a producer's uploads and display commands as typed calls.
pub trait Video {
    /// The producer's register writes begin.
    fn producing(&mut self);
    /// The producer's register writes are done; `uploads` are what they did.
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]);
    /// Apply `producer`'s `command` now.
    fn apply(&mut self, producer: &'static str, command: Command);
}
