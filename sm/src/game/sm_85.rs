pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::game_states::*;
use crate::game::sm_80::{
    HandleMusicQueue, QueueSfx1_Max6, ReadJoypadInputs, WaitForNMI_NoUpdate_Async,
};
use crate::game::sm_82::{CancelSoundEffects, HandleSoundEffects, QueueSamusMovementSfx};
use crate::game_state::Game;
use crate::rtl::{ReadReg, ReadRegWord, RomPtr, Unreachable, WriteReg, WriteRegWord};
pub use crate::snes::registers::*;

pub use crate::types::{
    __int16_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, uint16, uint16_t, uint32,
    uint32_t, uint8, uint8_t, CoroutineRet, SnesRegs, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MsgBoxConfig {
    pub modify_box_func: VoidP,
    pub draw_initial_tilemap: VoidP,
    pub message_tilemap: VoidP,
}
pub use crate::types::Buttons;
pub use crate::types::GameState;
pub use crate::types::{
    Consts_60, Ram3000, Ram3000_Menu, Ram3000_Misc, Ram3000_MsgBox, Ram3800, Ram4000,
    Ram4000_Backups,
};
#[inline]
unsafe fn RomPtr_85(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x850000u32) | ((addr) as u32), game)
}
pub const fnWriteLargeMessageBoxTilemap: uint32 = 8749658;
pub const fnWriteSmallMessageBoxTilemap: uint32 = 8749705;
pub const fnDrawShootButtonAndSetupPpuForLargeMessageBox: uint32 = 8750021;
pub const fnDrawRunButtonAndSetupPpuForLargeMessageBox: uint32 = 8750028;
pub const fnSetupPpuForSmallMessageBox: uint32 = 8750134;
pub const fnSetupPpuForLargeMessageBox: uint32 = 8750145;

static kMsgBoxSpecialButtonTilemapOffs: [uint16; 27] = [
    0, 0x12a, 0x12a, 0x12c, 0x12c, 0x12c, 0, 0, 0, 0, 0, 0, 0x120, 0, 0, 0, 0, 0, 0x12a, 0, 0, 0,
    0, 0, 0, 0, 0,
];
static kTileNumbersForButtonLetters: [uint16; 8] = [
    0x28e0, 0x3ce1, 0x2cf7, 0x38f8, 0x38d0, 0x38eb, 0x38f1, 0x284e,
];
unsafe fn InitializePpuForMessageBoxes(game: &mut Game) {
    game.g_ram[0x5f9..0x5fb].fill(0);
    WriteReg(HDMAEN as u16, 0, game);
    WriteReg(CGADD as u16, 0x19, game);
    WriteReg(CGDATA as u16, 0xb1, game);
    WriteReg(CGDATA as u16, 0x0b, game);
    WriteReg(CGDATA as u16, 0x1f, game);
    WriteReg(CGDATA as u16, 0, game);
    game.g_ram[0x33ea] = game.g_ram[0x85];
    game.g_ram[0x33eb] = game.g_ram[0x5b];
    game.g_ram[0x5b] = 88;
    game.g_ram[0x6a] = 23;
    game.g_ram[0x70] = 0;
    game.g_ram[0x73] = 0;
    WriteReg(COLDATA as u16, 0x20, game);
    WriteReg(COLDATA as u16, 0x40, game);
    WriteReg(COLDATA as u16, 0x80, game);
    ReadReg(BG3HOFS as u16, game);
    WriteReg(BG3HOFS as u16, 0, game);
    WriteReg(BG3HOFS as u16, 0, game);
    ReadReg(BG3VOFS as u16, game);
    WriteReg(BG3VOFS as u16, 0, game);
    WriteReg(BG3VOFS as u16, 0, game);
    game.g_ram[0x3000..0x3082].fill(0);
    WriteRegWord(VMADDL as u16, addr_unk_605880 as u16, game);
    ReadRegWord(RDVRAML as u16, game);
    WriteRegWord(DMAP1 as u16, 0x3981, game);
    WriteRegWord(A1T1L as u16, 0x4100, game);
    WriteRegWord(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, 0x700, game);
    WriteRegWord(DAS10 as u16, 0, game);
    WriteRegWord(A2A1H as u16, 0, game);
    WriteReg(VMAIN as u16, 0x80, game);
    WriteReg(MDMAEN as u16, 2, game);
}
unsafe fn ClearMessageBoxBg3Tilemap(game: &mut Game) {
    for tile in game.g_ram[0x3800..0x3f00].chunks_exact_mut(2) {
        tile.copy_from_slice(&14u16.to_le_bytes());
    }
    WriteRegWord(VMADDL as u16, addr_unk_605880 as u16, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, 0x3800, game);
    WriteRegWord(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, 0x700, game);
    WriteRegWord(DAS10 as u16, 0, game);
    WriteRegWord(A2A1H as u16, 0, game);
    WriteReg(VMAIN as u16, 0x80, game);
    WriteReg(MDMAEN as u16, 2, game);
}
static kLargeMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x284e, 0x284e, 0xe, 0xe, 0xe,
];
static kSmallMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e,
    0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];
unsafe fn WriteMessageTilemap(game: &mut Game) -> uint16 {
    game.g_ram[0x5a6..0x5a8].copy_from_slice(&112u16.to_le_bytes());
    game.g_ram[0x5a4..0x5a6].copy_from_slice(&124u16.to_le_bytes());
    game.g_ram[0x5a2..0x5a4].fill(0);
    game.g_ram[0x3000..0x30e0].fill(0);
    let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) as isize;
    let configs = game
        .g_rom
        .add(crate::rtl::lorom_offset(0x85869b) as usize)
        .cast::<MsgBoxConfig>();
    let start = (*configs.offset(message - 1)).message_tilemap;
    let length = (*configs.offset(message))
        .message_tilemap
        .wrapping_sub(start);
    game.message_box_das0l_value = length.wrapping_add(128);
    let mut remaining = i32::from(length >> 1);
    let mut word = 32u16;
    let mut source_offset = 0u16;
    loop {
        // Map every word, including the mapper's invalid-address diagnostics.
        let tile = RomPtr_85(start, game)
            .add(usize::from(source_offset))
            .cast::<u16>()
            .read_unaligned();
        let byte_offset = 2 * (usize::from(word) + 256);
        game.g_ram[0x3000..0x3800][byte_offset..byte_offset + 2]
            .copy_from_slice(&tile.to_le_bytes());
        word = word.wrapping_add(1);
        source_offset = source_offset.wrapping_add(2);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    word
}
fn WriteMessageBoxTilemapRow(mut word: u16, tiles: &[u16], game: &mut Game) {
    // Preserve the legacy table bounds and any writes before a bounds failure.
    for tile in tiles {
        let byte_offset = 2 * (usize::from(word) + 256);
        game.g_ram[0x3000..0x3800][byte_offset..byte_offset + 2]
            .copy_from_slice(&tile.to_le_bytes());
        word = word.wrapping_add(1);
    }
}
unsafe fn WriteLargeMessageBoxTilemap(game: &mut Game) {
    WriteMessageBoxTilemapRow(0, &kLargeMsgBoxTopBottomBorderTilemap, game);
    let end = WriteMessageTilemap(game);
    WriteMessageBoxTilemapRow(end, &kLargeMsgBoxTopBottomBorderTilemap, game);
}
unsafe fn WriteSmallMessageBoxTilemap(game: &mut Game) {
    WriteMessageBoxTilemapRow(0, &kSmallMsgBoxTopBottomBorderTilemap, game);
    let end = WriteMessageTilemap(game);
    WriteMessageBoxTilemapRow(end, &kSmallMsgBoxTopBottomBorderTilemap, game);
}
unsafe fn CallMsgBoxDraw(ea: uint32, game: &mut Game) {
    match ea {
        8749658 => {
            WriteLargeMessageBoxTilemap(game);
        }
        8749705 => {
            WriteSmallMessageBoxTilemap(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
fn MsgBoxMakeHdmaTable(game: &mut Game) {
    let radius = u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]);
    let upper = 0x7b00u16.wrapping_sub(radius) >> 8;
    let lower = radius.wrapping_add(0x7c00) >> 8;

    // Each scanline and its edge move together, so their difference stays constant.
    for word in game.g_ram[0x30bc..0x30f8].chunks_exact_mut(2) {
        word.copy_from_slice(&99u16.wrapping_sub(upper).to_le_bytes());
    }
    for word in game.g_ram[0x30f8..0x3134].chunks_exact_mut(2) {
        word.copy_from_slice(&148u16.wrapping_sub(lower).to_le_bytes());
    }
    game.g_ram[0x3134..0x31e0].fill(0);

    // Preserve the scratch words left after all 30 scanlines.
    game.g_ram[0x5a4..0x5a6].copy_from_slice(&lower.wrapping_add(30).to_le_bytes());
    game.g_ram[0x5a6..0x5a8].copy_from_slice(&178u16.to_le_bytes());
    game.g_ram[0x5a8..0x5aa].copy_from_slice(&upper.wrapping_sub(30).to_le_bytes());
    game.g_ram[0x5aa..0x5ac].copy_from_slice(&69u16.to_le_bytes());
}
unsafe fn SetupMessageBoxBg3YscrollHdma(game: &mut Game) {
    game.g_ram[0x3380..0x3387].copy_from_slice(&[0xff, 0x00, 0x30, 0xe1, 0xfe, 0x30, 0x00]);
    WriteReg(DMAP6 as uint16, 0x42 as uint8, game);
    WriteReg(BBAD6 as uint16, 0x12 as uint8, game);
    WriteReg(A1T6L as uint16, 0x80 as uint8, game);
    WriteReg(DAS6L as uint16, 0x80 as uint8, game);
    WriteReg(A1T6H as uint16, 0x33 as uint8, game);
    WriteReg(DAS6H as uint16, 0x33 as uint8, game);
    WriteReg(A1B6 as uint16, 0x7e as uint8, game);
    WriteReg(DAS60 as uint16, 0x7e as uint8, game);
    WriteReg(A2A6L as uint16, 0 as uint8, game);
    WriteReg(A2A6H as uint16, 0 as uint8, game);
    WriteReg(NTRL6 as uint16, 0 as uint8, game);
    MsgBoxMakeHdmaTable(game);
    WriteReg(HDMAEN as uint16, 0x40 as uint8, game);
}
unsafe fn SetupPpuForActiveMessageBox(r52: uint16, game: &mut Game) {
    SetupMessageBoxBg3YscrollHdma(game);
    WriteRegWord(VMADDL as u16, r52.wrapping_add(0x5800), game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, 0x3200, game);
    WriteRegWord(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, game.message_box_das0l_value, game);
    WriteRegWord(DAS10 as u16, 0, game);
    WriteRegWord(A2A1H as u16, 0, game);
    WriteReg(VMAIN as u16, 0x80, game);
    WriteReg(MDMAEN as u16, 2, game);
}
unsafe fn DrawSpecialButtonAndSetupPpuForLargeMessageBox(a: uint16, game: &mut Game) {
    let buttons = [
        kButton_A,
        kButton_B,
        kButton_X,
        kButton_Y,
        kButton_Select,
        kButton_L,
        kButton_R,
    ];
    let letter = buttons
        .iter()
        .position(|&button| a & button as u16 != 0)
        .unwrap_or(7);
    let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) as isize - 1;
    let offset = usize::from(kMsgBoxSpecialButtonTilemapOffs[message as usize]);
    game.g_ram[0x3200 + offset..0x3202 + offset]
        .copy_from_slice(&kTileNumbersForButtonLetters[letter].to_le_bytes());
    SetupPpuForActiveMessageBox(416, game);
}
unsafe fn DrawShootButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    let button = u16::from_le_bytes([game.g_ram[0x9b2], game.g_ram[0x9b3]]);
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
}
unsafe fn DrawRunButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    let button = u16::from_le_bytes([game.g_ram[0x9b6], game.g_ram[0x9b7]]);
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(button, game);
}
unsafe fn SetupPpuForSmallMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(448 as uint16, game);
}
unsafe fn SetupPpuForLargeMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(416 as uint16, game);
}
unsafe fn CallMsgBoxModify(ea: uint32, game: &mut Game) {
    match ea {
        8750021 => {
            DrawShootButtonAndSetupPpuForLargeMessageBox(game);
        }
        8750028 => {
            DrawRunButtonAndSetupPpuForLargeMessageBox(game);
        }
        8750134 => {
            SetupPpuForSmallMessageBox(game);
        }
        8750145 => {
            SetupPpuForLargeMessageBox(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
unsafe fn InitializeMessageBox(game: &mut Game) {
    let index = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) as isize - 1;
    let draw = (*game
        .g_rom
        .add(crate::rtl::lorom_offset(0x85869b) as usize)
        .cast::<MsgBoxConfig>()
        .offset(index))
    .draw_initial_tilemap;
    CallMsgBoxDraw(0x850000 | u32::from(draw), game);
    let modify = (*game
        .g_rom
        .add(crate::rtl::lorom_offset(0x85869b) as usize)
        .cast::<MsgBoxConfig>()
        .offset(index))
    .modify_box_func;
    CallMsgBoxModify(0x850000 | u32::from(modify), game);
}
static kSaveConfirmationSelectionTilemap: [uint16; 96] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd, 0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd,
    0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2cf8, 0x2ce4, 0x2cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x38cc,
    0x38cd, 0x3ced, 0x3cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];
unsafe fn ToggleSaveConfirmationSelection(game: &mut Game) {
    let selection = u16::from_le_bytes([game.g_ram[0x5f9], game.g_ram[0x5fa]]) ^ 2;
    game.g_ram[0x5f9..0x5fb].copy_from_slice(&selection.to_le_bytes());
    let row = if selection == 2 { 64 } else { 32 };
    WriteMessageBoxTilemapRow(128, &kSaveConfirmationSelectionTilemap[row..row + 32], game);
    WriteRegWord(VMADDL as uint16, addr_unk_6059A0 as uint16, game);
    WriteRegWord(DMAP1 as uint16, 0x1801 as uint16, game);
    WriteRegWord(A1T1L as uint16, 0x3200, game);
    WriteRegWord(A1B1 as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as uint16, 0x180 as uint16, game);
    WriteRegWord(DAS10 as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as uint16, 0 as uint16, game);
    WriteReg(VMAIN as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as uint16, 2 as uint8, game);
}
unsafe fn RestorePpuForMessageBox(game: &mut Game) {
    WriteRegWord(VMADDL as u16, addr_unk_605880 as u16, game);
    WriteRegWord(DMAP1 as u16, 0x1801, game);
    WriteRegWord(A1T1L as u16, 0x4100, game);
    WriteRegWord(A1B1 as u16, 0x7e, game);
    WriteRegWord(DAS1L as u16, 0x700, game);
    WriteRegWord(DAS10 as u16, 0, game);
    WriteRegWord(A2A1H as u16, 0, game);
    WriteReg(VMAIN as u16, 0x80, game);
    WriteReg(MDMAEN as u16, 2, game);
    game.g_ram[0x85] = game.g_ram[0x33ea];
    WriteReg(HDMAEN as u16, game.g_ram[0x33ea], game);
    game.g_ram[0x5b] = game.g_ram[0x33eb];
    game.g_ram[0x6a] = game.g_ram[0x69];
    game.g_ram[0x70] = game.g_ram[0x6e];
    game.g_ram[0x73] = game.g_ram[0x71];
    WriteReg(CGADD as u16, 0x19, game);
    WriteReg(CGDATA as u16, game.g_ram[0xc032], game);
    WriteReg(CGDATA as u16, game.g_ram[0xc033], game);
    WriteReg(CGDATA as u16, game.g_ram[0xc034], game);
    WriteReg(CGDATA as u16, game.g_ram[0xc035], game);
}
unsafe fn OpenMessageBox_Async(game: &mut Game) -> CoroutineRet {
    match game.g_ram[0x615] {
        0 => game.g_ram[0x5a2..0x5a4].fill(0),
        1 => {}
        _ => {
            game.g_ram[0x615] = 0;
            return 0;
        }
    }
    loop {
        let result = WaitForNMI_NoUpdate_Async(game);
        if result != 0 {
            game.g_ram[0x615] = 1;
            return result;
        }
        HandleMusicQueue(game);
        HandleSoundEffects(game);
        MsgBoxMakeHdmaTable(game);
        let radius = u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]);
        if radius == 0x1800 {
            break;
        }
        game.g_ram[0x5a2..0x5a4].copy_from_slice(&radius.wrapping_add(0x200).to_le_bytes());
    }
    game.g_ram[0x615] = 0;
    0
}
unsafe fn HandleMessageBoxInteraction_Async(game: &mut Game) -> CoroutineRet {
    enum Phase {
        Confirmation,
        Delay,
        Dismiss,
    }
    let mut phase = match game.g_ram[0x615] {
        0 => {
            let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]);
            if matches!(message, 23 | 28) {
                game.g_ram[0x5f9..0x5fb].fill(0);
                Phase::Confirmation
            } else {
                game.g_ram[0x77e..0x780].copy_from_slice(&10u16.to_le_bytes());
                if !matches!(message, 20 | 21 | 22 | 24) {
                    game.g_ram[0x77e..0x780].copy_from_slice(&360u16.to_le_bytes());
                }
                Phase::Delay
            }
        }
        1 => Phase::Confirmation,
        2 => Phase::Delay,
        3 => Phase::Dismiss,
        _ => {
            game.g_ram[0x615] = 0;
            return 0;
        }
    };
    loop {
        match phase {
            Phase::Confirmation => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 1;
                    return result;
                }
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                ReadJoypadInputs(game);
                let keys = u16::from_le_bytes([game.g_ram[0x8f], game.g_ram[0x90]]);
                if keys & kButton_A as u16 != 0 {
                    break;
                }
                if keys & kButton_B as u16 != 0 {
                    game.g_ram[0x5f9..0x5fb].copy_from_slice(&2u16.to_le_bytes());
                    break;
                }
                if keys & (kButton_Select | kButton_Left | kButton_Right) as u16 != 0 {
                    ToggleSaveConfirmationSelection(game);
                    QueueSfx1_Max6(0x37, game);
                }
            }
            Phase::Delay => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 2;
                    return result;
                }
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                let timer =
                    u16::from_le_bytes([game.g_ram[0x77e], game.g_ram[0x77f]]).wrapping_sub(1);
                game.g_ram[0x77e..0x780].copy_from_slice(&timer.to_le_bytes());
                if timer == 0 {
                    phase = Phase::Dismiss;
                }
            }
            Phase::Dismiss => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x615] = 3;
                    return result;
                }
                ReadJoypadInputs(game);
                let bug_fix_counter = u16::from_le_bytes([game.g_ram[0xc2], game.g_ram[0xc3]]);
                let keys = if bug_fix_counter == 0 {
                    u16::from_le_bytes([game.g_ram[0x8f], game.g_ram[0x90]])
                } else {
                    game.joypad1_lastkeys()
                };
                if keys != 0 {
                    break;
                }
            }
        }
    }
    game.g_ram[0x615] = 0;
    0
}
unsafe fn CloseMessageBox_Async(game: &mut Game) -> CoroutineRet {
    if matches!(game.g_ram[0x615], 0 | 1) {
        loop {
            let result = WaitForNMI_NoUpdate_Async(game);
            if result != 0 {
                game.g_ram[0x615] = 1;
                return result;
            }
            HandleMusicQueue(game);
            HandleSoundEffects(game);
            MsgBoxMakeHdmaTable(game);
            let radius =
                u16::from_le_bytes([game.g_ram[0x5a2], game.g_ram[0x5a3]]).wrapping_sub(0x200);
            game.g_ram[0x5a2..0x5a4].copy_from_slice(&radius.to_le_bytes());
            if radius & 0x8000 != 0 {
                break;
            }
        }
    }
    game.g_ram[0x615] = 0;
    0
}
pub unsafe fn DisplayMessageBox_Async(a: uint16, game: &mut Game) -> CoroutineRet {
    enum Phase {
        Open,
        Interact,
        Close,
        SaveDelay,
        SaveWait,
        OpenSaved,
        InteractSaved,
        CloseSaved,
        Restore,
    }
    let mut phase = match game.g_ram[0x614] {
        0 => {
            game.g_ram[0x1c1f..0x1c21].copy_from_slice(&a.to_le_bytes());
            CancelSoundEffects(game);
            InitializePpuForMessageBoxes(game);
            ClearMessageBoxBg3Tilemap(game);
            InitializeMessageBox(game);
            Phase::Open
        }
        2 => Phase::Open,
        3 => Phase::Interact,
        4 => Phase::Close,
        8 => Phase::SaveWait,
        5 => Phase::OpenSaved,
        6 => Phase::InteractSaved,
        7 => Phase::CloseSaved,
        _ => {
            game.g_ram[0x614] = 0;
            return 0;
        }
    };
    loop {
        match phase {
            Phase::Open | Phase::OpenSaved => {
                let saved = matches!(phase, Phase::OpenSaved);
                let result = OpenMessageBox_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 5 } else { 2 };
                    return result;
                }
                phase = if saved {
                    Phase::InteractSaved
                } else {
                    Phase::Interact
                };
            }
            Phase::Interact | Phase::InteractSaved => {
                let saved = matches!(phase, Phase::InteractSaved);
                let result = HandleMessageBoxInteraction_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 6 } else { 3 };
                    return result;
                }
                phase = if saved {
                    Phase::CloseSaved
                } else {
                    Phase::Close
                };
            }
            Phase::Close | Phase::CloseSaved => {
                let saved = matches!(phase, Phase::CloseSaved);
                let result = CloseMessageBox_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = if saved { 7 } else { 4 };
                    return result;
                }
                if saved {
                    game.g_ram[0x1c1f..0x1c21].copy_from_slice(&28u16.to_le_bytes());
                    phase = Phase::Restore;
                } else {
                    let message = u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]);
                    if message == 28
                        && u16::from_le_bytes([game.g_ram[0x5f9], game.g_ram[0x5fa]]) != 2
                    {
                        game.g_ram[0x1c1f..0x1c21].copy_from_slice(&24u16.to_le_bytes());
                        ClearMessageBoxBg3Tilemap(game);
                        QueueSfx1_Max6(0x2e, game);
                        game.g_ram[0x77e..0x780].copy_from_slice(&160u16.to_le_bytes());
                        phase = Phase::SaveDelay;
                    } else {
                        phase = Phase::Restore;
                    }
                }
            }
            Phase::SaveDelay => {
                HandleMusicQueue(game);
                HandleSoundEffects(game);
                phase = Phase::SaveWait;
            }
            Phase::SaveWait => {
                let result = WaitForNMI_NoUpdate_Async(game);
                if result != 0 {
                    game.g_ram[0x614] = 8;
                    return result;
                }
                let timer =
                    u16::from_le_bytes([game.g_ram[0x77e], game.g_ram[0x77f]]).wrapping_sub(1);
                game.g_ram[0x77e..0x780].copy_from_slice(&timer.to_le_bytes());
                if timer == 0 {
                    InitializeMessageBox(game);
                    phase = Phase::OpenSaved;
                } else {
                    phase = Phase::SaveDelay;
                }
            }
            Phase::Restore => {
                ClearMessageBoxBg3Tilemap(game);
                RestorePpuForMessageBox(game);
                QueueSamusMovementSfx(game);
                if u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) == 20 {
                    game.g_ram[0x998..0x99a]
                        .copy_from_slice(&(kGameState_12_Pausing as u16).to_le_bytes());
                }
                break;
            }
        }
    }
    game.g_ram[0x614] = 0;
    0
}
pub fn DisplayMessageBox_Poll(a: uint16, game: &mut Game) -> ::core::ffi::c_int {
    if a == u16::from_le_bytes([game.g_ram[0x1c1f], game.g_ram[0x1c20]]) {
        game.g_ram[0x1c1f..0x1c21].fill(0);
        return u16::from_le_bytes([game.g_ram[0x5f9], game.g_ram[0x5fa]]) as ::core::ffi::c_int;
    }
    game.g_ram[0xdc8..0xdca].copy_from_slice(&a.to_le_bytes());
    -1
}
pub fn DisplayMessageBox(a: uint16, game: &mut Game) {
    game.g_ram[0xdc8..0xdca].copy_from_slice(&a.to_le_bytes());
}
