//! The emulated backend's encoder for sm_game's typed display commands: each
//! command becomes the original's register writes, made now and in the
//! original's order and widths. Retires with the emulated PPU once the native
//! renderer is the authority (docs/handoff/owned-display-registers.md).
use crate::game::room_entry::PreparedRoomAssets;
use crate::game_state::Game;
use crate::rtl::{
    A1B1, A1T1L, A2A1H, A2A1L, BBAD0, BBAD1, BG1HOFS, BG1SC, BG12NBA, BG34NBA, BGMODE, CGADD,
    CGADSUB, CGDATA, CGWSEL, COLDATA, DAS1H, DAS1L, DAS10, DMAP1, HDMAEN, HTIMEH, HTIMEL, INIDISP,
    M7A, M7SEL, MDMAEN, MEMSEL, MOSAIC, NMITIMEN, NTRL1, OAMADDH, OAMADDL, OAMDATA, OBSEL, RDVRAML,
    SETINI, TM, TMW, TS, TSW, VMADDH, VMADDL, VMAIN, VTIMEH, VTIMEL, W12SEL, W34SEL, WBGLOG, WH0,
    WH1, WH2, WH3, WOBJLOG, WOBJSEL, WRDIVB, WRDIVH, WRDIVL, WRIO, WRMPYA, WRMPYB, read_reg,
    read_reg_word, write_reg, write_reg_word,
};
use sm_game::display::control::Brightness;
use sm_game::display::transfer::Source;
use sm_game::display::video::{
    Axis, Command, Control, Mode7Parameter, ScrollOffset, Upload, Video as _, WindowEdge,
    WindowHdma,
};

/// VMAIN's increment-after-high-byte mode for a word step of `increment`.
fn vram_increment_mode(increment: u16) -> u8 {
    0x80 | match increment {
        1 => 0,
        32 => 1,
        128 => 2,
        _ => unreachable!("a VRAM increment is 1, 32 or 128 words"),
    }
}

/// VMAIN: a one-word step, incrementing after the high byte.
const STEP_AFTER_HIGH: u8 = 0x80;
/// DMAP and BBAD for channel 1's CPU-to-VMDATA word transfers.
const TO_VRAM_DATA: u16 = u16::from_le_bytes([1, 0x18]);
/// DMAP and BBAD for channel 1's RDVRAM-to-CPU word transfers.
const FROM_VRAM_DATA: u16 = u16::from_le_bytes([0x81, 0x39]);

/// One register byte through the backend.
fn byte(register: u32, value: u8, game: &mut Game) {
    // SAFETY: `apply` and `copy_to_vram_now` check the backend is installed,
    // and `&mut Game` excludes other access.
    unsafe { write_reg(register as u16, value, game) }
}

/// A 16-bit store: its low byte to `register`, its high byte to the next
/// register up.
fn word(register: u32, value: u16, game: &mut Game) {
    // SAFETY: as for `byte`.
    unsafe { write_reg_word(register as u16, value, game) }
}

/// A scroll offset's register: BG1HOFS, BG1VOFS, BG2HOFS and so on.
fn scroll_register(scroll: ScrollOffset) -> u32 {
    let axis = match scroll.axis {
        Axis::Horizontal => 0,
        Axis::Vertical => 1,
    };
    BG1HOFS + 2 * u32::from(scroll.background.index()) + axis
}

/// M7A to M7Y, in parameter order.
fn mode7_register(parameter: Mode7Parameter) -> u32 {
    M7A + parameter as u32
}

fn control_register(control: Control) -> u32 {
    match control {
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
        Control::Brightness(_) => INIDISP,
        Control::ObjectLayout(_) => OBSEL,
        Control::BackgroundMode(_) => BGMODE,
        Control::Mosaic(_) => MOSAIC,
        Control::Tilemap(background, _) => BG1SC + u32::from(background.index()),
        Control::CharactersBg1Bg2(_) => BG12NBA,
        Control::CharactersBg3Bg4(_) => BG34NBA,
        Control::VramStep(_) => VMAIN,
        Control::Mode7Selection(_) => M7SEL,
        Control::Mode7Latch(parameter, _) => mode7_register(parameter),
        Control::Window1Left(_) => WH0,
        Control::Window1Right(_) => WH1,
        Control::Window2Left(_) => WH2,
        Control::Window2Right(_) => WH3,
        Control::WindowLogicBackgrounds(_) => WBGLOG,
        Control::WindowLogicSpritesColor(_) => WOBJLOG,
        Control::ScreenSettings(_) => SETINI,
        Control::Interrupts(_) => NMITIMEN,
        Control::MemorySpeed(_) => MEMSEL,
        Control::ObjectAddressLow(_) => OAMADDL,
        Control::ObjectAddressHigh(_) => OAMADDH,
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
            word(VMADDL, address, game);
            // The word write rewrites VMADDL's low byte with the same value.
            word(
                VMAIN,
                u16::from_le_bytes([vram_increment_mode(increment), address.to_le_bytes()[0]]),
                game,
            );
            channel1(TO_VRAM_DATA, source, bytes, game);
            byte(MDMAEN, 2, game);
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
        Command::TilesNowBytePort {
            address,
            source,
            bytes,
        } => {
            game.producing();
            let [low, high] = address.to_le_bytes();
            byte(VMADDL, low, game);
            byte(VMADDH, high, game);
            byte(VMAIN, STEP_AFTER_HIGH, game);
            channel1(TO_VRAM_DATA, source, bytes, game);
            byte(MDMAEN, 2, game);
            game.produced(producer, &[word_step_tiles(address, source, bytes)]);
        }
        Command::TilemapNow {
            address,
            source,
            bytes,
        } => {
            game.producing();
            word(VMADDL, address, game);
            channel1_words(TO_VRAM_DATA, source, bytes, game);
            byte(VMAIN, STEP_AFTER_HIGH, game);
            byte(MDMAEN, 2, game);
            game.produced(producer, &[word_step_tiles(address, source, bytes)]);
        }
        Command::ReadTilemapNow {
            address,
            destination,
            bytes,
        } => {
            word(VMADDL, address, game);
            // SAFETY: as for `byte`.
            unsafe { read_reg_word(RDVRAML as u16, game) };
            channel1_words(FROM_VRAM_DATA, destination, bytes, game);
            byte(VMAIN, STEP_AFTER_HIGH, game);
            byte(MDMAEN, 2, game);
        }
        Command::ReadVramNow {
            address,
            destination,
            bytes,
        } => {
            let [mode, target] = FROM_VRAM_DATA.to_le_bytes();
            word(VMADDL, address, game);
            byte(VMAIN, STEP_AFTER_HIGH, game);
            byte(DMAP1, mode, game);
            byte(BBAD1, target, game);
            word(A1T1L, destination.address, game);
            byte(A1B1, destination.bank, game);
            word(DAS1L, bytes, game);
            byte(DAS10, 0, game);
            word(A2A1L, 0, game);
            byte(NTRL1, 0, game);
            byte(MDMAEN, 2, game);
        }
        Command::VramPort { address, step } => {
            let [low, high] = address.to_le_bytes();
            byte(VMADDL, low, game);
            byte(VMADDH, high, game);
            byte(VMAIN, step, game);
        }
        Command::TilesThroughPort { source, bytes } => {
            channel1(TO_VRAM_DATA, source, bytes, game);
            byte(MDMAEN, 2, game);
        }
        Command::TilesThroughPortBytes { source, bytes } => {
            let [mode, target] = TO_VRAM_DATA.to_le_bytes();
            let [count_low, count_high] = bytes.to_le_bytes();
            byte(DMAP1, mode, game);
            byte(BBAD1, target, game);
            byte(DAS1L, count_low, game);
            byte(DAS1H, count_high, game);
            word(A1T1L, source.address, game);
            byte(A1B1, source.bank, game);
            byte(MDMAEN, 2, game);
        }
        Command::ObjectBytes(bytes) => {
            for value in bytes {
                byte(OAMDATA, value, game);
            }
        }
        Command::ClearCpuIo => {
            for register in [
                WRIO, WRMPYA, WRMPYB, WRDIVL, WRDIVH, WRDIVB, HTIMEL, HTIMEH, VTIMEL, VTIMEH,
                MDMAEN,
            ] {
                byte(register, 0, game);
            }
        }
        Command::FixedColor(color) => byte(COLDATA, color.byte(), game),
        Command::WindowHdma(hdma) => window_hdma(hdma, game),
        Command::Control(control) => byte(control_register(control), control.byte(), game),
        Command::Colors { index, colors } => {
            byte(CGADD, index, game);
            for color in colors {
                let [low, high] = color.to_le_bytes();
                byte(CGDATA, low, game);
                byte(CGDATA, high, game);
            }
        }
        Command::Scroll { scroll, offset } => scroll_latch(scroll, offset, game),
        Command::ScrollAfterRead { scroll, offset } => {
            // SAFETY: as for `byte`.
            unsafe { read_reg(scroll_register(scroll) as u16, game) };
            scroll_latch(scroll, offset, game);
        }
        Command::Mode7Word(parameter, value) => word(mode7_register(parameter), value, game),
        Command::HdmaScrollTarget { channel, scroll } => {
            let target = scroll_register(scroll).to_le_bytes()[0];
            byte(BBAD0 + (u32::from(channel) << 4), target, game);
        }
        Command::ScrollHdma {
            channel,
            scroll,
            table,
        } => scroll_hdma(channel, scroll, table, game),
        Command::StopDma => word(MDMAEN, 0, game),
    }
}

/// Channel 1's mode and target as one word, then its source address, bank and
/// byte count.
fn channel1(mode: u16, source: Source, bytes: u16, game: &mut Game) {
    word(DMAP1, mode, game);
    word(A1T1L, source.address, game);
    byte(A1B1, source.bank, game);
    word(DAS1L, bytes, game);
}

/// Channel 1 as the message boxes set it: every register a word, so the bank's
/// word also clears the count's low byte before the count is written, and the
/// indirect bank, table address and line counter are cleared after.
fn channel1_words(mode: u16, source: Source, bytes: u16, game: &mut Game) {
    word(DMAP1, mode, game);
    word(A1T1L, source.address, game);
    word(A1B1, u16::from(source.bank), game);
    word(DAS1L, bytes, game);
    word(DAS10, 0, game);
    word(A2A1H, 0, game);
}

fn word_step_tiles(address: u16, source: Source, bytes: u16) -> Upload {
    Upload::Tiles {
        address,
        increment: 1,
        source,
        bytes,
    }
}

fn scroll_latch(scroll: ScrollOffset, offset: u16, game: &mut Game) {
    let register = scroll_register(scroll);
    let [low, high] = offset.to_le_bytes();
    byte(register, low, game);
    byte(register, high, game);
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
    let base = 0x4300 | (u32::from(channel) << 4);
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
        byte(base + offset, value, game);
    }
}

/// An indirect two-write HDMA channel to a scroll offset, in the message box's
/// order ($85:8363): mode, target, each table byte followed by the matching
/// indirect address byte, the table bank then the indirect bank, then the
/// cursors.
fn scroll_hdma(channel: u8, scroll: ScrollOffset, table: Source, game: &mut Game) {
    const INDIRECT_TWO_WRITES: u8 = 0x42;
    let [table_low, table_high] = table.address.to_le_bytes();
    let base = 0x4300 | (u32::from(channel) << 4);
    for (offset, value) in [
        (0, INDIRECT_TWO_WRITES),
        (1, scroll_register(scroll).to_le_bytes()[0]),
        (2, table_low),
        (5, table_low),
        (3, table_high),
        (6, table_high),
        (4, table.bank),
        (7, table.bank),
        (8, 0),
        (9, 0),
        (10, 0),
    ] {
        byte(base + offset, value, game);
    }
}

/// `CopyToVramNow`: forced blank, the VRAM address, channel 1, the VRAM step,
/// the start (reading the room's prepared assets when given), then full
/// brightness.
pub(crate) fn copy_to_vram_now(
    address: u16,
    source: Source,
    bytes: u16,
    assets: Option<&PreparedRoomAssets>,
    game: &mut Game,
) {
    assert!(!game.g_snes.is_null(), "SNES backend must be installed");
    byte(INIDISP, Brightness::BLANK_DARK.byte(), game);
    word(VMADDL, address, game);
    channel1(TO_VRAM_DATA, source, bytes, game);
    byte(VMAIN, STEP_AFTER_HIGH, game);
    // SAFETY: as for `byte`.
    unsafe { crate::game::display_refresh::start_room_dma(2, assets, game) };
    byte(INIDISP, Brightness::FULL.byte(), game);
}
