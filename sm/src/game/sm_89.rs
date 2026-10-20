use crate::game::callback_tables::callback_table;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::RomPtr;

pub use crate::types::Func_V;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct FxDef {
        pub door_ptr, base_y_pos, target_y_pos, y_vel: u16;
        pub timer, type_0, default_layer_blend, layer3_layer_blend, fx_liquid_options_,
            palette_fx_bitset, animtiles_bitset, palette_blend: u8;
    }
}
pub use crate::types::GameState;
const RomPtr_83: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x83>;
#[inline]
unsafe fn get_FxDef(a: u16, game: &mut Game) -> *mut FxDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut FxDef
}
pub unsafe fn LoadFxEntry(v0: u16, game: &mut Game) {
    let FD = get_FxDef(
        (ram_word!(game, room_layer3_asm_ptr) as i32 + 16 * (v0 as i32 & 7)) as u16,
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
        let v2: i32 = (*FD).palette_blend as i32 >> 1;
        set_ram_word!(
            game.palette_buffer[25] = crate::rtl::rom_table_word!(game, 0x89aa02, v2 as isize)
        );
        set_ram_word!(
            game.palette_buffer[26] =
                crate::rtl::rom_table_word!(game, 0x89aa02, (v2 + 1) as isize)
        );
        set_ram_word!(
            game.palette_buffer[27] =
                crate::rtl::rom_table_word!(game, 0x89aa02, (v2 + 2) as isize)
        );
    } else {
        set_ram_word!(game.palette_buffer[27] = 0);
    };
}
pub fn nullsub_106(game: &mut Game) {}
static kFxTypeFuncPtrs: [Option<Func_V>; 23] = callback_table! { unsafe Func_V;
    nullsub_106, FxTypeFunc_2_Lava, FxTypeFunc_4_Acid, FxTypeFunc_6_Water, FxTypeFunc_8_Spores,
    FxTypeFunc_A_Rain, FxTypeFunc_C, nullsub_106, nullsub_106, nullsub_106, nullsub_106,
    nullsub_106, nullsub_106, nullsub_106, nullsub_106, nullsub_106, FxTypeFunc_20,
    FxTypeFunc_22_ScrollingSky, FxTypeFunc_24, FxTypeFunc_26_TourianEntranceStatue,
    FxTypeFunc_28_CeresRidley, FxTypeFunc_CeresElevator, FxTypeFunc_2C_Haze,
};
pub unsafe fn LoadFXHeader(game: &mut Game) {
    let mut entry_ptr: u16 = ram_word!(game, room_layer3_asm_ptr);
    if entry_ptr == 0 {
        return;
    }
    loop {
        let door_ptr: u16 = (*get_FxDef(entry_ptr, game)).door_ptr;
        if door_ptr == 0 {
            break;
        }
        if door_ptr == 0xffff {
            return;
        }
        if door_ptr == ram_word!(game, door_def_ptr) {
            break;
        }
        entry_ptr = (entry_ptr).wrapping_add(16);
    }
    let FD = get_FxDef(entry_ptr, game);
    set_ram_word!(game.fx_base_y_pos = (*FD).base_y_pos);
    set_ram_word!(game.fx_target_y_pos = (*FD).target_y_pos);
    set_ram_word!(game.fx_y_vel = (*FD).y_vel);
    set_ram_word!(game.fx_timer = (*FD).timer as u16);
    set_ram_word!(game.fx_layer_blending_config_a = (*FD).default_layer_blend as u16);
    set_ram_word!(game.fx_layer_blending_config_b = (*FD).layer3_layer_blend as u16);
    set_ram_word!(game.fx_liquid_options = (*FD).fx_liquid_options_ as u16);
    if (*FD).palette_blend != 0 {
        let v4: i32 = (*FD).palette_blend as i32 >> 1;
        set_ram_word!(
            game.target_palettes[25] = crate::rtl::rom_table_word!(game, 0x89aa02, v4 as isize)
        );
        set_ram_word!(
            game.target_palettes[26] =
                crate::rtl::rom_table_word!(game, 0x89aa02, (v4 + 1) as isize)
        );
        set_ram_word!(
            game.target_palettes[27] =
                crate::rtl::rom_table_word!(game, 0x89aa02, (v4 + 2) as isize)
        );
    } else {
        set_ram_word!(game.target_palettes[27] = 0);
    }
    set_ram_word!(game.fx_type = (*FD).type_0 as u16);
    if (*FD).type_0 != 0 {
        set_ram_word!(
            game.fx_tilemap_ptr =
                crate::rtl::rom_table_word!(game, 0x83abf0, ((*FD).type_0 >> 1) as isize)
        );
        kFxTypeFuncPtrs[((*FD).type_0 >> 1) as usize].expect("non-null function pointer")(game);
    }
    if (*FD).palette_fx_bitset != 0 {
        let mut bits: i32 = (*FD).palette_fx_bitset as i32;
        let v11 = RomPtr_83(
            crate::rtl::rom_table_word!(game, 0x83ac46, ram_word!(game, area_index) as isize),
            game,
        );
        let mut j: i32 = 0;
        while j != 16 {
            if bits & 1 != 0 {
                SpawnPalfxObject(*(v11.offset(j as isize) as *mut u16), game);
            }
            j += 2;
            bits >>= 1;
        }
    }
    if (*FD).animtiles_bitset != 0 {
        let mut bits_0: i32 = (*FD).animtiles_bitset as i32;
        let v14 = RomPtr_83(
            crate::rtl::rom_table_word!(game, 0x83ac56, ram_word!(game, area_index) as isize),
            game,
        );
        let mut k: i32 = 0;
        while k != 16 {
            if bits_0 & 1 != 0 {
                SpawnAnimtiles(*(v14.offset(k as isize) as *mut u16), game);
            }
            k += 2;
            bits_0 >>= 1;
        }
    }
}
pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    let mut v1: i16 = 0;
    if (ram_word!(game, ceres_status) & 0x8000) != 0 {
        if (112i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0
            && (144i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0
            && ram_word!(game, samus_y_pos) as i32 - 128i32 & 0x8000 != 0
            && ram_word!(game, samus_y_pos) as i32 - 75i32 & 0x8000 == 0
            && ram_word!(game, samus_y_speed) == 0
            && ram_word!(game, samus_y_subspeed) == 0
            && ram_word!(game, game_state) as i32 == kGameState_8_MainGameplay as i32
        {
            CallSomeSamusCode(2, game);
            set_ram_words! { game, 0; screen_fade_delay, screen_fade_counter }
            set_ram_word!(game.game_state = 32);
        }
        crate::update_place! { reference fresh0 = *(ram_ptr!(game, u16; 0x7e1, 2)) => (*fresh0).wrapping_sub(1); }
        if *(ram_ptr!(game, i16; 0x7e1, 2)) < 0 {
            let v0: i32 = ((6u16).wrapping_mul(*(ram_ptr!(game, u16; 0x7e1)))) as i32 >> 1;
            *(ram_ptr!(game, u16; 0x7e1, 2)) =
                crate::rtl::rom_table_word!(game, 0x89ad5f, v0 as isize);
            set_ram_word!(
                game.reg_M7B = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 1) as isize)
            );
            set_ram_word!(game.reg_M7C = -(ram_word!(game, reg_M7B) as i32) as u16);
            set_ram_word!(
                game.reg_M7A = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 2) as isize)
            );
            set_ram_word!(game.reg_M7D = ram_word!(game, reg_M7A));
            if *(ram_ptr!(game, i16; 0x7e1)) < 0 {
                v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_sub(1);
                if *(ram_ptr!(game, u16; 0x7e1)) == 0x8001 {
                    v1 = 0;
                }
            } else {
                v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_add(1);
                if *(ram_ptr!(game, u16; 0x7e1)) == 67 {
                    v1 = -32700i16;
                }
            }
            *(ram_ptr!(game, u16; 0x7e1)) = v1 as u16;
        }
    }
}
