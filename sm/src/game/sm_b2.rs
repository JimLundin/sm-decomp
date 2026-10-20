pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::projectile_types::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6};
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_90::ProjectileReflection;
use crate::game::sm_a0::{
    CosineMult8bit, EnemyDeathAnimation, EnemyFunc_BBBF, Enemy_ItemDrop_LowerNorfairSpacePirate,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SineMult8bit,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::abs16;
use crate::rtl::{RomPtr, Unreachable};
use ::core::ptr::null_mut;

pub use crate::types::Consts_A0;
pub use crate::types::Consts_B2;
pub use crate::types::{Consts_86, EnemyDef, Vulnerability};
pub use crate::types::{EnemyBase, EnemyData, LongPtr, Point16U, ProjectileType, Rect16U, VoidP};
crate::game::constants::typed_constants! { Consts_B2;
    addr_kSpacePirates_Ilist_FC48 = 64584, addr_kSpacePirates_Ilist_FC0E = 64526,
    addr_kSpacePirates_Ilist_FBE6 = 64486, addr_kSpacePirates_Ilist_FBC6 = 64454,
    addr_kSpacePirates_Ilist_FB8C = 64396, addr_kSpacePirates_Ilist_FB64 = 64356,
    addr_kSpacePirates_Ilist_FB58 = 64344, addr_kSpacePirates_Ilist_FB4C = 64332,
    addr_kSpacePirates_Ilist_F51A = 62746, addr_kSpacePirates_Ilist_F4EA = 62698,
    addr_kSpacePirates_Ilist_F4CC = 62668, addr_kSpacePirates_Ilist_F4A4 = 62628,
    addr_kSpacePirates_Ilist_F492 = 62610, addr_kSpacePirates_Ilist_F462 = 62562,
    addr_kSpacePirates_Ilist_F420 = 62496, addr_kSpacePirates_Ilist_F3B2 = 62386,
    addr_kSpacePirates_Ilist_F34A = 62282, addr_kSpacePirates_Ilist_F32E = 62254,
    addr_kSpacePirates_Ilist_F2F8 = 62200, addr_kSpacePirates_Ilist_F2DA = 62170,
    addr_kSpacePirates_Ilist_F2B2 = 62130, addr_kSpacePirates_Ilist_F2A0 = 62112,
    addr_kSpacePirates_Ilist_F270 = 62064, addr_kSpacePirates_Ilist_F22E = 61998,
    addr_kSpacePirates_Ilist_F1C4 = 61892, addr_kSpacePirates_Ilist_F15C = 61788,
    addr_kSpacePirates_Ilist_EDF6 = 60918, addr_off_B2EDAC = 60844,
    addr_kSpacePirates_Ilist_EDA4 = 60836, addr_kSpacePirates_Ilist_ED80 = 60800,
    addr_kSpacePirates_Ilist_ED36 = 60726, addr_off_B2ECEC = 60652,
    addr_kSpacePirates_Ilist_ECE4 = 60644, addr_kSpacePirates_Ilist_ECC0 = 60608,
}
pub use crate::types::Consts_B4;
pub type Enemy_SpacePirates = crate::game::enemy_layouts::EnemyLayout72;

const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_Vulnerability(a: u16, game: &mut Game) -> *mut Vulnerability {
    RomPtr((0xb40000u32) | ((a) as u32), game) as *mut Vulnerability
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_SpacePirates(o: i32) -> Enemy_SpacePirates = [0xf78, o as isize];
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_CancelBeam_B2 => Enemy_SwitchToFrozenAi(game);
    Enemy_NormalFrozenAI_B2 => NormalEnemyFrozenAI(game);
    WalkingSpacePirates_Powerbomb => NormalEnemyPowerBombAi(game);
}
pub unsafe fn WalkingSpacePirates_Touch(game: &mut Game) {
    if (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .frozen_timer
        == 0
    {
        NormalEnemyTouchAi(game);
    }
}
pub unsafe fn WalkingSpacePirates_Shot(game: &mut Game) {
    if (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .enemy_ptr as i32
        == addr_kEnemyDef_F593 as i32
    {
        WalkingSpacePirates_8789(game);
    } else {
        NormalEnemyShotAi(game);
    };
}
pub unsafe fn WalkingSpacePirates_8789(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
    set_ram_word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    if (*E).base.health == 0 {
        let v1: u16 = ram_word!(game, cur_enemy_index);
        if (*E).base.enemy_ptr as i32 == addr_kEnemyDef_F593 as i32 {
            (*E).var_B = 0;
            EnemyDeathAnimation(v1, 4, game);
            Enemy_ItemDrop_LowerNorfairSpacePirate(v1, game);
        } else {
            (*E).var_B = 0;
            EnemyDeathAnimation(v1, 4, game);
        }
    }
}
pub unsafe fn WalkingSpacePirates_87C8(game: &mut Game) {
    let mut current_block: u64;
    if ((*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .enemy_ptr as i32
        == addr_kEnemyDef_F593 as i32)
    {
        let mut r18: u16 = ram_word!(
            game,
            projectile_type[ram_word!(game, collision_detection_index)]
        );
        if (r18 as i32 & kProjectileType_TypeMask as i32) - kProjectileType_PowerBomb as i32
            & 0x8000
            != 0
        {
            let mut enemy_ptr: u16 = 0;
            let mut vulnerability_ptr: u16 = 0;
            enemy_ptr = (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
                .base
                .enemy_ptr;
            vulnerability_ptr = (*get_EnemyDef_A2(enemy_ptr, game)).vulnerability_ptr;
            if vulnerability_ptr == 0 {
                vulnerability_ptr = addr_stru_B4EC1C as u16;
            }
            let mut r20: u16 = vulnerability_ptr;
            if (r18 as i32 & kProjectileType_TypeMask as i32) != 0 {
                if r18 as i32 & kProjectileType_TypeMask as i32 != kProjectileType_Missile as i32
                    && r18 as i32 & kProjectileType_TypeMask as i32
                        != kProjectileType_SuperMissile as i32
                {
                    current_block = 1528265894032410192;
                } else {
                    let mut v3: i16 = (((r18) & ((kProjectileType_TypeMask) as u16)) >> 8) as i16;
                    let mut v = get_Vulnerability((r20).wrapping_add((v3) as u16), game);
                    if ((*v).plasma_ice_wave & 0xf) != 0 && ((*v).plasma_ice_wave & 0xf) != 15 {
                        current_block = 1528265894032410192;
                    } else {
                        current_block = 17407779659766490442;
                    }
                }
            } else {
                let mut v2 = get_Vulnerability((r20 as i32 + (r18 as i32 & 0xf)) as u16, game);
                if ((*v2).power & 0xf) != 0 && ((*v2).power & 0xf) != 15 {
                    current_block = 1528265894032410192;
                } else {
                    current_block = 17407779659766490442;
                }
            }
            if current_block == 1528265894032410192 {
            } else {
                WalkingSpacePirates_883E(game);
                current_block = 15976848397966268834;
            }
        } else {
            current_block = 15976848397966268834;
        }
        match current_block {
            1528265894032410192 => {}
            _ => return,
        }
    }
    WalkingSpacePirates_8789(game);
}
pub unsafe fn WalkingSpacePirates_883E(game: &mut Game) {
    if (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .enemy_ptr as i32
        != addr_kEnemyDef_F593 as i32
    {
        WalkingSpacePirates_8789(game);
        return;
    }
    let v0: u16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let v1: i32 = ram_word!(game, collision_detection_index) as i32;
    let r18: u16 = ram_word!(game, projectile_type[v1]);
    if r18 as i32 & kProjectileType_TypeMask as i32 == kProjectileType_SuperMissile as i32 {
        if ram_word!(game, projectile_variables[v1]) == 0 {
            return;
        }
    } else if (r18 as i32 & kProjectileType_TypeMask as i32) - kProjectileType_PowerBomb as i32
        & 0x8000
        == 0
    {
        return;
    }
    (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .invincibility_timer = 10;
    let mut v2: u16 = 0;
    if (ram_word!(game, projectile_dir[v1]) & 0xf) == 7 {
        v2 = 1;
    } else if (ram_word!(game, projectile_dir[v1]) & 0xf) == 2 {
        v2 = 8;
    } else {
        v2 = 5;
    }
    set_ram_word!(game.projectile_dir[v1] = v2);
    ProjectileReflection(v0, game);
    QueueSfx2_Max6(0x66, game);
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16;
    @prefix {
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (*jp.offset(0) as i16 as i32) << 16,
            game,
        ) == 0
        {
            return jp.offset(1);
        }
    };
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight [addr_kSpacePirates_Ilist_ED36 as i32 as usize as *const u16] {

        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_C = ((*E).var_C) ^ (1u16);
        if (*E).var_C != 0 {
            return addr_off_B2ECEC as i32 as usize as *const u16;
        }
    };
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft [addr_off_B2EDAC as i32 as usize as *const u16] {

        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_C = ((*E).var_C) ^ (1u16);
        if (*E).var_C != 0 {
            return addr_kSpacePirates_Ilist_EDF6 as i32 as usize as *const u16;
        }
    };
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16;
    SpacePirates_Instr_RandomNewDirFaceR [result as usize as *const u16] {
        let result: u16 = addr_kSpacePirates_Ilist_ED36 as u16;
        let v4: u16 = (NextRandom(game)) & (1u16);
        (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game)).var_C = v4;
        if v4 != 0 {
            return addr_off_B2ECEC as i32 as usize as *const u16;
        }
    };
    SpacePirates_Instr_RandomNewDirFaceL [result as usize as *const u16] {
        let result: u16 = addr_off_B2EDAC as u16;
        let v4: u16 = (NextRandom(game)) & (1u16);
        (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game)).var_C = v4;
        if v4 != 0 {
            return addr_kSpacePirates_Ilist_EDF6 as i32 as usize as *const u16;
        }
    };
    SpacePirates_Instr_PrepareWallJumpR [jp] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_B = ((*E).base.x_pos).wrapping_add((*E).parameter_2);
        (*E).var_D = ((*E).base.x_pos as i32 + ((*E).parameter_2 as i32 >> 1)) as u16;
        (*E).var_E = (*E).base.y_pos;
        (*E).var_F = 64;
    };
    SpacePirates_Instr_PrepareWallJumpL [jp] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).var_B = ((*E).base.x_pos).wrapping_sub((*E).parameter_2);
        (*E).var_D = ((*E).base.x_pos as i32 - ((*E).parameter_2 as i32 >> 1)) as u16;
        (*E).var_E = (*E).base.y_pos;
        (*E).var_F = 192;
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpacePirates_Instr_FireLaserL [jp] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        set_ram_word!(
            game.cinematic_var8 = *(RomPtr_A0((*E).base.enemy_ptr, game) as *mut u16).offset(3)
        );
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_sub(24)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        game.eproj_spawn_r22 = 0;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 0, game);
    };
    SpacePirates_Instr_FireLaserR [jp] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(24)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        game.eproj_spawn_r22 = 1;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
}
pub unsafe fn SpacePirates_Instr_SetEnemyFunc(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game)).var_A = *jp.offset(0);
    jp.offset(1)
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpacePirates_Instr_PlaySfx [jp] { QueueSfx2_Max6(0x66, game); };
}
pub unsafe fn WallSpacePirates_Init(game: &mut Game) {
    let mut v0: u16 = addr_kSpacePirates_Ilist_ED36 as u16;
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).parameter_1 & 1) != 0 {
        v0 = addr_off_B2EDAC as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).var_20 = 190;
    (*E).var_21 = 66;
    (*E).var_22 = 2;
    if ((*E).parameter_1 & 0x8000) == 0 {
        (*E).var_20 = ((*E).var_20).wrapping_add(2);
        (*E).var_21 = ((*E).var_21).wrapping_sub(2);
        (*E).var_22 = ((*E).var_22).wrapping_add(2);
    }
    let mut v2: u16 = ((0xb2f034i32) as u16);
    if ((*E).parameter_1 & 1) != 0 {
        v2 = ((0xb2f0c8i32) as u16);
    }
    (*E).var_A = v2;
    let mut v3: u16 = 0;
    if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
        v3 = ((*E).base.x_pos) & (0xfff8u16);
    } else {
        v3 = (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16;
    }
    (*E).base.x_pos = v3;
}
crate::game::address_dispatch! {
    pub unsafe fn CallSpacePiratesEnemyFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11698251 => [()]; 11726900 => SpacePirates_Func_1; 11726927 => [()];
        11726928 => SpacePirates_Func_2; 11727048 => SpacePirates_Func_3; 11727075 => [()];
        11730506 => [()]; 11727076 => SpacePirates_Func_4; 11728553 => SpacePirates_Func_5;
        11728612 => SpacePirates_Func_6; 11728919 => SpacePirates_Func_11;
        11728972 => SpacePirates_Func_12; 11729040 => SpacePirates_Func_13;
        11729093 => SpacePirates_Func_14; 11729161 => SpacePirates_Func_15;
        11729285 => SpacePirates_F985; 11729345 => SpacePirates_F9C1; 11729429 => SpacePirates_FA15;
        11729497 => SpacePirates_FA59; 11729557 => SpacePirates_FA95; 11729641 => SpacePirates_FAE9;
        11730244 => [WalkingSpacePirates_FD44(game)]; 11730382 => [WalkingSpacePirates_FDCE(game)];
    }
}
pub unsafe fn WallSpacePirates_Main(game: &mut Game) {
    let v0 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    CallSpacePiratesEnemyFunc(
        (((*v0).ai_var_A) as u32) | (0xb20000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn SpacePirates_Func_1(k: u16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x20, game) != 0 {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ED80 as u16, timer = 1}
    }
}
pub unsafe fn SpacePirates_Func_2(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos =
        ((*E).var_D).wrapping_add(SineMult8bit((*E).var_F, ((*E).parameter_2 >> 1), game));
    (*E).base.y_pos =
        ((*E).var_E).wrapping_sub(CosineMult8bit((*E).var_F, ((*E).parameter_2 >> 2), game));
    let v1: u16 = ((*(&raw mut (*E).var_F as *mut u8).offset(0))
        .wrapping_sub(*(&raw mut (*E).var_22 as *mut u8).offset(0))) as u16;
    (*E).var_F = v1;
    if v1 == (*E).var_20 {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_EDA4 as u16, timer = 1}
        let mut v2: u16 = 0;
        if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
            v2 = ((*E).base.x_pos) & (0xfff8u16);
        } else {
            v2 = (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16;
        }
        (*E).base.x_pos = v2;
    }
}
pub unsafe fn SpacePirates_Func_3(k: u16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x20, game) != 0 {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ECC0 as u16, timer = 1}
    }
}
pub unsafe fn SpacePirates_Func_4(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos =
        ((*E).var_D).wrapping_add(SineMult8bit((*E).var_F, ((*E).parameter_2 >> 1), game));
    (*E).base.y_pos =
        ((*E).var_E).wrapping_sub(CosineMult8bit((*E).var_F, ((*E).parameter_2 >> 2), game));
    let v1: u16 = ((*(&raw mut (*E).var_22 as *mut u8).offset(0))
        .wrapping_add(*(&raw mut (*E).var_F as *mut u8).offset(0))) as u16;
    (*E).var_F = v1;
    if v1 == (*E).var_21 {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ECE4 as u16, timer = 1}
        let mut v2: u16 = 0;
        if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
            v2 = ((*E).base.x_pos) & (0xfff8u16);
        } else {
            v2 = (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16;
        }
        (*E).base.x_pos = v2;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpacePirates_Instr_20 [jp.offset(1)] {
        (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .palette_index = *jp.offset(0);
    };
    SpacePirates_Instr_16 [jp.offset(1)] { QueueSfx2_Max6(*jp.offset(0), game); };
    SpacePirates_Instr_15 [jp.offset(3)] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        game.eproj_spawn_rect = Rect16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
            w: *jp.offset(1),
            h: *jp.offset(2),
        };
        SpawnEprojWithGfx(
            *jp.offset(0),
            ram_word!(game, cur_enemy_index),
            addr_kEproj_PirateClaw as u16,
            game,
        );
    };
    SpacePirates_Instr_18 [(*E).var_C as usize as *const u16] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).base.instruction_timer = 1;
        (*E).var_C =
            (if ((*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 & 0x8000) != 0 {
                addr_kSpacePirates_Ilist_F420 as i32
            } else {
                addr_kSpacePirates_Ilist_F22E as i32
            }) as u16;
    };
    SpacePirates_Instr_17 [jp] { (*Get_SpacePirates(k as i32, game)).var_00 = 0; };
}
pub unsafe fn NinjaSpacePirates_Init(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = (if ((*E).parameter_1 & 1) != 0 {
        addr_kSpacePirates_Ilist_F4CC as i32
    } else {
        addr_kSpacePirates_Ilist_F2DA as i32
    }) as u16;
    (*E).var_C = (*E).base.current_instruction;
    let x_pos: u16 = (*E).base.x_pos;
    if ((*E).parameter_1 & 1) != 0 {
        (*E).var_E = x_pos;
        (*E).var_F = ((*E).parameter_2).wrapping_add(x_pos);
    } else {
        (*E).var_F = x_pos;
        (*E).var_E = (x_pos).wrapping_sub((*E).parameter_2);
    }
    let mut r20: u16 = ((((*E).var_F).wrapping_sub((*E).var_E)) >> 1);
    (*E).var_D = ((*E).var_E).wrapping_add(r20);
    let mut r18: u16 = 0;
    let mut R22: u16 = 0;
    r20 = ((r20 as u8 as i32) << 8) as u16;
    loop {
        r18 = (r18).wrapping_add(32);
        R22 = (R22).wrapping_add(r18);
        if (R22 as i32 - r20 as i32 & 0x8000 == 0) {
            break;
        }
    }
    (*E).var_B = r18;
    let R24: u16 = ((R22 as i32 & 0xff00) >> 8) as u16;
    (*E).var_F = ((*E).var_D).wrapping_add(R24);
    (*E).var_E = ((*E).var_D).wrapping_sub(R24);
    let mut sps_var_E: u16 = (*E).var_E;
    if ((*E).parameter_1 & 1) == 0 {
        sps_var_E = (*E).var_F;
    }
    (*E).base.x_pos = sps_var_E;
    (*E).var_A = ((0xb2804bi32) as u16);
    (*E).var_08 = (*E).base.y_pos;
    let mut v6: u16 = 0;
    let mut v7: u16 = 0;
    for _ in 0..=15 {
        *(ram_ptr!(game, u16; 0xc200)).offset(((v7 as i32 >> 1) + 240) as isize) =
            crate::rtl::rom_table_word!(game, 0xb28727, (v6 >> 1) as isize);
        v6 = (v6).wrapping_add(2);
        v7 = (v7).wrapping_add(2);
    }
}
pub unsafe fn NinjaSpacePirates_Main(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    CallSpacePiratesEnemyFunc(
        (((*E).var_A) as u32) | (0xb20000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn SpacePirates_Func_5(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)))) as i16)
        .wrapping_sub(128)
        >= 0
    {
        SpacePirates_Func_8(game);
    } else {
        (*E).base.current_instruction =
            (if (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 & 0x8000 != 0 {
                addr_kSpacePirates_Ilist_F420 as i32
            } else {
                addr_kSpacePirates_Ilist_F22E as i32
            }) as u16;
        (*E).var_C = (*E).base.current_instruction;
        (*E).base.instruction_timer = 1;
    };
}
pub unsafe fn SpacePirates_Func_6(k: u16, game: &mut Game) {
    if SpacePirates_Func_8(game) == 0
        && SpacePirates_Func_10(game) == 0
        && SpacePirates_Func_9(game) == 0
    {
        SpacePirates_Func_7(k, game);
    }
}
pub unsafe fn SpacePirates_Func_7(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    if ((*E).base.frame_counter & 0x3f) == 0 {
        if (*E).base.x_pos == (*E).var_E {
            if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0 {
                return;
            }
            (*E).base.current_instruction = addr_kSpacePirates_Ilist_F15C as u16;
        } else {
            if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0 {
                return;
            }
            (*E).base.current_instruction = addr_kSpacePirates_Ilist_F34A as u16;
        }
        (*E).base.instruction_timer = 1;
    }
}
pub unsafe fn SpacePirates_Func_8(game: &mut Game) -> u16 {
    let mut v0: u16 = 8;
    while ram_word!(game, projectile_type[(v0 >> 1)]) == 0 {
        v0 = (v0).wrapping_sub(2);
        if (v0 & 0x8000) != 0 {
            return 0;
        }
    }
    let v1: i32 = v0 as i32 >> 1;
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16((ram_word!(game, projectile_x_pos[v1])).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub(32)
        >= 0
        || ((abs16((ram_word!(game, projectile_y_pos[v1])).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(32)
            >= 0
    {
        return 0;
    }
    enemy_instruction! {[(*E).base]; instruction = (if (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 & 0x8000 != 0 {
        addr_kSpacePirates_Ilist_F462 as i32
    } else {
        addr_kSpacePirates_Ilist_F270 as i32
    }) as u16, timer = 1}
    1
}
pub unsafe fn SpacePirates_Func_9(game: &mut Game) -> u16 {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16(((*E).var_D).wrapping_sub(ram_word!(game, samus_x_pos)))) as i16).wrapping_sub(32)
        >= 0
    {
        return 0;
    }
    let mut v1: u16 = addr_kSpacePirates_Ilist_F1C4 as u16;
    if (*E).base.x_pos == (*E).var_E {
        v1 = addr_kSpacePirates_Ilist_F3B2 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
    1
}
pub unsafe fn SpacePirates_Func_10(game: &mut Game) -> u16 {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub(40)
        >= 0
        || ((abs16((ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(40)
            >= 0
    {
        return 0;
    }
    enemy_instruction! {[(*E).base]; instruction = (if ((*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 & 0x8000) != 0 {
        addr_kSpacePirates_Ilist_F51A as i32
    } else {
        addr_kSpacePirates_Ilist_F32E as i32
    }) as u16, timer = 1}
    1
}
pub unsafe fn SpacePirates_Func_11(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_sub((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).var_00 = ((*E).var_00).wrapping_add(32);
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_D) as i16) < 0 {
        (*E).var_A = ((0xb2f84ci32) as u16);
    }
}
pub unsafe fn SpacePirates_Func_12(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_sub((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let v2: u16 = ((*E).var_00).wrapping_sub(32);
    (*E).var_00 = v2;
    if v2 == 0 {
        (*E).var_A = ((0xb2804bi32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2F8 as u16, timer = 1}
        (*E).base.x_pos = (*E).var_E;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_Func_13(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_add((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).var_00 = ((*E).var_00).wrapping_add(32);
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_D) as i16) >= 0 {
        (*E).var_A = ((0xb2f8c5i32) as u16);
    }
}
pub unsafe fn SpacePirates_Func_14(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_add((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let v2: u16 = ((*E).var_00).wrapping_sub(32);
    (*E).var_00 = v2;
    if v2 == 0 {
        (*E).var_A = ((0xb2804bi32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F4EA as u16, timer = 1}
        (*E).base.x_pos = (*E).var_F;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_Func_15(k: u16, game: &mut Game) {
    if SpacePirates_Func_8(game) == 0 && SpacePirates_Func_10(game) == 0 {
        SpacePirates_Func_16(game);
    }
}
pub unsafe fn SpacePirates_Func_16(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16(((*E).var_D).wrapping_sub(ram_word!(game, samus_x_pos)))) as i16).wrapping_sub(32)
        < 0
    {
        let mut v2: u16 = 0;
        loop {
            v2 = (NextRandom(game)) & (3u16);
            if (v2 != 0) {
                break;
            }
        }
        let mut v3: i16 = 0;
        if (*E).base.x_pos == (*E).var_E {
            v3 = 4;
        }
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xb2f959, (v2 as i32 + v3 as i32) as isize), timer = 1}
    }
}
pub unsafe fn SpacePirates_Instr_19(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_SpacePirates(k as i32, game);
    (*E).var_00 = 1536;
    (*E).var_03 =
        ((*E).var_D as i32 + ((((*E).var_F).wrapping_sub((*E).var_D)) as i32 >> 1)) as u16;
    jp
}
pub unsafe fn SpacePirates_F985(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.y_pos =
        ((*E).base.y_pos).wrapping_sub((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    let mut v2: i16 = (((*E).var_00) as i16).wrapping_sub(64);
    (*E).var_00 = v2 as u16;
    if v2 < 0 {
        (*E).var_A = ((0xb2f9c1i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2A0 as u16, timer = 1}
        (*E).var_00 = 1536;
    }
}
pub unsafe fn SpacePirates_F9C1(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(5);
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).var_00 as *mut u8).offset(1) as u32) << 16
            | *(&raw mut (*E).var_00 as *mut u8).offset(0) as u16 as u32) as i32,
        game,
    ) != 0
        || {
            v2 = (((*E).var_00) as i16).wrapping_sub(64);
            (*E).var_00 = v2 as u16;
            v2 < 0
        }
        || (v2 as u16 & 0xff00) == 0
    {
        (*E).var_A = ((0xb2fa15i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2B2 as u16, timer = 1}
        (*E).base.y_pos = (*E).var_08;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_FA15(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    let v2: u16 = ((*E).base.x_pos).wrapping_sub(2);
    (*E).base.x_pos = v2;
    if ((v2) as i16).wrapping_sub(((*E).var_E) as i16) < 0 {
        (*E).base.x_pos = (*E).var_E;
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2F8 as u16, timer = 1}
        (*E).var_A = ((0xb2804bi32) as u16);
    }
}
pub unsafe fn SpacePirates_Instr_21(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_SpacePirates(k as i32, game);
    (*E).var_00 = 1536;
    (*E).var_03 =
        ((*E).var_E as i32 + ((((*E).var_D).wrapping_sub((*E).var_E)) as i32 >> 1)) as u16;
    jp
}
pub unsafe fn SpacePirates_FA59(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.y_pos =
        ((*E).base.y_pos).wrapping_sub((*(&raw mut (*E).var_00 as *mut u8).offset(1)) as u16);
    let mut v2: i16 = (((*E).var_00) as i16).wrapping_sub(64);
    (*E).var_00 = v2 as u16;
    if v2 < 0 {
        (*E).var_A = ((0xb2fa95i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F492 as u16, timer = 1}
        (*E).var_00 = 1536;
    }
}
pub unsafe fn SpacePirates_FA95(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E = Get_SpacePirates(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(5);
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).var_00 as *mut u8).offset(1) as u32) << 16
            | *(&raw mut (*E).var_00 as *mut u8).offset(0) as u16 as u32) as i32,
        game,
    ) != 0
        || {
            v2 = (((*E).var_00) as i16).wrapping_sub(64);
            (*E).var_00 = v2 as u16;
            v2 < 0
        }
        || (v2 as u16 & 0xff00) == 0
    {
        (*E).var_A = ((0xb2fae9i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F4A4 as u16, timer = 1}
        (*E).base.y_pos = (*E).var_08;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_FAE9(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    let v2: u16 = ((*E).base.x_pos).wrapping_add(2);
    (*E).base.x_pos = v2;
    if ((v2) as i16).wrapping_sub(((*E).var_F) as i16) >= 0 {
        (*E).base.x_pos = (*E).var_F;
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F4EA as u16, timer = 1}
        (*E).var_A = ((0xb2804bi32) as u16);
    }
}
pub unsafe fn SpacePirates_FB11(k: u16, game: &mut Game) {
    let E = Get_SpacePirates(k as i32, game);
    CreateSpriteAtPos(
        ((*E).base.x_pos).wrapping_sub(8),
        ((*E).base.y_pos).wrapping_add(28),
        10,
        0,
        game,
    );
    CreateSpriteAtPos(
        ((*E).base.x_pos).wrapping_add(8),
        ((*E).base.y_pos).wrapping_add(28),
        10,
        0,
        game,
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpacePirates_Instr_12 [jp.offset(1)] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_sub(24)),
            y: (((*E).base.y_pos).wrapping_sub(*jp.offset(0))),
        };
        game.eproj_spawn_r22 = 0;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 0, game);
    };
    SpacePirates_Instr_14 [jp.offset(1)] {
        let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(24)),
            y: (((*E).base.y_pos).wrapping_sub(*jp.offset(0))),
        };
        game.eproj_spawn_r22 = 1;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
    SpacePirates_Instr_11 [jp.offset(1)] {
        (*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game)).var_A = *jp.offset(0);
    };
    SpacePirates_Instr_13 [] {
        if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x10, game) != 0 {
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
                ((*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
                    .base
                    .x_pos) as i16,
            ) >= 0
            {
                return addr_kSpacePirates_Ilist_FC0E as i32 as usize as *const u16;
            }
            addr_kSpacePirates_Ilist_FB8C as i32 as usize as *const u16
        } else {
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
                ((*Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game))
                    .base
                    .x_pos) as i16,
            ) >= 0
            {
                return addr_kSpacePirates_Ilist_FB64 as i32 as usize as *const u16;
            }
            addr_kSpacePirates_Ilist_FBE6 as i32 as usize as *const u16
        }
    };
}
pub unsafe fn WalkingSpacePirates_Init(game: &mut Game) {
    let mut v0: u16 = addr_kSpacePirates_Ilist_FB64 as u16;
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).parameter_1 & 1) != 0 {
        v0 = addr_kSpacePirates_Ilist_FBE6 as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).var_A = ((0xb2804bi32) as u16);
    (*E).var_F = ((*E).parameter_2).wrapping_add((*E).base.x_pos);
    (*E).var_E = ((*E).base.x_pos).wrapping_sub((*E).parameter_2);
}
pub unsafe fn WalkingSpacePirates_Main(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    CallSpacePiratesEnemyFunc(
        (((*E).var_A) as u32) | (0xb20000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    if ((*E).parameter_1 & 0x8000) != 0 {
        WalkingSpacePirates_FE4B(game);
    }
}
pub unsafe fn WalkingSpacePirates_FD44(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x10, game) != 0 {
        let mut v1: u16 = addr_kSpacePirates_Ilist_FB8C as u16;
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0 {
            v1 = addr_kSpacePirates_Ilist_FC0E as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
    } else if Enemy_MoveDown(ram_word!(game, cur_enemy_index), (1i16 as i32) << 16, game) != 0 {
        let x_pos: u16 = (*E).base.x_pos;
        (*E).var_00 = x_pos;
        (*E).base.x_pos = (x_pos).wrapping_sub(17);
        let result: u8 =
            Enemy_MoveDown(ram_word!(game, cur_enemy_index), (1i16 as i32) << 16, game);
        (*E).base.x_pos = (*E).var_00;
        if result != 0 {
            EnemyFunc_BBBF(ram_word!(game, cur_enemy_index), (-9i16 as i32) << 16, game);
            if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -14337i32, game) == 0
                && (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_E) as i16) >= 0
            {
                return;
            }
        }
        enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_FBC6 as u16, timer = 1}
    }
}
pub unsafe fn WalkingSpacePirates_FDCE(game: &mut Game) {
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x10, game) != 0 {
        let mut v1: u16 = addr_kSpacePirates_Ilist_FB8C as u16;
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0 {
            v1 = addr_kSpacePirates_Ilist_FC0E as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
    } else if Enemy_MoveDown(ram_word!(game, cur_enemy_index), (1i16 as i32) << 16, game) != 0 {
        let x_pos: u16 = (*E).base.x_pos;
        (*E).var_00 = x_pos;
        (*E).base.x_pos = (x_pos).wrapping_add(16);
        let result: u8 =
            Enemy_MoveDown(ram_word!(game, cur_enemy_index), (1i16 as i32) << 16, game);
        (*E).base.x_pos = (*E).var_00;
        if result == 0
            || Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 14336, game) != 0
            || (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_F) as i16) >= 0
        {
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_FC48 as u16, timer = 1}
        }
    }
}
pub unsafe fn WalkingSpacePirates_FE4B(game: &mut Game) {
    let mut v0: u16 = 8;
    while ram_word!(game, projectile_type[(v0 >> 1)]) == 0 {
        v0 = (v0).wrapping_sub(2);
        if (v0 & 0x8000) != 0 {
            return;
        }
    }
    let v1: i32 = v0 as i32 >> 1;
    let E = Get_SpacePirates(ram_word!(game, cur_enemy_index) as i32, game);
    if ((abs16((ram_word!(game, projectile_x_pos[v1])).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub(32)
        < 0
        && ((abs16((ram_word!(game, projectile_y_pos[v1])).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(32)
            < 0
    {
        enemy_instruction! {[(*E).base]; instruction = (if (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32 & 0x8000 != 0 {
            addr_kSpacePirates_Ilist_FB58 as i32
        } else {
            addr_kSpacePirates_Ilist_FB4C as i32
        }) as u16, timer = 1}
    }
}
