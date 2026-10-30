//! Message boxes, input confirmation, animation and saved-game feedback.
pub use crate::game::constants::bank_60::*;
use crate::game::message_composition as composition;
use crate::game::music::HandleMusicQueue;
pub use crate::game::prelude::*;

use crate::game::music::QueueSfx1_Max6;
use crate::game::music::{advance_sound_effects, cancel_effects};

use crate::rtl::ReadReg;

const MESSAGE_CONFIG_BASE: usize = crate::rtl::lorom_offset(0x85869b) as usize;

unsafe fn InitializePpuForMessageBoxes(game: &mut Game) {
    unsafe {
        game.ram().fill(0x5f9..0x5fb, 0);
        crate::rtl::register_writes! { game;
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
        use sm_game::display::control::FixedColor;
        crate::game::display_transfers::set_fixed_color(
            &[
                FixedColor::red(0),
                FixedColor::green(0),
                FixedColor::blue(0),
            ],
            game,
        );
        ReadReg(BG3HOFS as u16, game);
        crate::rtl::register_writes! { game;
            byte(BG3HOFS as u16, 0); byte(BG3HOFS as u16, 0);
        }
        ReadReg(BG3VOFS as u16, game);
        crate::rtl::register_writes! { game;
            byte(BG3VOFS as u16, 0); byte(BG3VOFS as u16, 0);
        }
        game.ram().fill(0x3000..0x3082, 0);
        WriteRegWord(VMADDL as u16, addr_unk_605880 as u16, game);
        ReadRegWord(RDVRAML as u16, game);
        crate::rtl::register_writes! { game;
            word(DMAP1 as u16, 0x3981); word(A1T1L as u16, 0x4100);
            word(A1B1 as u16, 0x7e); word(DAS1L as u16, 0x700);
            word(DAS10 as u16, 0); word(A2A1H as u16, 0);
            byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
        }
    }
}
unsafe fn ClearMessageBoxBg3Tilemap(game: &mut Game) {
    unsafe {
        word_transfers!(game, ascending, 896; little(0x3800) <- value(14));
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, addr_unk_605880 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3800); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x700); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
#[forbid(unsafe_code)]
fn message_tile(start: u16, byte_offset: u16, game: &mut Game) -> u16 {
    let address = 0x850000 | u32::from(start);
    if start & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    crate::rom::RomRecord {
        image: game.capture_native_rom(),
        base: crate::rtl::lorom_offset(address) as usize,
    }
    .word(usize::from(byte_offset))
}

#[forbid(unsafe_code)]
fn WriteMessageTilemap(game: &mut Game) -> u16 {
    game.ram().copy_in(0x5a6, &112u16.to_le_bytes());
    game.ram().copy_in(0x5a4, &124u16.to_le_bytes());
    game.ram().fill(0x5a2..0x5a4, 0);
    game.ram().fill(0x3000..0x30e0, 0);
    let message =
        u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)]) as isize;
    let configs = game.capture_native_rom();
    let start = configs.table_word(MESSAGE_CONFIG_BASE, 3 * (message - 1) + 2);
    let length = configs
        .table_word(MESSAGE_CONFIG_BASE, 3 * message + 2)
        .wrapping_sub(start);
    game.message_box_das0l_value = composition::dma_length(length);
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
fn WriteMessageBoxTilemapRow(mut word: u16, tiles: &[u16], game: &mut Game) {
    // Preserve the legacy table bounds and any writes before a bounds failure.
    for tile in tiles {
        let byte_offset = composition::destination_byte_offset(word);
        game.ram()
            .copy_in(0x3000 + byte_offset, &tile.to_le_bytes());
        word = word.wrapping_add(1);
    }
}
crate::game::callback_dispatch! {
    unsafe fn CallMsgBoxDraw(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @body checked;
    @functions [fn (game: &mut Game)];
    [8749658] WriteLargeMessageBoxTilemap [] {
        WriteMessageBoxTilemapRow(0, composition::border(composition::BoxSize::Large), game);
        let end = WriteMessageTilemap(game);
        WriteMessageBoxTilemapRow(end, composition::border(composition::BoxSize::Large), game);
    };
    [8749705] WriteSmallMessageBoxTilemap [] {
        WriteMessageBoxTilemapRow(0, composition::border(composition::BoxSize::Small), game);
        let end = WriteMessageTilemap(game);
        WriteMessageBoxTilemapRow(end, composition::border(composition::BoxSize::Small), game);
    };
}
fn MsgBoxMakeHdmaTable(game: &mut Game) {
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
unsafe fn SetupMessageBoxBg3YscrollHdma(game: &mut Game) {
    unsafe {
        game.ram().copy_in(0x3380, &composition::HDMA_DESCRIPTOR);
        crate::rtl::register_writes! { game;
            byte(DMAP6 as u16, 0x42); byte(BBAD6 as u16, 0x12);
            byte(A1T6L as u16, 0x80); byte(DAS6L as u16, 0x80);
            byte(A1T6H as u16, 0x33); byte(DAS6H as u16, 0x33);
            byte(A1B6 as u16, 0x7e); byte(DAS60 as u16, 0x7e);
            byte(A2A6L as u16, 0); byte(A2A6H as u16, 0);
            byte(NTRL6 as u16, 0);
        }
        MsgBoxMakeHdmaTable(game);
        WriteReg(HDMAEN as u16, 0x40, game);
    }
}
unsafe fn SetupPpuForActiveMessageBox(r52: u16, game: &mut Game) {
    unsafe {
        SetupMessageBoxBg3YscrollHdma(game);
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, r52.wrapping_add(0x5800)); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, game.message_box_das0l_value); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
unsafe fn DrawSpecialButtonAndSetupPpuForLargeMessageBox(a: u16, game: &mut Game) {
    unsafe {
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
crate::game::callback_dispatch! {
    unsafe fn CallMsgBoxModify(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [unsafe fn (game: &mut Game)];
    [8750021] DrawShootButtonAndSetupPpuForLargeMessageBox [] {
        let button = u16::from_le_bytes([game.ram().read_byte(0x9b2), game.ram().read_byte(0x9b3)]);
        DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
    };
    [8750028] DrawRunButtonAndSetupPpuForLargeMessageBox [] {
        let button = u16::from_le_bytes([game.ram().read_byte(0x9b6), game.ram().read_byte(0x9b7)]);
        DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
    };
    @forward [unsafe fn (game: &mut Game)];
    [8750134] SetupPpuForSmallMessageBox => SetupPpuForActiveMessageBox(composition::map_offset(composition::BoxSize::Small), game);
    [8750145] SetupPpuForLargeMessageBox => SetupPpuForActiveMessageBox(composition::map_offset(composition::BoxSize::Large), game);
}
unsafe fn InitializeMessageBox(game: &mut Game) {
    unsafe {
        let index = u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)])
            as isize
            - 1;
        let draw = game
            .capture_native_rom()
            .table_word(MESSAGE_CONFIG_BASE, 3 * index + 1);
        CallMsgBoxDraw(0x850000 | u32::from(draw), game);
        let modify = game
            .capture_native_rom()
            .table_word(MESSAGE_CONFIG_BASE, 3 * index);
        CallMsgBoxModify(0x850000 | u32::from(modify), game);
    }
}
unsafe fn DrawSaveConfirmationSelection(game: &mut Game) {
    unsafe {
        let selection =
            u16::from_le_bytes([game.ram().read_byte(0x5f9), game.ram().read_byte(0x5fa)]);
        WriteMessageBoxTilemapRow(128, composition::selection_row(selection), game);
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, addr_unk_6059A0 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x3200); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x180); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
    }
}
unsafe fn RestorePpuForMessageBox(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, addr_unk_605880 as u16); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, 0x4100); word(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x700); word(DAS10 as u16, 0);
            word(A2A1H as u16, 0); byte(VMAIN as u16, 0x80);
            byte(MDMAEN as u16, 2);
        }
        game.ram().write_byte(0x85, game.ram().read_byte(0x33ea));
        WriteReg(HDMAEN as u16, game.ram().read_byte(0x33ea), game);
        game.ram().write_byte(0x5b, game.ram().read_byte(0x33eb));
        game.ram().write_byte(0x6a, game.ram().read_byte(0x69));
        game.ram().write_byte(0x70, game.ram().read_byte(0x6e));
        game.ram().write_byte(0x73, game.ram().read_byte(0x71));
        crate::rtl::register_writes! { game;
            byte(CGADD as u16, 0x19); byte(CGDATA as u16, game.ram().read_byte(0xc032));
            byte(CGDATA as u16, game.ram().read_byte(0xc033)); byte(CGDATA as u16, game.ram().read_byte(0xc034));
            byte(CGDATA as u16, game.ram().read_byte(0xc035));
        }
    }
}
unsafe fn advance_audio(game: &mut Game) {
    unsafe {
        HandleMusicQueue(game);
        advance_sound_effects(game, |channel, value, meaning, game| {
            crate::native_audio::emit(
                crate::native_audio::sound_cue(
                    channel,
                    sm_data::OriginalSoundEffect(value),
                    meaning,
                ),
                sm_data::CuePhase::DialogueSound,
                game,
            )
        });
    }
}

// Only this bridge enters the remaining unsafe presentation/input implementation.
struct LegacyDialogue<'a>(&'a mut Game);
impl sm_game::dialogue::Context for LegacyDialogue<'_> {
    type Id = u16;
    fn effect(
        &mut self,
        state: sm_game::dialogue::Dialogue<u16>,
        effect: sm_game::dialogue::Effect,
    ) -> sm_game::dialogue::Dialogue<u16> {
        use {crate::game::dialogue_ram, sm_game::dialogue::Effect::*};
        dialogue_ram::publish(state, self.0);
        unsafe {
            match effect {
                CancelSounds => cancel_effects(6, self.0),
                PrepareDisplay => InitializePpuForMessageBoxes(self.0),
                ClearDisplay => ClearMessageBoxBg3Tilemap(self.0),
                PrepareMessage => InitializeMessageBox(self.0),
                SaveSound => QueueSfx1_Max6(0x2e, self.0),
                AdvanceAudio => advance_audio(self.0),
                RestoreDisplay => RestorePpuForMessageBox(self.0),
                ResumeMovementSound => crate::game::player_projectiles::restore_sound(self.0),
                SelectPause => dialogue_ram::select_pause(self.0),
                DrawRadius => MsgBoxMakeHdmaTable(self.0),
                DrawSelection => DrawSaveConfirmationSelection(self.0),
                SelectionSound => QueueSfx1_Max6(0x37, self.0),
            }
        }
        dialogue_ram::import(self.0)
    }
    fn wait(
        &mut self,
        state: sm_game::dialogue::Dialogue<u16>,
    ) -> (sm_game::dialogue::Dialogue<u16>, sm_game::dialogue::Outcome) {
        use {crate::game::dialogue_ram, sm_game::dialogue::Outcome};
        dialogue_ram::publish(state, self.0);
        let result = crate::game::display_refresh::wait(false, self.0);
        let outcome =
            std::num::NonZeroU8::new(result).map_or(Outcome::Complete, Outcome::Suspended);
        (dialogue_ram::import(self.0), outcome)
    }
    fn input(
        &mut self,
        state: sm_game::dialogue::Dialogue<u16>,
    ) -> (sm_game::dialogue::Dialogue<u16>, sm_game::dialogue::Input) {
        use crate::game::dialogue_ram;
        dialogue_ram::publish(state, self.0);
        crate::game::controller_input::sample(self.0);
        (dialogue_ram::import(self.0), dialogue_ram::input(self.0))
    }
}
pub unsafe fn display(message: u16, game: &mut Game) -> CoroutineRet {
    use {crate::game::dialogue_ram, sm_game::dialogue};
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
pub fn poll(a: u16, game: &mut Game) -> i32 {
    if a == u16::from_le_bytes([game.ram().read_byte(0x1c1f), game.ram().read_byte(0x1c20)]) {
        game.ram().fill(0x1c1f..0x1c21, 0);
        return u16::from_le_bytes([game.ram().read_byte(0x5f9), game.ram().read_byte(0x5fa)])
            as i32;
    }
    game.ram().copy_in(0xdc8, &a.to_le_bytes());
    -1
}
pub fn queue(a: u16, game: &mut Game) {
    game.ram().copy_in(0xdc8, &a.to_le_bytes());
}
