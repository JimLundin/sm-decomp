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

pub use crate::types::Func_V;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FxDef {
    pub door_ptr: u16,
    pub base_y_pos: u16,
    pub target_y_pos: u16,
    pub y_vel: u16,
    pub timer: u8,
    pub type_0: u8,
    pub default_layer_blend: u8,
    pub layer3_layer_blend: u8,
    pub fx_liquid_options_: u8,
    pub palette_fx_bitset: u8,
    pub animtiles_bitset: u8,
    pub palette_blend: u8,
}
pub use crate::types::GameState;
#[inline]
unsafe fn RomPtr_83(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0x830000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn get_FxDef(a: u16, game: &mut Game) -> *mut FxDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut FxDef
}
pub unsafe fn LoadFxEntry(v0: u16, game: &mut Game) {
    let FD: *mut FxDef = get_FxDef(
        (ram_word!(game, room_layer3_asm_ptr) as i32 + 16i32 * (v0 as i32 & 7i32)) as u16,
        game,
    );
    set_ram_word!(game.fx_base_y_pos = (*FD).base_y_pos);
    set_ram_word!(game.fx_target_y_pos = (*FD).target_y_pos);
    set_ram_word!(game.fx_y_vel = (*FD).y_vel);
    set_ram_word!(game.fx_timer = (*FD).timer as u16);
    set_ram_word!(game.fx_layer_blending_config_a = (*FD).default_layer_blend as u16);
    set_ram_word!(game.fx_layer_blending_config_b = (*FD).layer3_layer_blend as u16);
    set_ram_word!(game.fx_liquid_options = (*FD).fx_liquid_options_ as u16);
    if (*FD).palette_blend != 0 {
        let v2: i32 = (*FD).palette_blend as i32 >> 1i32;
        set_ram_word!(
            game.palette_buffer[25] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset(v2 as isize)
        );
        set_ram_word!(
            game.palette_buffer[26] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset((v2 + 1i32) as isize)
        );
        set_ram_word!(
            game.palette_buffer[27] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset((v2 + 2i32) as isize)
        );
    } else {
        set_ram_word!(game.palette_buffer[27] = 0u16);
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
    let mut entry_ptr: u16 = ram_word!(game, room_layer3_asm_ptr);
    if entry_ptr == 0u16 {
        return;
    }
    loop {
        let door_ptr: u16 = (*get_FxDef(entry_ptr, game)).door_ptr;
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
    set_ram_word!(game.fx_timer = (*FD).timer as u16);
    set_ram_word!(game.fx_layer_blending_config_a = (*FD).default_layer_blend as u16);
    set_ram_word!(game.fx_layer_blending_config_b = (*FD).layer3_layer_blend as u16);
    set_ram_word!(game.fx_liquid_options = (*FD).fx_liquid_options_ as u16);
    if (*FD).palette_blend != 0 {
        let v4: i32 = (*FD).palette_blend as i32 >> 1i32;
        set_ram_word!(
            game.target_palettes[25] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset(v4 as isize)
        );
        set_ram_word!(
            game.target_palettes[26] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset((v4 + 1i32) as isize)
        );
        set_ram_word!(
            game.target_palettes[27] = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89aa02))
                as *const u8 as *mut u16)
                .offset((v4 + 2i32) as isize)
        );
    } else {
        set_ram_word!(game.target_palettes[27] = 0u16);
    }
    set_ram_word!(game.fx_type = (*FD).type_0 as u16);
    if (*FD).type_0 != 0 {
        set_ram_word!(
            game.fx_tilemap_ptr = *(game.g_rom.offset(crate::rtl::lorom_offset(0x83abf0))
                as *const u8 as *mut u16)
                .offset(((*FD).type_0 as i32 >> 1i32) as isize)
        );
        kFxTypeFuncPtrs[((*FD).type_0 as i32 >> 1i32) as usize].expect("non-null function pointer")(
            game,
        );
    }
    if (*FD).palette_fx_bitset != 0 {
        let mut bits: i32 = (*FD).palette_fx_bitset as i32;
        let v11: *const u8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac46)) as *const u8 as *mut u16)
                .offset(ram_word!(game, area_index) as isize),
            game,
        );
        let mut j: i32 = 0i32;
        while j != 16i32 {
            if bits & 1i32 != 0 {
                SpawnPalfxObject(*(v11.offset(j as isize) as *const u8 as *mut u16), game);
            }
            j += 2i32;
            bits >>= 1i32;
        }
    }
    if (*FD).animtiles_bitset != 0 {
        let mut bits_0: i32 = (*FD).animtiles_bitset as i32;
        let v14: *const u8 = RomPtr_83(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x83ac56)) as *const u8 as *mut u16)
                .offset(ram_word!(game, area_index) as isize),
            game,
        );
        let mut k: i32 = 0i32;
        while k != 16i32 {
            if bits_0 & 1i32 != 0 {
                SpawnAnimtiles(*(v14.offset(k as isize) as *const u8 as *mut u16), game);
            }
            k += 2i32;
            bits_0 >>= 1i32;
        }
    }
}
pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    let mut v1: i16 = 0;
    if (ram_word!(game, ceres_status) as i32 & 0x8000i32) != 0i32 {
        if (112i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16
            && (144i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16
            && ram_word!(game, samus_y_pos) as i32 - 128i32 & 0x8000i32 != 0
            && ram_word!(game, samus_y_pos) as i32 - 75i32 & 0x8000i32 == 0
            && ram_word!(game, samus_y_speed) == 0
            && ram_word!(game, samus_y_subspeed) == 0
            && ram_word!(game, game_state) as i32 == kGameState_8_MainGameplay as i32
        {
            CallSomeSamusCode(2u16, game);
            set_ram_word!(game.screen_fade_delay = 0u16);
            set_ram_word!(game.screen_fade_counter = 0u16);
            set_ram_word!(game.game_state = 32u16);
        }
        let ref mut fresh0 =
            *((&raw mut game.g_ram as *mut u8).offset(0x7e1).offset(2) as *mut u8 as *mut u16);
        *fresh0 = (*fresh0).wrapping_sub(1);
        if *((&raw mut game.g_ram as *mut u8).offset(0x7e1).offset(2) as *mut u8 as *mut i16) < 0i16
        {
            let v0: i32 = ((6u16)
                .wrapping_mul(*((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16)))
                as i32
                >> 1i32;
            *((&raw mut game.g_ram as *mut u8).offset(0x7e1).offset(2) as *mut u8 as *mut u16) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f)) as *const u8 as *mut u16)
                    .offset(v0 as isize);
            set_ram_word!(
                game.reg_M7B = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f)) as *const u8
                    as *mut u16)
                    .offset((v0 + 1i32) as isize)
            );
            set_ram_word!(game.reg_M7C = -(ram_word!(game, reg_M7B) as i32) as u16);
            set_ram_word!(
                game.reg_M7A = *(game.g_rom.offset(crate::rtl::lorom_offset(0x89ad5f)) as *const u8
                    as *mut u16)
                    .offset((v0 + 2i32) as isize)
            );
            set_ram_word!(game.reg_M7D = ram_word!(game, reg_M7A));
            if *((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut i16) < 0i16 {
                v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16)) as i16)
                    .wrapping_sub(1i16);
                if *((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16) == 0x8001u16 {
                    v1 = 0i16;
                }
            } else {
                v1 = ((*((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16)) as i16)
                    .wrapping_add(1i16);
                if *((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16) == 67u16 {
                    v1 = -(32700i32) as i16;
                }
            }
            *((&raw mut game.g_ram as *mut u8).offset(0x7e1) as *mut u16) = v1 as u16;
        }
    }
}
