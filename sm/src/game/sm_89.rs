pub use crate::game::constants::game_states::*;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::{
    FxTypeFunc_20, FxTypeFunc_22_ScrollingSky, FxTypeFunc_24, FxTypeFunc_26_TourianEntranceStatue,
    FxTypeFunc_28_CeresRidley, FxTypeFunc_2C_Haze, FxTypeFunc_2_Lava, FxTypeFunc_4_Acid,
    FxTypeFunc_6_Water, FxTypeFunc_8_Spores, FxTypeFunc_A_Rain, FxTypeFunc_C,
    FxTypeFunc_CeresElevator,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game_state::Game;
use crate::rtl::RomPtr;

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
#[inline]
unsafe fn RomPtr_83(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr((0x830000u32) | ((addr) as u32), game);
}
#[inline]
unsafe fn get_FxDef(mut a: uint16, game: &mut Game) -> *mut FxDef {
    return RomPtr((0x830000u32) | ((a) as u32), game) as *mut FxDef;
}
pub unsafe fn LoadFxEntry(mut v0: uint16, game: &mut Game) {
    let mut FD: *mut FxDef = get_FxDef(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7cd) as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int * (v0 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
            as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = (*FD).base_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = (*FD).target_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = (*FD).y_vel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = (*FD).timer as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) =
        (*FD).default_layer_blend as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16) =
        (*FD).layer3_layer_blend as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16) =
        (*FD).fx_liquid_options_ as uint16;
    if (*FD).palette_blend != 0 {
        let mut v2: ::core::ffi::c_int =
            (*FD).palette_blend as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(25) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(26) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(27) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(27) =
            0 as uint16;
    };
}
pub fn nullsub_106(game: &mut Game) {}
static kFxTypeFuncPtrs: [Option<Func_V>; 23] = unsafe {
    [
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_2_Lava as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_4_Acid as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_6_Water as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_8_Spores as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_A_Rain as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_C as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_106 as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_20 as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_22_ScrollingSky as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_24 as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_26_TourianEntranceStatue as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_28_CeresRidley as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_CeresElevator as unsafe fn(&mut Game) -> ()),
        Some(FxTypeFunc_2C_Haze as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn LoadFXHeader(game: &mut Game) {
    let mut entry_ptr: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x7cd) as *mut uint16);
    if entry_ptr == 0u16 {
        return;
    }
    loop {
        let mut door_ptr: uint16 = (*get_FxDef(entry_ptr, game)).door_ptr;
        if door_ptr == 0 {
            break;
        }
        if door_ptr == 0xffffu16 {
            return;
        }
        if door_ptr == *((&raw mut game.g_ram as *mut uint8).offset(0x78d) as *mut uint16) {
            break;
        }
        entry_ptr = (entry_ptr).wrapping_add(16u16);
    }
    let mut FD: *mut FxDef = get_FxDef(entry_ptr, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = (*FD).base_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = (*FD).target_y_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = (*FD).y_vel;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = (*FD).timer as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1982) as *mut uint16) =
        (*FD).default_layer_blend as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1984) as *mut uint16) =
        (*FD).layer3_layer_blend as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197e) as *mut uint16) =
        (*FD).fx_liquid_options_ as uint16;
    if (*FD).palette_blend != 0 {
        let mut v4: ::core::ffi::c_int =
            (*FD).palette_blend as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(25) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset(v4 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(26) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(27) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02)) as *const uint8 as *mut uint16)
                .offset((v4 + 2 as ::core::ffi::c_int) as isize);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(27) =
            0 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16) = (*FD).type_0 as uint16;
    if (*FD).type_0 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1964) as *mut uint16) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83abf0)) as *const uint8 as *mut uint16)
                .offset(((*FD).type_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        kFxTypeFuncPtrs[((*FD).type_0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(game);
    }
    if (*FD).palette_fx_bitset != 0 {
        let mut bits: ::core::ffi::c_int = (*FD).palette_fx_bitset as ::core::ffi::c_int;
        let mut v11: *const uint8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac46)) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
                ),
            game,
        );
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j != 16i32 {
            if bits & 1 as ::core::ffi::c_int != 0 {
                SpawnPalfxObject(
                    *(v11.offset(j as isize) as *const uint8 as *mut uint16),
                    game,
                );
            }
            j += 2 as ::core::ffi::c_int;
            bits >>= 1 as ::core::ffi::c_int;
        }
    }
    if (*FD).animtiles_bitset != 0 {
        let mut bits_0: ::core::ffi::c_int = (*FD).animtiles_bitset as ::core::ffi::c_int;
        let mut v14: *const uint8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac56)) as *const uint8 as *mut uint16)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
                ),
            game,
        );
        let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while k != 16i32 {
            if bits_0 & 1 as ::core::ffi::c_int != 0 {
                SpawnAnimtiles(
                    *(v14.offset(k as isize) as *const uint8 as *mut uint16),
                    game,
                );
            }
            k += 2 as ::core::ffi::c_int;
            bits_0 >>= 1 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    let mut v1: int16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        if (112i16).wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
        ) < 0i16
            && (144i16).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
            ) >= 0i16
            && *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 128 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 75 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16) == 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16)
                as ::core::ffi::c_int
                == kGameState_8_MainGameplay as ::core::ffi::c_int
        {
            CallSomeSamusCode(2 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 32 as uint16;
        }
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2)
            as *mut uint8 as *mut uint16);
        *fresh0 = (*fresh0).wrapping_sub(1);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
            as *mut int16)
            < 0i16
        {
            let mut v0: ::core::ffi::c_int = ((6u16)
                .wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16)))
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
                as *mut uint16) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f))
                as *const uint8 as *mut uint16)
                .offset(v0 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f)) as *const uint8
                    as *mut uint16)
                    .offset((v0 + 1 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) =
                -(*((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f)) as *const uint8
                    as *mut uint16)
                    .offset((v0 + 2 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut int16) < 0i16 {
                v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16)) as i16)
                    .wrapping_sub(1i16);
                if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) == 0x8001u16
                {
                    v1 = 0 as int16;
                }
            } else {
                v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16)) as i16)
                    .wrapping_add(1i16);
                if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) == 67u16 {
                    v1 = -(32700 as ::core::ffi::c_int) as int16;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) = v1 as uint16;
        }
    }
}
