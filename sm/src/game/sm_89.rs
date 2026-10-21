use crate::game::liquid_effects::{initialize_acid, initialize_lava, initialize_water};
pub use crate::game::prelude::*;

use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::{
    FxTypeFunc_20, FxTypeFunc_22_ScrollingSky, FxTypeFunc_24, FxTypeFunc_26_TourianEntranceStatue,
    FxTypeFunc_28_CeresRidley, FxTypeFunc_2C_Haze, FxTypeFunc_8_Spores, FxTypeFunc_A_Rain,
    FxTypeFunc_C, FxTypeFunc_CeresElevator,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct FxDef {
        pub door_ptr, base_y_pos, target_y_pos, y_vel: u16;
        pub timer, type_0, default_layer_blend, layer3_layer_blend, fx_liquid_options_,
            palette_fx_bitset, animtiles_bitset, palette_blend: u8;
    }
}

const RomPtr_83: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x83>;
#[inline]
unsafe fn get_FxDef(a: u16, game: &mut Game) -> *mut FxDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut FxDef
}

macro_rules! apply_room_fx_definition {
    ($game:ident, $definition:ident, $palette_field:ident) => {
        word!($game.fx_base_y_pos = (*$definition).base_y_pos);
        word!($game.fx_target_y_pos = (*$definition).target_y_pos);
        word!($game.fx_y_vel = (*$definition).y_vel);
        word!($game.fx_timer = (*$definition).timer as u16);
        word!($game.fx_layer_blending_config_a = (*$definition).default_layer_blend as u16);
        word!($game.fx_layer_blending_config_b = (*$definition).layer3_layer_blend as u16);
        word!($game.fx_liquid_options = (*$definition).fx_liquid_options_ as u16);
        if (*$definition).palette_blend != 0 {
            let v2: i32 = (*$definition).palette_blend as i32 >> 1;
            word!(
                $game.$palette_field[25] =
                    crate::rtl::rom_table_word!($game, 0x89aa02, v2 as isize)
            );
            word!(
                $game.$palette_field[26] =
                    crate::rtl::rom_table_word!($game, 0x89aa02, (v2 + 1) as isize)
            );
            word!(
                $game.$palette_field[27] =
                    crate::rtl::rom_table_word!($game, 0x89aa02, (v2 + 2) as isize)
            );
        } else {
            word!($game.$palette_field[27] = 0);
        }
    };
}

pub unsafe fn LoadFxEntry(v0: u16, game: &mut Game) {
    let FD = get_FxDef(
        (word!(game.room_layer3_asm_ptr) as i32 + 16 * (v0 as i32 & 7)) as u16,
        game,
    );
    apply_room_fx_definition!(game, FD, palette_buffer);
}
pub fn nullsub_106(game: &mut Game) {}
static kFxTypeFuncPtrs: [Func_V; 23] = callback_table! { nonnull Func_V;
    nullsub_106, initialize_lava, initialize_acid, initialize_water, FxTypeFunc_8_Spores,
    FxTypeFunc_A_Rain, FxTypeFunc_C, nullsub_106, nullsub_106, nullsub_106, nullsub_106,
    nullsub_106, nullsub_106, nullsub_106, nullsub_106, nullsub_106, FxTypeFunc_20,
    FxTypeFunc_22_ScrollingSky, FxTypeFunc_24, FxTypeFunc_26_TourianEntranceStatue,
    FxTypeFunc_28_CeresRidley, FxTypeFunc_CeresElevator, FxTypeFunc_2C_Haze,
};

macro_rules! spawn_room_effect_bitset {
    (
        $definition:ident, $bitset_field:ident, $game:ident, $object_table:literal,
        $spawn_object:path $(,)?
    ) => {
        if (*$definition).$bitset_field != 0 {
            let mut bits: i32 = (*$definition).$bitset_field as i32;
            let v11 = RomPtr_83(
                crate::rtl::rom_table_word!($game, $object_table, word!($game.area_index) as isize),
                $game,
            );
            let mut j: i32 = 0;
            while j != 16 {
                if bits & 1 != 0 {
                    $spawn_object(*(v11.offset(j as isize) as *mut u16), $game);
                }
                j += 2;
                bits >>= 1;
            }
        }
    };
}

pub unsafe fn LoadFXHeader(game: &mut Game) {
    let mut entry_ptr: u16 = word!(game.room_layer3_asm_ptr);
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
        if door_ptr == word!(game.door_def_ptr) {
            break;
        }
        entry_ptr = (entry_ptr).wrapping_add(16);
    }
    let FD = get_FxDef(entry_ptr, game);
    apply_room_fx_definition!(game, FD, target_palettes);
    word!(game.fx_type = (*FD).type_0 as u16);
    if (*FD).type_0 != 0 {
        word!(
            game.fx_tilemap_ptr =
                crate::rtl::rom_table_word!(game, 0x83abf0, ((*FD).type_0 >> 1) as isize)
        );
        kFxTypeFuncPtrs[((*FD).type_0 >> 1) as usize](game);
    }
    spawn_room_effect_bitset!(FD, palette_fx_bitset, game, 0x83ac46, SpawnPalfxObject);
    spawn_room_effect_bitset!(FD, animtiles_bitset, game, 0x83ac56, SpawnAnimtiles);
}
pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    let mut v1: i16 = 0;
    if (word!(game.ceres_status) & 0x8000) != 0 {
        if (112i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0
            && (144i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0
            && word!(game.samus_y_pos) as i32 - 128i32 & 0x8000 != 0
            && word!(game.samus_y_pos) as i32 - 75i32 & 0x8000 == 0
            && word!(game.samus_y_speed) == 0
            && word!(game.samus_y_subspeed) == 0
            && word!(game.game_state) as i32 == kGameState_8_MainGameplay as i32
        {
            CallSomeSamusCode(2, game);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = 32);
        }
        crate::update_place! { unaligned_word fresh0 = &raw mut (*(ram_ptr!(game, u16; 0x7e1, 2))) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
        if *(ram_ptr!(game, i16; 0x7e1, 2)) < 0 {
            let v0: i32 = ((6u16).wrapping_mul(*(ram_ptr!(game, u16; 0x7e1)))) as i32 >> 1;
            *(ram_ptr!(game, u16; 0x7e1, 2)) =
                crate::rtl::rom_table_word!(game, 0x89ad5f, v0 as isize);
            word!(game.reg_M7B = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 1) as isize));
            word!(game.reg_M7C = -(word!(game.reg_M7B) as i32) as u16);
            word!(game.reg_M7A = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 2) as isize));
            word!(game.reg_M7D = word!(game.reg_M7A));
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
