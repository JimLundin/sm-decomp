//! Ordered display refresh and the NMI request/completion handshake, plus the
//! gameplay and pause display setup, graphics loads and tilemap queues.
#![forbid(unsafe_code)]
use crate::constants::bank_60::{
    addr_unk_6053E0, addr_unk_605000, addr_unk_605400, addr_unk_605880,
};
use crate::decompression::{self, decompress_to_ram};
use crate::display::control::{
    BgMode, FixedColor, Layers, MainScreen, MainWindowMask, ScreenSettings, SubScreen,
    SubWindowMask,
};
use crate::display::registers::{
    A1B0, A1B1, A1T0L, A1T1L, A2A1L, BBAD1, BG1HOFS, BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA,
    BGMODE, CGADD, CGADSUB, CGWSEL, COLDATA, DAS0L, DAS1H, DAS1L, DAS10, DMAP0, DMAP1, HDMAEN,
    HTIMEH, HTIMEL, INIDISP, M7A, M7SEL, MDMAEN, MEMSEL, MOSAIC, NMITIMEN, NTRL1, OAMADDH, OAMADDL,
    OAMDATA, OBSEL, RDVRAML, SETINI, TM, TMW, TS, TSW, VMADDH, VMADDL, VMAIN, VMDATAL, VTIMEH,
    VTIMEL, W12SEL, W34SEL, WBGLOG, WH0, WH1, WH2, WH3, WMADDH, WMADDL, WMADDM, WOBJLOG, WOBJSEL,
    WRDIVB, WRDIVH, WRDIVL, WRIO, WRMPYA, WRMPYB,
};
use crate::display::sequence::{Registers, register_writes, write_register_and_mirror};
use crate::display::transfer::{ScrollingLayout, setup_transfer};
use crate::ram::{CompatibilityRam, RawRam, checked_word, checked_word as at, word, word_arrays};
use crate::scene::state::DisplayError;
use crate::word_transfers;

/// What the display refresh needs besides RAM, registers and the cartridge:
/// owned display stores and the uploads the compatibility runtime still owns.
pub trait Host:
    CompatibilityRam
    + crate::ram::WorkRamAccess
    + Registers
    + crate::word_transfers::LoadedRom
    + crate::cartridge::LoadedImage
    + crate::decompression::DecompressionHost
    + crate::records::Records
{
    /// Store `value` at `at` through the owned display state.
    fn write_owned_byte(&mut self, at: usize, value: u8);
    /// The HDMA object table as the compatibility projection reads it.
    fn hdma_objects(&self) -> crate::hdma::HdmaObjects;
    /// Start the channel-1 graphics DMA for a load from `source`, reading the
    /// prepared room image when it holds that source.
    fn start_graphics_dma(&mut self, source: u32);
    fn upload_samus(&mut self);
    fn upload_animated_tiles(&mut self);
    fn process_mode7(&mut self);
    fn process_readbacks(&mut self);
    fn flush_tilemaps(&mut self);
    /// Fill VRAM's low bytes with `fill` from address 0, resetting the
    /// address as one word or two bytes.
    fn fill_vram_low_bytes(&mut self, fill: u8, word_reset: bool);
}

fn fill_owned<H: Host + ?Sized>(game: &mut H, range: core::ops::Range<usize>, value: u8) {
    for at in range {
        game.write_owned_byte(at, value);
    }
}

/// Set the forced-blank mirror bit for the next display refresh.
pub fn set_blank<H: Host + ?Sized>(blank: bool, game: &mut H) {
    game.write_owned_byte(
        0x51,
        game.compatibility_byte(0x51) & 0x7f | if blank { 0x80 } else { 0 },
    );
}

pub fn pause_graphics<H: Host + ?Sized>(paused: bool, game: &mut H) {
    let (objects, bg1, bg2, bg3) = if paused {
        (1, 0x31, 0x38, 0x58)
    } else {
        (3, 0x51, 0x49, 0x5a)
    };
    register_writes! { game;
        mirror(OBSEL as u16, objects, 0x52, false); show_mirror(BgMode::MODE1_BG3_PRIORITY, 0x55, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, bg1, 0x58, true); mirror(BG2SC as u16, bg2, 0x59, true);
        mirror(BG3SC as u16, bg3, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
    }
    if paused {
        register_writes! { game;
            show_mirror(MainScreen(Layers::of(true, true, true, false, true)), 0x69, false); mirror(MOSAIC as u16, 0, 0x57, false);
        }
        for offset in 0x74..0x77 {
            game.write_owned_byte(offset, game.compatibility_byte(offset) & 0xe0);
        }
        game.write_owned_byte(0x71, 0);
    }
}

/// The ordered CPU I/O and OAM reset.
pub fn initialize_io_and_oam<H: Host + ?Sized>(brightness: Option<u8>, game: &mut H) {
    register_writes! { game;
        mirror(NMITIMEN as u16, 1, 0x84, false); byte(WRIO as u16, 0);
        byte(WRMPYA as u16, 0); byte(WRMPYB as u16, 0);
        byte(WRDIVL as u16, 0); byte(WRDIVH as u16, 0);
        byte(WRDIVB as u16, 0); byte(HTIMEL as u16, 0);
        byte(HTIMEH as u16, 0); byte(VTIMEL as u16, 0);
        byte(VTIMEH as u16, 0); byte(MDMAEN as u16, 0);
        mirror(HDMAEN as u16, 0, 0x85, false); mirror(MEMSEL as u16, 1, 0x86, false);
    }
    if let Some(value) = brightness {
        write_register_and_mirror(game, INIDISP as u16, value, 0x51, false);
    }
    register_writes! { game;
        mirror(OBSEL as u16, 3, 0x52, false);
        mirror(OAMADDL as u16, 0, 0x53, false);
        mirror(OAMADDH as u16, 0x80, 0x54, false);
        byte(OAMDATA as u16, 0); byte(OAMDATA as u16, 0);
    }
}

/// Gameplay display registers, cleared tilemaps and the common graphics.
pub fn initialize_gameplay<H: Host + ?Sized>(game: &mut H) {
    initialize_io_and_oam(Some(0x80), game);
    register_writes! { game;
        show_mirror(BgMode::MODE1_BG3_PRIORITY, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
        mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
        mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
        mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        byte(VMAIN as u16, 0); mirror(W12SEL as u16, 0, 0x60, false);
        mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
        mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
        mirror(WH1 as u16, 0, 0x64, false); mirror(WH2 as u16, 0, 0x65, false);
        mirror(WH3 as u16, 0, 0x66, false); mirror(WBGLOG as u16, 0, 0x67, false);
        mirror(WOBJLOG as u16, 0, 0x68, false); show_mirror(MainScreen(Layers::of(true, true, true, false, true)), 0x69, false);
        show_mirror(MainWindowMask(Layers::NONE), 0x6c, false); show_mirror(SubScreen(Layers::NONE), 0x6b, false);
        show_mirror(SubWindowMask(Layers::NONE), 0x6d, false); mirror(CGWSEL as u16, 0, 0x6e, false);
        mirror(CGADSUB as u16, 0, 0x71, false); show(FixedColor::all(0));
        show_mirror(ScreenSettings::STANDARD, 0x77, false);
    }
    word!(game.oam_next_ptr = 0);
    word_transfers!(game, ascending, 1023; native(0x3000) <- value(0));
    word_transfers!(game, ascending, 1023; native(0x4000) <- value(0x6f));
    word_transfers!(game, ascending, 127; native(0x4000) <- value(0x2c0f));
    GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2e00).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5000, 0x1000).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5800, 0x800).load(game);
    word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x9a8000, 0));
}

/// Read the room's BG2 tilemap back into RAM for the pause screen.
pub fn capture_pause_background<H: Host + ?Sized>(game: &mut H) {
    register_writes! { game;
        word(VMADDL as u16,
        (u16::from(game.compatibility_byte(0x59) & 0xfc) << 8) | 1,); byte(VMAIN as u16, 0x80);
        byte(DMAP1 as u16, 0x81); byte(BBAD1 as u16, 0x39);
        word(A1T1L as u16, 0xdf5c); byte(A1B1 as u16, 0x7e);
        word(DAS1L as u16, 0x1000); byte(DAS10 as u16, 0);
        word(A2A1L as u16, 0); byte(NTRL1 as u16, 0);
        byte(MDMAEN as u16, 2);
    }
}

pub fn load_pause_assets<H: Host + crate::menus::host::Host + ?Sized>(game: &mut H) {
    GraphicsAsset::VramWords(0xb68000, 0, 0x4000).load(game);
    GraphicsAsset::VramWords(0xb6c000, 0x2000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
    GraphicsAsset::VramWords(0xb6e000, 0x3800, 0x800).load(game);
    GraphicsAsset::WramBytes(0xb6e400, 0x7e3400, 0x400).load(game);
    GraphicsAsset::WramBytes(0xb6e800, 0x7e3800, 0x800).load(game);
    crate::menus::menu_tilemap::prepare_equipment_frame(game);
}

/// Upload the pause map and the area label.
pub fn upload_pause_map<
    H: Host + crate::menus::host::Host + crate::runtime_access::RuntimeAccess + ?Sized,
>(
    game: &mut H,
) {
    word! {game; reg_BG1HOFS = word!(game.reg_BG4HOFS); reg_BG1VOFS = word!(game.reg_BG4VOFS); }
    register_writes! { game;
        byte(VMADDL as u16, 0); byte(VMADDH as u16, 0x30);
        byte(VMAIN as u16, 0x80);
    }
    crate::menus::map_menu::render_map(crate::menus::map_menu::MapScreen::Pause, game);
    setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x7e4000, 0x1000, game);
    register_writes! { game;
        byte(MDMAEN as u16, 2); byte(VMADDL as u16, 0xaa);
        byte(VMADDH as u16, 0x38); byte(VMAIN as u16, 0x80);
        byte(DMAP1 as u16, 1); byte(BBAD1 as u16, 0x18);
        byte(DAS1L as u16, 0x18); byte(DAS1H as u16, 0);
    }
    let mut area: u16 = room_area(game);
    if room_area(game) as i32 - 7i32 & 0x8000 == 0 {
        area = 0;
    }
    register_writes! { game;
        word(A1T1L as u16,
        crate::gameplay_tables::area_label(game, area as isize),); byte(A1B1 as u16, 0x82);
        byte(MDMAEN as u16, 2);
    }
}

fn room_area<H: crate::runtime_access::RuntimeAccess + ?Sized>(game: &mut H) -> u16 {
    use crate::session_access::SessionAccess;
    game.runtime().session().room.area
}

pub fn reset_gameplay_layers<H: Host + ?Sized>(clear_current_irq: bool, game: &mut H) {
    word!(game.fx_layer_blending_config_a = 0);
    if clear_current_irq {
        word!(game.cur_irq_handler = 0);
    }
    reset_layer_mirrors(game);
    game.write_owned_byte(0x55, 9);
}

pub fn reset_layer_mirrors<H: Host + ?Sized>(game: &mut H) {
    game.write_owned_byte(0x6e, 0);
    game.write_owned_byte(0x71, 0);
    game.write_owned_byte(0x69, 16);
    fill_owned(game, 0x6b..0x6e, 0);
}

pub fn fade<H: Host + ?Sized>(fade_in: bool, game: &mut H) {
    let remaining = word!(game.screen_fade_counter).wrapping_sub(1);
    let due = (remaining as i16) < 0;
    word!(
        game.screen_fade_counter = if due {
            word!(game.screen_fade_delay)
        } else {
            remaining
        }
    );
    if !due {
        return;
    }
    let brightness = game.compatibility_byte(0x51) & 0xf;
    let next = if fade_in {
        if brightness == 15 {
            return;
        }
        brightness + 1
    } else {
        match brightness {
            0 => return,
            1 => 0x80,
            value => value - 1,
        }
    };
    game.write_owned_byte(0x51, next);
}

pub fn wait<H: Host + ?Sized>(update: bool, game: &mut H) -> u8 {
    if game.compatibility_byte(0x782) != 0 {
        game.write_owned_byte(0x782, 0);
        return 0;
    }
    if update {
        game.write_owned_byte(0x5b4, 1);
    }
    game.write_owned_byte(0x782, 1);
    1
}

pub fn enable<H: Host + ?Sized>(enabled: bool, game: &mut H) {
    let mask = if enabled { 0x80 } else { 0 };
    let value = game.compatibility_byte(0x84) & 0x7f | mask;
    game.write_register(NMITIMEN as u16, game.compatibility_byte(0x84) & 0x7f | mask);
    // The hardware write may change RAM; publish the value captured before it.
    game.write_owned_byte(0x84, value);
}

fn upload_latches<H: Host + ?Sized>(register: u16, source: usize, count: u16, game: &mut H) {
    for index in 0..count {
        let address = register + index;
        let offset = source + usize::from(index) * 2;
        game.write_register(address, game.compatibility_byte(offset));
        game.write_register(address, game.compatibility_byte(offset + 1));
    }
}

fn upload_registers<H: Host + ?Sized>(game: &mut H) {
    register_writes! { game;
        byte(NMITIMEN as u16, game.compatibility_byte(0x84)); byte(INIDISP as u16, game.compatibility_byte(0x51));
        byte(OBSEL as u16, game.compatibility_byte(0x52)); byte(BGMODE as u16, game.compatibility_byte(0x55));
        byte(MOSAIC as u16, game.compatibility_byte(0x57)); byte(BG1SC as u16, game.compatibility_byte(0x58));
        byte(BG2SC as u16, game.compatibility_byte(0x59)); byte(BG3SC as u16, game.compatibility_byte(0x5a));
        byte(BG4SC as u16, game.compatibility_byte(0x5c)); byte(BG12NBA as u16, game.compatibility_byte(0x5d));
        byte(BG34NBA as u16, game.compatibility_byte(0x5e)); byte(M7SEL as u16, game.compatibility_byte(0x5f));
        byte(W12SEL as u16, game.compatibility_byte(0x60)); byte(W34SEL as u16, game.compatibility_byte(0x61));
        byte(WOBJSEL as u16, game.compatibility_byte(0x62)); byte(WH0 as u16, game.compatibility_byte(0x63));
        byte(WH1 as u16, game.compatibility_byte(0x64)); byte(WH2 as u16, game.compatibility_byte(0x65));
        byte(WH3 as u16, game.compatibility_byte(0x66)); byte(WBGLOG as u16, game.compatibility_byte(0x67));
        byte(WOBJLOG as u16, game.compatibility_byte(0x68));
    }
    game.write_owned_byte(0x6a, game.compatibility_byte(0x69));
    register_writes! { game;
        byte(TM as u16, game.compatibility_byte(0x69)); byte(TMW as u16, game.compatibility_byte(0x6c));
        byte(TS as u16, game.compatibility_byte(0x6b)); byte(TSW as u16, game.compatibility_byte(0x6d));
        byte(CGWSEL as u16, game.compatibility_byte(0x6f)); byte(CGADSUB as u16, game.compatibility_byte(0x72));
    }
    game.write_owned_byte(0x70, game.compatibility_byte(0x6e));
    game.write_owned_byte(0x73, game.compatibility_byte(0x71));
    register_writes! { game;
        byte(COLDATA as u16, game.compatibility_byte(0x74)); byte(COLDATA as u16, game.compatibility_byte(0x75));
        byte(COLDATA as u16, game.compatibility_byte(0x76)); byte(SETINI as u16, game.compatibility_byte(0x77));
    }

    // Scroll and Mode 7 words latch low then high through the same register.
    upload_latches(BG1HOFS as u16, 0xb1, 8, game);
    game.write_owned_byte(0x7ec, game.compatibility_byte(0x56));
    if game.compatibility_byte(0x55) & 7 == 7 || game.compatibility_byte(0x56) & 7 == 7 {
        upload_latches(M7A as u16, 0x78, 6, game);
    }
}

fn upload_palette_and_oam<H: Host + ?Sized>(game: &mut H) {
    // Channel 0 uploads the OAM entries and their high table from RAM.
    register_writes! { game;
        word(DMAP0 as u16, 0x0400); word(A1T0L as u16, 0x0370);
        byte(A1B0 as u16, 0); word(DAS0L as u16, 0x0220);
        word(OAMADDL as u16, 0);
    }

    // Channel 1 uploads palette_buffer to CGRAM.
    setup_transfer(0x2200, 0x7ec000, 0x0200, game);
    register_writes! { game;
        byte(CGADD as u16, 0); byte(MDMAEN as u16, 3);
    }
    crate::probe::hit("palette.dma.uploaded");
}

struct Refresh<'a, H: ?Sized>(&'a mut H);

// The host's uploads either complete or keep their original panic/failure
// behavior; this adapter never produces a portable DisplayError.
impl<H: Host + ?Sized> crate::scene::commit::RefreshHost for Refresh<'_, H> {
    fn pending_nmi(&self) -> u8 {
        self.0.compatibility_byte(0x5b4)
    }
    fn palette_and_objects(&mut self) -> Result<(), DisplayError> {
        upload_palette_and_oam(self.0);
        Ok(())
    }
    fn samus(&mut self) -> Result<(), DisplayError> {
        self.0.upload_samus();
        Ok(())
    }
    fn animations(&mut self) -> Result<(), DisplayError> {
        self.0.upload_animated_tiles();
        Ok(())
    }
    fn controls(&mut self) -> Result<(), DisplayError> {
        upload_registers(self.0);
        Ok(())
    }
    fn hdma_mask(&self, slot: usize) -> u16 {
        self.0.hdma_objects().objects[slot].channels
    }
    fn hdma_bank_slot(&self, slot: usize) -> u8 {
        self.0.hdma_objects().objects[slot].bank_slot as u8
    }
    fn hdma_table(&self, slot: usize) -> u16 {
        self.0.hdma_objects().objects[slot].table
    }
    fn bind_hdma(&mut self, bank_slot: u16, table: u16) -> Result<(), DisplayError> {
        self.0.write_register_word(A1T0L as u16 + bank_slot, table);
        Ok(())
    }
    fn mode(&self) -> u8 {
        self.0.compatibility_byte(0x55)
    }
    fn fake_mode(&self) -> u8 {
        self.0.compatibility_byte(0x56)
    }
    fn mode7(&mut self) -> Result<(), DisplayError> {
        self.0.process_mode7();
        Ok(())
    }
    fn tilemaps(&mut self) -> Result<(), DisplayError> {
        self.0.flush_tilemaps();
        Ok(())
    }
    fn readbacks(&mut self) -> Result<(), DisplayError> {
        self.0.process_readbacks();
        Ok(())
    }
    fn hdma_enable(&self) -> u8 {
        self.0.compatibility_byte(0x85)
    }
    fn publish_hdma(&mut self, value: u8) -> Result<(), DisplayError> {
        self.0.write_register(HDMAEN as u16, value);
        Ok(())
    }
    fn set_pending_nmi(&mut self, value: u8) {
        self.0.write_owned_byte(0x5b4, value)
    }
    fn missed(&self) -> u8 {
        self.0.compatibility_byte(0x5ba)
    }
    fn set_missed(&mut self, value: u8) {
        self.0.write_owned_byte(0x5ba, value)
    }
    fn max_missed(&self) -> u8 {
        self.0.compatibility_byte(0x5bb)
    }
    fn set_max_missed(&mut self, value: u8) {
        self.0.write_owned_byte(0x5bb, value)
    }
    fn frame_byte(&self) -> u8 {
        self.0.compatibility_byte(0x5b5)
    }
    fn set_frame_byte(&mut self, value: u8) {
        self.0.write_owned_byte(0x5b5, value)
    }
    fn frame_word(&self) -> u16 {
        let game = &*self.0;
        word!(game.nmi_frame_counter_word)
    }
    fn set_frame_word(&mut self, value: u16) {
        let game = &mut *self.0;
        word!(game.nmi_frame_counter_word = value);
    }
    fn including_lag(&self) -> u16 {
        let game = &*self.0;
        word!(game.nmi_frame_counter_including_lag)
    }
    fn set_including_lag(&mut self, value: u16) {
        let game = &mut *self.0;
        word!(game.nmi_frame_counter_including_lag = value);
    }
}

/// Run the display refresh in the original order.
pub fn run<H: Host + ?Sized>(game: &mut H) {
    crate::scene::commit::refresh(&mut Refresh(game))
        .expect("native display refresh adapter cannot return an error");
}

// Graphics loads keep source, destination, then transfer size together.
// Decompression uses RAM offsets; VRAM uses word addresses. WRAM transfers
// retain all three destination-register bytes, including ignored bank bits.
#[derive(Clone, Copy, Debug)]
pub enum GraphicsAsset {
    Decompress(u32, usize),
    VramWords(u32, u16, u16),
    VramLowBytes(u32, u16, u16),
    VramHighBytes(u32, u16, u16),
    WramBytes(u32, u32, u16),
}

impl GraphicsAsset {
    pub fn load_planes<H: Host + ?Sized>(self, fill: u8, word_reset: bool, game: &mut H) {
        Self::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
        game.fill_vram_low_bytes(fill, word_reset);
        self.load(game);
        Self::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
    }

    pub fn load<H: Host + ?Sized>(self, game: &mut H) {
        crate::probe::hit(match &self {
            Self::Decompress(..) => "graphics-load-decompress",
            Self::VramWords(..) => "graphics-load-vram-words",
            Self::VramLowBytes(..) => "graphics-load-vram-low",
            Self::VramHighBytes(..) => "graphics-load-vram-high",
            Self::WramBytes(..) => "graphics-load-wram",
        });
        let (mode, source, size, vram) = match self {
            Self::Decompress(source, destination) => {
                decompress_to_ram(source, destination, game);
                return;
            }
            Self::VramWords(source, destination, size) => {
                ([1, 0x18], source, size, Some((destination, 0x80)))
            }
            Self::VramLowBytes(source, destination, size) => {
                ([0, 0x18], source, size, Some((destination, 0)))
            }
            Self::VramHighBytes(source, destination, size) => {
                ([0, 0x19], source, size, Some((destination, 0x80)))
            }
            Self::WramBytes(source, destination, size) => {
                register_writes! { game;
                    byte(WMADDL as u16, destination as u8);
                    byte(WMADDM as u16, (destination >> 8) as u8);
                    byte(WMADDH as u16, (destination >> 16) as u8);
                }
                ([0, 0x80], source, size, None)
            }
        };
        if let Some((destination, vmain)) = vram {
            register_writes! { game;
                byte(VMADDL as u16, destination as u8); byte(VMADDH as u16, (destination >> 8) as u8);
                byte(VMAIN as u16, vmain);
            }
        }
        // The old packed descriptor read its adjacent mode/register bytes natively.
        setup_transfer(u16::from_ne_bytes(mode), source, size, game);
        game.start_graphics_dma(source);
    }
}

// The VRAM decoder output reads back through the PPU data port.
struct DecompressionVram<'a, 'b, H: ?Sized> {
    game: &'a mut H,
    prepared: Option<core::slice::Iter<'b, u8>>,
}
impl<H: Host + ?Sized> decompression::Output for DecompressionVram<'_, '_, H> {
    fn next_byte(&mut self) -> u8 {
        match self.prepared.as_mut() {
            Some(stream) => *stream.next().expect("validated prepared VRAM room stream"),
            None => decompression::next_byte(self.game),
        }
    }
    fn read(&mut self, position: i32) -> u8 {
        read_ppu_byte(position as u16, self.game)
    }
    fn write(&mut self, position: i32, value: u8, back_reference: bool) {
        if back_reference {
            self.game
                .write_register_word(VMADDL as u16, (position >> 1) as u16);
        }
        self.game
            .write_register((VMDATAL as i32 + (position & 1)) as u16, value);
    }
}
fn read_ppu_byte<H: Host + ?Sized>(addr: u16, game: &mut H) -> u8 {
    game.write_register_word(VMADDL as u16, addr >> 1);
    game.read_register_word(RDVRAML as u16);
    let data: u16 = game.read_register_word(RDVRAML as u16);
    (if addr & 1 != 0 {
        (data as i32 & 0xff00) >> 8
    } else {
        data as i32
    }) as u8
}
pub fn decompress_to_vram<H: Host + ?Sized>(src: u32, dst_addr: u16, game: &mut H) {
    game.set_decompress_source(src);
    decompression::decode(
        i32::from(dst_addr),
        &mut DecompressionVram {
            game,
            prepared: None,
        },
    );
}
pub fn decompress_prepared_vram<H: Host + ?Sized>(stream: &[u8], destination: u16, game: &mut H) {
    decompression::decode(
        i32::from(destination),
        &mut DecompressionVram {
            game,
            prepared: Some(stream.iter()),
        },
    );
}

fn mult8x8(a: u8, b: u8) -> u16 {
    (a as i32 * b as i32) as u16
}

pub fn queue_scrolling_tiles<H: Host + ?Sized>(k: u16, layout: ScrollingLayout, game: &mut H) {
    if word!(game.irq_enable_mode7) != 0 {
        return;
    }
    let row = matches!(layout, ScrollingLayout::Row);
    crate::probe::hit(match (row, k != 0) {
        (false, false) => "scroll_tiles.column.bg1",
        (false, true) => "scroll_tiles.column.bg2",
        (true, false) => "scroll_tiles.row.bg1",
        (true, true) => "scroll_tiles.row.bg2",
    });
    let product = mult8x8(
        word!(game.blocks_to_update_y_block) as u8,
        word!(game.room_width_in_blocks) as u8,
    );
    let mut source_offset = product
        .wrapping_add(word!(game.blocks_to_update_x_block))
        .wrapping_mul(2)
        .wrapping_add(2);
    if k != 0 {
        source_offset = source_offset.wrapping_add(0x9600);
    }
    let source = 0x10000 + usize::from(source_offset);
    let parameters = (if row { 0x964 } else { 0x956 }) + usize::from(k);
    // Columns store the wrapped size first; rows store the unwrapped size first.
    if row {
        let within_screen = word!(game.vram_blocks_to_update_x_block) & 15;
        set_native_word(parameters, 4 * (16 - within_screen), game);
        set_native_word(parameters + 2, 4 * (within_screen + 1), game);
    } else {
        let wrapped = 4 * (word!(game.vram_blocks_to_update_y_block) & 15);
        set_native_word(parameters + 2, wrapped, game);
        set_native_word(parameters, (wrapped ^ 0x3f) + 1, game);
    }
    let tile_row = mult8x8(word!(game.vram_blocks_to_update_y_block) as u8 & 15, 0x40);
    let tile_column = word!(game.vram_blocks_to_update_x_block) & 31;
    let origin = tile_row.wrapping_add(2 * tile_column);
    let mut screen = if tile_column >= 16 {
        addr_unk_6053E0 as u16
    } else {
        addr_unk_605000 as u16
    };
    if k != 0 {
        screen = screen.wrapping_sub(word!(game.size_of_bg2));
    }
    set_native_word(parameters + 4, origin.wrapping_add(screen), game);
    let wrapped_destination = if row {
        let mut wrapped_screen = if tile_column >= 16 {
            addr_unk_605000 as u16
        } else {
            addr_unk_605400 as u16
        };
        if k != 0 {
            wrapped_screen = wrapped_screen.wrapping_sub(word!(game.size_of_bg2));
        }
        tile_row.wrapping_add(wrapped_screen)
    } else {
        (2 * tile_column).wrapping_add(screen)
    };
    set_native_word(parameters + 6, wrapped_destination, game);
    let buffer_base = if row {
        word_arrays::bg1_column_update_tilemap_top_halves
    } else {
        word_arrays::bg1_column_update_tilemap_left_halves
    };
    let buffer = buffer_base + if k != 0 { 264 } else { 0 };
    let half_bytes = if row { 68 } else { 64 };
    let wrapped_source = native_word(parameters, game).wrapping_add(buffer as u16);
    set_native_word(parameters + 8, wrapped_source, game);
    set_native_word(
        parameters + 10,
        wrapped_source.wrapping_add(half_bytes),
        game,
    );
    let mut source_delta = 0u16;
    for block_index in 0..if row { 17 } else { 16 } {
        let block = native_word(source + 2 * usize::from(source_delta >> 1), game);
        let first = buffer + 4 * block_index;
        let second = first + usize::from(half_bytes);
        let destinations = if row {
            [first, first + 2, second, second + 2]
        } else {
            [first, second, first + 2, second + 2]
        };
        let flip = usize::from((block >> 10) & 3);
        let tile_flip = (block & 0x0c00) << 4;
        let first_tile = usize::from(block & 0x03ff) * 4;
        for (corner, destination) in destinations.into_iter().enumerate() {
            // Read each tile immediately before storing it: staging may overlap the table.
            let source = 0xa000 + 2 * (first_tile + (corner ^ flip));
            let tile = at!(game, source;) ^ tile_flip;
            at!(game, destination; = tile);
        }
        // The column stride is reread after drawing each block.
        source_delta = source_delta.wrapping_add(if row {
            2
        } else {
            word!(game.room_width_in_blocks).wrapping_mul(2)
        });
    }
    set_native_word(
        parameters + 12,
        native_word(parameters + 12, game).wrapping_add(1),
        game,
    );
}

pub fn native_word<H: Host + ?Sized>(offset: usize, game: &H) -> u16 {
    u16::from_ne_bytes([
        game.compatibility_byte(offset),
        game.compatibility_byte(offset + 1),
    ])
}

pub fn set_native_word<H: Host + ?Sized>(offset: usize, value: u16, game: &mut H) {
    let bytes = value.to_ne_bytes();
    game.write_owned_byte(offset, bytes[0]);
    game.write_owned_byte(offset + 1, bytes[1]);
}

/// Publish a native queue record, evaluating each source after preceding stores.
pub fn queue_native_transfer<H: Host + ?Sized>(
    mut cursor: u16,
    size: u16,
    source: impl FnOnce(&H) -> u16,
    bank: u8,
    destination: impl FnOnce(&H) -> u16,
    game: &mut H,
) -> u16 {
    set_native_word(0xd0 + usize::from(cursor), size, game);
    cursor = cursor.wrapping_add(2);
    let source = source(game);
    set_native_word(0xd0 + usize::from(cursor), source, game);
    cursor = cursor.wrapping_add(2);
    game.write_owned_byte(0xd0 + usize::from(cursor), bank);
    cursor = cursor.wrapping_add(1);
    let destination = destination(game);
    set_native_word(0xd0 + usize::from(cursor), destination, game);
    cursor = cursor.wrapping_add(2);
    word!(game.vram_write_queue_tail = cursor);
    cursor
}

pub fn queue_fx_clear<H: Host + ?Sized>(game: &mut H) {
    word_transfers!(game, descending, 1920; native(0x4efe) <- value(6222));
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    set_native_word(entry, 3840, game);
    set_native_word(entry + 2, 0x4000, game);
    // The bank store is deliberately a native word, overlapping the destination low byte.
    set_native_word(entry + 4, 126, game);
    set_native_word(entry + 5, addr_unk_605880 as u16, game);
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

fn write_readback_word<H: Host + ?Sized>(game: &mut H, offset: usize, value: u16) {
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(offset, bytes[0]);
    game.ram().write_byte(offset + 1, bytes[1]);
}

/// Queue a VRAM readback of `size` bytes into WRAM at `destination`, from
/// the screen base in the register mirror at `register` plus `vram_offset`.
pub fn enqueue_readback<H: Host + ?Sized>(
    register: usize,
    vram_offset: u16,
    destination: u16,
    size: u16,
    retain_mode: bool,
    game: &mut H,
) {
    let tail = checked_word!(game.vram_read_queue_tail);
    let entry = 0x340 + usize::from(tail);
    let target =
        ((u16::from(game.ram().read_byte(register)) & 0xfc) << 8).wrapping_add(vram_offset);
    write_readback_word(game, entry, target);
    if !retain_mode {
        write_readback_word(game, entry + 2, 129);
    }
    // These stores overlap. Stage five of X-ray retains the preceding mode byte.
    write_readback_word(game, entry + 3, 57);
    write_readback_word(game, entry + 4, destination);
    write_readback_word(game, entry + 6, 126);
    write_readback_word(game, entry + 7, size);
    checked_word!(game.vram_read_queue_tail = tail.wrapping_add(9));
}
