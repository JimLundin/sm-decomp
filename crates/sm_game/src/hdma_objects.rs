//! HDMA objects: allocation, channel setup, the per-frame program interpreter and the room effect reset.
use crate::cinematic::cinematic_native_record;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::display::registers::{DMAP0, HDMAEN, MDMAEN};
use crate::display::sequence::Registers;
use crate::display_refresh::DisplayOwner;
use crate::imported::word as table_word;
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word};
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;
use crate::session_access::SessionAccess;

/// The HDMA effects whose state the host's session view projects.
/// Where a wave pair's two-entry indirect HDMA table is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveTable {
    /// Phantoon's wide wave, at $7E:9000 in the bosses' shared block.
    Phantoon,
    /// The intro's wavy Samus, at $7E:0D82: the saved button configuration
    /// words, which the original reuses for the table.
    Intro,
}

#[derive(Clone, Copy)]
pub enum HdmaEffect {
    /// Step the current object's program past its wait.
    AdvanceProgram,
    InitializePhantoonWave,
    InitializeEyeBeam,
    ResetWave,
    InitializeWavePair {
        table: WaveTable,
        primary: u16,
    },
    SetPhantoonMotion {
        direction: u16,
        speed: u16,
    },
    ScrollWeather(crate::visual_state::Weather),
    BeginHaze {
        color: u16,
    },
    AdvanceHaze(crate::visual_state::HazePhase),
    SelectDraygonLayer,
    SelectPhantoonLayer,
    AdvanceWave(crate::visual_state::HdmaWave),
    AdvanceEyeBeam(crate::visual_state::EyeBeamPhase),
    AdvanceXrayBackground,
    AdvanceXrayBeam(crate::player_transitions::xray::BeamPhase),
    InitializeSuitPickup,
    AdvanceSuitPickup {
        gravity: bool,
    },
    /// Mother Brain's rainbow beam, which runs on the compatibility host.
    InitializeRainbowBeam,
    AdvanceRainbowBeam,
}
use crate::cinematic::set_fixed_color;
use crate::menus::prelude::HdmaObjectDefinition;
use sm_data::title_screen::GRADIENT_ADDRESS;

/// Spawn the BG3 scroll object in the last slot.
pub fn spawn_bg3_scroll<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let words = hdma_definition(&HdmaObjectDefinition::new(0x43, 0x11, 0xd8d0));
    place_hdma(Some(5), 0x88, |index| words[index], game);
}

fn hdma_definition(args: &HdmaObjectDefinition) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

/// Spawn an HDMA object through the host; returns its slot.
pub fn spawn_hdma<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    args: &HdmaObjectDefinition,
    game: &mut H,
) -> u16 {
    crate::menus::host::MenuRuntime::spawn_hdma(game, bank, args)
}

/// The host's HDMA spawn on the owned table.
pub fn spawn_hdma_owned<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    args: &HdmaObjectDefinition,
    game: &mut H,
) -> u16 {
    let words = hdma_definition(args);
    spawn_hdma_words(bank, |index| words[index], game)
}

pub fn spawn_hdma_words<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
) -> u16 {
    place_hdma(None, bank, read_word, game)
}

/// Current common-table access through the actual HDMA source owner.
pub trait HdmaTable {
    fn hdma_word(&self, word: crate::hdma::Word) -> u16;
    fn store_hdma_word(
        &mut self,
        word: crate::hdma::Word,
        value: u16,
        order: crate::scene::state::WordOrder,
    );
    fn store_hdma_byte(&mut self, word: crate::hdma::Word, high: bool, value: u8);
}

/// Initialize object `slot`, or the first free one, from bank `bank`'s
/// definition, then its channel's DMA control and target registers once
/// the table is published. Returns the byte slot, or `u16::MAX` when every
/// object is busy.
fn place_hdma<H: crate::menus::host::Host + ?Sized>(
    slot: Option<usize>,
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
) -> u16 {
    let slot = slot.or_else(|| {
        (0..crate::hdma::OBJECTS).find(|&slot| crate::hdma_word!(&*game, slot, Channels) == 0)
    });
    let Some(slot) = slot else {
        return u16::MAX;
    };
    crate::hdma_word!(game, slot, PreInstruction => 0x84b8);
    crate::hdma_word!(game, slot, PreInstructionBank => 0x88);
    crate::hdma_word!(game, slot, InstructionList => read_word(1));
    crate::hdma_word!(game, slot, InstructionTimer => 1);
    crate::hdma_word!(game, slot, Timer => 0);
    crate::hdma_word!(game, slot, A => 0);
    crate::hdma_word!(game, slot, B => 0);
    crate::hdma_word!(game, slot, C => 0);
    crate::hdma_word!(game, slot, D => 0);
    crate::hdma_word!(game, slot, Channels => 4 << slot);
    crate::hdma_word!(game, slot, BankSlot => (u16::from(bank) << 8) | channel(slot));
    let control = read_word(0);
    let register = DMAP0 as u16 + channel(slot);
    game.write_dma_register(register, control as u8);
    game.write_dma_register(register + 1, (control >> 8) as u8);
    2 * slot as u16
}

/// Object `slot`'s DMA channel register offset.
fn channel(slot: usize) -> u16 {
    0x20 + 0x10 * slot as u16
}

/// Set or clear the flag that runs the objects each frame (bit 15).
pub fn set_hdma_enabled<H: HdmaTable + ?Sized>(enabled: bool, game: &mut H) {
    let value = (game.hdma_word(crate::hdma::Word::Enabled) & 0x7fff) | (u16::from(enabled) << 15);
    game.store_hdma_word(crate::hdma::Word::Enabled, value, WordOrder::Little);
}

/// Free the object at byte slot `slot` by clearing its channel mask, as the
/// bosses do when their effect ends. A slot past the table (a failed spawn's
/// `u16::MAX`) keeps the original's store to the word it names.
pub fn free_hdma_object<H>(slot: u16, game: &mut H)
where
    H: crate::ram::WorkRamAccess
        + crate::ram::CompatibilityRam
        + crate::runtime_access::RuntimeAccess
        + HdmaTable
        + ?Sized,
{
    let index = usize::from(slot >> 1);
    if index < crate::hdma::OBJECTS {
        crate::hdma_word!(game, index, Channels => 0);
    } else {
        checked_word!(game.HDMA_OBJECT_CHANNELS_BITMASK[index] = 0);
    }
}

/// Clear the whole flag word, as the map screen does on entry.
pub fn clear_hdma_flags<H: HdmaTable + ?Sized>(game: &mut H) {
    game.store_hdma_word(crate::hdma::Word::Enabled, 0, WordOrder::Little);
}

/// Free every object and clear the HDMA channel enable mirror.
pub fn clear_hdma_channels<
    H: crate::display::preparation::Access + DisplayOwner + HdmaTable + ?Sized,
>(
    game: &mut H,
) {
    crate::frame_display_byte!(game, crate::display::preparation::Field::HdmaEnable => 0);
    for index in (0..crate::hdma::OBJECTS).rev() {
        crate::hdma_word!(game, index, Channels => 0);
    }
}

pub fn clear_hdma<
    H: Registers + crate::display::preparation::Access + DisplayOwner + HdmaTable + ?Sized,
>(
    game: &mut H,
) {
    crate::display::sequence::register_writes! { game;
        byte(MDMAEN as u16, 0); byte(HDMAEN as u16, 0);
    }
    clear_hdma_channels(game);
}

pub fn initialize_room_effects<H>(game: &mut H)
where
    H: Registers
        + crate::display::preparation::Access
        + crate::ram::CompatibilityRam
        + DisplayOwner
        + SessionAccess
        + crate::fx_domain::Controls
        + ?Sized,
{
    use crate::display::registers::{WMADDH, WMADDL, WMADDM};
    word! { game, 0; EARTHQUAKE_SFX_INDEX, EARTHQUAKE_SFX_TIMER }
    if matches!(
        word!(game.ROOM_PTR),
        0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
    ) {
        {
            let fx_value = 0xffff;
            let mut fx = game.fx_controls();
            fx.earthquake_sound_timer = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EarthquakeSoundTimer]);
        };
    }
    word!(game.DEBUG_DISABLE_MINIMAP = 0);
    for channel in (0x20..0x80u16).step_by(0x10) {
        for register in [0, 1, 2, 3, 4, 5, 6, 8, 9] {
            let value = if register == 1 { 0x13 } else { 0 };
            game.write_register(0x4300 + channel + register, value);
        }
    }
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.surface.fraction, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.surface.whole, -1i32 as u16);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.lava_acid.fraction, 0);
    crate::liquid::LiquidWords::set(
        game,
        |words| &mut words.liquid.lava_acid.whole,
        -1i32 as u16,
    );
    {
        let fx_value = 0;
        let mut fx = game.fx_controls();
        fx.hud_scroll[0] = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::HudHorizontal]);
    };
    {
        let fx_value = 0;
        let mut fx = game.fx_controls();
        fx.hud_scroll[1] = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::HudVertical]);
    };
    {
        let fx_value = 0;
        let mut fx = game.fx_controls();
        fx.effect_scroll[0] = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::EffectHorizontal]);
    };
    {
        let fx_value = 0;
        let mut fx = game.fx_controls();
        fx.effect_scroll[1] = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::EffectVertical]);
    };
    word!(game.IRQ_ENABLE_MODE7 = 0);
    word!(game.CAMERA_DISTANCE_INDEX = 0);
    game.session_mut().mechanisms.statue_animation = 0;
    game.session_mut().mechanisms.statue_finished = 0;
    crate::gameplay_context::set_effect(game, crate::room_view::EffectWord::EarthquakeTimer, 0);
    word!(game.PHANTOM_RELATED_LAYER_FLAG = 0);
    let session = game.session_mut();
    session.projectiles.power_bomb.explosion_status = 0;
    session.projectiles.power_bomb.flag = 0;
    session.room.power_bomb_radius = 0;
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.tilemap, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.kind, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.base_fraction, 0x8000);
    crate::liquid::LiquidWords::set(game, |words| &mut words.height, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.target, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.velocity, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.options, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.delay, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.tide_phase, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.offset.fraction, 0);
    crate::liquid::LiquidWords::set(game, |words| &mut words.liquid.offset.whole, 0);
    {
        let fx_value = 2;
        let mut room_blending = game.room_blending();
        room_blending.primary = fx_value;
        game.publish_room_blending(room_blending, true);
    };
    {
        crate::display_scroll::write(game, crate::display_scroll::Word::Bg3H, 0);
        crate::display_scroll::write(game, crate::display_scroll::Word::Bg3V, 0);
    }
    game.session_mut().room.camera.layer2 = [0, 0];
    word!(game.ROOM_LOADING_IRQ_HANDLER = 0);
    crate::word_transfers! { game, ascending, 1;
        native(0x602) <- value(0x8800), native(0x605) <- value(0x8800),
        native(0x601) <- value(0x83e1), native(0x604) <- value(0x83e1),
    }
    crate::display::sequence::register_writes! { game;
        byte(WMADDL as u16, 0xf0); byte(WMADDM as u16, 0xff);
        byte(WMADDH as u16, 1);
    }
    crate::frame_display_byte!(game, crate::display::preparation::Field::HdmaEnable => 0);
    set_fixed_color([32, 64, 0x80], game);
    // Mosaic, main layers, colour selection and math, BG character and screen bases.
    for (field, value) in [
        (crate::display::preparation::Field::Mosaic, 0),
        (crate::display::preparation::Field::MainLayers, 19),
        (crate::display::preparation::Field::ColorSelection, 0),
        (crate::display::preparation::Field::ColorMath, 0),
        (crate::display::preparation::Field::Characters0, 0),
        (crate::display::preparation::Field::Characters1, 4),
        (crate::display::preparation::Field::Screens1, 73),
        (crate::display::preparation::Field::Screens2, 90),
        (crate::display::preparation::Field::Background3Map, 90),
    ] {
        crate::display::preparation::Access::store_display_control_byte(
            &mut *game,
            field,
            value,
            crate::display::preparation::Store::Owned,
        );
    }
}

/// Run the HDMA object handler ($88:84B9) with `handler_carry`, the carry it
/// is entered with: the main game loop's, set from power-on, or clear when a
/// message box restores the display.
pub fn update_hdma<H>(handler_carry: bool, game: &mut H)
where
    H: crate::hdma_dispatch::DispatchHost + crate::hdma_program::LeaseHdma + ?Sized,
{
    game.handle_music_queue();
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) == 0
        && crate::session_part::get(game, |session| {
            &session.projectiles.power_bomb.explosion_status
        }) & 0x4000
            != 0
    {
        crate::session_part::update(
            game,
            |session| &mut session.projectiles.power_bomb.explosion_status,
            |status| *status = 0x8000,
        );
        spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 0x28, 0x8ace), game);
        spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 0x29, 0x8b80), game);
    }
    {
        let fx_value = game.room_blending().primary;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if game.hdma_word(crate::hdma::Word::Enabled) & 0x8000 != 0 {
        crate::hdma_program::run_objects(handler_carry, game);
        crate::layer_blending::update(game);
    }
}

pub(crate) fn refresh_title_gradient<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let mut cursor = table_word(
        &game.visual_tables().title_gradients,
        game,
        GRADIENT_ADDRESS,
        ((word!(game.MODE7_ZOOM) & 0xf0) >> 4) as isize,
    );
    for slot in 0usize.. {
        let value = cinematic_native_record::<0x8c>(cursor, game).word(0);
        game.store_source_word(LiveOwner::HdmaTable1, 2 * slot, WordOrder::Little, value);
        if value == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
    }
}

pub fn advance_title_gradient<H: crate::menus::host::Host + ?Sized>(backdrop: bool, game: &mut H) {
    if backdrop {
        refresh_title_gradient(game);
    }
    if crate::sequence_state::Access::sequence_flow(&*game).cinematic_function == 0x9b68 {
        crate::hdma_effects::apply_leased(HdmaEffect::AdvanceProgram, 0, game);
    }
}

pub fn advance_intro_crossfade<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if checked_word!(game.EPROJ_X_POS[0]) == 1 {
        crate::hdma_effects::apply_leased(HdmaEffect::AdvanceProgram, 0, game);
    }
}

/// Readonly capture of the actual current common-table source values.
pub fn read_table<H: HdmaTable + ?Sized>(game: &H) -> crate::hdma::HdmaObjects {
    let mut objects = crate::hdma::HdmaObjects::default();
    crate::hdma_program::table_words(&mut objects, |at, word| {
        let key = match at {
            0x18b0 => crate::hdma::Word::Enabled,
            0x18b2 => crate::hdma::Word::Selected,
            _ => {
                let offset = at - 0x18b4;
                crate::hdma::Word::Object {
                    index: (offset % 12) / 2,
                    field: crate::hdma::ObjectWord::ALL[offset / 12],
                }
            }
        };
        *word = game.hdma_word(key);
    });
    objects
}

/// Nonleased interpreter access; callbacks and words share the same host owner.
pub(crate) struct Bridged<'a, H: ?Sized> {
    game: &'a mut H,
}

impl<'a, H: ?Sized> Bridged<'a, H> {
    pub(crate) fn load(game: &'a mut H) -> Self {
        Self { game }
    }
}

impl<H: HdmaTable + ?Sized> HdmaTable for Bridged<'_, H> {
    fn hdma_word(&self, word: crate::hdma::Word) -> u16 {
        self.game.hdma_word(word)
    }
    fn store_hdma_word(&mut self, word: crate::hdma::Word, value: u16, order: WordOrder) {
        self.game.store_hdma_word(word, value, order);
    }
    fn store_hdma_byte(&mut self, word: crate::hdma::Word, high: bool, value: u8) {
        self.game.store_hdma_byte(word, high, value);
    }
}

impl<H: crate::hdma_dispatch::DispatchHost + ?Sized> crate::hdma_program::Interpreter
    for Bridged<'_, H>
{
    type Game = H;
    fn byte(&self, at: usize) -> u8 {
        self.game.ram().read_byte(at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.game.ram().write_byte(at, value);
    }
    fn image(&self) -> sm_data::cartridge::RomImage {
        self.game.native_image()
    }
    fn foreign<T>(&mut self, action: impl FnOnce(&mut H) -> T) -> T {
        action(self.game)
    }
}

pub(crate) fn write_channel_register<H: crate::menus::host::Host + ?Sized>(
    register: u16,
    value: u8,
    game: &mut H,
) {
    game.write_register(register, value);
}

pub(crate) fn restart_title_gradient<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.store_display_control_byte(DisplayField::ColorSelection, 0, DisplayStore::Named);
    refresh_title_gradient(game);
}
