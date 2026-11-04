//! HDMA objects: allocation, channel setup, the per-frame program interpreter and the room effect reset.
use crate::cinematic::cinematic_native_record;
use crate::cinematic_state::Word as CinematicWord;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::display::registers::DMAP0;
use crate::display::video::{Axis, Background, Command, ScrollOffset, Video};
use crate::display_refresh::DisplayOwner;
use crate::imported::word as table_word;
use crate::projection::Field;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;
use crate::session::HookSlot;
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
pub fn spawn_bg3_scroll<H: Allocator + ?Sized>(game: &mut H) {
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
pub fn spawn_hdma<H: crate::menus::host::CommonRuntime + ?Sized>(
    bank: u8,
    args: &HdmaObjectDefinition,
    game: &mut H,
) -> u16 {
    crate::menus::host::CommonRuntime::spawn_hdma(game, bank, args)
}

/// The HDMA object allocator's host: the object table, and the DMA channel
/// registers a new object's definition sets.
pub trait Allocator: HdmaTable {
    /// Write the DMA control and target registers at `register`, control first.
    fn write_channel_control(&mut self, register: u16, control: u16);
}
impl<T: crate::menus::host::MenuRuntime + HdmaTable + ?Sized> Allocator for T {
    fn write_channel_control(&mut self, register: u16, control: u16) {
        let [low, high] = control.to_le_bytes();
        self.write_dma_register(register, low);
        self.write_dma_register(register + 1, high);
    }
}

/// The host's HDMA spawn on the owned table.
pub fn spawn_hdma_owned<H: Allocator + ?Sized>(
    bank: u8,
    args: &HdmaObjectDefinition,
    game: &mut H,
) -> u16 {
    let words = hdma_definition(args);
    spawn_hdma_words(bank, |index| words[index], game)
}

pub fn spawn_hdma_words<H: Allocator + ?Sized>(
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
fn place_hdma<H: Allocator + ?Sized>(
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
    game.write_channel_control(DMAP0 as u16 + channel(slot), read_word(0));
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
    H: crate::runtime_access::SessionRam + HdmaTable + ?Sized,
{
    let index = usize::from(slot >> 1);
    if index < crate::hdma::OBJECTS {
        crate::hdma_word!(game, index, Channels => 0);
    } else {
        // LEGACY(original-overflow): the word the slot names past the channel table.
        crate::legacy_overflow::ram_write_word_le(
            game,
            crate::ram::word_arrays::HDMA_OBJECT_CHANNELS_BITMASK + 2 * index,
            0,
        );
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

/// The producer name of the HDMA resets' commands.
const HDMA_RESET: &str = "hdma reset";

pub fn clear_hdma<
    H: Video + crate::display::preparation::Access + DisplayOwner + HdmaTable + ?Sized,
>(
    game: &mut H,
) {
    game.apply(HDMA_RESET, Command::StopDma);
    clear_hdma_channels(game);
}

pub fn initialize_room_effects<H>(game: &mut H)
where
    H: Video
        + crate::display::preparation::Access
        + crate::ram::CompatibilityRam
        + DisplayOwner
        + SessionAccess
        + crate::fx_domain::Controls
        + ?Sized,
{
    {
        let mut fx = game.fx_controls();
        fx.earthquake_sound_index = 0;
        fx.earthquake_sound_timer = 0;
        game.publish_fx_controls(
            fx,
            &[
                crate::fx_state::Field::EarthquakeSoundIndex,
                crate::fx_state::Field::EarthquakeSoundTimer,
            ],
        );
    }
    if matches!(
        game.session().room_key(),
        0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
    ) {
        {
            let fx_value = 0xffff;
            let mut fx = game.fx_controls();
            fx.earthquake_sound_timer = fx_value;
            game.publish_fx_controls(fx, &[crate::fx_state::Field::EarthquakeSoundTimer]);
        };
    }
    game.session_mut().minimap.disabled = 0;
    game.words_stored();
    game.apply(
        HDMA_RESET,
        Command::ClearHdmaChannels {
            first: 2,
            target: ScrollOffset {
                background: Background::Bg4,
                axis: Axis::Horizontal,
            },
        },
    );
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
    game.session_mut().room.mode7_irq = 0;
    game.words_stored();
    game.session_mut().room.camera.distance_index = 0;
    game.words_stored();
    game.session_mut().mechanisms.statue_animation = 0;
    game.session_mut().mechanisms.statue_finished = 0;
    crate::gameplay_context::set_effect(game, crate::room_view::EffectWord::EarthquakeTimer, 0);
    game.session_mut().room.fx_layer_flags = 0;
    game.words_stored();
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
    game.session_mut().room.entry.irq_policy = 0;
    // Both pause hooks become $88:83E1: the bank and address high byte from
    // $8800 a byte up, then the address.
    let room = &mut game.session_mut().room;
    for hook in [HookSlot::Pause, HookSlot::Unpause] {
        room.set_hook_bank(hook, 0x88);
        room.set_hook_address(hook, 0x83e1u16.to_le_bytes());
    }
    game.words_stored();
    game.apply(HDMA_RESET, Command::WramPort { address: 0x01_fff0 });
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
pub fn update_hdma<H: UpdateHost + ?Sized>(handler_carry: bool, game: &mut H) {
    game.advance_music_queue();
    if crate::sequence_word!(game.TIME_IS_FROZEN_FLAG) == 0
        && crate::explosion_window::explosion(game)
            .bomb
            .explosion_status
            & 0x4000
            != 0
    {
        crate::explosion_window::with_explosion(game, |_, e| {
            e.bomb.explosion_status = 0x8000;
        });
        spawn_hdma_owned(0x88, &HdmaObjectDefinition::new(0x40, 0x28, 0x8ace), game);
        spawn_hdma_owned(0x88, &HdmaObjectDefinition::new(0x40, 0x29, 0x8b80), game);
    }
    {
        let fx_value = game.room_blending().primary;
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
    if game.hdma_word(crate::hdma::Word::Enabled) & 0x8000 != 0 {
        game.run_hdma_programs(handler_carry);
        crate::layer_blending::update(game);
    }
}

/// What the HDMA object handler reaches beyond its object table: the
/// sequence words, the allocator, the FX controls and display mirrors, and
/// three rules: the music queue's step, the room key the layer blending reads,
/// and the objects' programs. Every host that can lease its Session to the
/// interpreter is one, running the programs on a fresh lease; a Session lease
/// is one and runs them on itself.
pub trait UpdateHost:
    Allocator
    + crate::sequence_state::Access
    + crate::fx_domain::Controls
    + crate::display::preparation::Access
{
    /// Advance the music queue, as the handler does first.
    fn advance_music_queue(&mut self);
    /// The current room's key.
    fn blending_room_key(&mut self) -> u16;
    /// Run every active object's program once.
    fn run_hdma_programs(&mut self, handler_carry: bool);
}
impl<T> UpdateHost for T
where
    T: crate::hdma_dispatch::DispatchHost
        + crate::hdma_program::LeaseHdma
        + crate::menus::host::MenuRuntime
        + ?Sized,
{
    fn advance_music_queue(&mut self) {
        self.handle_music_queue();
    }
    fn blending_room_key(&mut self) -> u16 {
        use crate::session_access::SessionAccess as _;
        self.runtime().session().room_key()
    }
    fn run_hdma_programs(&mut self, handler_carry: bool) {
        crate::hdma_program::run_objects(handler_carry, self);
    }
}

/// What the title gradient's refresh reaches: the Mode 7 zoom, the colour
/// selection, HDMA table 1, the gradient table and its native records. Every
/// menu host is one; a Session lease is one too.
pub trait GradientHost:
    crate::projection::Views
    + crate::display::preparation::Access
    + crate::fx_domain::Sources
    + crate::cartridge::NativeDiagnostics
    + crate::cartridge::LoadedImage
{
    fn gradient_tables(&self) -> &crate::visual_tables::ImportedTables;
}
impl<T: crate::menus::host::Host + ?Sized> GradientHost for T {
    fn gradient_tables(&self) -> &crate::visual_tables::ImportedTables {
        self.visual_tables()
    }
}

pub(crate) fn refresh_title_gradient<H: GradientHost + ?Sized>(game: &mut H) {
    let mut cursor = table_word(
        &game.gradient_tables().title_gradients,
        game,
        GRADIENT_ADDRESS,
        ((game.view_word(Field::Cinematic(CinematicWord::Mode7Zoom)) & 0xf0) >> 4) as isize,
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

/// Refresh the title gradient's table from the zoom, then step its program.
pub fn advance_title_gradient_backdrop<H>(game: &mut H)
where
    H: GradientHost + crate::sequence_state::Access + crate::hdma_effects::HdmaEffects + ?Sized,
{
    refresh_title_gradient(game);
    advance_title_gradient(game);
}

/// Step the title gradient's program once the title's cinematic reaches it.
pub fn advance_title_gradient<H>(game: &mut H)
where
    H: crate::sequence_state::Access + crate::hdma_effects::HdmaEffects + ?Sized,
{
    if crate::sequence_state::Access::sequence_flow(&*game).cinematic_function == 0x9b68 {
        game.advance_hdma_program();
    }
}

/// Step the intro crossfade's program once the cinematic asks for it.
pub fn advance_intro_crossfade<H>(game: &mut H)
where
    H: crate::projection::Views + crate::hdma_effects::HdmaEffects + ?Sized,
{
    if game.view_word(Field::Cinematic(CinematicWord::Var13)) == 1 {
        game.advance_hdma_program();
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

/// Set one parameter byte of the HDMA channel `bank_slot` selects, as an
/// HDMA program command does.
pub(crate) fn write_channel_byte<H: Video + ?Sized>(
    bank_slot: u8,
    register: sm_data::hdma_programs::ChannelRegister,
    value: u8,
    game: &mut H,
) {
    game.apply(
        "hdma program",
        Command::HdmaChannelByte {
            bank_slot,
            register,
            value,
        },
    );
}

pub(crate) fn restart_title_gradient<H: GradientHost + ?Sized>(game: &mut H) {
    game.store_display_control_byte(DisplayField::ColorSelection, 0, DisplayStore::Named);
    refresh_title_gradient(game);
}
