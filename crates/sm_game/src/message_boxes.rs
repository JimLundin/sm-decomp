//! Message boxes, input confirmation, animation and saved-game feedback.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
pub use crate::constants::bank_60::*;
pub use crate::menus::prelude::*;
use crate::message_composition as composition;
#[allow(unused_imports)]
use crate::ram::RawRam as _;

const MESSAGE_CONFIG_BASE: usize = sm_data::cartridge::lorom_offset(0x85869b) as usize;

fn InitializePpuForMessageBoxes<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        game.ram().fill(0x5f9..0x5fb, 0);
        crate::display::sequence::register_writes! { game;
            byte(HDMAEN as u16, 0); byte(CGADD as u16, 0x19);
            byte(CGDATA as u16, 0xb1); byte(CGDATA as u16, 0x0b);
            byte(CGDATA as u16, 0x1f); byte(CGDATA as u16, 0);
        }
        game.ram().write_byte(0x33ea, game.ram().read_byte(0x85));
        game.ram().write_byte(0x33eb, game.ram().read_byte(0x5b));
        game.ram().write_byte(0x5b, 88);
        game.ram().write_byte(0x6a, 23);
        game.ram().write_byte(0x70, 0);
        game.ram().write_byte(0x73, 0);
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
        game.ram().fill(0x3000..0x3082, 0);
        game.write_register_word(VMADDL as u16, addr_unk_605880 as u16);
        game.read_register_word(RDVRAML as u16);
        crate::display::sequence::register_writes! { game;
            word(DMAP1 as u16, 0x3981); word(A1T1L as u16, 0x4100);
            word(A1B1 as u16, 0x7e); word(DAS1L as u16, 0x700);
            word(DAS10 as u16, 0); word(A2A1H as u16, 0);
            byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
        }
    }
}
fn ClearMessageBoxBg3Tilemap<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        crate::word_transfers!(game, ascending, 896; little(0x3800) <- value(14));
        crate::display::sequence::register_writes! { game;
            word(VMADDL as u16, addr_unk_605880 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3800); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x700); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
#[forbid(unsafe_code)]
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

#[forbid(unsafe_code)]
fn WriteMessageTilemap<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u16 {
    game.ram().copy_in(0x5a6, &112u16.to_le_bytes());
    game.ram().copy_in(0x5a4, &124u16.to_le_bytes());
    game.ram().fill(0x5a2..0x5a4, 0);
    game.ram().fill(0x3000..0x30e0, 0);
    let message =
        u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)]) as isize;
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
        game.ram()
            .copy_in(0x3000 + byte_offset, &tile.to_le_bytes());
        if cursor.advance() {
            break;
        }
    }
    cursor.word
}
fn WriteMessageBoxTilemapRow<H: crate::menus::host::Host + ?Sized>(
    mut word: u16,
    tiles: &[u16],
    game: &mut H,
) {
    // Preserve the legacy table bounds and any writes before a bounds failure.
    for tile in tiles {
        let byte_offset = composition::destination_byte_offset(word);
        game.ram()
            .copy_in(0x3000 + byte_offset, &tile.to_le_bytes());
        word = word.wrapping_add(1);
    }
}
fn WriteLargeMessageBoxTilemap<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    WriteMessageBoxTilemapRow(0, composition::border(composition::BoxSize::Large), game);
    let end = WriteMessageTilemap(game);
    WriteMessageBoxTilemapRow(end, composition::border(composition::BoxSize::Large), game);
}
fn WriteSmallMessageBoxTilemap<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    WriteMessageBoxTilemapRow(0, composition::border(composition::BoxSize::Small), game);
    let end = WriteMessageTilemap(game);
    WriteMessageBoxTilemapRow(end, composition::border(composition::BoxSize::Small), game);
}
/// The original's draw table, keyed by routine address.
fn CallMsgBoxDraw<H: crate::menus::host::Host + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        8749658 => WriteLargeMessageBoxTilemap(game),
        8749705 => WriteSmallMessageBoxTilemap(game),
        _ => {
            game.rtl_unreachable();
        }
    }
}
fn MsgBoxMakeHdmaTable<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let radius = u16::from_le_bytes([game.ram().read_byte(0x5a2), game.ram().read_byte(0x5a3)]);
    let values = composition::radius_values(radius);
    for at in (0x30bc..0x30f8).step_by(2) {
        game.ram().copy_in(at, &values.top_scroll.to_le_bytes());
    }
    for at in (0x30f8..0x3134).step_by(2) {
        game.ram().copy_in(at, &values.bottom_scroll.to_le_bytes());
    }
    game.ram().fill(0x3134..0x31e0, 0);
    // Retain the final scratch publication order after all scanline writes.
    game.ram().copy_in(0x5a4, &values.scratch[0].to_le_bytes());
    game.ram().copy_in(0x5a6, &values.scratch[1].to_le_bytes());
    game.ram().copy_in(0x5a8, &values.scratch[2].to_le_bytes());
    game.ram().copy_in(0x5aa, &values.scratch[3].to_le_bytes());
}
fn SetupMessageBoxBg3YscrollHdma<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        game.ram().copy_in(0x3380, &composition::HDMA_DESCRIPTOR);
        crate::display::sequence::register_writes! { game;
            byte(DMAP6 as u16, 0x42); byte(BBAD6 as u16, 0x12);
            byte(A1T6L as u16, 0x80); byte(DAS6L as u16, 0x80);
            byte(A1T6H as u16, 0x33); byte(DAS6H as u16, 0x33);
            byte(A1B6 as u16, 0x7e); byte(DAS60 as u16, 0x7e);
            byte(A2A6L as u16, 0); byte(A2A6H as u16, 0);
            byte(NTRL6 as u16, 0);
        }
        MsgBoxMakeHdmaTable(game);
        game.write_register(HDMAEN as u16, 0x40);
    }
}
fn SetupPpuForActiveMessageBox<H: crate::menus::host::Host + ?Sized>(r52: u16, game: &mut H) {
    {
        SetupMessageBoxBg3YscrollHdma(game);
        crate::display::sequence::register_writes! { game;
            word(VMADDL as u16, r52.wrapping_add(0x5800)); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, game.message_box_das0l_value()); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
fn DrawSpecialButtonAndSetupPpuForLargeMessageBox<H: crate::menus::host::Host + ?Sized>(
    a: u16,
    game: &mut H,
) {
    {
        let tile = composition::button_letter(a);
        let message =
            u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)])
                as isize
                - 1;
        let offset = composition::special_button_offset(message);
        game.ram().copy_in(0x3200 + offset, &tile.to_le_bytes());
        SetupPpuForActiveMessageBox(composition::map_offset(composition::BoxSize::Large), game);
    }
}
fn DrawShootButtonAndSetupPpuForLargeMessageBox<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
) {
    {
        let button = u16::from_le_bytes([game.ram().read_byte(0x9b2), game.ram().read_byte(0x9b3)]);
        DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
    }
}
fn DrawRunButtonAndSetupPpuForLargeMessageBox<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        let button = u16::from_le_bytes([game.ram().read_byte(0x9b6), game.ram().read_byte(0x9b7)]);
        DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
    }
}
fn SetupPpuForSmallMessageBox<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        SetupPpuForActiveMessageBox(composition::map_offset(composition::BoxSize::Small), game)
    };
}
fn SetupPpuForLargeMessageBox<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        SetupPpuForActiveMessageBox(composition::map_offset(composition::BoxSize::Large), game)
    };
}
/// The original's modify table, keyed by routine address.
fn CallMsgBoxModify<H: crate::menus::host::Host + ?Sized>(ea: u32, game: &mut H) {
    {
        match ea {
            8750021 => DrawShootButtonAndSetupPpuForLargeMessageBox(game),
            8750028 => DrawRunButtonAndSetupPpuForLargeMessageBox(game),
            8750134 => SetupPpuForSmallMessageBox(game),
            8750145 => SetupPpuForLargeMessageBox(game),
            _ => {
                game.rtl_unreachable();
            }
        }
    }
}
fn InitializeMessageBox<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        let index = u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)])
            as isize
            - 1;
        let draw = game
            .native_image()
            .table_word(MESSAGE_CONFIG_BASE, 3 * index + 1);
        CallMsgBoxDraw(0x850000 | u32::from(draw), game);
        let modify = game
            .native_image()
            .table_word(MESSAGE_CONFIG_BASE, 3 * index);
        CallMsgBoxModify(0x850000 | u32::from(modify), game);
    }
}
fn DrawSaveConfirmationSelection<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        let selection =
            u16::from_le_bytes([game.ram().read_byte(0x5f9), game.ram().read_byte(0x5fa)]);
        WriteMessageBoxTilemapRow(128, composition::selection_row(selection), game);
        crate::display::sequence::register_writes! { game;
            word(VMADDL as u16, addr_unk_6059A0 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x180); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
fn RestorePpuForMessageBox<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        crate::display::sequence::register_writes! { game;
            word(VMADDL as u16, addr_unk_605880 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x4100); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x700); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
        game.ram().write_byte(0x85, game.ram().read_byte(0x33ea));
        game.write_register(HDMAEN as u16, game.ram().read_byte(0x33ea));
        game.ram().write_byte(0x5b, game.ram().read_byte(0x33eb));
        game.ram().write_byte(0x6a, game.ram().read_byte(0x69));
        game.ram().write_byte(0x70, game.ram().read_byte(0x6e));
        game.ram().write_byte(0x73, game.ram().read_byte(0x71));
        crate::display::sequence::register_writes! { game;
            byte(CGADD as u16, 0x19); byte(CGDATA as u16, game.ram().read_byte(0xc032));
            byte(CGDATA as u16, game.ram().read_byte(0xc033)); byte(CGDATA as u16, game.ram().read_byte(0xc034));
            byte(CGDATA as u16, game.ram().read_byte(0xc035));
        }
    }
}
fn advance_audio<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        game.advance_dialogue_audio();
    }
}

// Only this bridge enters the remaining unsafe presentation/input implementation.
struct LegacyDialogue<'a, H: ?Sized>(&'a mut H);
impl<H: crate::menus::host::Host + ?Sized> crate::dialogue::Context for LegacyDialogue<'_, H> {
    type Id = u16;
    fn effect(
        &mut self,
        state: crate::dialogue::Dialogue<u16>,
        effect: crate::dialogue::Effect,
    ) -> crate::dialogue::Dialogue<u16> {
        use {crate::dialogue::Effect::*, crate::dialogue_ram};
        dialogue_ram::publish(state, self.0);
        {
            match effect {
                CancelSounds => {
                    for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
                        self.0.queue_sound(channel, sound, 6);
                    }
                }
                PrepareDisplay => InitializePpuForMessageBoxes(self.0),
                ClearDisplay => ClearMessageBoxBg3Tilemap(self.0),
                PrepareMessage => InitializeMessageBox(self.0),
                SaveSound => self.0.queue_sound(0, 0x2e, 6),
                AdvanceAudio => advance_audio(self.0),
                RestoreDisplay => RestorePpuForMessageBox(self.0),
                ResumeMovementSound => self.0.restore_movement_sound(),
                SelectPause => dialogue_ram::select_pause(self.0),
                DrawRadius => MsgBoxMakeHdmaTable(self.0),
                DrawSelection => DrawSaveConfirmationSelection(self.0),
                SelectionSound => self.0.queue_sound(0, 0x37, 6),
            }
        }
        dialogue_ram::import(self.0)
    }
    fn wait(
        &mut self,
        state: crate::dialogue::Dialogue<u16>,
    ) -> (crate::dialogue::Dialogue<u16>, crate::dialogue::Outcome) {
        use {crate::dialogue::Outcome, crate::dialogue_ram};
        dialogue_ram::publish(state, self.0);
        let result = self.0.display_wait(false);
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
        &mut LegacyDialogue(game),
    );
    dialogue_ram::publish(state, game);
    match outcome {
        dialogue::Outcome::Complete => 0,
        dialogue::Outcome::Suspended(code) => code.get(),
    }
}
pub fn poll<H: crate::menus::host::Host + ?Sized>(a: u16, game: &mut H) -> i32 {
    if a == u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)]) {
        game.ram().fill(0x1c1f..0x1c21, 0);
        return u16::from_le_bytes([game.ram().read_byte(0x5f9), game.ram().read_byte(0x5fa)])
            as i32;
    }
    game.ram().copy_in(0xdc8, &a.to_le_bytes());
    -1
}
pub fn queue<H: crate::menus::host::Host + ?Sized>(a: u16, game: &mut H) {
    game.ram().copy_in(0xdc8, &a.to_le_bytes());
}
