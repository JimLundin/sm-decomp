//! HDMA objects: allocation, channel setup, the per-frame program interpreter and the room effect reset.
use crate::cinematic::cinematic_native_record;
use crate::display::registers::{DMAP0, HDMAEN, MDMAEN};
use crate::display::sequence::Registers;
use crate::display_refresh::DisplayOwner;
use crate::imported::word as table_word;
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word};
use crate::session_access::SessionAccess;

/// The HDMA effects whose state the host's session view projects.
#[derive(Clone, Copy)]
pub enum HdmaEffect {
    /// Step the current object's program past its wait.
    AdvanceProgram,
    InitializePhantoonWave,
    InitializeEyeBeam,
    ResetWave,
    InitializeWavePair {
        base: usize,
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

/// The HDMA object table's owner: a lease's Session, or what `with_table`
/// opens for a host that holds none.
pub trait HdmaTable {
    fn with_hdma_table<T>(&mut self, action: impl FnOnce(&mut crate::hdma::HdmaObjects) -> T) -> T;
}

/// Run `action` on the HDMA table: the owned Session's under one lease when
/// one can open, otherwise a copy decoded from RAM and stored back if
/// `action` changed it.
pub fn with_table<H, T>(game: &mut H, action: impl FnOnce(&mut crate::hdma::HdmaObjects) -> T) -> T
where
    H: crate::ram::WorkRamAccess + crate::runtime_access::RuntimeAccess + ?Sized,
{
    use crate::session_access::SessionAccess;
    if game.runtime_admissible() {
        action(&mut game.runtime().session_mut().hdma)
    } else {
        let mut bridge = Bridged::load(game);
        let before = bridge.objects;
        let result = action(&mut bridge.objects);
        if bridge.objects != before {
            bridge.store();
        }
        result
    }
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
    let placed = game.with_hdma_table(|table| {
        let slot = slot.or_else(|| (0..6).find(|&slot| table.objects[slot].channels == 0))?;
        let object = &mut table.objects[slot];
        object.pre_instruction = 0x84b8;
        object.pre_instruction_bank = 0x88;
        object.instruction_list = read_word(1);
        object.instruction_timer = 1;
        object.timer = 0;
        object.a = 0;
        object.b = 0;
        object.c = 0;
        object.d = 0;
        object.channels = 4 << slot;
        object.bank_slot = (u16::from(bank) << 8) | channel(slot);
        Some(slot)
    });
    let Some(slot) = placed else {
        return u16::MAX;
    };
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
    game.with_hdma_table(|table| {
        table.enabled = (table.enabled & 0x7fff) | (u16::from(enabled) << 15);
    });
}

/// Free the object at byte slot `slot` by clearing its channel mask, as the
/// bosses do when their effect ends. A slot past the table (a failed spawn's
/// `u16::MAX`) keeps the original's store to the word it names.
pub fn free_hdma_object<H>(slot: u16, game: &mut H)
where
    H: crate::ram::WorkRamAccess
        + crate::ram::CompatibilityRam
        + crate::runtime_access::RuntimeAccess
        + ?Sized,
{
    let index = usize::from(slot >> 1);
    if index < crate::hdma::OBJECTS {
        with_table(game, |table| table.objects[index].channels = 0);
    } else {
        checked_word!(game.HDMA_OBJECT_CHANNELS_BITMASK[index] = 0);
    }
}

/// Clear the whole flag word, as the map screen does on entry.
pub fn clear_hdma_flags<H: HdmaTable + ?Sized>(game: &mut H) {
    game.with_hdma_table(|table| table.enabled = 0);
}

/// Free every object and clear the HDMA channel enable mirror.
pub fn clear_hdma_channels<H: DisplayOwner + HdmaTable + ?Sized>(game: &mut H) {
    game.write_owned_byte(0x85, 0);
    game.with_hdma_table(|table| {
        for object in table.objects.iter_mut().rev() {
            object.channels = 0;
        }
    });
}

pub fn clear_hdma<H: Registers + DisplayOwner + HdmaTable + ?Sized>(game: &mut H) {
    crate::display::sequence::register_writes! { game;
        byte(MDMAEN as u16, 0); byte(HDMAEN as u16, 0);
    }
    clear_hdma_channels(game);
}

pub fn initialize_room_effects<H>(game: &mut H)
where
    H: Registers + crate::ram::CompatibilityRam + DisplayOwner + SessionAccess + ?Sized,
{
    use crate::display::registers::{WMADDH, WMADDL, WMADDM};
    word! { game, 0; EARTHQUAKE_SFX_INDEX, EARTHQUAKE_SFX_TIMER }
    if matches!(
        word!(game.ROOM_PTR),
        0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
    ) {
        word!(game.EARTHQUAKE_SFX_TIMER = 0xffff);
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
    word!(game.HUD_BG3_XPOS = 0);
    word!(game.HUD_BG3_YPOS = 0);
    word!(game.BG3_XPOS = 0);
    word!(game.BG3_YPOS = 0);
    word!(game.IRQ_ENABLE_MODE7 = 0);
    word!(game.CAMERA_DISTANCE_INDEX = 0);
    crate::room_view::set_effect(game, crate::room_view::EffectWord::StatueAnimation, 0);
    crate::room_view::set_effect(game, crate::room_view::EffectWord::StatueFinished, 0);
    crate::room_view::set_effect(game, crate::room_view::EffectWord::EarthquakeTimer, 0);
    word!(game.PHANTOM_RELATED_LAYER_FLAG = 0);
    word!(game.POWER_BOMB_EXPLOSION_STATUS = 0);
    word!(game.POWER_BOMB_FLAG = 0);
    word!(game.POWER_BOMB_EXPLOSION_RADIUS = 0);
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
    word!(game.FX_LAYER_BLENDING_CONFIG_A = 2);
    word! { game, 0;
        REG_BG3HOFS, REG_BG3VOFS,
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
    game.write_owned_byte(0x85, 0);
    set_fixed_color([32, 64, 0x80], game);
    // Mosaic, main layers, colour selection and math, BG character and screen bases.
    for (at, value) in [
        (0x57, 0),
        (0x69, 19),
        (0x6f, 0),
        (0x72, 0),
        (0x5d, 0),
        (0x5e, 4),
        (0x59, 73),
        (0x5a, 90),
        (0x5b, 90),
    ] {
        game.write_owned_byte(at, value);
    }
}

pub fn update_hdma<H>(game: &mut H)
where
    H: crate::hdma_dispatch::DispatchHost + crate::hdma_program::LeaseHdma + ?Sized,
{
    game.handle_music_queue();
    if checked_word!(game.TIME_IS_FROZEN_FLAG) == 0
        && checked_word!(game.POWER_BOMB_EXPLOSION_STATUS) & 0x4000 != 0
    {
        checked_word!(game.POWER_BOMB_EXPLOSION_STATUS = 0x8000);
        spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 0x28, 0x8ace), game);
        spawn_hdma(0x88, &HdmaObjectDefinition::new(0x40, 0x29, 0x8b80), game);
    }
    checked_word!(game.FX_LAYER_BLENDING_CONFIG_C = checked_word!(game.FX_LAYER_BLENDING_CONFIG_A));
    if checked_word!(game.HDMA_OBJECTS_ENABLE_FLAG) & 0x8000 != 0 {
        crate::hdma_program::run_objects(game);
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
        word!(game.HDMA_TABLE_1[slot] = value);
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
    if crate::flow::current(game).cinematic_function == 0x9b68 {
        crate::hdma_effects::apply_leased(HdmaEffect::AdvanceProgram, 0, game);
    }
}

pub fn advance_intro_crossfade<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if checked_word!(game.EPROJ_X_POS[0]) == 1 {
        crate::hdma_effects::apply_leased(HdmaEffect::AdvanceProgram, 0, game);
    }
}

/// The table's RAM words, decoded.
pub fn read_table<H: crate::ram::WorkRamAccess + ?Sized>(game: &H) -> crate::hdma::HdmaObjects {
    let ram = game.ram();
    let mut objects = crate::hdma::HdmaObjects::default();
    crate::hdma_program::table_words(&mut objects, |at, word| {
        *word = u16::from_le_bytes([ram.read_byte(at), ram.read_byte(at + 1)]);
    });
    objects
}

/// The table decoded from RAM, for hosts that cannot open a lease now.
pub(crate) struct Bridged<'a, H: ?Sized> {
    game: &'a mut H,
    objects: crate::hdma::HdmaObjects,
}

impl<'a, H: crate::ram::WorkRamAccess + ?Sized> Bridged<'a, H> {
    pub(crate) fn load(game: &'a mut H) -> Self {
        let mut bridge = Self {
            game,
            objects: crate::hdma::HdmaObjects::default(),
        };
        bridge.reload();
        bridge
    }
    fn reload(&mut self) {
        self.objects = read_table(self.game);
    }
    pub(crate) fn store(&mut self) {
        let ram = self.game.ram();
        crate::hdma_program::table_words(&mut self.objects, |at, word| {
            let [low, high] = word.to_le_bytes();
            ram.write_byte(at, low);
            ram.write_byte(at + 1, high);
        });
    }
}

impl<H: crate::hdma_dispatch::DispatchHost + ?Sized> crate::hdma_program::Interpreter
    for Bridged<'_, H>
{
    type Game = H;
    fn objects(&mut self) -> &mut crate::hdma::HdmaObjects {
        &mut self.objects
    }
    fn byte(&self, at: usize) -> u8 {
        self.game.ram().read_byte(at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.game.ram().write_byte(at, value);
    }
    fn image(&self) -> crate::enemy_script::ScriptImage {
        self.game.native_image().into()
    }
    fn foreign<T>(&mut self, action: impl FnOnce(&mut H) -> T) -> T {
        self.store();
        let result = action(self.game);
        self.reload();
        result
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
    game.ram().write_byte(0x6f, 0);
    refresh_title_gradient(game);
}
