//! Message boxes, input confirmation, animation and saved-game feedback.
pub use crate::constants::bank_60::*;
use crate::display::preparation::{Field, Store};
pub use crate::menus::prelude::*;
use crate::message_composition as composition;
use composition::BoxSize;

const MESSAGE_CONFIG_BASE: usize = sm_data::cartridge::lorom_offset(0x85869b) as usize;

/// The first half of "Initialise PPU for message boxes" ($85:8143), after its
/// first lag frame: save and replace the BG3 and blending registers.
fn prepare_display<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::display::sequence::register_writes! { game;
        byte(HDMAEN as u16, 0); byte(CGADD as u16, 0x19);
        byte(CGDATA as u16, 0xb1); byte(CGDATA as u16, 0x0b);
        byte(CGDATA as u16, 0x1f); byte(CGDATA as u16, 0);
    }
    game.screen_write_byte(
        Point::new(Role::MessageDescriptor, 0x6a),
        game.display_control_byte(Field::HdmaEnable),
    );
    game.screen_write_byte(
        Point::new(Role::MessageDescriptor, 0x6b),
        game.display_control_byte(Field::Background3Map),
    );
    game.store_display_control_byte(Field::Background3Map, 88, Store::Owned);
    game.store_display_control_byte(Field::PreviousMainLayers, 23, Store::Owned);
    game.store_display_control_byte(Field::PreviousColorSelection, 0, Store::Owned);
    game.store_display_control_byte(Field::PreviousColorMath, 0, Store::Owned);
    use crate::display::control::FixedColor;
    // One COLDATA write per colour, in order, as the compat sink did.
    crate::display::sequence::register_writes! { game;
        show(FixedColor::red(0)); show(FixedColor::green(0)); show(FixedColor::blue(0));
    }
    game.read_register(BG3HOFS as u16);
    crate::display::sequence::register_writes! { game;
        byte(BG3HOFS as u16, 0); byte(BG3HOFS as u16, 0);
    }
    game.read_register(BG3VOFS as u16);
    crate::display::sequence::register_writes! { game;
        byte(BG3VOFS as u16, 0); byte(BG3VOFS as u16, 0);
    }
    game.screen_write_slice(Point::new(Role::Pause, 0), &[0; 0x82]);
}
/// Read the BG3 tilemap the box covers back to $7E:4100.
fn save_background<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.write_register_word(VMADDL as u16, ADDR_UNK_605880 as u16);
    game.read_register_word(RDVRAML as u16);
    crate::display::sequence::register_writes! { game;
        word(DMAP1 as u16, 0x3981); word(A1T1L as u16, 0x4100);
        word(A1B1 as u16, 0x7e); word(DAS1L as u16, 0x700);
        word(DAS10 as u16, 0); word(A2A1H as u16, 0);
        byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
    }
}
/// DMA the BG3 tilemap at `source` to VRAM $5880.
fn upload_background<H: crate::menus::host::Host + ?Sized>(source: u16, game: &mut H) {
    crate::display::sequence::register_writes! { game;
        word(VMADDL as u16, ADDR_UNK_605880 as u16); word(DMAP1 as u16, 0x1801);
        word(A1T1L as u16, source); word(A1B1 as u16, 0x7e);
        word(DAS1L as u16, 0x700); word(DAS10 as u16, 0);
        word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
        byte(MDMAEN as u16, 2);
    }
}
fn message_tile<H: crate::menus::host::Host + ?Sized>(
    start: u16,
    byte_offset: u16,
    game: &mut H,
) -> u16 {
    let address = 0x850000 | u32::from(start);
    if start & 0x8000 == 0 {
        game.native_rom_diagnostic(address);
    }
    sm_data::cartridge::RomRecord {
        image: game.native_image(),
        base: sm_data::cartridge::lorom_offset(address) as usize,
    }
    .word(usize::from(byte_offset))
}

fn write_message_tilemap<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u16 {
    game.dialogue_set_scratch(1, 112);
    game.dialogue_set_scratch(0, 124);
    game.dialogue_set_radius(0);
    game.screen_write_slice(Point::new(Role::Pause, 0), &[0; 0xe0]);
    let message = game.dialogue_active_index() as isize;
    let configs = game.native_image();
    let start = configs.table_word(MESSAGE_CONFIG_BASE, 3 * (message - 1) + 2);
    let length = configs
        .table_word(MESSAGE_CONFIG_BASE, 3 * message + 2)
        .wrapping_sub(start);
    game.set_message_box_das0l_value(composition::dma_length(length));
    let mut cursor = composition::TextCursor::new(length);
    loop {
        // Keep every fresh media query before the selected destination check.
        let tile = message_tile(start, cursor.source_offset, game);
        let byte_offset = composition::destination_byte_offset(cursor.word);
        game.screen_write_slice(
            Point::new(Role::MessageText, byte_offset - 0x200),
            &tile.to_le_bytes(),
        );
        if cursor.advance() {
            break;
        }
    }
    cursor.word
}
fn write_message_box_tilemap_row<H: crate::menus::host::Host + ?Sized>(
    word: u16,
    tiles: &[u16],
    game: &mut H,
) {
    game.screen_apply(|workspace| crate::message_workspace::row(workspace, word, tiles));
}
/// The message's routine from its configuration row: 0 modifies, 1 draws.
fn message_routine<H: crate::menus::host::Host + ?Sized>(column: isize, game: &H) -> u32 {
    let index = game.dialogue_active_index() as isize - 1;
    let routine = game
        .native_image()
        .table_word(MESSAGE_CONFIG_BASE, 3 * index + column);
    0x850000 | u32::from(routine)
}
/// The box size from the original's draw table, keyed by routine address.
fn drawn_size<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> Option<BoxSize> {
    match message_routine(1, game) {
        0x85825a => Some(BoxSize::Large),
        0x858289 => Some(BoxSize::Small),
        _ => {
            game.rtl_unreachable();
            None
        }
    }
}
/// The original's modify table: its button letter and the size it uploads.
fn modified_size<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
) -> Option<(Option<usize>, BoxSize)> {
    match message_routine(0, game) {
        0x8583c5 => Some((Some(0x9b2), BoxSize::Large)),
        0x8583cc => Some((Some(0x9b6), BoxSize::Large)),
        0x858436 => Some((None, BoxSize::Small)),
        0x858441 => Some((None, BoxSize::Large)),
        _ => {
            game.rtl_unreachable();
            None
        }
    }
}
fn draw_top_border<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if let Some(size) = drawn_size(game) {
        write_message_box_tilemap_row(0, composition::border(size), game);
    }
}
fn write_message<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if let Some(size) = drawn_size(game) {
        let end = write_message_tilemap(game);
        write_message_box_tilemap_row(end, composition::border(size), game);
    }
}
/// Draw the bound button's letter, then set up the BG3 Y scroll HDMA channel
/// ($85:8363) before its table.
fn prepare_scroll<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let Some((button, _)) = modified_size(game) else {
        return;
    };
    if let Some(binding) = button {
        let buttons = game.dialogue_binding(binding == 0x9b2);
        let message = game.dialogue_active_index() as isize - 1;
        let offset = composition::special_button_offset(message);
        game.screen_write_slice(
            Point::new(Role::MessageText, offset),
            &composition::button_letter(buttons).to_le_bytes(),
        );
    }
    game.screen_write_slice(
        Point::new(Role::MessageDescriptor, 0),
        &composition::HDMA_DESCRIPTOR,
    );
    crate::display::sequence::register_writes! { game;
        byte(DMAP6 as u16, 0x42); byte(BBAD6 as u16, 0x12);
        byte(A1T6L as u16, 0x80); byte(DAS6L as u16, 0x80);
        byte(A1T6H as u16, 0x33); byte(DAS6H as u16, 0x33);
        byte(A1B6 as u16, 0x7e); byte(DAS60 as u16, 0x7e);
        byte(A2A6L as u16, 0); byte(A2A6H as u16, 0);
        byte(NTRL6 as u16, 0);
    }
}
fn msg_box_make_hdma_table<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let radius = game.dialogue_radius();
    let values = game.screen_apply(|workspace| crate::message_workspace::radius(workspace, radius));
    // Retain the final scratch publication order after all scanline writes.
    for (index, value) in values.scratch.into_iter().enumerate() {
        game.dialogue_set_scratch(index, value);
    }
}
/// DMA the composed box from $7E:3200 to its BG3 rows ($85:831E).
fn upload_message<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let Some((_, size)) = modified_size(game) else {
        return;
    };
    let destination = composition::map_offset(size).wrapping_add(0x5800);
    crate::display::sequence::register_writes! { game;
        word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1801);
        word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
        word(DAS1L as u16, game.message_box_das0l_value()); word(DAS10 as u16, 0);
        word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
        byte(MDMAEN as u16, 2);
    }
}
fn upload_selection<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::display::sequence::register_writes! { game;
        word(VMADDL as u16, ADDR_UNK_6059A0 as u16); word(DMAP1 as u16, 0x1801);
        word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
        word(DAS1L as u16, 0x180); word(DAS10 as u16, 0);
        word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
        byte(MDMAEN as u16, 2);
    }
}
/// The second half of "Restore PPU" ($85:861A), after its second lag frame.
fn restore_display<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.store_display_control_byte(
        Field::HdmaEnable,
        game.screen_byte(Point::new(Role::MessageDescriptor, 0x6a)),
        Store::Owned,
    );
    game.write_register(
        HDMAEN as u16,
        game.screen_byte(Point::new(Role::MessageDescriptor, 0x6a)),
    );
    game.store_display_control_byte(
        Field::Background3Map,
        game.screen_byte(Point::new(Role::MessageDescriptor, 0x6b)),
        Store::Owned,
    );
    game.store_display_control_byte(
        Field::PreviousMainLayers,
        game.display_control_byte(Field::MainLayers),
        Store::Owned,
    );
    game.store_display_control_byte(
        Field::PreviousColorSelection,
        game.display_control_byte(Field::NextColorSelection),
        Store::Owned,
    );
    game.store_display_control_byte(
        Field::PreviousColorMath,
        game.display_control_byte(Field::NextColorMath),
        Store::Owned,
    );
    crate::display::sequence::register_writes! { game;
        byte(CGADD as u16, 0x19); byte(CGDATA as u16, game.palette_byte(false, 0x32));
        byte(CGDATA as u16, game.palette_byte(false, 0x33)); byte(CGDATA as u16, game.palette_byte(false, 0x34));
        byte(CGDATA as u16, game.palette_byte(false, 0x35));
    }
}

// Each effect publishes control state before reaching the current presentation owner.
struct PresentationDialogue<'a, H: ?Sized>(&'a mut H);
impl<H: crate::menus::host::Host + ?Sized> crate::dialogue::Context
    for PresentationDialogue<'_, H>
{
    type Id = u16;
    fn effect(
        &mut self,
        state: crate::dialogue::Dialogue<u16>,
        effect: crate::dialogue::Effect,
    ) -> crate::dialogue::Dialogue<u16> {
        use {crate::dialogue::Effect::*, crate::dialogue_ram};
        dialogue_ram::publish(state, self.0);
        let game = &mut *self.0;
        match effect {
            CancelSounds => {
                for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
                    game.queue_sound(channel, sound, 6);
                }
            }
            ClearSelection => game.dialogue_set_selection(0),
            PrepareDisplay => prepare_display(game),
            SaveBackground => save_background(game),
            FillBackground => {
                game.screen_apply(|workspace| crate::message_workspace::background(workspace));
            }
            UploadBackground => upload_background(0x3800, game),
            DrawTopBorder => draw_top_border(game),
            WriteMessage => write_message(game),
            PrepareScroll => prepare_scroll(game),
            EnableScroll => game.write_register(HDMAEN as u16, 0x40),
            UploadMessage => upload_message(game),
            SaveSound => game.queue_sound(0, 0x2e, 6),
            AdvanceAudio => {
                game.handle_music_queue();
                game.advance_dialogue_sound_effects();
            }
            AdvanceSoundEffects => game.advance_dialogue_sound_effects(),
            RestoreBackground => upload_background(0x4100, game),
            RestoreDisplay => restore_display(game),
            UpdateHdma => game.run_hdma_objects(),
            ResumeMovementSound => game.restore_movement_sound(),
            SelectPause => dialogue_ram::select_pause(game),
            DrawRadius => msg_box_make_hdma_table(game),
            StageSelection => {
                let selection = game.dialogue_selection();
                write_message_box_tilemap_row(128, composition::selection_row(selection), game);
            }
            UploadSelection => upload_selection(game),
            SelectionSound => game.queue_sound(0, 0x37, 6),
        }
        dialogue_ram::import(self.0)
    }
    fn wait(
        &mut self,
        state: crate::dialogue::Dialogue<u16>,
    ) -> (crate::dialogue::Dialogue<u16>, crate::dialogue::Outcome) {
        use {crate::dialogue::Outcome, crate::dialogue_ram};
        dialogue_ram::publish(state, self.0);
        let result = crate::display_refresh::wait(false, self.0);
        let outcome =
            std::num::NonZeroU8::new(result).map_or(Outcome::Complete, Outcome::Suspended);
        (dialogue_ram::import(self.0), outcome)
    }
    fn input(
        &mut self,
        state: crate::dialogue::Dialogue<u16>,
    ) -> (crate::dialogue::Dialogue<u16>, crate::dialogue::Input) {
        use crate::dialogue_ram;
        dialogue_ram::publish(state, self.0);
        crate::menus::controller_input::sample(self.0);
        (dialogue_ram::import(self.0), dialogue_ram::input(self.0))
    }
}
pub fn display<H: crate::menus::host::Host + ?Sized>(message: u16, game: &mut H) -> CoroutineRet {
    use {crate::dialogue, crate::dialogue_ram};
    let state = dialogue_ram::import(game);
    let (state, outcome) = dialogue::run(
        state,
        dialogue_ram::message(message),
        &mut PresentationDialogue(game),
    );
    dialogue_ram::publish(state, game);
    match outcome {
        dialogue::Outcome::Complete => 0,
        dialogue::Outcome::Suspended(code) => code.get(),
    }
}
pub fn poll<H: crate::menus::host::Host + ?Sized>(a: u16, game: &mut H) -> i32 {
    if a == game.dialogue_active_index() {
        game.dialogue_clear_active();
        return i32::from(game.dialogue_selection());
    }
    game.dialogue_set_request(a);
    -1
}
pub fn queue<H: crate::menus::host::Host + ?Sized>(a: u16, game: &mut H) {
    game.dialogue_set_request(a);
}
