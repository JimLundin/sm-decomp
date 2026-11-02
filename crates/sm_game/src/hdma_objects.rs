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

pub fn initialize_room_effects<H: crate::owned_hdma_update::lifecycle::ResetHost>(game: &mut H) {
    crate::owned_hdma_update::lifecycle::ResetHost::room_initialize_special_effects(game);
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
        && crate::cinematic_access::load::<crate::explosion_window::Explosion, _>(game)
            .bomb
            .explosion_status
            & 0x4000
            != 0
    {
        crate::cinematic_access::with(game, |_, e: &mut crate::explosion_window::Explosion| {
            e.bomb.explosion_status = 0x8000;
        });
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
    objects: Option<crate::hdma::HdmaObjects>,
}

impl<'a, H: crate::hdma_dispatch::DispatchHost + ?Sized> Bridged<'a, H> {
    pub(crate) fn load(game: &'a mut H) -> Self {
        let objects = game
            .hdma_raw_table_interval()
            .then(|| capture_raw_table(game));
        Self { game, objects }
    }
}

impl<H: crate::hdma_dispatch::DispatchHost + ?Sized> HdmaTable for Bridged<'_, H> {
    fn hdma_word(&self, word: crate::hdma::Word) -> u16 {
        self.objects.as_ref().map_or_else(
            || self.game.hdma_word(word),
            |objects| *table_field(objects, word),
        )
    }
    fn store_hdma_word(&mut self, word: crate::hdma::Word, value: u16, order: WordOrder) {
        // The original actual compatibility publication preserves poisoning,
        // byte order, idempotent epoch changes and source-publication rank.
        self.game.store_hdma_word(word, value, order);
        if let Some(objects) = &mut self.objects {
            *table_field_mut(objects, word) = u16::from_le_bytes(order.encode(value));
        }
    }
    fn store_hdma_byte(&mut self, word: crate::hdma::Word, high: bool, value: u8) {
        self.game.store_hdma_byte(word, high, value);
        if let Some(objects) = &mut self.objects {
            let target = table_field_mut(objects, word);
            let mut bytes = target.to_le_bytes();
            bytes[usize::from(high)] = value;
            *target = u16::from_le_bytes(bytes);
        }
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
        // No table value survives while the actual Game callback owns access.
        self.objects.take();
        let result = action(self.game);
        self.objects = self
            .game
            .hdma_raw_table_interval()
            .then(|| capture_raw_table(self.game));
        result
    }
}

fn capture_raw_table<H: crate::hdma_dispatch::DispatchHost + ?Sized>(
    host: &H,
) -> crate::hdma::HdmaObjects {
    let mut objects = crate::hdma::HdmaObjects::default();
    crate::hdma_program::table_words(&mut objects, |at, word| {
        *word = u16::from_le_bytes([host.ram().read_byte(at), host.ram().read_byte(at + 1)])
    });
    objects
}
fn table_field(objects: &crate::hdma::HdmaObjects, word: crate::hdma::Word) -> &u16 {
    use crate::hdma::{ObjectWord, Word};
    match word {
        Word::Enabled => &objects.enabled,
        Word::Selected => &objects.selected,
        Word::Object { index, field } => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            let object = &objects.objects[index];
            match field {
                ObjectWord::Channels => &object.channels,
                ObjectWord::BankSlot => &object.bank_slot,
                ObjectWord::InstructionList => &object.instruction_list,
                ObjectWord::Table => &object.table,
                ObjectWord::InstructionTimer => &object.instruction_timer,
                ObjectWord::PreInstruction => &object.pre_instruction,
                ObjectWord::PreInstructionBank => &object.pre_instruction_bank,
                ObjectWord::Timer => &object.timer,
                ObjectWord::A => &object.a,
                ObjectWord::B => &object.b,
                ObjectWord::C => &object.c,
                ObjectWord::D => &object.d,
                ObjectWord::E => &object.e,
                ObjectWord::F => &object.f,
            }
        }
    }
}
fn table_field_mut(objects: &mut crate::hdma::HdmaObjects, word: crate::hdma::Word) -> &mut u16 {
    use crate::hdma::{ObjectWord, Word};
    match word {
        Word::Enabled => &mut objects.enabled,
        Word::Selected => &mut objects.selected,
        Word::Object { index, field } => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            let object = &mut objects.objects[index];
            match field {
                ObjectWord::Channels => &mut object.channels,
                ObjectWord::BankSlot => &mut object.bank_slot,
                ObjectWord::InstructionList => &mut object.instruction_list,
                ObjectWord::Table => &mut object.table,
                ObjectWord::InstructionTimer => &mut object.instruction_timer,
                ObjectWord::PreInstruction => &mut object.pre_instruction,
                ObjectWord::PreInstructionBank => &mut object.pre_instruction_bank,
                ObjectWord::Timer => &mut object.timer,
                ObjectWord::A => &mut object.a,
                ObjectWord::B => &mut object.b,
                ObjectWord::C => &mut object.c,
                ObjectWord::D => &mut object.d,
                ObjectWord::E => &mut object.e,
                ObjectWord::F => &mut object.f,
            }
        }
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
