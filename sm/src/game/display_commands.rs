//! The emulated backend's encoder for sm_game's typed display commands: each
//! command becomes the original's register writes, made now and in the
//! original's order and widths. Retires with the emulated PPU once the native
//! renderer is the authority (docs/handoff/owned-display-registers.md).
use crate::game::room_entry::PreparedRoomAssets;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::rtl::{
    A1B0, A1B1, A1T0L, A1T1L, A2A1H, A2A1L, BBAD0, BBAD1, BG1HOFS, BG1SC, BG12NBA, BG34NBA, BGMODE,
    CGADD, CGADSUB, CGDATA, CGWSEL, COLDATA, DAS0L, DAS1H, DAS1L, DAS10, DMAP0, DMAP1, HDMAEN,
    HTIMEH, HTIMEL, INIDISP, M7A, M7SEL, MDMAEN, MEMSEL, MOSAIC, NMITIMEN, NTRL1, OAMADDH, OAMADDL,
    OAMDATA, OBSEL, RDVRAML, SETINI, TM, TMW, TS, TSW, VMADDH, VMADDL, VMAIN, VMDATAH, VMDATAL,
    VTIMEH, VTIMEL, W12SEL, W34SEL, WBGLOG, WH0, WH1, WH2, WH3, WMADDH, WMADDL, WMADDM, WOBJLOG,
    WOBJSEL, WRDIVB, WRDIVH, WRDIVL, WRIO, WRMPYA, WRMPYB, read_device, read_reg, write_device,
    write_reg,
};
use sm_data::hdma_programs::ChannelRegister;
use sm_game::display::control::Brightness;
use sm_game::display::preparation::Field;
use sm_game::display::transfer::Source;
use sm_game::display::video::{
    Axis, Command, Control, GraphicsTransfer, Mode7Parameter, ScrollOffset, Upload, WindowEdge,
    WindowHdma,
};
use sm_game::scene::irq::Output;

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
/// DMAP and BBAD for byte transfers to OAMDATA.
const TO_OAM_DATA: u16 = u16::from_le_bytes([0, 0x04]);
/// DMAP and BBAD for byte transfers to CGDATA.
const TO_CGRAM_DATA: u16 = u16::from_le_bytes([0, 0x22]);

/// Where the encoder's register accesses go: the bus, as the game's own code
/// makes them, a device interval for a lease's device-only command (`Device`),
/// or the lease's transfer port (`Lease`).
pub(crate) trait Wire {
    fn byte(&mut self, register: u32, value: u8);
    fn read(&mut self, register: u32) -> u8;
    /// The producer's register writes begin.
    fn producing(&mut self);
    /// The producer's register writes are done; `uploads` are what they did.
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]);
    /// Start the DMA channels in `mask`, reading the room's prepared image when
    /// given.
    fn start_room_dma(&mut self, mask: u8, assets: Option<&PreparedRoomAssets>);
}

impl crate::game::display_commands::Wire for Game {
    fn byte(&mut self, register: u32, value: u8) {
        // SAFETY: `apply` and `copy_to_vram_now` check the backend is installed,
        // and `&mut Game` excludes other access.
        unsafe { write_reg(register as u16, value, self) }
    }
    fn read(&mut self, register: u32) -> u8 {
        // SAFETY: as for `byte`.
        unsafe { read_reg(register as u16, self) }
    }
    fn producing(&mut self) {
        Game::producing(self);
    }
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]) {
        Game::produced(self, producer, uploads);
    }
    fn start_room_dma(&mut self, mask: u8, assets: Option<&PreparedRoomAssets>) {
        // SAFETY: as for `byte`.
        unsafe { crate::game::display_refresh::start_room_dma(mask, assets, self) }
    }
}

/// The device-register interval a lease's device-only command runs in: the same
/// writes, which leave Samus authority and the lease current.
struct Device<'a>(&'a mut Game);

impl Wire for Device<'_> {
    fn byte(&mut self, register: u32, value: u8) {
        // SAFETY: `apply_device` checks the backend is installed, and the
        // borrow of Game excludes other access.
        unsafe { write_device(register as u16, value, self.0) }
    }
    fn read(&mut self, register: u32) -> u8 {
        // SAFETY: as for `byte`.
        unsafe { read_device(register, self.0) }
    }
    fn producing(&mut self) {
        unreachable!("a device-only command produces no upload")
    }
    fn produced(&mut self, _producer: &'static str, _uploads: &[Upload]) {
        unreachable!("a device-only command produces no upload")
    }
    fn start_room_dma(&mut self, _mask: u8, _assets: Option<&PreparedRoomAssets>) {
        unreachable!("a device-only command starts no DMA")
    }
}

/// A lease's transfer port: device registers through the device interval, and
/// each DMA start through `Runtime::start_dma`, after the port has published the
/// lease (`Runtime::publish`).
pub(crate) struct Lease<'a, 'g>(pub(crate) &'a mut Runtime<'g>);

impl Wire for Lease<'_, '_> {
    fn byte(&mut self, register: u32, value: u8) {
        if register == MDMAEN && value != 0 {
            self.0.start_dma(value, None);
        } else {
            self.0.write_device(register as u16, value);
        }
    }
    fn read(&mut self, register: u32) -> u8 {
        self.0.read_device(register)
    }
    fn producing(&mut self) {
        self.0.producing();
    }
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]) {
        self.0.produced(producer, uploads);
    }
    fn start_room_dma(&mut self, mask: u8, assets: Option<&PreparedRoomAssets>) {
        match assets {
            Some(assets) => {
                let (first, bytes) = assets.raw_window();
                self.0.start_dma(
                    mask,
                    Some(&crate::execution::DmaRomSource::new(first, bytes)),
                );
            }
            None => self.0.start_dma(mask, None),
        }
    }
}

/// One register byte through `wire`.
pub(crate) fn byte(register: u32, value: u8, wire: &mut impl Wire) {
    wire.byte(register, value);
}

/// A 16-bit store: its low byte to `register`, its high byte to the next
/// register up.
pub(crate) fn word(register: u32, value: u16, wire: &mut impl Wire) {
    let [low, high] = value.to_le_bytes();
    wire.byte(register, low);
    wire.byte(register + 1, high);
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

/// The register a display mirror field is committed to.
fn committed_register(field: Field) -> u32 {
    match field {
        Field::NmiEnable => NMITIMEN,
        Field::Brightness => INIDISP,
        Field::ObjectLayout => OBSEL,
        Field::Mode => BGMODE,
        Field::Mosaic => MOSAIC,
        Field::Screens0 => BG1SC,
        Field::Screens1 => BG1SC + 1,
        Field::Screens2 => BG1SC + 2,
        Field::Screens3 => BG1SC + 3,
        Field::Characters0 => BG12NBA,
        Field::Characters1 => BG34NBA,
        Field::AffineSelection => M7SEL,
        Field::WindowSelection0 => W12SEL,
        Field::WindowSelection1 => W34SEL,
        Field::WindowSelection2 => WOBJSEL,
        Field::WindowPositions0 => WH0,
        Field::WindowPositions1 => WH1,
        Field::WindowPositions2 => WH2,
        Field::WindowPositions3 => WH3,
        Field::WindowLogic0 => WBGLOG,
        Field::WindowLogic1 => WOBJLOG,
        Field::MainLayers => TM,
        Field::MainWindow => TMW,
        Field::SubLayers => TS,
        Field::SubWindow => TSW,
        Field::ColorSelection => CGWSEL,
        Field::ColorMath => CGADSUB,
        Field::FixedColor0 | Field::FixedColor1 | Field::FixedColor2 => COLDATA,
        Field::Geometry => SETINI,
        field => unreachable!("{field:?} is not committed to the display"),
    }
}

/// The register each scanline interrupt output reaches.
fn irq_register(output: Output) -> u32 {
    match output {
        Output::Background3Map => BG1SC + 2,
        Output::ColorWindow => CGWSEL,
        Output::ColorArithmetic => CGADSUB,
        Output::MainLayers => TM,
        Output::VerticalLow => VTIMEL,
        Output::VerticalHigh => VTIMEH,
        Output::HorizontalLow => HTIMEL,
        Output::HorizontalHigh => HTIMEH,
        Output::InterruptEnable => NMITIMEN,
    }
}

/// An HDMA program's channel register, as an offset from the channel's first.
fn channel_register_offset(register: ChannelRegister) -> u32 {
    match register {
        ChannelRegister::Control => 0,
        ChannelRegister::Target => 1,
        ChannelRegister::TableBank => 4,
        ChannelRegister::IndirectBank => 7,
    }
}

/// DMAP and BBAD for a graphics load's channel.
fn graphics_mode(transfer: GraphicsTransfer) -> u16 {
    u16::from_le_bytes(match transfer {
        GraphicsTransfer::VramWords => [1, 0x18],
        GraphicsTransfer::VramLowBytes => [0, 0x18],
        GraphicsTransfer::VramHighBytes => [0, 0x19],
        GraphicsTransfer::WramBytes => [0, 0x80],
    })
}

/// Apply `command` from `producer` to the emulated backend.
pub(crate) fn apply(producer: &'static str, command: Command, game: &mut Game) {
    assert!(!game.g_snes.is_null(), "SNES backend must be installed");
    encode(producer, command, game);
}

/// Apply `command` from `producer` on the lease: the lease is published, then
/// the same writes go through its transfer port, so every DMA start reads the
/// Session's bytes and the bytes a readback writes return to the Session.
pub(crate) fn apply_on_lease(producer: &'static str, command: Command, runtime: &mut Runtime<'_>) {
    runtime.publish();
    encode(producer, command, &mut Lease(runtime));
}

/// `command`'s register writes through `game`.
fn encode(producer: &'static str, command: Command, game: &mut impl Wire) {
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
            // The prefetch read of the data word, low byte then high.
            game.read(RDVRAML);
            game.read(RDVRAML + 1);
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
        Command::ObjectsAndPaletteNow {
            objects,
            object_bytes,
            palette,
            palette_bytes,
        } => {
            game.producing();
            word(DMAP0, TO_OAM_DATA, game);
            word(A1T0L, objects.address, game);
            byte(A1B0, objects.bank, game);
            word(DAS0L, object_bytes, game);
            word(OAMADDL, 0, game);
            channel1(TO_CGRAM_DATA, palette, palette_bytes, game);
            byte(CGADD, 0, game);
            byte(MDMAEN, 3, game);
            game.produced(
                producer,
                &[
                    Upload::Objects {
                        address: 0,
                        source: usize::from(objects.address),
                        bytes: object_bytes,
                    },
                    Upload::Palette {
                        index: 0,
                        source: usize::from(palette.address),
                        bytes: palette_bytes,
                    },
                ],
            );
        }
        command => registers(command, game),
    }
}

/// Whether `command` reaches only device registers: no DMA transfer starts, and
/// no WRAM, ROM or Samus state moves. MDMAEN writes are left out even when they
/// start nothing (`StopDma`, `ClearCpuIo`): the room background check
/// snapshots RAM at each one.
pub(crate) fn device_only(command: &Command) -> bool {
    !matches!(
        command,
        Command::TilesNow { .. }
            | Command::TilesNowBytePort { .. }
            | Command::TilemapNow { .. }
            | Command::ReadTilemapNow { .. }
            | Command::ReadVramNow { .. }
            | Command::TilesThroughPort { .. }
            | Command::TilesThroughPortBytes { .. }
            | Command::ObjectsAndPaletteNow { .. }
            | Command::StopDma
            | Command::ClearCpuIo
    )
}

/// Apply device-only `command` (`device_only`) to the emulated backend in a
/// device interval, with the same writes `apply` makes.
pub(crate) fn apply_device(command: Command, game: &mut Game) {
    assert!(!game.g_snes.is_null(), "SNES backend must be installed");
    assert!(device_only(&command), "a display command that moves RAM");
    // A command that left RAM moved would leave the lease's Session behind it:
    // the lease neither exports before nor imports after a device interval.
    #[cfg(sm_verify_dll)]
    let epoch = game.compatibility_write_epoch();
    registers(command, &mut Device(game));
    #[cfg(sm_verify_dll)]
    assert_eq!(
        epoch,
        game.compatibility_write_epoch(),
        "a device-only display command moved RAM"
    );
}

/// Apply device-only `command` on the lease's transfer port, with the same
/// writes `apply` makes: a port that has its own observation seams makes many.
pub(crate) fn apply_device_on_lease(command: Command, runtime: &mut Runtime<'_>) {
    assert!(device_only(&command), "a display command that moves RAM");
    registers(command, &mut Lease(runtime));
}

/// An animated tile upload on the lease's transfer port: channel 0 from bank
/// $87 to VMDATA, a word at a time, in the original's register order. It reads
/// the cartridge alone, so the lease needs no publish.
pub(crate) fn animated_tiles_on_lease(
    request: sm_game::visual_state::Upload,
    runtime: &mut Runtime<'_>,
) {
    const BANK: u8 = 0x87;
    let mut lease = Lease(runtime);
    lease.producing();
    word(A1T0L, request.source, &mut lease);
    byte(A1B0, BANK, &mut lease);
    word(DMAP0, TO_VRAM_DATA, &mut lease);
    word(DAS0L, request.size, &mut lease);
    word(VMADDL, request.vram, &mut lease);
    byte(VMAIN, STEP_AFTER_HIGH, &mut lease);
    byte(MDMAEN, 1, &mut lease);
    lease.produced(
        "animated tiles",
        &[Upload::Tiles {
            address: request.vram,
            increment: 1,
            source: Source {
                address: request.source,
                bank: BANK,
            },
            bytes: request.size,
        }],
    );
}

/// The VRAM data word as `read_register_word` reads it: the low byte, then the
/// high byte, which steps the port.
pub(crate) fn read_vram_data_on_lease(runtime: &mut Runtime<'_>) -> u16 {
    let mut lease = Lease(runtime);
    let low = lease.read(RDVRAML);
    let high = lease.read(RDVRAML + 1);
    u16::from_le_bytes([low, high])
}

/// The commands that write registers only, through `game`.
fn registers(command: Command, game: &mut impl Wire) {
    match command {
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
        Command::WramPort { address } => {
            let [low, middle, high, _] = address.to_le_bytes();
            byte(WMADDL, low, game);
            byte(WMADDM, middle, game);
            byte(WMADDH, high, game);
        }
        Command::VramAddress(address) => word(VMADDL, address, game),
        Command::VramData { high, value } => {
            byte(if high { VMDATAH } else { VMDATAL }, value, game)
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
            game.read(scroll_register(scroll));
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
        Command::Commit(field, value) => byte(committed_register(field), value, game),
        Command::ScrollByte { scroll, value } => byte(scroll_register(scroll), value, game),
        Command::IrqOutput(output, value) => byte(irq_register(output), value, game),
        Command::HdmaTable { bank_slot, table } => word(A1T0L + u32::from(bank_slot), table, game),
        Command::HdmaChannelByte {
            bank_slot,
            register,
            value,
        } => byte(
            DMAP0 + channel_register_offset(register) + u32::from(bank_slot),
            value,
            game,
        ),
        Command::ClearHdmaChannels { first, target } => {
            let target = scroll_register(target).to_le_bytes()[0];
            for channel in first..8 {
                let base = 0x4300 | (u32::from(channel) << 4);
                for (offset, value) in [
                    (0, 0),
                    (1, target),
                    (2, 0),
                    (3, 0),
                    (4, 0),
                    (5, 0),
                    (6, 0),
                    (8, 0),
                    (9, 0),
                ] {
                    byte(base + offset, value, game);
                }
            }
        }
        Command::GraphicsChannel {
            transfer,
            source,
            bytes,
        } => channel1(graphics_mode(transfer), source, bytes, game),
        Command::TilesNow { .. }
        | Command::TilesNowBytePort { .. }
        | Command::TilemapNow { .. }
        | Command::ReadTilemapNow { .. }
        | Command::ReadVramNow { .. }
        | Command::ObjectsAndPaletteNow { .. } => unreachable!("`apply` encodes the uploads"),
    }
}

/// Channel 1's mode and target as one word, then its source address, bank and
/// byte count.
fn channel1(mode: u16, source: Source, bytes: u16, game: &mut impl Wire) {
    word(DMAP1, mode, game);
    word(A1T1L, source.address, game);
    byte(A1B1, source.bank, game);
    word(DAS1L, bytes, game);
}

/// Channel 1 as the message boxes set it: every register a word, so the bank's
/// word also clears the count's low byte before the count is written, and the
/// indirect bank, table address and line counter are cleared after.
fn channel1_words(mode: u16, source: Source, bytes: u16, game: &mut impl Wire) {
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

fn scroll_latch(scroll: ScrollOffset, offset: u16, game: &mut impl Wire) {
    let register = scroll_register(scroll);
    let [low, high] = offset.to_le_bytes();
    byte(register, low, game);
    byte(register, high, game);
}

/// An indirect one-register HDMA channel to a window 1 edge, in the original's
/// order: mode, target, table address and bank, indirect bank, then the cursors.
fn window_hdma(hdma: WindowHdma, game: &mut impl Wire) {
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
fn scroll_hdma(channel: u8, scroll: ScrollOffset, table: Source, game: &mut impl Wire) {
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
    copy_to_vram(address, source, bytes, assets, game);
}

/// `CopyToVramNow` on the lease, published first, through its transfer port.
pub(crate) fn copy_to_vram_on_lease(
    address: u16,
    source: Source,
    bytes: u16,
    assets: Option<&PreparedRoomAssets>,
    runtime: &mut Runtime<'_>,
) {
    runtime.publish();
    copy_to_vram(address, source, bytes, assets, &mut Lease(runtime));
}

fn copy_to_vram(
    address: u16,
    source: Source,
    bytes: u16,
    assets: Option<&PreparedRoomAssets>,
    game: &mut impl Wire,
) {
    byte(INIDISP, Brightness::BLANK_DARK.byte(), game);
    word(VMADDL, address, game);
    channel1(TO_VRAM_DATA, source, bytes, game);
    byte(VMAIN, STEP_AFTER_HIGH, game);
    game.start_room_dma(2, assets);
    byte(INIDISP, Brightness::FULL.byte(), game);
}
