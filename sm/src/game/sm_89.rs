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
use crate::ram_map::{ram_word, set_ram_word};
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
unsafe fn RomPtr_83(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x830000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn get_FxDef(a: uint16, game: &mut Game) -> *mut FxDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut FxDef
}
pub unsafe fn LoadFxEntry(v0: uint16, game: &mut Game) {
    let FD: *mut FxDef = get_FxDef(
        (ram_word!(game, room_layer3_asm_ptr) as i32 + 16 as i32 * (v0 as i32 & 7 as i32))
            as uint16,
        game,
    );
    set_ram_word!(game.fx_base_y_pos = (*FD).base_y_pos);
    set_ram_word!(game.fx_target_y_pos = (*FD).target_y_pos);
    set_ram_word!(game.fx_y_vel = (*FD).y_vel);
    set_ram_word!(game.fx_timer = (*FD).timer as uint16);
    set_ram_word!(game.fx_layer_blending_config_a = (*FD).default_layer_blend as uint16);
    set_ram_word!(game.fx_layer_blending_config_b = (*FD).layer3_layer_blend as uint16);
    set_ram_word!(game.fx_liquid_options = (*FD).fx_liquid_options_ as uint16);
    if (*FD).palette_blend != 0 {
        let v2: i32 = (*FD).palette_blend as i32 >> 1 as i32;
        set_ram_word!(
            game.palette_buffer[25] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset(v2 as isize)
        );
        set_ram_word!(
            game.palette_buffer[26] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset((v2 + 1 as i32) as isize)
        );
        set_ram_word!(
            game.palette_buffer[27] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset((v2 + 2 as i32) as isize)
        );
    } else {
        set_ram_word!(game.palette_buffer[27] = 0 as uint16);
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
    let mut entry_ptr: uint16 = ram_word!(game, room_layer3_asm_ptr);
    if entry_ptr == 0u16 {
        return;
    }
    loop {
        let door_ptr: uint16 = (*get_FxDef(entry_ptr, game)).door_ptr;
        if door_ptr == 0 {
            break;
        }
        if door_ptr == 0xffffu16 {
            return;
        }
        if door_ptr == ram_word!(game, door_def_ptr) {
            break;
        }
        entry_ptr = (entry_ptr).wrapping_add(16u16);
    }
    let FD: *mut FxDef = get_FxDef(entry_ptr, game);
    set_ram_word!(game.fx_base_y_pos = (*FD).base_y_pos);
    set_ram_word!(game.fx_target_y_pos = (*FD).target_y_pos);
    set_ram_word!(game.fx_y_vel = (*FD).y_vel);
    set_ram_word!(game.fx_timer = (*FD).timer as uint16);
    set_ram_word!(game.fx_layer_blending_config_a = (*FD).default_layer_blend as uint16);
    set_ram_word!(game.fx_layer_blending_config_b = (*FD).layer3_layer_blend as uint16);
    set_ram_word!(game.fx_liquid_options = (*FD).fx_liquid_options_ as uint16);
    if (*FD).palette_blend != 0 {
        let v4: i32 = (*FD).palette_blend as i32 >> 1 as i32;
        set_ram_word!(
            game.target_palettes[25] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset(v4 as isize)
        );
        set_ram_word!(
            game.target_palettes[26] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset((v4 + 1 as i32) as isize)
        );
        set_ram_word!(
            game.target_palettes[27] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const uint8 as *mut uint16)
                .offset((v4 + 2 as i32) as isize)
        );
    } else {
        set_ram_word!(game.target_palettes[27] = 0 as uint16);
    }
    set_ram_word!(game.fx_type = (*FD).type_0 as uint16);
    if (*FD).type_0 != 0 {
        set_ram_word!(
            game.fx_tilemap_ptr = *(game.g_rom.offset(crate::rtl::lorom_offset(0x83abf0))
                as *const uint8 as *mut uint16)
                .offset(((*FD).type_0 as i32 >> 1 as i32) as isize)
        );
        kFxTypeFuncPtrs[((*FD).type_0 as i32 >> 1 as i32) as usize]
            .expect("non-null function pointer")(game);
    }
    if (*FD).palette_fx_bitset != 0 {
        let mut bits: i32 = (*FD).palette_fx_bitset as i32;
        let v11: *const uint8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac46)) as *const uint8 as *mut uint16)
                .offset(ram_word!(game, area_index) as isize),
            game,
        );
        let mut j: i32 = 0 as i32;
        while j != 16i32 {
            if bits & 1 as i32 != 0 {
                SpawnPalfxObject(
                    *(v11.offset(j as isize) as *const uint8 as *mut uint16),
                    game,
                );
            }
            j += 2 as i32;
            bits >>= 1 as i32;
        }
    }
    if (*FD).animtiles_bitset != 0 {
        let mut bits_0: i32 = (*FD).animtiles_bitset as i32;
        let v14: *const uint8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac56)) as *const uint8 as *mut uint16)
                .offset(ram_word!(game, area_index) as isize),
            game,
        );
        let mut k: i32 = 0 as i32;
        while k != 16i32 {
            if bits_0 & 1 as i32 != 0 {
                SpawnAnimtiles(
                    *(v14.offset(k as isize) as *const uint8 as *mut uint16),
                    game,
                );
            }
            k += 2 as i32;
            bits_0 >>= 1 as i32;
        }
    }
}
pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    let mut v1: int16 = 0;
    if (ram_word!(game, ceres_status) as i32 & 0x8000 as i32) != 0i32 {
        if (112i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16
            && (144i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16
            && ram_word!(game, samus_y_pos) as i32 - 128 as i32 & 0x8000 as i32 != 0
            && ram_word!(game, samus_y_pos) as i32 - 75 as i32 & 0x8000 as i32 == 0
            && ram_word!(game, samus_y_speed) == 0
            && ram_word!(game, samus_y_subspeed) == 0
            && ram_word!(game, game_state) as i32 == kGameState_8_MainGameplay as i32
        {
            CallSomeSamusCode(2 as uint16, game);
            set_ram_word!(game.screen_fade_delay = 0 as uint16);
            set_ram_word!(game.screen_fade_counter = 0 as uint16);
            set_ram_word!(game.game_state = 32 as uint16);
        }
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2)
            as *mut uint8 as *mut uint16);
        *fresh0 = (*fresh0).wrapping_sub(1);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
            as *mut int16)
            < 0i16
        {
            let v0: i32 = ((6u16)
                .wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16)))
                as i32
                >> 1 as i32;
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e1).offset(2) as *mut uint8
                as *mut uint16) = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f))
                as *const uint8 as *mut uint16)
                .offset(v0 as isize);
            set_ram_word!(
                game.reg_M7B = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f))
                    as *const uint8 as *mut uint16)
                    .offset((v0 + 1 as i32) as isize)
            );
            set_ram_word!(game.reg_M7C = -(ram_word!(game, reg_M7B) as i32) as uint16);
            set_ram_word!(
                game.reg_M7A = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f))
                    as *const uint8 as *mut uint16)
                    .offset((v0 + 2 as i32) as isize)
            );
            set_ram_word!(game.reg_M7D = ram_word!(game, reg_M7A));
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
                    v1 = -(32700 as i32) as int16;
                }
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x7e1) as *mut uint16) = v1 as uint16;
        }
    }
}
