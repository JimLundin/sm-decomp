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
unsafe fn RomPtr_85(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x850000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
pub const fnWriteLargeMessageBoxTilemap: uint32 = 8749658;
pub const fnWriteSmallMessageBoxTilemap: uint32 = 8749705;
pub const fnDrawShootButtonAndSetupPpuForLargeMessageBox: uint32 = 8750021;
pub const fnDrawRunButtonAndSetupPpuForLargeMessageBox: uint32 = 8750028;
pub const fnSetupPpuForSmallMessageBox: uint32 = 8750134;
pub const fnSetupPpuForLargeMessageBox: uint32 = 8750145;

static kMsgBoxSpecialButtonTilemapOffs: [uint16; 27] = [
    0 as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0x12c as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x120 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0x12a as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
    0 as ::core::ffi::c_int as uint16,
];
static kTileNumbersForButtonLetters: [uint16; 8] = [
    0x28e0 as ::core::ffi::c_int as uint16,
    0x3ce1 as ::core::ffi::c_int as uint16,
    0x2cf7 as ::core::ffi::c_int as uint16,
    0x38f8 as ::core::ffi::c_int as uint16,
    0x38d0 as ::core::ffi::c_int as uint16,
    0x38eb as ::core::ffi::c_int as uint16,
    0x38f1 as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
];
unsafe fn InitializePpuForMessageBoxes(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16) = 0 as uint16;
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb1 as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0xb as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0x1f as uint8, game);
    WriteReg(CGDATA as ::core::ffi::c_int as uint16, 0 as uint8, game);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .msgbox
        .backup_of_enabled_hdma_channels = *(&raw mut game.g_ram as *mut uint8).offset(0x85);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .misc
        .field_3E8[3 as ::core::ffi::c_int as usize] =
        *(&raw mut game.g_ram as *mut uint8).offset(0x5b);
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) = 88 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x6a) = 23 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x70) = 0 as uint8;
    *(&raw mut game.g_ram as *mut uint8).offset(0x73) = 0 as uint8;
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x20 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
    WriteReg(COLDATA as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    ReadReg(BG3HOFS as ::core::ffi::c_int as uint16, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3HOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    ReadReg(BG3VOFS as ::core::ffi::c_int as uint16, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(BG3VOFS as ::core::ffi::c_int as uint16, 0 as uint8, game);
    let mut i: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap as *mut uint16 as *mut uint8)
            .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
        game,
    );
    ReadRegWord(RDVRAML as ::core::ffi::c_int as uint16, game);
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x3981 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn ClearMessageBoxBg3Tilemap(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 1790 as int16;
    loop {
        *((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800))
            .cinematic_bg_tilemap as *mut uint16 as *mut uint8)
            .offset(v0 as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 0xe as uint16;
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as int16;
        if !(v0 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int) {
            break;
        }
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0x3800) as *mut Ram3800 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
static kLargeMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x280f as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
static kSmallMsgBoxTopBottomBorderTilemap: [uint16; 32] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0x284e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
unsafe fn WriteMessageTilemap(game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16) = 112 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a4) as *mut uint16) = 124 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x5a2) as *mut uint16) = 0 as uint16;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 112 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[i as usize] = 0 as uint16;
        i += 1;
    }
    let mut r0: uint16 = (*(game.g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f) as *mut uint16)
                as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as isize,
        ))
    .message_tilemap as uint16;
    let mut r9: uint16 = ((*(game.g_rom.offset(
        (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut MsgBoxConfig)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f) as *mut uint16) as isize))
    .message_tilemap as ::core::ffi::c_int
        - r0 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = r9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    r9 = (r9 as ::core::ffi::c_int + 128 as ::core::ffi::c_int) as uint16;
    game.message_box_das0l_value = r9;
    let mut v1: uint16 = 32 as uint16;
    let mut v2: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            *((RomPtr_85 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r0 as uint16_t, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16);
        v1 = v1.wrapping_add(1);
        v2 = (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    return v1;
}
unsafe fn WriteLargeMessageBoxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kLargeMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap(game);
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(i_0 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kLargeMsgBoxTopBottomBorderTilemap
                [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i_0 = i_0.wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
unsafe fn WriteSmallMessageBoxTilemap(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 32 as ::core::ffi::c_int {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap[(i + 256 as ::core::ffi::c_int) as usize] =
            kSmallMsgBoxTopBottomBorderTilemap[i as usize];
        i += 1;
    }
    let mut i_0: uint16 = WriteMessageTilemap(game);
    let mut n: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    let mut v1: uint16 = 0 as uint16;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(i_0 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kSmallMsgBoxTopBottomBorderTilemap
                [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        i_0 = i_0.wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
}
unsafe fn CallMsgBoxDraw(mut ea: uint32, game: &mut Game) {
    match ea {
        8749658 => {
            WriteLargeMessageBoxTilemap(game);
            return;
        }
        8749705 => {
            WriteSmallMessageBoxTilemap(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
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
    WriteReg(DMAP6 as ::core::ffi::c_int as uint16, 0x42 as uint8, game);
    WriteReg(BBAD6 as ::core::ffi::c_int as uint16, 0x12 as uint8, game);
    WriteReg(A1T6L as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(DAS6L as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(A1T6H as ::core::ffi::c_int as uint16, 0x33 as uint8, game);
    WriteReg(DAS6H as ::core::ffi::c_int as uint16, 0x33 as uint8, game);
    WriteReg(A1B6 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(DAS60 as ::core::ffi::c_int as uint16, 0x7e as uint8, game);
    WriteReg(A2A6L as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(A2A6H as ::core::ffi::c_int as uint16, 0 as uint8, game);
    WriteReg(NTRL6 as ::core::ffi::c_int as uint16, 0 as uint8, game);
    MsgBoxMakeHdmaTable(game);
    WriteReg(HDMAEN as ::core::ffi::c_int as uint16, 0x40 as uint8, game);
}
unsafe fn SetupPpuForActiveMessageBox(mut r52: uint16, game: &mut Game) {
    SetupMessageBoxBg3YscrollHdma(game);
    r52 = (r52 as ::core::ffi::c_int + 22528 as ::core::ffi::c_int) as uint16;
    WriteRegWord(VMADDL as ::core::ffi::c_int as uint16, r52, game);
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(
        DAS1L as ::core::ffi::c_int as uint16,
        game.message_box_das0l_value,
        game,
    );
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn DrawSpecialButtonAndSetupPpuForLargeMessageBox(mut a: uint16, game: &mut Game) {
    let mut v1: uint16 = 0 as uint16;
    if a as ::core::ffi::c_int & kButton_A as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        v1 = 2 as uint16;
        if a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            v1 = 4 as uint16;
            if a as ::core::ffi::c_int & kButton_X as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                v1 = 6 as uint16;
                if a as ::core::ffi::c_int & kButton_Y as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    v1 = 8 as uint16;
                    if a as ::core::ffi::c_int & kButton_Select as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        v1 = 10 as uint16;
                        if a as ::core::ffi::c_int & kButton_L as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            v1 = 12 as uint16;
                            if a as ::core::ffi::c_int & kButton_R as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                v1 = 14 as uint16;
                            }
                        }
                    }
                }
            }
        }
    }
    *(((&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
        .pause_menu_map_tilemap as *mut uint16)
        .offset(256) as *mut uint16 as *mut uint8)
        .offset(
            kMsgBoxSpecialButtonTilemapOffs[(*((&raw mut game.g_ram as *mut uint8).offset(0x1c1f)
                as *mut uint16) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int as isize,
        ) as *mut uint16) = kTileNumbersForButtonLetters
        [(v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    SetupPpuForActiveMessageBox(416 as uint16, game);
}
unsafe fn DrawShootButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16),
        game,
    );
}
unsafe fn DrawRunButtonAndSetupPpuForLargeMessageBox(game: &mut Game) {
    DrawSpecialButtonAndSetupPpuForLargeMessageBox(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9b6) as *mut uint16),
        game,
    );
}
unsafe fn SetupPpuForSmallMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(448 as uint16, game);
}
unsafe fn SetupPpuForLargeMessageBox(game: &mut Game) {
    SetupPpuForActiveMessageBox(416 as uint16, game);
}
unsafe fn CallMsgBoxModify(mut ea: uint32, game: &mut Game) {
    match ea {
        8750021 => {
            DrawShootButtonAndSetupPpuForLargeMessageBox(game);
            return;
        }
        8750028 => {
            DrawRunButtonAndSetupPpuForLargeMessageBox(game);
            return;
        }
        8750134 => {
            SetupPpuForSmallMessageBox(game);
            return;
        }
        8750145 => {
            SetupPpuForLargeMessageBox(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn InitializeMessageBox(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1c1f)
        as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    CallMsgBoxDraw(
        ((*(game.g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .draw_initial_tilemap as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
        game,
    );
    CallMsgBoxModify(
        ((*(game.g_rom.offset(
            (((0x85869b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x85869b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut MsgBoxConfig)
            .offset(v0 as isize))
        .modify_box_func as ::core::ffi::c_int
            | 0x850000 as ::core::ffi::c_int) as uint32,
        game,
    );
}
static kSaveConfirmationSelectionTilemap: [uint16; 96] = [
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3cf8 as ::core::ffi::c_int as uint16,
    0x3ce4 as ::core::ffi::c_int as uint16,
    0x3cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2ced as ::core::ffi::c_int as uint16,
    0x2cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3cf8 as ::core::ffi::c_int as uint16,
    0x3ce4 as ::core::ffi::c_int as uint16,
    0x3cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2ced as ::core::ffi::c_int as uint16,
    0x2cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x2cf8 as ::core::ffi::c_int as uint16,
    0x2ce4 as ::core::ffi::c_int as uint16,
    0x2cf2 as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x38cc as ::core::ffi::c_int as uint16,
    0x38cd as ::core::ffi::c_int as uint16,
    0x3ced as ::core::ffi::c_int as uint16,
    0x3cee as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0x3c4e as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
    0xe as ::core::ffi::c_int as uint16,
];
unsafe fn ToggleSaveConfirmationSelection(game: &mut Game) {
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16);
    *fresh7 = (*fresh7 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as uint16;
    let mut v0: uint16 = 64 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5f9) as *mut uint16) as ::core::ffi::c_int
        == 2 as ::core::ffi::c_int
    {
        v0 = 128 as uint16;
    }
    let mut v1: uint16 = 128 as uint16;
    let mut r52: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
    loop {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .pause_menu_map_tilemap
            [(v1 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as usize] =
            kSaveConfirmationSelectionTilemap
                [(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
        v1 = v1.wrapping_add(1);
        v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
        r52 -= 1;
        if !(r52 != 0) {
            break;
        }
    }
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_6059A0 as ::core::ffi::c_int as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 512 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x180 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
}
unsafe fn RestorePpuForMessageBox(game: &mut Game) {
    WriteRegWord(
        VMADDL as ::core::ffi::c_int as uint16,
        addr_unk_605880 as ::core::ffi::c_int as uint16,
        game,
    );
    WriteRegWord(
        DMAP1 as ::core::ffi::c_int as uint16,
        0x1801 as uint16,
        game,
    );
    WriteRegWord(
        A1T1L as ::core::ffi::c_int as uint16,
        (((&raw mut game.g_ram as *mut uint8).offset(0x4000) as *mut Ram4000 as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 256 as ::core::ffi::c_long) as uint16,
        game,
    );
    WriteRegWord(A1B1 as ::core::ffi::c_int as uint16, 0x7e as uint16, game);
    WriteRegWord(DAS1L as ::core::ffi::c_int as uint16, 0x700 as uint16, game);
    WriteRegWord(DAS10 as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteRegWord(A2A1H as ::core::ffi::c_int as uint16, 0 as uint16, game);
    WriteReg(VMAIN as ::core::ffi::c_int as uint16, 0x80 as uint8, game);
    WriteReg(MDMAEN as ::core::ffi::c_int as uint16, 2 as uint8, game);
    *(&raw mut game.g_ram as *mut uint8).offset(0x85) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels;
    WriteReg(
        HDMAEN as ::core::ffi::c_int as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .msgbox
            .backup_of_enabled_hdma_channels,
        game,
    );
    *(&raw mut game.g_ram as *mut uint8).offset(0x5b) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x3000) as *mut Ram3000))
            .misc
            .field_3E8[3 as ::core::ffi::c_int as usize];
    *(&raw mut game.g_ram as *mut uint8).offset(0x6a) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x69);
    *(&raw mut game.g_ram as *mut uint8).offset(0x70) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x6e);
    *(&raw mut game.g_ram as *mut uint8).offset(0x73) =
        *(&raw mut game.g_ram as *mut uint8).offset(0x71);
    WriteReg(CGADD as ::core::ffi::c_int as uint16, 0x19 as uint8, game);
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(25) as uint8,
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(25)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(26) as uint8,
        game,
    );
    WriteReg(
        CGDATA as ::core::ffi::c_int as uint16,
        *(((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(26)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ),
        game,
    );
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
