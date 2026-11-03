//! The emulated backend's encoder for sm_game's typed display commands: each
//! command becomes the original's register writes, made now and in the
//! original's order and widths. Retires with the emulated PPU once the native
//! renderer is the authority (docs/handoff/owned-display-registers.md).
use crate::game_state::Game;
use crate::rtl::{
    A1B1, A1T1L, CGADSUB, CGWSEL, COLDATA, DAS1L, DMAP1, HDMAEN, MDMAEN, TM, TMW, TS, TSW, VMADDL,
    VMAIN, W12SEL, W34SEL, WH0, WH1, WOBJSEL, write_reg, write_reg_word,
};
use sm_game::display::video::{Command, Control, Upload, Video as _, WindowEdge, WindowHdma};

/// VMAIN's increment-after-high-byte mode for a word step of `increment`.
fn vram_increment_mode(increment: u16) -> u8 {
    0x80 | match increment {
        1 => 0,
        32 => 1,
        128 => 2,
        _ => unreachable!("a VRAM increment is 1, 32 or 128 words"),
    }
}

/// Apply `command` from `producer` to the emulated backend.
pub(crate) fn apply(producer: &'static str, command: Command, game: &mut Game) {
    assert!(!game.g_snes.is_null(), "SNES backend must be installed");
    match command {
        Command::TilesNow {
            address,
            increment,
            source,
            bytes,
        } => {
            game.producing();
            // SAFETY: the backend is installed, and `&mut Game` excludes other access.
            unsafe {
                write_reg_word(VMADDL as u16, address, game);
                // The word write rewrites VMADDL's low byte with the same value.
                write_reg_word(
                    VMAIN as u16,
                    u16::from_le_bytes([vram_increment_mode(increment), address.to_le_bytes()[0]]),
                    game,
                );
                write_reg_word(DMAP1 as u16, u16::from_le_bytes([1, 0x18]), game);
                write_reg_word(A1T1L as u16, source.address, game);
                write_reg(A1B1 as u16, source.bank, game);
                write_reg_word(DAS1L as u16, bytes, game);
                write_reg(MDMAEN as u16, 2, game);
            }
            game.produced(
                producer,
                &[Upload::Tiles {
                    address,
                    increment,
                    source,
                    bytes,
                }],
            );
        }
        // SAFETY: as for `TilesNow`.
        Command::FixedColor(color) => unsafe { write_reg(COLDATA as u16, color.byte(), game) },
        Command::WindowHdma(hdma) => window_hdma(hdma, game),
        Command::Control(control) => {
            let register = match control {
                Control::MainScreen(_) => TM,
                Control::SubScreen(_) => TS,
                Control::MainWindowMask(_) => TMW,
                Control::SubWindowMask(_) => TSW,
                Control::WindowsBg1Bg2(_) => W12SEL,
                Control::WindowsBg3Bg4(_) => W34SEL,
                Control::WindowsSpritesColor(_) => WOBJSEL,
                Control::ColorSelection(_) => CGWSEL,
                Control::ColorMath(_) => CGADSUB,
                Control::HdmaChannels(_) => HDMAEN,
            };
            // SAFETY: as for `TilesNow`.
            unsafe { write_reg(register as u16, control.byte(), game) }
        }
    }
}

/// An indirect one-register HDMA channel to a window 1 edge, in the original's
/// order: mode, target, table address and bank, indirect bank, then the cursors.
fn window_hdma(hdma: WindowHdma, game: &mut Game) {
    const INDIRECT_ONE_REGISTER: u8 = 0x40;
    let WindowHdma {
        channel,
        edge,
        table,
        indirect_bank,
    } = hdma;
    let target = match edge {
        WindowEdge::Left => WH0,
        WindowEdge::Right => WH1,
    };
    let [table_low, table_high] = table.address.to_le_bytes();
    let base = 0x4300 | (u16::from(channel) << 4);
    for (offset, value) in [
        (0, INDIRECT_ONE_REGISTER),
        (1, target.to_le_bytes()[0]),
        (2, table_low),
        (3, table_high),
        (4, table.bank),
        (7, indirect_bank),
        (5, 0),
        (6, 0),
        (8, 0),
        (9, 0),
        (10, 0),
    ] {
        // SAFETY: the caller checked the backend is installed.
        unsafe { write_reg(base + offset, value, game) }
    }
}
