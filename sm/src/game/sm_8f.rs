use crate::c_library::strcpy;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, NextRandom, QueueSfx2_Max6, SetEventHappened,
};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::{
    FxTypeFunc_20, FxTypeFunc_2C_Haze, RoomCode_ScrollingSkyLand, RoomMainAsm_ScrollingSkyOcean,
    RoomSetupAsm_ScrollingSkyOcean,
};
use crate::game::sm_89::RoomCode_CeresElevatorShaft;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_94::Samus_MoveDown_NoSolidColl;
use crate::game::sm_a6::HandleCeresRidleyGetawayCutscene;
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::{mov24, Mult8x8, RomPtr, Unreachable, WriteReg};
pub use crate::snes::registers::*;
pub use crate::types::EnemyData;
pub use crate::types::{CoroutineRet, LongPtr, SnesRegs, VoidP};
use ::core::ffi::c_char;
use ::core::ptr::null_mut;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RoomDefStateSelect_E6E5_Finish {
    pub code_ptr: VoidP,
}
pub use crate::types::Consts_87;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_86, DoorDef, RoomDefRoomstate};
crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
}
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
#[inline]
unsafe fn get_DoorDef(a: u16, game: &mut Game) -> *mut DoorDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: u16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_RoomDefStateSelect_E6E5_Finish(
    a: u16,
    game: &mut Game,
) -> *mut RoomDefStateSelect_E6E5_Finish {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefStateSelect_E6E5_Finish
}
crate::game::constants::typed_constants! { i32;
    fnPauseHook_DraygonRoom = 0x8fc8f6, fnUnpauseHook_DraygonRoom = 0x8fc8fc,
}
pub unsafe fn RoomSetup_AfterSavingAnimals(game: &mut Game) {
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x3d, 0xb, 0xbb30 as VoidP), game);
    word!(game.earthquake_type = 24);
    word!(game.earthquake_timer = -1i32 as u16);
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RoomSetup_AutoDestroyWallAfterEscape => SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10, 0x87, 0xb964 as VoidP),
        game,
    );
    RoomSetup_TurnWallIntoShotblocks => SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xf, 0xa, 0xb9ed as VoidP),
        game,
    );
}
pub unsafe fn RoomSetup_ShakeDuringEscape(game: &mut Game) {
    word!(game.earthquake_type = 6);
    word!(game.earthquake_timer = -1i32 as u16);
    FxTypeFunc_20(game);
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RoomSetup_ScrollingSkyLand => FxTypeFunc_20(game);
    RoomSetup_ScrollingSkyOcean => RoomSetupAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomSetup_RunStatueUnlockAnims(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Kraid as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Phantoon as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Draygon as u16, game);
    SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Ridley as u16, game);
}
pub unsafe fn DoorCode_StartWreckedShipTreadmillWest(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillRight as u16, game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x4, 0x9, 0xb64b as VoidP), game);
}
// Each door callback writes its listed scroll cells in order: 0 red, 1 blue, 2 green.
macro_rules! door_scroll_changes {
    ($game:ident; $($name:ident { $($cell:literal => $color:literal),+ $(,)? };)+) => {
        $(pub unsafe fn $name($game: &mut Game) {
            $(*ram_ptr!($game; 0xcd20, $cell) = $color;)+
        })+
    };
}

door_scroll_changes! { game;
    DoorCode_Scroll6_Green { 6 => 2 };
    DoorCode_Scroll0_Blue { 0 => 1 };
    DoorCode_Scroll13_Blue { 19 => 1 };
    DoorCode_Scroll_4Blue_8Green { 4 => 0, 8 => 2 };
    DoorCode_Scroll_8toB_Red { 8 => 0, 9 => 0, 10 => 0, 11 => 0 };
    DoorCode_Scroll_LotsRed { 2 => 0, 3 => 0, 4 => 0, 5 => 0, 11 => 0, 12 => 0, 13 => 0, 17 => 0 };
    DoorCode_Scroll_1_4_Green { 1 => 2, 4 => 2 };
    DoorCode_Scroll_2_Blue { 2 => 1 };
    DoorCode_Scroll_17_Blue { 23 => 1 };
    DoorCode_Scroll_4_Blue { 4 => 1 };
}
pub use self::DoorCode_Scroll6_Green as DoorCode_Scroll_6_Green;
door_scroll_changes! { game;
    DoorCode_Scroll_3_Green { 3 => 2 };
    DoorCode_SetScroll_0 { 24 => 2, 28 => 2 };
    DoorCode_SetScroll_1 { 5 => 1, 6 => 1 };
    DoorCode_SetScroll_2 { 29 => 1 };
    DoorCode_SetScroll_3 { 2 => 2, 3 => 2 };
    DoorCode_SetScroll_4 { 0 => 0, 1 => 2 };
    DoorCode_SetScroll_5 { 11 => 2 };
    DoorCode_SetScroll_6 { 28 => 0, 29 => 1 };
    DoorCode_SetScroll_7 { 4 => 0 };
    DoorCode_SetScroll_8 { 32 => 2, 36 => 2, 37 => 2 };
}
pub use self::DoorCode_Scroll_2_Blue as DoorCode_SetScroll_9;
door_scroll_changes! { game;
    DoorCode_SetScroll_10 { 0 => 2 };
    DoorCode_SetScroll_11 { 6 => 2, 7 => 2 };
    DoorCode_SetScroll_12 { 1 => 1, 2 => 0 };
    DoorCode_SetScroll_13 { 1 => 1, 3 => 0 };
    DoorCode_SetScroll_14 { 0 => 0, 4 => 1 };
    DoorCode_SetScroll_15 { 2 => 1, 3 => 1 };
    DoorCode_SetScroll_16 { 0 => 2, 1 => 2 };
    DoorCode_SetScroll_17 { 1 => 2 };
    DoorCode_SetScroll_18 { 15 => 2, 18 => 2 };
}
pub use self::DoorCode_Scroll6_Green as DoorCode_SetScroll_19;
door_scroll_changes! { game;
    DoorCode_SetScroll_20 { 0 => 2, 1 => 1 };
    DoorCode_SetScroll_21 { 2 => 2 };
    DoorCode_SetScroll_22 { 3 => 0, 4 => 0, 6 => 1, 7 => 1, 8 => 1 };
    DoorCode_SetScroll_23 { 1 => 1, 2 => 1, 3 => 1, 4 => 2, 6 => 0 };
    DoorCode_SetScroll_24 { 0 => 1, 1 => 1 };
    DoorCode_SetScroll_25 { 1 => 0, 0 => 1 };
    DoorCode_SetScroll_26 { 10 => 2 };
    DoorCode_SetScroll_28 { 0 => 2, 2 => 2 };
    DoorCode_SetScroll_29 { 6 => 1, 7 => 1, 8 => 0 };
    DoorCode_SetScroll_30 { 2 => 0, 3 => 1 };
    DoorCode_SetScroll_31 { 7 => 2 };
    DoorCode_SetScroll_32 { 1 => 0, 2 => 1 };
    DoorCode_SetScroll_33 { 3 => 0, 0 => 1 };
    DoorCode_SetScroll_34 { 1 => 1, 4 => 0 };
    DoorCode_SetScroll_35 { 0 => 1, 1 => 0, 2 => 0, 3 => 0 };
}
pub use self::DoorCode_SetScroll_10 as DoorCode_SetScroll_36;
door_scroll_changes! { game;
    DoorCode_SetScroll_37 { 0 => 1, 1 => 1, 4 => 0 };
    DoorCode_SetScroll_38 { 0 => 1, 3 => 0 };
}
pub use self::DoorCode_Scroll0_Blue as DoorCode_SetScroll_39;
door_scroll_changes! { game;
    DoorCode_SetScroll_40 { 0 => 1, 1 => 0 };
    DoorCode_SetScroll_41 { 24 => 1 };
    DoorCode_SetScroll_42 { 3 => 0, 2 => 1 };
    DoorCode_SetScroll_43 { 14 => 0 };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RoomCode_ScrollingSkyLand_ => RoomCode_ScrollingSkyLand(game);
    RoomCode_ScrollingSkyOcean_ => RoomMainAsm_ScrollingSkyOcean(game);
}
pub unsafe fn RoomCode_GenRandomExplodes_Nonblank(game: &mut Game) {
    if word!(game.time_is_frozen_flag) == 0 && (word!(game.nmi_frame_counter_word) & 1) == 0 {
        let mut Random: u16 = NextRandom(game);
        let x: u16 = (word!(game.layer1_x_pos)).wrapping_add((Random as u8) as u16);
        let y: u16 = (word!(game.layer1_y_pos)).wrapping_add((byte!(Random; 1)) as u16);
        let prod: u16 = Mult8x8((y >> 4) as u8, word!(game.room_width_in_blocks) as u8);
        if (*(ram_ptr!(game, u16; 0x10002)).offset((prod as i32 + (x as i32 >> 4)) as isize)
            & 0x3ff)
            != 255
        {
            RoomCode_GenRandomExplodes(x, y, game);
        }
    }
}
pub unsafe fn RoomCode_ScrollingSkyLand_Shakes(game: &mut Game) {
    RoomCode_ScrollingSkyLand(game);
    RoomCode_GenRandomExplodes_Nonblank(game);
    word!(game.earthquake_timer, |fresh17| (fresh17) | (0x8000u16));
}
pub unsafe fn RoomCode_ExplodeShakes(game: &mut Game) {
    RoomCode_GenRandomExplodes_Nonblank(game);
    word!(game.earthquake_timer, |fresh16| (fresh16) | (0x8000u16));
}
pub unsafe fn RoomCode_GenRandomExplodes_4th(game: &mut Game) {
    if word!(game.time_is_frozen_flag) == 0 && (word!(game.nmi_frame_counter_word) & 3) == 0 {
        let mut Random: u16 = NextRandom(game);
        RoomCode_GenRandomExplodes(
            (word!(game.layer1_x_pos)).wrapping_add((Random as u8) as u16),
            (word!(game.layer1_y_pos)).wrapping_add((byte!(Random; 1)) as u16),
            game,
        );
    }
}
pub unsafe fn RoomCode_GenRandomExplodes(x_r18: u16, y_r20: u16, game: &mut Game) {
    static kRoomCode_GenRandomExplodes_Sprite: [u8; 8] = [3, 3, 9, 12, 12, 18, 18, 21];
    static kRoomCode_GenRandomExplodes_Sfx: [u8; 8] = [0x24, 0, 0, 0x25, 0, 0, 0, 0];
    let v1: u16 = (NextRandom(game)) & (0xfu16);
    if v1 < 8 {
        let v2: u16 = kRoomCode_GenRandomExplodes_Sfx[v1 as usize] as u16;
        if v2 != 0 {
            QueueSfx2_Max6(v2, game);
        }
    }
    CreateSpriteAtPos(
        x_r18,
        y_r20,
        kRoomCode_GenRandomExplodes_Sprite[(v1 as i32 & 7) as usize] as u16,
        0,
        game,
    );
}
pub unsafe fn RoomCode_ScrollRightDachora(game: &mut Game) {
    if *ram_ptr!(game; 0xcd20, 11) == 2
        && word!(game.layer1_y_pos) < 0x500
        && word!(game.layer1_x_pos) < 0x380
    {
        word!(game.layer1_x_pos, |fresh15| (fresh15 as i32
            + ((word!(game.layer1_x_pos) >= 0x380) as i32 + 3))
            as u16);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RoomCode_8FC8C8 => SpawnEprojWithRoomGfx(addr_kEproj_PrePhantomRoom as u16, 0, game);
    RoomCode_SetupShaktoolRoomPlm =>
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0, 0, 0xb8eb as VoidP), game);
}
pub unsafe fn RoomCode_SetPauseCodeForDraygon(game: &mut Game) {
    mov24(
        ram_ptr!(game, LongPtr; 0x601),
        fnPauseHook_DraygonRoom as u32,
    );
    mov24(
        ram_ptr!(game, LongPtr; 0x604),
        fnUnpauseHook_DraygonRoom as u32,
    );
}
pub unsafe fn PauseHook_DraygonRoom(game: &mut Game) {
    word!(game.irqhandler_next_handler = 4);
}
pub unsafe fn UnpauseHook_DraygonRoom(game: &mut Game) -> CoroutineRet {
    if word!(game.hdma_object_channels_bitmask[1]) == 8 {
        word!(game.irqhandler_next_handler = 12);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn RoomCode_SetCollectedMap(game: &mut Game) {
    let v0: u16 = (*(ram_ptr!(game, u16; 0xd908, word!(game.area_index) as isize))) | (1u16);
    *(ram_ptr!(game, u16; 0xd908, word!(game.area_index) as isize)) = v0;
    word!(game.has_area_map = v0);
}
pub unsafe fn RoomCode_SetZebesTimebombEvent(game: &mut Game) {
    SetEventHappened(0xe, game);
    word!(game.earthquake_type = 18);
    word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetLightHorizRoomShake(game: &mut Game) {
    word!(game.earthquake_type = 18);
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 18;
    *(ram_ptr!(game, u16; 0x7e1)) = 0;
    word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetMediumHorizRoomShake(game: &mut Game) {
    word!(game.earthquake_type = 21);
    word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_Escape4_SetMediumHorizRoomShake(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10, 0x10, 0xb968 as VoidP),
        game,
    );
    word!(game.earthquake_type = 21);
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 21;
    *(ram_ptr!(game, u16; 0x7e1)) = 0;
    word!(game.earthquake_timer = -1i32 as u16);
}
pub unsafe fn RoomCode_SetCeresDoorSolid(game: &mut Game) {
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0xf, 0x26, 0xba48 as VoidP), game);
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma(game: &mut Game) {
    FxTypeFunc_2C_Haze(game);
}
pub unsafe fn RoomCode_CeresColorMathHdma_BgBase(game: &mut Game) {
    *ram_ptr!(game; 0x5d) = 102;
    FxTypeFunc_2C_Haze(game);
    word!(game.hdma_data_table_in_ceres = 9);
}
pub unsafe fn DoorCode_StartWreckedSkipTreadmill_East(game: &mut Game) {
    SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillLeft as u16, game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x4, 0x9, 0xb64f as VoidP), game);
}
door_scroll_changes! { game;
    DoorCode_SetScroll_44 { 1 => 1 };
}
pub use self::DoorCode_SetScroll_10 as DoorCode_SetScroll_45;
door_scroll_changes! { game;
    DoorCode_SetScroll_46 { 3 => 0, 4 => 1 };
    DoorCode_SetScroll_47 { 41 => 1 };
    DoorCode_SetScroll_48 { 40 => 2, 46 => 2 };
    DoorCode_SetScroll_49 { 6 => 0, 7 => 0, 8 => 0, 9 => 0, 10 => 0, 11 => 0 };
}
pub unsafe fn DoorCode_SetupElevatubeFromSouth(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x7e1, 4)) = -256i32 as u16;
    *(ram_ptr!(game, u16; 0x7e1, 2)) = 2496;
    *(ram_ptr!(game, u16; 0x7e1, 6)) = -32i32 as u16;
    CallSomeSamusCode(0, game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x1, 0, 0xb8f9 as VoidP), game);
}
pub unsafe fn DoorCode_SetupElevatubeFromNorth(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x7e1, 4)) = 256;
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 2),
        b"@\0" as *const u8 as *const c_char,
    );
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 6),
        b" \0" as *const u8 as *const c_char,
    );
    CallSomeSamusCode(0, game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x1, 0, 0xb8f9 as VoidP), game);
}
pub unsafe fn RoomCode_Elevatube(game: &mut Game) {
    word!(game.samus_x_pos = 128);
    word!(game.samus_x_subpos = 0);
    let amt: i32 = (*(ram_ptr!(game, u16; 0x7e1, 4)) as i16 as i32) << 8;
    let mut v0: u16 = ((amt as u32).wrapping_add(*(ram_ptr!(game, u32; 0x7e1))) >> 16) as u16;
    crate::update_place! { reference fresh13 = *(ram_ptr!(game, u16; 0x7e1)) => (*fresh13 as i32 + amt) as u16; }
    *(ram_ptr!(game, u16; 0x7e1, 2)) = v0;
    Samus_MoveDown_NoSolidColl(amt, game);
    if ((*(ram_ptr!(game, u16; 0x7e1, 6)) as i32 + *(ram_ptr!(game, u16; 0x7e1, 4)) as i32 + 3616)
        as u16)
        < 0x1c41
    {
        crate::update_place! { reference fresh14 = *(ram_ptr!(game, u16; 0x7e1, 4)) => (*fresh14).wrapping_add(*(ram_ptr!(game, u16; 0x7e1, 6))); }
    }
}
pub unsafe fn DoorCode_ResetElevatubeNorthExit(game: &mut Game) {
    CallSomeSamusCode(1, game);
}
pub unsafe fn DoorCode_ResetElevatubeSouthExit(game: &mut Game) {
    *(ram_ptr!(game, u16; 0xcd20)) = 514;
    CallSomeSamusCode(1, game);
}
door_scroll_changes! { game;
    DoorCode_SetScroll_50 { 10 => 0, 11 => 1 };
}
pub use self::DoorCode_SetScroll_14 as DoorCode_SetScroll_53;
door_scroll_changes! { game;
    DoorCode_SetScroll_54 { 0 => 0, 1 => 1 };
    DoorCode_SetScroll_55 { 9 => 0, 10 => 1 };
    DoorCode_SetScroll_56 { 0 => 0, 2 => 0, 1 => 1 };
}
pub use self::DoorCode_SetScroll_44 as DoorCode_SetScroll_57;
door_scroll_changes! { game;
    DoorCode_SetScroll_58 { 6 => 1 };
}
pub use self::DoorCode_SetScroll_7 as DoorCode_SetScroll_59;
door_scroll_changes! { game;
    DoorCode_SetScroll_61 { 4 => 0, 7 => 0 };
    DoorCode_SetScroll_62 { 2 => 0, 1 => 1 };
}
pub use self::DoorCode_SetScroll_16 as DoorCode_SetScroll_64;
pub use self::DoorCode_SetScroll_28 as DoorCode_SetScroll_63;
door_scroll_changes! { game;
    DoorCode_SetScroll_65 { 24 => 1, 25 => 0 };
}
pub unsafe fn DoorCode_CeresElevatorShaft(game: &mut Game) {
    crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x56, true, game);
    word! { game, 256; reg_M7A, reg_M7D }
    word! { game, 0; reg_M7B, reg_M7C }
    word!(game.reg_M7X = 128);
    word!(game.reg_M7Y = 1008);
    word!(game.irq_enable_mode7 = 1);
    strcpy(
        ram_ptr!(game, c_char; 0x7e1),
        b"\"\0" as *const u8 as *const c_char,
    );
    strcpy(
        ram_ptr!(game, c_char; 0x7e1, 2),
        b"<\0" as *const u8 as *const c_char,
    );
}
pub unsafe fn DoorCode_CeresElevatorShaft_2(game: &mut Game) {
    *ram_ptr!(game; 0x56) = 9;
    word!(game.irq_enable_mode7 = 0);
}
pub unsafe fn RoomCode_CeresElevatorShaft_(game: &mut Game) {
    RoomCode_CeresElevatorShaft(game);
}
static kRoomCode_SpawnCeresFallingDebris_Tab: [u16; 16] = [
    0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110, 0x130, 0x150, 0x170,
    0x190,
];
pub unsafe fn RoomCode_SpawnCeresFallingDebris(game: &mut Game) {
    if word!(game.ceres_status) != 0
        && {
            crate::update_place! { reference fresh12 = *(ram_ptr!(game, u16; 0x7e1)) => (*fresh12).wrapping_sub(1); }
            *(ram_ptr!(game, i16; 0x7e1)) < 0
        }
    {
        *ram_ptr!(game; 0x7e1, 0) = 8;
        *ram_ptr!(game; 0x7e1, 1) = 0;
        let v0: u16 = (if word!(game.random_number) & 0x8000 != 0 {
            addr_stru_869742 as i32
        } else {
            addr_stru_869734 as i32
        }) as u16;
        SpawnEprojWithRoomGfx(
            v0,
            kRoomCode_SpawnCeresFallingDebris_Tab
                [(word!(game.random_number) as i32 & 0xf) as usize],
            game,
        );
    }
}
pub unsafe fn RoomCode_HandleCeresRidleyGetaway(game: &mut Game) {
    if word!(game.ceres_status) & 1 != 0 {
        HandleCeresRidleyGetawayCutscene(game);
    }
}

macro_rules! select_room_earthquake_update {
    (
        $update_earthquake:ident, $game:ident, $timer_reference:ident, $earthquake_kind:ident,
        $ending_kind:literal, $random_threshold:literal, $starting_kind:literal $(,)?
    ) => {
        let $update_earthquake: bool = if *(ram_ptr!($game, u16; 0x7e1)) != 0 {
            crate::update_place! {
                reference $timer_reference = *(ram_ptr!($game, u16; 0x7e1))
                    => (*$timer_reference).wrapping_sub(1);
            }
            (*$timer_reference == 0)
                && ({
                    $earthquake_kind = $ending_kind;
                    true
                })
        } else {
            (NextRandom($game) < $random_threshold)
                && ({
                    strcpy(
                        ram_ptr!($game, c_char; 0x7e1),
                        b"*\0" as *const u8 as *const c_char,
                    );
                    $earthquake_kind = $starting_kind;
                    true
                })
        };
    };
}

pub unsafe fn RoomCode_ShakeScreenHorizDiag(game: &mut Game) {
    let mut v0: u16 = 0;

    select_room_earthquake_update!(update_earthquake, game, fresh11, v0, 18, 0x200, 23);
    if update_earthquake {
        word!(game.earthquake_type = v0);
    }
    RoomCode_GenRandomExplodes_4th(game);
}
pub unsafe fn RoomCode_ShakeScreenHorizDiagStrong(game: &mut Game) {
    let mut v0: i16 = 0;

    select_room_earthquake_update!(update_earthquake, game, fresh10, v0, 21, 0x180, 26);
    if update_earthquake {
        *(ram_ptr!(game, u16; 0x7e1, 2)) = v0 as u16;
    }
    word!(game.earthquake_type = *(ram_ptr!(game, u16; 0x7e1, 2)));
    RoomCode_GenRandomExplodes_4th(game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoomDefStateSelect(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
        call (k, game); default Unreachable(game) as u16;
    {
        9430502 => RoomDefStateSelect_Finish; 9430527 => RoomDefStateSelect_TourianBoss01;
        9430546 => RoomDefStateSelect_IsEventSet; 9430569 => RoomDefStateSelect_IsBossDead;
        9430610 => RoomDefStateSelect_MorphBallMissiles; 9430633 => RoomDefStateSelect_PowerBombs;
    }
}
pub unsafe fn HandleRoomDefStateSelect(k: u16, game: &mut Game) {
    let mut v1: u16 = (k).wrapping_add(11);
    loop {
        let event_pointer: u16 = (*get_RoomDefStateSelect_E6E5_Finish(v1, game)).code_ptr;
        v1 = CallRoomDefStateSelect(
            ((event_pointer) as u32) | (0x8f0000u32),
            (v1).wrapping_add(2),
            game,
        );
        if (v1 == 0) {
            break;
        }
    }
}
pub unsafe fn RoomDefStateSelect_Finish(k: u16, game: &mut Game) -> u16 {
    word!(game.roomdefroomstate_ptr = k);
    0
}
pub unsafe fn RoomDefStateSelect_TourianBoss01(k: u16, game: &mut Game) -> u16 {
    if CheckBossBitForCurArea(1, game) & 1 == 0 {
        return (k).wrapping_add(2);
    }
    let v1 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_IsEventSet(k: u16, game: &mut Game) -> u16 {
    let v1 = RomPtr_8F(k, game);
    if CheckEventHappened(*v1 as u16, game) != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut u16), game)
    } else {
        (k).wrapping_add(3)
    }
}
pub unsafe fn RoomDefStateSelect_IsBossDead(k: u16, game: &mut Game) -> u16 {
    let v1 = RomPtr_8F(k, game);
    if CheckBossBitForCurArea(*v1 as u16, game) & 1 != 0 {
        RoomDefStateSelect_Finish(*(v1.offset(1) as *mut u16), game)
    } else {
        (k).wrapping_add(3)
    }
}
pub unsafe fn RoomDefStateSelect_MorphBallMissiles(k: u16, game: &mut Game) -> u16 {
    if (word!(game.collected_items) & 4) == 0 || word!(game.samus_max_missiles) == 0 {
        return (k).wrapping_add(2);
    }
    let v1 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
pub unsafe fn RoomDefStateSelect_PowerBombs(k: u16, game: &mut Game) -> u16 {
    if word!(game.samus_max_power_bombs) == 0 {
        return (k).wrapping_add(2);
    }
    let v1 = RomPtr_8F(k, game) as *const u16;
    RoomDefStateSelect_Finish(*v1, game)
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoomSetupCode(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9408916 => RoomSetup_AfterSavingAnimals; 9408937 => RoomSetup_AutoDestroyWallAfterEscape;
        9408946 => RoomSetup_TurnWallIntoShotblocks; 9408957 => RoomSetup_ShakeDuringEscape;
        9408969 => RoomSetup_ScrollingSkyLand; 9408974 => RoomSetup_ScrollingSkyOcean;
        9408983 => RoomSetup_RunStatueUnlockAnims; 9423048 => RoomCode_8FC8C8;
        9423059 => RoomCode_SetupShaktoolRoomPlm; 9423069 => RoomCode_SetPauseCodeForDraygon;
        9423114 => RoomCode_SetCollectedMap;
        9408955 | 9408956 | 9408979 | 9408980 | 9408981 | 9408982 | 9409012 | 9409013 | 9409014
                | 9409015 | 9423047 | 9423056 | 9423057 | 9423058 | 9423068 | 9423134 => [()];
        9423135 => RoomCode_SetZebesTimebombEvent; 9423155 => RoomCode_SetLightHorizRoomShake;
        9423174 => RoomCode_SetMediumHorizRoomShake;
        9423187 => RoomCode_Escape4_SetMediumHorizRoomShake; 9423214 => RoomCode_SetCeresDoorSolid;
        9423222 => RoomCode_CeresColorMathHdma; 9423227 => RoomCode_CeresColorMathHdma_BgBase;
    }
}
pub unsafe fn RunRoomSetupCode(game: &mut Game) {
    let mut RoomDefRoomstate = get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game);
    if (*RoomDefRoomstate).room_setup_code != 0 {
        CallRoomSetupCode(
            (((*RoomDefRoomstate).room_setup_code) as u32) | (0x8f0000u32),
            game,
        );
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallDoorDefSetupCode(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9419121 => DoorCode_StartWreckedShipTreadmillWest; 9419137 => DoorCode_Scroll6_Green;
        9419148 => DoorCode_Scroll0_Blue; 9419159 => DoorCode_Scroll13_Blue;
        9419170 => DoorCode_Scroll_4Blue_8Green; 9419187 => DoorCode_Scroll_8toB_Red;
        9419210 => DoorCode_Scroll_LotsRed; 9419249 => DoorCode_Scroll_1_4_Green;
        9419264 => DoorCode_Scroll_2_Blue; 9419275 => DoorCode_Scroll_17_Blue;
        9419286 => DoorCode_Scroll_4_Blue; 9419297 => DoorCode_Scroll_6_Green;
        9419308 => DoorCode_Scroll_3_Green; 9420039 => DoorCode_SetScroll_0;
        9420054 => DoorCode_SetScroll_1; 9420069 => DoorCode_SetScroll_2;
        9420080 => DoorCode_SetScroll_3; 9420095 => DoorCode_SetScroll_4;
        9420112 => DoorCode_SetScroll_5; 9420123 => DoorCode_SetScroll_6;
        9420140 => DoorCode_SetScroll_7; 9420151 => DoorCode_SetScroll_8;
        9420170 => DoorCode_SetScroll_9; 9420181 => DoorCode_SetScroll_10;
        9420192 => DoorCode_SetScroll_11; 9420207 => DoorCode_SetScroll_12;
        9420224 => DoorCode_SetScroll_13; 9420241 => DoorCode_SetScroll_14;
        9420258 => DoorCode_SetScroll_15; 9420273 => DoorCode_SetScroll_16;
        9420288 => DoorCode_SetScroll_17; 9420299 => DoorCode_SetScroll_18;
        9420314 => DoorCode_SetScroll_19; 9420325 => DoorCode_SetScroll_20;
        9420342 => DoorCode_SetScroll_21; 9420702 => DoorCode_SetScroll_22;
        9420731 => DoorCode_SetScroll_23; 9420762 => DoorCode_SetScroll_24;
        9420777 => DoorCode_SetScroll_25; 9420794 => DoorCode_SetScroll_26;
        9420822 => DoorCode_SetScroll_28; 9420837 => DoorCode_SetScroll_29;
        9420858 => DoorCode_SetScroll_30; 9420875 => DoorCode_SetScroll_31;
        9420886 => DoorCode_SetScroll_32; 9420903 => DoorCode_SetScroll_33;
        9420920 => DoorCode_SetScroll_34; 9420937 => DoorCode_SetScroll_35;
        9420962 => DoorCode_SetScroll_36; 9420973 => DoorCode_SetScroll_37;
        9420994 => DoorCode_SetScroll_38; 9421011 => DoorCode_SetScroll_39;
        9421022 => DoorCode_SetScroll_40; 9421039 => DoorCode_SetScroll_41;
        9421050 => DoorCode_SetScroll_42; 9421067 => DoorCode_SetScroll_43;
        9429464 => DoorCode_StartWreckedSkipTreadmill_East; 9429480 => DoorCode_SetScroll_44;
        9429491 => DoorCode_SetScroll_45; 9429502 => DoorCode_SetScroll_46;
        9429519 => DoorCode_SetScroll_47; 9429530 => DoorCode_SetScroll_48;
        9429545 => DoorCode_SetScroll_49; 9429612 => DoorCode_SetupElevatubeFromSouth;
        9429649 => DoorCode_SetupElevatubeFromNorth; 9429761 => DoorCode_ResetElevatubeNorthExit;
        9429769 => DoorCode_ResetElevatubeSouthExit; 9429784 => DoorCode_SetScroll_50;
        9429829 => DoorCode_SetScroll_53; 9429846 => DoorCode_SetScroll_54;
        9429863 => DoorCode_SetScroll_55; 9429880 => DoorCode_SetScroll_56;
        9429901 => DoorCode_SetScroll_57; 9429912 => DoorCode_SetScroll_58;
        9429923 => DoorCode_SetScroll_59; 9429945 => DoorCode_SetScroll_61;
        9429960 => DoorCode_SetScroll_62; 9429977 => DoorCode_SetScroll_63;
        9430208 => DoorCode_SetScroll_64; 9430223 => DoorCode_SetScroll_65;
        9430240 => DoorCode_CeresElevatorShaft; 9430291 => DoorCode_CeresElevatorShaft_2;
    }
}
pub unsafe fn RunDoorSetupCode(game: &mut Game) {
    let DD = get_DoorDef(word!(game.door_def_ptr), game);
    if (*DD).door_setup_code != 0 {
        CallDoorDefSetupCode((((*DD).door_setup_code) as u32) | (0x8f0000u32), game);
    }
}
pub unsafe fn RoomCode_CrocomireShaking(game: &mut Game) {
    let mut ai_var_D: u16 = 0;
    if ((*(ram_ptr!(game, EnemyData; 0xf78))).properties & 0x200) == 0 {
        if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_A == 64 {
            word!(
                game.reg_BG1VOFS = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                    + word!(game.bg1_y_offset) as i32
                    + word!(game.layer1_y_pos) as i32) as u16
            );
        } else if ((*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_B & 0x400) != 0 {
            let fresh5 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
            fresh5.write_unaligned(fresh5.read_unaligned().wrapping_sub(1));
            if fresh5.read_unaligned() as i32 + 7 & 0x8000 != 0 {
                ai_var_D = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                    - 2i32 * ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32 + 7))
                    as u16;
            } else {
                ai_var_D = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
            }
            word!(game.reg_BG1VOFS, wrapping_add(ai_var_D));
            word!(game.reg_BG2VOFS = (ai_var_D).wrapping_sub(48));
        } else if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_C == 34
            && (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D != 0
        {
            let fresh7 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D;
            fresh7.write_unaligned(fresh7.read_unaligned().wrapping_sub(1));
            if ((*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D & 1) != 0 {
                word!(game.layer1_x_pos, wrapping_sub(4));
            } else {
                word!(game.layer1_x_pos, wrapping_add(4));
            }
        }
    }
}
static kRoomCode_RidleyRoomShaking_X: [i16; 8] = [0, 2, 2, 2, 0, -2, -2, -2];
static kRoomCode_RidleyRoomShaking_Y: [i16; 8] = [-2, -2, 0, 2, 2, 2, 0, -2];
pub unsafe fn RoomCode_RidleyRoomShaking(game: &mut Game) {
    let ai_var_A: u16 = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
    if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A != 0 {
        let fresh0 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
        fresh0.write_unaligned(fresh0.read_unaligned().wrapping_sub(1));
        let v1: i32 = ai_var_A as i32 - 1i32;
        word!(
            game.reg_BG1HOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16)
        );
        word!(
            game.reg_BG2HOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16)
        );
        word!(
            game.reg_BG1VOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16)
        );
        word!(
            game.reg_BG2VOFS,
            wrapping_add((kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16)
        );
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallRoomCode(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9421078 => RoomCode_ScrollingSkyLand_; 9421083 => RoomCode_ScrollingSkyOcean_;
        9421088 => RoomCode_ScrollingSkyLand_Shakes; 9421092 => RoomCode_ExplodeShakes;
        9421286 => RoomCode_ScrollRightDachora; 9429686 => RoomCode_Elevatube;
        9430303 => RoomCode_CeresElevatorShaft_; 9430308 => [()];
        9430309 => RoomCode_SpawnCeresFallingDebris; 9430385 => RoomCode_HandleCeresRidleyGetaway;
        9430396 => RoomCode_ShakeScreenHorizDiag; 9430432 => RoomCode_GenRandomExplodes_4th;
        9430436 => RoomCode_ShakeScreenHorizDiagStrong; 9431245 => RoomCode_CrocomireShaking;
        9431376 => RoomCode_RidleyRoomShaking;
    }
}
pub unsafe fn RunRoomMainCode(game: &mut Game) {
    if word!(game.room_main_code_ptr) != 0 {
        CallRoomCode(
            ((word!(game.room_main_code_ptr)) as u32) | (0x8f0000u32),
            game,
        );
    }
}
