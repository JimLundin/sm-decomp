//! Ordered display refresh and the NMI request/completion handshake, plus the
//! gameplay and pause display setup, graphics loads and tilemap queues.
use crate::constants::bank_60::ADDR_UNK_605880;
use crate::decompression::{self, decompress_to_ram};
use crate::display::control::{BgMode, FixedColor, Layers, ScreenSettings};
use crate::display::preparation::Field as DisplayField;
use crate::display::registers::{
    A1B0, A1T0L, BG1HOFS, BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA, BGMODE, CGADD, CGADSUB,
    CGWSEL, COLDATA, DAS0L, DMAP0, HDMAEN, HTIMEL, INIDISP, M7A, M7SEL, MDMAEN, MOSAIC, NMITIMEN,
    OAMADDL, OBSEL, RDVRAML, SETINI, TM, TMW, TS, TSW, VTIMEL, W12SEL, W34SEL, WBGLOG, WH0, WH1,
    WH2, WH3, WOBJLOG, WOBJSEL,
};
use crate::display::sequence::{Registers, register_writes};
use crate::display::transfer::{ScrollingLayout, TilemapQueue, setup_transfer};
use crate::display::video::{
    Background, Command, Control, Mirror, Upload, apply_controls, apply_mirrored,
};
use crate::ram::CompatibilityRam;
use crate::scene::irq;
use crate::scene::state::DisplayError;
use crate::word_transfers;
use core::convert::Infallible;

/// Stores to the display's RAM mirrors, which the owned display state holds
/// when the Session owns the presentation.
pub trait DisplayOwner {
    /// Store `value` at `at` through the owned display state.
    fn write_owned_byte(&mut self, at: usize, value: u8);
}

/// What the display refresh needs besides RAM, registers and the cartridge:
/// owned display stores and the uploads the compatibility runtime still owns.
pub trait Host:
    crate::fx_domain::Controls
    + crate::display::preparation::Access
    + CompatibilityRam
    + crate::ram::WorkRamAccess
    + Registers
    + crate::word_transfers::LoadedRom
    + crate::cartridge::LoadedImage
    + crate::decompression::DecompressionHost
    + crate::records::Records
    + DisplayOwner
    + crate::display::queued_tilemaps::Scrolling
    + crate::display::video::Video
    + crate::hdma_objects::HdmaTable
{
    /// The HDMA object table as the compatibility projection reads it.
    fn hdma_objects(&self) -> crate::hdma::HdmaObjects;
    /// Start the channel-1 graphics DMA for a load from `source`, reading the
    /// prepared room image when it holds that source.
    fn start_graphics_dma(&mut self, source: u32);
    fn upload_samus(&mut self);
    fn upload_animated_tiles(&mut self);
    fn process_mode7(&mut self);
    fn process_readbacks(&mut self);
    /// The VRAM readback queue's tail ($7E:0360), the byte offset of its
    /// next 9-byte record at $7E:0340.
    fn readback_queue_tail(&self) -> u16;
    fn set_readback_queue_tail(&mut self, tail: u16);
    /// Store byte `offset` of the VRAM readback queue's records ($7E:0340).
    fn store_readback_byte(&mut self, offset: usize, value: u8);
    fn flush_tilemaps(&mut self);
    /// Fill VRAM's low bytes with `fill` from address 0, resetting the
    /// address as one word or two bytes.
    fn fill_vram_low_bytes(&mut self, fill: u8, word_reset: bool);
}

/// The producer name of the pause screen's display commands.
const PAUSE: &str = "pause display";
/// The producer name of the display initialisation's commands.
const INITIALISE: &str = "display initialisation";
/// BG1, BG2, BG3 and the sprites: gameplay's main screen.
const GAMEPLAY_LAYERS: Layers = Layers::of(true, true, true, false, true);

/// Set the forced-blank mirror bit for the next display refresh.
pub fn set_blank<H: Host + ?Sized>(blank: bool, game: &mut H) {
    crate::frame_display_byte!(game, crate::display::preparation::Field::Brightness => crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Brightness) & 0x7f | if blank { 0x80 } else { 0 });
}

pub fn pause_graphics<H: Host + ?Sized>(paused: bool, game: &mut H) {
    let (objects, bg1, bg2, bg3) = if paused {
        (1, 0x31, 0x38, 0x58)
    } else {
        (3, 0x51, 0x49, 0x5a)
    };
    apply_controls(
        game,
        PAUSE,
        &[
            (
                Control::ObjectLayout(objects),
                Mirror::After(DisplayField::ObjectLayout),
            ),
            (
                Control::BackgroundMode(BgMode::MODE1_BG3_PRIORITY.byte()),
                Mirror::After(DisplayField::Mode),
            ),
            (
                Control::CharactersBg1Bg2(0),
                Mirror::Before(DisplayField::Characters0),
            ),
            (
                Control::CharactersBg3Bg4(4),
                Mirror::Before(DisplayField::Characters1),
            ),
            (
                Control::Tilemap(Background::Bg1, bg1),
                Mirror::Before(DisplayField::Screens0),
            ),
            (
                Control::Tilemap(Background::Bg2, bg2),
                Mirror::Before(DisplayField::Screens1),
            ),
            (
                Control::Tilemap(Background::Bg3, bg3),
                Mirror::Before(DisplayField::Screens2),
            ),
            (
                Control::Tilemap(Background::Bg4, 0),
                Mirror::Before(DisplayField::Screens3),
            ),
        ],
    );
    if paused {
        apply_controls(
            game,
            PAUSE,
            &[
                (
                    Control::MainScreen(GAMEPLAY_LAYERS),
                    Mirror::After(DisplayField::MainLayers),
                ),
                (Control::Mosaic(0), Mirror::After(DisplayField::Mosaic)),
            ],
        );
        for field in [
            crate::display::preparation::Field::FixedColor0,
            crate::display::preparation::Field::FixedColor1,
            crate::display::preparation::Field::FixedColor2,
        ] {
            crate::frame_display_byte!(game, field => crate::display::preparation::Access::display_control_byte(&*game, field) & 0xe0)
        }
        crate::frame_display_byte!(game, crate::display::preparation::Field::NextColorMath => 0);
    }
}

/// The ordered CPU I/O and OAM reset.
pub fn initialize_io_and_oam<H: Host + ?Sized>(
    brightness: Option<crate::display::control::Brightness>,
    game: &mut H,
) {
    apply_mirrored(
        game,
        INITIALISE,
        Control::Interrupts(1),
        DisplayField::NmiEnable,
        false,
    );
    game.apply(INITIALISE, Command::ClearCpuIo);
    apply_controls(
        game,
        INITIALISE,
        &[
            (
                Control::HdmaChannels(0),
                Mirror::After(DisplayField::HdmaEnable),
            ),
            (
                Control::MemorySpeed(1),
                Mirror::After(DisplayField::MemorySelect),
            ),
        ],
    );
    if let Some(brightness) = brightness {
        apply_mirrored(
            game,
            INITIALISE,
            Control::Brightness(brightness),
            DisplayField::Brightness,
            false,
        );
    }
    apply_controls(
        game,
        INITIALISE,
        &[
            (
                Control::ObjectLayout(3),
                Mirror::After(DisplayField::ObjectLayout),
            ),
            (
                Control::ObjectAddressLow(0),
                Mirror::After(DisplayField::ObjectAddressLow),
            ),
            (
                Control::ObjectAddressHigh(0x80),
                Mirror::After(DisplayField::ObjectAddressHigh),
            ),
        ],
    );
    game.apply(INITIALISE, Command::ObjectBytes([0, 0]));
}

/// Gameplay display registers, cleared tilemaps and the common graphics.
pub fn initialize_gameplay<H: Host + crate::sprite_drawing::ObjectAccess + ?Sized>(game: &mut H) {
    initialize_io_and_oam(Some(crate::display::control::Brightness::BLANK_DARK), game);
    // The original clears the BG1/BG2 window selection twice.
    apply_controls(
        game,
        INITIALISE,
        &[
            (
                Control::BackgroundMode(BgMode::MODE1_BG3_PRIORITY.byte()),
                Mirror::After(DisplayField::Mode),
            ),
            (Control::Mosaic(0), Mirror::After(DisplayField::Mosaic)),
            (
                Control::CharactersBg1Bg2(0),
                Mirror::Before(DisplayField::Characters0),
            ),
            (
                Control::CharactersBg3Bg4(4),
                Mirror::Before(DisplayField::Characters1),
            ),
            (
                Control::Tilemap(Background::Bg1, 0x51),
                Mirror::Before(DisplayField::Screens0),
            ),
            (
                Control::Tilemap(Background::Bg2, 0x49),
                Mirror::Before(DisplayField::Screens1),
            ),
            (
                Control::Tilemap(Background::Bg3, 0x5a),
                Mirror::Before(DisplayField::Screens2),
            ),
            (
                Control::Tilemap(Background::Bg4, 0),
                Mirror::Before(DisplayField::Screens3),
            ),
            (Control::VramStep(0), Mirror::None),
            (
                Control::WindowsBg1Bg2(0),
                Mirror::After(DisplayField::WindowSelection0),
            ),
            (
                Control::WindowsBg1Bg2(0),
                Mirror::After(DisplayField::WindowSelection0),
            ),
            (
                Control::WindowsBg3Bg4(0),
                Mirror::After(DisplayField::WindowSelection1),
            ),
            (
                Control::WindowsSpritesColor(0),
                Mirror::After(DisplayField::WindowSelection2),
            ),
            (
                Control::Window1Left(0),
                Mirror::After(DisplayField::WindowPositions0),
            ),
            (
                Control::Window1Right(0),
                Mirror::After(DisplayField::WindowPositions1),
            ),
            (
                Control::Window2Left(0),
                Mirror::After(DisplayField::WindowPositions2),
            ),
            (
                Control::Window2Right(0),
                Mirror::After(DisplayField::WindowPositions3),
            ),
            (
                Control::WindowLogicBackgrounds(0),
                Mirror::After(DisplayField::WindowLogic0),
            ),
            (
                Control::WindowLogicSpritesColor(0),
                Mirror::After(DisplayField::WindowLogic1),
            ),
            (
                Control::MainScreen(GAMEPLAY_LAYERS),
                Mirror::After(DisplayField::MainLayers),
            ),
            (
                Control::MainWindowMask(Layers::NONE),
                Mirror::After(DisplayField::MainWindow),
            ),
            (
                Control::SubScreen(Layers::NONE),
                Mirror::After(DisplayField::SubLayers),
            ),
            (
                Control::SubWindowMask(Layers::NONE),
                Mirror::After(DisplayField::SubWindow),
            ),
            (
                Control::ColorSelection(0),
                Mirror::After(DisplayField::NextColorSelection),
            ),
            (
                Control::ColorMath(0),
                Mirror::After(DisplayField::NextColorMath),
            ),
        ],
    );
    game.apply(INITIALISE, Command::FixedColor(FixedColor::all(0)));
    apply_mirrored(
        game,
        INITIALISE,
        Control::ScreenSettings(ScreenSettings::STANDARD.byte()),
        DisplayField::Geometry,
        false,
    );
    crate::sprite_drawing::reset_cursor(game);
    word_transfers!(game, ascending, 1023; native(0x3000) <- value(0));
    word_transfers!(game, ascending, 1023; native(0x4000) <- value(0x6f));
    word_transfers!(game, ascending, 127; native(0x4000) <- value(0x2c0f));
    GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
    GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2e00).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5000, 0x1000).load(game);
    GraphicsAsset::VramWords(0x7e4000, 0x5800, 0x800).load(game);
    // The common palettes ($9A:8000) into the palette buffer, in order.
    let colors: [u16; 256] = word_transfers::rom_words(game, 0x9a_8000);
    for (index, color) in colors.into_iter().enumerate() {
        game.set_view_word(crate::projection::Field::PaletteColor(index), color);
    }
}

/// Read the room's BG2 tilemap back into RAM for the pause screen.
pub fn capture_pause_background<H: Host + ?Sized>(game: &mut H) {
    // The BG2 tilemap's base, one word in, as the original addresses it.
    let address = (u16::from(
        crate::display::preparation::Access::display_control_byte(
            &*game,
            crate::display::preparation::Field::Screens1,
        ) & 0xfc,
    ) << 8)
        | 1;
    game.apply(
        PAUSE,
        Command::ReadVramNow {
            address,
            destination: crate::display::transfer::Source {
                address: 0xdf5c,
                bank: 0x7e,
            },
            bytes: 0x1000,
        },
    );
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
pub fn upload_pause_map<H: Host + crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        crate::menus::prelude::display_scroll!(
            game.Bg1H = crate::menus::prelude::display_scroll!(game.Bg4H)
        );
        crate::menus::prelude::display_scroll!(
            game.Bg1V = crate::menus::prelude::display_scroll!(game.Bg4V)
        );
    }
    const STEP_AFTER_HIGH: u8 = 0x80;
    game.apply(
        PAUSE,
        Command::VramPort {
            address: 0x3000,
            step: STEP_AFTER_HIGH,
        },
    );
    crate::menus::map_menu::render_map(crate::menus::map_menu::MapScreen::Pause, game);
    game.apply(
        PAUSE,
        Command::TilesThroughPort {
            source: crate::display::transfer::Source {
                address: 0x4000,
                bank: 0x7e,
            },
            bytes: 0x1000,
        },
    );
    game.apply(
        PAUSE,
        Command::VramPort {
            address: 0x38aa,
            step: STEP_AFTER_HIGH,
        },
    );
    // The label is looked up before the channel's first byte, which the
    // original wrote first: neither the lookup's ROM table nor the room's
    // area can see a channel register.
    let mut area: u16 = room_area(game);
    if (room_area(game) as i32 - 7i32) & 0x8000 == 0 {
        area = 0;
    }
    let label = crate::gameplay_tables::area_label(game, area as isize);
    game.apply(
        PAUSE,
        Command::TilesThroughPortBytes {
            source: crate::display::transfer::Source {
                address: label,
                bank: 0x82,
            },
            bytes: 0x18,
        },
    );
}

fn room_area<H: crate::runtime_access::RuntimeAccess + ?Sized>(game: &mut H) -> u16 {
    use crate::session_access::SessionAccess;
    game.runtime().session().room.area
}

pub fn reset_gameplay_layers<H: Host + ?Sized>(clear_current_irq: bool, game: &mut H) {
    crate::sequence_state::with_words_projected(
        game,
        |_, words: &mut crate::session_words::SessionWords| {
            words.fx_layer_blending_config_a = 0;
        },
    );
    if clear_current_irq {
        crate::frame_display_word!(game.CurrentInterrupt = 0);
    }
    reset_layer_mirrors(game);
    crate::frame_display_byte!(game, crate::display::preparation::Field::Mode => 9);
}

pub fn reset_layer_mirrors<H: Host + ?Sized>(game: &mut H) {
    crate::frame_display_byte!(game, crate::display::preparation::Field::NextColorSelection => 0);
    crate::frame_display_byte!(game, crate::display::preparation::Field::NextColorMath => 0);
    crate::frame_display_byte!(game, crate::display::preparation::Field::MainLayers => 16);
    for field in [
        crate::display::preparation::Field::SubLayers,
        crate::display::preparation::Field::MainWindow,
        crate::display::preparation::Field::SubWindow,
    ] {
        crate::display::preparation::Access::store_display_control_byte(
            &mut *game,
            field,
            0,
            crate::display::preparation::Store::Owned,
        );
    }
}

pub fn fade<H: Host + crate::screen_fade::Owner + ?Sized>(fade_in: bool, game: &mut H) {
    let due = game.tick_screen_fade();
    if !due {
        return;
    }
    let brightness = crate::display::preparation::Access::display_control_byte(
        &*game,
        crate::display::preparation::Field::Brightness,
    ) & 0xf;
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
    crate::frame_display_byte!(game, crate::display::preparation::Field::Brightness => next);
}

pub fn wait<H: crate::display::preparation::Access + CompatibilityRam + DisplayOwner + ?Sized>(
    update: bool,
    game: &mut H,
) -> u8 {
    if crate::display::preparation::Access::display_control_byte(
        &*game,
        crate::display::preparation::Field::NmiAcknowledgment,
    ) != 0
    {
        crate::frame_display_byte!(game, crate::display::preparation::Field::NmiAcknowledgment => 0);
        return 0;
    }
    if update {
        crate::frame_display_byte!(game, crate::display::preparation::Field::NmiPending => 1);
    }
    crate::frame_display_byte!(game, crate::display::preparation::Field::NmiAcknowledgment => 1);
    1
}

pub fn enable<
    H: Registers + crate::display::preparation::Access + CompatibilityRam + DisplayOwner + ?Sized,
>(
    enabled: bool,
    game: &mut H,
) {
    let mask = if enabled { 0x80 } else { 0 };
    let value = (crate::display::preparation::Access::display_control_byte(
        &*game,
        crate::display::preparation::Field::NmiEnable,
    ) & 0x7f)
        | mask;
    game.write_register(NMITIMEN as u16, value);
    // The hardware write may change RAM; publish the value captured before it.
    crate::frame_display_byte!(game, crate::display::preparation::Field::NmiEnable => value);
}

/// The scanline interrupt's controls and display through a host's projection
/// and registers: the owned display state's words where the Session owns the
/// presentation, RAM otherwise, as a lease or the compatibility game holds them.
pub struct ScanlineIrq<'a, H: ?Sized>(pub &'a mut H);

impl<H: crate::display::preparation::Access + CompatibilityRam + ?Sized> irq::Controls
    for ScanlineIrq<'_, H>
{
    fn read_control(&mut self, field: irq::Control) -> u16 {
        use crate::display::preparation::{Access, Field, Place, Word};
        match field {
            irq::Control::Current => Access::display_history_word(&*self.0, Word::CurrentInterrupt),
            irq::Control::Pending => Access::display_history_word(&*self.0, Word::PendingInterrupt),
            irq::Control::Enable => {
                Place::Controls(Field::NmiEnable, Field::HdmaEnable).read(&*self.0)
            }
            irq::Control::PreviousCre => self.0.view_word(crate::projection::Field::PreviousCre),
            irq::Control::CurrentCre => self.0.view_word(crate::projection::Field::CurrentCre),
            irq::Control::TransitionFlag => {
                self.0.view_word(crate::projection::Field::DoorTransition)
            }
        }
    }
    fn write_control(&mut self, field: irq::Store, value: u16) {
        use crate::display::preparation::{Field, Place, Word};
        let place = match field {
            irq::Store::Current => Place::History(Word::CurrentInterrupt),
            irq::Store::Pending => Place::History(Word::PendingInterrupt),
            irq::Store::Enable => Place::Controls(Field::NmiEnable, Field::HdmaEnable),
        };
        place.write(self.0, value);
    }
}
impl<H: crate::display::preparation::Access + CompatibilityRam + Registers + ?Sized> irq::Display
    for ScanlineIrq<'_, H>
{
    type Error = Infallible;

    fn read_mirror(&mut self, field: irq::Mirror) -> u8 {
        use crate::display::preparation::{Access, Field};
        Access::display_control_byte(
            &*self.0,
            match field {
                irq::Mirror::Background3Map => Field::Background3Map,
                irq::Mirror::ColorWindow => Field::PreviousColorSelection,
                irq::Mirror::ColorArithmetic => Field::PreviousColorMath,
                irq::Mirror::MainLayers => Field::PreviousMainLayers,
                irq::Mirror::InterruptEnable => Field::NmiEnable,
            },
        )
    }
    fn write_display(&mut self, field: irq::Output, value: u8) -> Result<(), Infallible> {
        let register = match field {
            irq::Output::Background3Map => BG3SC,
            irq::Output::ColorWindow => CGWSEL,
            irq::Output::ColorArithmetic => CGADSUB,
            irq::Output::MainLayers => TM,
            irq::Output::VerticalLow => VTIMEL,
            irq::Output::VerticalHigh => VTIMEL + 1,
            irq::Output::HorizontalLow => HTIMEL,
            irq::Output::HorizontalHigh => HTIMEL + 1,
            irq::Output::InterruptEnable => NMITIMEN,
        };
        self.0.write_register(register as u16, value);
        Ok(())
    }
}

/// Enable the scanline interrupt through `game`'s projection and registers.
pub fn enable_scanline_irq<
    H: crate::display::preparation::Access + CompatibilityRam + Registers + ?Sized,
>(
    immediate: bool,
    game: &mut H,
) {
    let Ok(()) = irq::enable(&mut ScanlineIrq(game), immediate);
}

/// Disable the scanline interrupt in `game`'s projected enable mirror.
pub fn disable_scanline_irq<H: crate::display::preparation::Access + CompatibilityRam + ?Sized>(
    game: &mut H,
) {
    irq::disable(&mut ScanlineIrq(game));
}

fn upload_affine_latches<H: Host + ?Sized>(game: &mut H) {
    use crate::display::preparation::{Access, Field};
    for (index, (low, high)) in [
        (Field::AffineWords0Low, Field::AffineWords0High),
        (Field::AffineWords1Low, Field::AffineWords1High),
        (Field::AffineWords2Low, Field::AffineWords2High),
        (Field::AffineWords3Low, Field::AffineWords3High),
        (Field::AffineWords4Low, Field::AffineWords4High),
        (Field::AffineWords5Low, Field::AffineWords5High),
    ]
    .into_iter()
    .enumerate()
    {
        let register = (M7A as u16) + u16::try_from(index).expect("six affine latches");
        game.write_register(register, Access::display_control_byte(&*game, low));
        // The low register callback precedes the high field observation.
        game.write_register(register, Access::display_control_byte(&*game, high));
    }
}

fn upload_registers<H: Host + ?Sized>(game: &mut H) {
    register_writes! { game;
        byte(NMITIMEN as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::NmiEnable)); byte(INIDISP as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Brightness));
        byte(OBSEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::ObjectLayout)); byte(BGMODE as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Mode));
        byte(MOSAIC as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Mosaic)); byte(BG1SC as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Screens0));
        byte(BG2SC as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Screens1)); byte(BG3SC as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Screens2));
        byte(BG4SC as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Screens3)); byte(BG12NBA as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Characters0));
        byte(BG34NBA as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Characters1)); byte(M7SEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::AffineSelection));
        byte(W12SEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowSelection0)); byte(W34SEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowSelection1));
        byte(WOBJSEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowSelection2)); byte(WH0 as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowPositions0));
        byte(WH1 as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowPositions1)); byte(WH2 as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowPositions2));
        byte(WH3 as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowPositions3)); byte(WBGLOG as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowLogic0));
        byte(WOBJLOG as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::WindowLogic1));
    }
    crate::frame_display_byte!(game, crate::display::preparation::Field::PreviousMainLayers => crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::MainLayers));
    register_writes! { game;
        byte(TM as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::MainLayers)); byte(TMW as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::MainWindow));
        byte(TS as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::SubLayers)); byte(TSW as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::SubWindow));
        byte(CGWSEL as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::ColorSelection)); byte(CGADSUB as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::ColorMath));
    }
    crate::frame_display_byte!(game, crate::display::preparation::Field::PreviousColorSelection => crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::NextColorSelection));
    crate::frame_display_byte!(game, crate::display::preparation::Field::PreviousColorMath => crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::NextColorMath));
    register_writes! { game;
        byte(COLDATA as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::FixedColor0)); byte(COLDATA as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::FixedColor1));
        byte(COLDATA as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::FixedColor2)); byte(SETINI as u16, crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::Geometry));
    }

    // Scroll and Mode 7 words latch low then high through the same register.
    for (index, word) in crate::display_scroll::WORDS.into_iter().enumerate() {
        let register = (BG1HOFS as u16) + u16::try_from(index).expect("eight scroll registers");
        game.write_register(register, game.scroll_byte(word, false));
        // Re-sample after the low latch: the register callback may change it.
        game.write_register(register, game.scroll_byte(word, true));
    }
    crate::frame_display_byte!(game, crate::display::preparation::Field::CommittedFakeMode => crate::display::preparation::Access::display_control_byte(&*game, crate::display::preparation::Field::FakeMode));
    if crate::display::preparation::Access::display_control_byte(
        &*game,
        crate::display::preparation::Field::Mode,
    ) & 7
        == 7
        || crate::display::preparation::Access::display_control_byte(
            &*game,
            crate::display::preparation::Field::FakeMode,
        ) & 7
            == 7
    {
        upload_affine_latches(game);
    }
}

fn upload_palette_and_oam<H: Host + ?Sized>(game: &mut H) {
    game.producing();
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
    game.produced(
        "palette and objects",
        &[
            Upload::Objects {
                address: 0,
                source: 0x0370,
                bytes: 0x0220,
            },
            Upload::Palette {
                index: 0,
                source: 0xc000,
                bytes: 0x0200,
            },
        ],
    );
    crate::probe::hit("palette.dma.uploaded");
}

struct Refresh<'a, H: ?Sized>(&'a mut H);

// The host's uploads either complete or keep their original panic/failure
// behavior; this adapter never produces a portable DisplayError.
impl<H: Host + ?Sized> crate::scene::commit::RefreshHost for Refresh<'_, H> {
    fn pending_nmi(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::NmiPending,
        )
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
        self.0.hdma_word(crate::hdma::Word::Object {
            index: slot,
            field: crate::hdma::ObjectWord::Channels,
        })
    }
    fn hdma_bank_slot(&self, slot: usize) -> u8 {
        self.0.hdma_word(crate::hdma::Word::Object {
            index: slot,
            field: crate::hdma::ObjectWord::BankSlot,
        }) as u8
    }
    fn hdma_table(&self, slot: usize) -> u16 {
        self.0.hdma_word(crate::hdma::Word::Object {
            index: slot,
            field: crate::hdma::ObjectWord::Table,
        })
    }
    fn bind_hdma(&mut self, bank_slot: u16, table: u16) -> Result<(), DisplayError> {
        self.0.write_register_word(A1T0L as u16 + bank_slot, table);
        Ok(())
    }
    fn mode(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::Mode,
        )
    }
    fn fake_mode(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::FakeMode,
        )
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
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::HdmaEnable,
        )
    }
    fn publish_hdma(&mut self, value: u8) -> Result<(), DisplayError> {
        self.0.write_register(HDMAEN as u16, value);
        Ok(())
    }
    fn set_pending_nmi(&mut self, value: u8) {
        crate::frame_display_byte!(self.0, crate::display::preparation::Field::NmiPending => value)
    }
    fn missed(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::NmiMissed,
        )
    }
    fn set_missed(&mut self, value: u8) {
        crate::frame_display_byte!(self.0, crate::display::preparation::Field::NmiMissed => value)
    }
    fn max_missed(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::NmiMaxMissed,
        )
    }
    fn set_max_missed(&mut self, value: u8) {
        crate::frame_display_byte!(self.0, crate::display::preparation::Field::NmiMaxMissed => value)
    }
    fn frame_byte(&self) -> u8 {
        crate::display::preparation::Access::display_control_byte(
            &*self.0,
            crate::display::preparation::Field::NmiFrameByte,
        )
    }
    fn set_frame_byte(&mut self, value: u8) {
        crate::frame_display_byte!(self.0, crate::display::preparation::Field::NmiFrameByte => value)
    }
    fn frame_word(&self) -> u16 {
        let game = &*self.0;
        crate::frame_display_word!(game.NmiFrameWordPair)
    }
    fn set_frame_word(&mut self, value: u16) {
        let game = &mut *self.0;
        crate::frame_display_word!(game.NmiFrameWordPair = value);
    }
    fn including_lag(&self) -> u16 {
        let game = &*self.0;
        crate::frame_display_word!(game.NmiIncludingLagPair)
    }
    fn set_including_lag(&mut self, value: u16) {
        let game = &mut *self.0;
        crate::frame_display_word!(game.NmiIncludingLagPair = value);
    }
}

/// Run the display refresh in the original order.
pub fn run<H: Host + ?Sized>(game: &mut H) {
    crate::scene::commit::refresh(&mut Refresh(game))
        .expect("native display refresh adapter cannot return an error");
}

/// The producer name of the graphics loads' and decompression's commands.
const GRAPHICS: &str = "graphics load";

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
                game.apply(
                    GRAPHICS,
                    Command::WramPort {
                        address: destination,
                    },
                );
                ([0, 0x80], source, size, None)
            }
        };
        if let Some((address, step)) = vram {
            game.apply(GRAPHICS, Command::VramPort { address, step });
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
                .apply(GRAPHICS, Command::VramAddress((position >> 1) as u16));
        }
        let high = position & 1 != 0;
        self.game.apply(GRAPHICS, Command::VramData { high, value });
    }
}
fn read_ppu_byte<H: Host + ?Sized>(addr: u16, game: &mut H) -> u8 {
    game.apply(GRAPHICS, Command::VramAddress(addr >> 1));
    // The prefetch read, then the word.
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

pub(crate) fn mult8x8(a: u8, b: u8) -> u16 {
    u16::from(a) * u16::from(b)
}

pub fn queue_scrolling_tiles<H: crate::display::queued_tilemaps::Scrolling + ?Sized>(
    k: u16,
    layout: ScrollingLayout,
    game: &mut H,
) {
    crate::display::queued_tilemaps::queue_scrolling(
        crate::display::queued_tilemaps::Selection {
            layout,
            layer_offset: k,
        },
        game,
    );
}

pub fn set_native_word<H: DisplayOwner + ?Sized>(offset: usize, value: u16, game: &mut H) {
    let bytes = value.to_ne_bytes();
    game.write_owned_byte(offset, bytes[0]);
    game.write_owned_byte(offset + 1, bytes[1]);
}

/// Publish a native queue record, evaluating each source after preceding stores.
pub fn queue_native_transfer<H: DisplayOwner + TilemapQueue + ?Sized>(
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
    game.set_tilemap_queue_tail(cursor);
    cursor
}

pub fn queue_fx_clear<H: CompatibilityRam + DisplayOwner + TilemapQueue + ?Sized>(game: &mut H) {
    word_transfers!(game, descending, 1920; native(0x4efe) <- value(6222));
    let tail = game.tilemap_queue_tail();
    let entry = 0xd0 + usize::from(game.tilemap_queue_tail());
    set_native_word(entry, 3840, game);
    set_native_word(entry + 2, 0x4000, game);
    // The bank store is deliberately a native word, overlapping the destination low byte.
    set_native_word(entry + 4, 126, game);
    set_native_word(entry + 5, ADDR_UNK_605880 as u16, game);
    game.set_tilemap_queue_tail(tail.wrapping_add(7));
}

fn write_readback_word<H: Host + ?Sized>(game: &mut H, offset: usize, value: u16) {
    let bytes = value.to_ne_bytes();
    game.store_readback_byte(offset, bytes[0]);
    game.store_readback_byte(offset + 1, bytes[1]);
}

/// Queue a VRAM readback of `size` bytes into WRAM at `destination`, from
/// the screen base in the register mirror `register` plus `vram_offset`.
pub fn enqueue_readback<H: Host + ?Sized>(
    register: crate::display::preparation::Field,
    vram_offset: u16,
    destination: u16,
    size: u16,
    retain_mode: bool,
    game: &mut H,
) {
    let tail = game.readback_queue_tail();
    let entry = usize::from(tail);
    let target =
        ((u16::from(game.display_control_byte(register)) & 0xfc) << 8).wrapping_add(vram_offset);
    write_readback_word(game, entry, target);
    if !retain_mode {
        write_readback_word(game, entry + 2, 129);
    }
    // These stores overlap. Stage five of X-ray retains the preceding mode byte.
    write_readback_word(game, entry + 3, 57);
    write_readback_word(game, entry + 4, destination);
    write_readback_word(game, entry + 6, 126);
    write_readback_word(game, entry + 7, size);
    game.set_readback_queue_tail(tail.wrapping_add(9));
}
