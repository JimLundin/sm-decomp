use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::{
    FxTypeFunc_20, FxTypeFunc_22_ScrollingSky, FxTypeFunc_24, FxTypeFunc_26_TourianEntranceStatue,
    FxTypeFunc_28_CeresRidley, FxTypeFunc_2C_Haze, FxTypeFunc_2_Lava, FxTypeFunc_4_Acid,
    FxTypeFunc_6_Water, FxTypeFunc_8_Spores, FxTypeFunc_A_Rain, FxTypeFunc_C,
    FxTypeFunc_CeresElevator,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::rtl::{g_ram, g_rom, RomPtr};

pub use crate::types::{
    __int16_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, uint16, uint16_t, uint32_t,
    uint8, uint8_t, Func_V,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FxDef {
    pub door_ptr: uint16,
    pub base_y_pos: uint16,
    pub target_y_pos: uint16,
    pub y_vel: uint16,
    pub timer: uint8,
    pub type_0: uint8,
    pub default_layer_blend: uint8,
    pub layer3_layer_blend: uint8,
    pub fx_liquid_options_: uint8,
    pub palette_fx_bitset: uint8,
    pub animtiles_bitset: uint8,
    pub palette_blend: uint8,
}
pub use crate::types::GameState;
pub const kGameState_44_TransitionFromDemo: GameState = 44;
pub const kGameState_43_TransitionFromDemo: GameState = 43;
pub const kGameState_42_PlayingDemo: GameState = 42;
pub const kGameState_41_TransitionToDemo: GameState = 41;
pub const kGameState_40_TransitionToDemo: GameState = 40;
pub const kGameState_39_EndingAndCredits: GameState = 39;
pub const kGameState_38_SamusEscapesFromZebes: GameState = 38;
pub const kGameState_37_CeresGoesBoomWithSamus: GameState = 37;
pub const kGameState_36_WhitingOutFromTimeUp: GameState = 36;
pub const kGameState_35_TimeUp: GameState = 35;
pub const kGameState_34_CeresGoesBoom: GameState = 34;
pub const kGameState_33_BlackoutFromCeres: GameState = 33;
pub const kGameState_32_MadeItToCeresElevator: GameState = 32;
pub const kGameState_31_SetUpNewGame: GameState = 31;
pub const kGameState_30_IntroCinematic: GameState = 30;
pub const kGameState_29_DebugGameOverMenu: GameState = 29;
pub const kGameState_28_Unused: GameState = 28;
pub const kGameState_27_ReserveTanksAuto: GameState = 27;
pub const kGameState_26_GameOverMenu: GameState = 26;
pub const kGameState_25_SamusNoHealth: GameState = 25;
pub const kGameState_24_SamusNoHealth: GameState = 24;
pub const kGameState_23_SamusNoHealth: GameState = 23;
pub const kGameState_22_SamusNoHealth: GameState = 22;
pub const kGameState_21_SamusNoHealth: GameState = 21;
pub const kGameState_20_SamusNoHealth: GameState = 20;
pub const kGameState_19_SamusNoHealth: GameState = 19;
pub const kGameState_18_Unpausing: GameState = 18;
pub const kGameState_17_Unpausing: GameState = 17;
pub const kGameState_16_Unpausing: GameState = 16;
pub const kGameState_15_Paused: GameState = 15;
pub const kGameState_14_Paused: GameState = 14;
pub const kGameState_13_Pausing: GameState = 13;
pub const kGameState_12_Pausing: GameState = 12;
pub const kGameState_11_LoadingNextRoom: GameState = 11;
pub const kGameState_10_LoadingNextRoom: GameState = 10;
pub const kGameState_9_HitDoorBlock: GameState = 9;
pub const kGameState_8_MainGameplay: GameState = 8;
pub const kGameState_7_MainGameplayFadeIn: GameState = 7;
pub const kGameState_6_LoadingGameData: GameState = 6;
pub const kGameState_5_FileSelectMap: GameState = 5;
pub const kGameState_4_FileSelectMenus: GameState = 4;
pub const kGameState_3_Unused: GameState = 3;
pub const kGameState_2_GameOptionsMenu: GameState = 2;
pub const kGameState_1_OpeningCinematic: GameState = 1;
pub const kGameState_0_Reset: GameState = 0;
#[inline]
unsafe extern "C" fn RomPtr_83(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x830000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn get_FxDef(mut a: uint16) -> *mut FxDef {
    return RomPtr((0x830000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut FxDef;
}
#[no_mangle]
pub unsafe extern "C" fn LoadFxEntry(mut v0: uint16) {
    let mut FD: *mut FxDef = get_FxDef(
        (*((&raw mut g_ram as *mut uint8).offset(0x7cd as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * (v0 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16,
    );
    *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).base_y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).target_y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).y_vel;
    *((&raw mut g_ram as *mut uint8).offset(0x1980 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).timer as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).default_layer_blend as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1984 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).layer3_layer_blend as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).fx_liquid_options_ as uint16;
    if (*FD).palette_blend != 0 {
        let mut v2: ::core::ffi::c_int =
            (*FD).palette_blend as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(25 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v2 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(26 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(27 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xc000 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(27 as ::core::ffi::c_int as isize) = 0 as uint16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn nullsub_106() {}
static mut kFxTypeFuncPtrs: [Option<Func_V>; 23] = unsafe {
    [
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_2_Lava as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_4_Acid as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_6_Water as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_8_Spores as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_A_Rain as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_C as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(nullsub_106 as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_20 as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_22_ScrollingSky as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_24 as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_26_TourianEntranceStatue as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_28_CeresRidley as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_CeresElevator as unsafe extern "C" fn() -> ()),
        Some(FxTypeFunc_2C_Haze as unsafe extern "C" fn() -> ()),
    ]
};
#[no_mangle]
pub unsafe extern "C" fn LoadFXHeader() {
    let mut entry_ptr: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0x7cd as ::core::ffi::c_int as isize)
        as *mut uint16);
    if entry_ptr as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return;
    }
    loop {
        let mut door_ptr: uint16 = (*get_FxDef(entry_ptr)).door_ptr;
        if door_ptr == 0 {
            break;
        }
        if door_ptr as ::core::ffi::c_int == 0xffff as ::core::ffi::c_int {
            return;
        }
        if door_ptr as ::core::ffi::c_int
            == *((&raw mut g_ram as *mut uint8).offset(0x78d as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
        {
            break;
        }
        entry_ptr = (entry_ptr as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as uint16;
    }
    let mut FD: *mut FxDef = get_FxDef(entry_ptr);
    *((&raw mut g_ram as *mut uint8).offset(0x1978 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).base_y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0x197a as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).target_y_pos;
    *((&raw mut g_ram as *mut uint8).offset(0x197c as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).y_vel;
    *((&raw mut g_ram as *mut uint8).offset(0x1980 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).timer as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1982 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).default_layer_blend as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x1984 as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).layer3_layer_blend as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0x197e as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).fx_liquid_options_ as uint16;
    if (*FD).palette_blend != 0 {
        let mut v4: ::core::ffi::c_int =
            (*FD).palette_blend as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(25 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset(v4 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(26 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(27 as ::core::ffi::c_int as isize) = *(g_rom.offset(
            (((0x89aa02 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x89aa02 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut uint16)
            .offset((v4 + 2 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xc200 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(27 as ::core::ffi::c_int as isize) = 0 as uint16;
    }
    *((&raw mut g_ram as *mut uint8).offset(0x196e as ::core::ffi::c_int as isize)
        as *mut uint16) = (*FD).type_0 as uint16;
    if (*FD).type_0 != 0 {
        *((&raw mut g_ram as *mut uint8).offset(0x1964 as ::core::ffi::c_int as isize)
            as *mut uint16) = *(g_rom.offset(
            (((0x83abf0 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x83abf0 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(((*FD).type_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        kFxTypeFuncPtrs[((*FD).type_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")();
    }
    if (*FD).palette_fx_bitset != 0 {
        let mut bits: ::core::ffi::c_int = (*FD).palette_fx_bitset as ::core::ffi::c_int;
        let mut v11: *const uint8 = RomPtr_83(
            *(g_rom.offset(
                (((0x83ac46 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x83ac46 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as uint16_t,
        );
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j != 16 as ::core::ffi::c_int {
            if bits & 1 as ::core::ffi::c_int != 0 {
                SpawnPalfxObject(*(v11.offset(j as isize) as *const uint8 as *mut uint16));
            }
            j += 2 as ::core::ffi::c_int;
            bits >>= 1 as ::core::ffi::c_int;
        }
    }
    if (*FD).animtiles_bitset != 0 {
        let mut bits_0: ::core::ffi::c_int = (*FD).animtiles_bitset as ::core::ffi::c_int;
        let mut v14: *const uint8 = RomPtr_83(
            *(g_rom.offset(
                (((0x83ac56 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x83ac56 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut g_ram as *mut uint8).offset(0x79f as ::core::ffi::c_int as isize)
                        as *mut uint16) as isize,
                ) as uint16_t,
        );
        let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while k != 16 as ::core::ffi::c_int {
            if bits_0 & 1 as ::core::ffi::c_int != 0 {
                SpawnAnimtiles(*(v14.offset(k as isize) as *const uint8 as *mut uint16));
            }
            k += 2 as ::core::ffi::c_int;
            bits_0 >>= 1 as ::core::ffi::c_int;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RoomCode_CeresElevatorShaft() {
    let mut v1: int16 = 0;
    if *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if ((112 as ::core::ffi::c_int
            - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
            && (144 as ::core::ffi::c_int
                - *((&raw mut g_ram as *mut uint8).offset(0xaf6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int) as int16
                as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            && *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            && *((&raw mut g_ram as *mut uint8).offset(0xafa as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                - 75 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            && *((&raw mut g_ram as *mut uint8).offset(0xb2e as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            && *((&raw mut g_ram as *mut uint8).offset(0xb2c as ::core::ffi::c_int as isize)
                as *mut uint16)
                == 0
            && *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                == kGameState_8_MainGameplay as ::core::ffi::c_int
        {
            CallSomeSamusCode(2 as uint16);
            *((&raw mut g_ram as *mut uint8).offset(0x723 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x725 as ::core::ffi::c_int as isize)
                as *mut uint16) = 0 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x998 as ::core::ffi::c_int as isize)
                as *mut uint16) = 32 as uint16;
        }
        let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
            as *mut uint16);
        *fresh0 = (*fresh0).wrapping_sub(1);
        if (*((&raw mut g_ram as *mut uint8)
            .offset(0x7e1 as ::core::ffi::c_int as isize)
            .offset(2 as ::core::ffi::c_int as isize) as *mut uint8 as *mut int16)
            as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            let mut v0: ::core::ffi::c_int = (6 as ::core::ffi::c_int
                * *((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int)
                as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut g_ram as *mut uint8)
                .offset(0x7e1 as ::core::ffi::c_int as isize)
                .offset(2 as ::core::ffi::c_int as isize) as *mut uint8
                as *mut uint16) = *(g_rom.offset(
                (((0x89ad5f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x89ad5f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize);
            *((&raw mut g_ram as *mut uint8).offset(0x7a as ::core::ffi::c_int as isize)
                as *mut uint16) = *(g_rom.offset(
                (((0x89ad5f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x89ad5f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 + 1 as ::core::ffi::c_int) as isize);
            *((&raw mut g_ram as *mut uint8).offset(0x7c as ::core::ffi::c_int as isize)
                as *mut uint16) = -(*((&raw mut g_ram as *mut uint8)
                .offset(0x7a as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int) as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x78 as ::core::ffi::c_int as isize)
                as *mut uint16) = *(g_rom.offset(
                (((0x89ad5f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x89ad5f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 + 2 as ::core::ffi::c_int) as isize);
            *((&raw mut g_ram as *mut uint8).offset(0x7e as ::core::ffi::c_int as isize)
                as *mut uint16) = *((&raw mut g_ram as *mut uint8)
                .offset(0x78 as ::core::ffi::c_int as isize)
                as *mut uint16);
            if (*((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut int16) as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                if *((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == 0x8001 as ::core::ffi::c_int
                {
                    v1 = 0 as int16;
                }
            } else {
                v1 = (*((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as int16;
                if *((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    == 67 as ::core::ffi::c_int
                {
                    v1 = -(32700 as ::core::ffi::c_int) as int16;
                }
            }
            *((&raw mut g_ram as *mut uint8).offset(0x7e1 as ::core::ffi::c_int as isize)
                as *mut uint16) = v1 as uint16;
        }
    }
}
