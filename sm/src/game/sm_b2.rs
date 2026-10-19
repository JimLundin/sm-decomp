pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::projectile_types::*;
use crate::game::enemy_layouts::enemy_layout;
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
use crate::rtl::abs16;
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::Consts_A0;
pub use crate::types::Consts_B2;
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, EnemyBase, EnemyData, LongPtr,
    Point16U, ProjectileType, Rect16U, VoidP,
};
pub use crate::types::{Consts_86, EnemyDef, Vulnerability};
pub const addr_kSpacePirates_Ilist_FC48: Consts_B2 = 64584;
pub const addr_kSpacePirates_Ilist_FC0E: Consts_B2 = 64526;
pub const addr_kSpacePirates_Ilist_FBE6: Consts_B2 = 64486;
pub const addr_kSpacePirates_Ilist_FBC6: Consts_B2 = 64454;
pub const addr_kSpacePirates_Ilist_FB8C: Consts_B2 = 64396;
pub const addr_kSpacePirates_Ilist_FB64: Consts_B2 = 64356;
pub const addr_kSpacePirates_Ilist_FB58: Consts_B2 = 64344;
pub const addr_kSpacePirates_Ilist_FB4C: Consts_B2 = 64332;
pub const addr_kSpacePirates_Ilist_F51A: Consts_B2 = 62746;
pub const addr_kSpacePirates_Ilist_F4EA: Consts_B2 = 62698;
pub const addr_kSpacePirates_Ilist_F4CC: Consts_B2 = 62668;
pub const addr_kSpacePirates_Ilist_F4A4: Consts_B2 = 62628;
pub const addr_kSpacePirates_Ilist_F492: Consts_B2 = 62610;
pub const addr_kSpacePirates_Ilist_F462: Consts_B2 = 62562;
pub const addr_kSpacePirates_Ilist_F420: Consts_B2 = 62496;
pub const addr_kSpacePirates_Ilist_F3B2: Consts_B2 = 62386;
pub const addr_kSpacePirates_Ilist_F34A: Consts_B2 = 62282;
pub const addr_kSpacePirates_Ilist_F32E: Consts_B2 = 62254;
pub const addr_kSpacePirates_Ilist_F2F8: Consts_B2 = 62200;
pub const addr_kSpacePirates_Ilist_F2DA: Consts_B2 = 62170;
pub const addr_kSpacePirates_Ilist_F2B2: Consts_B2 = 62130;
pub const addr_kSpacePirates_Ilist_F2A0: Consts_B2 = 62112;
pub const addr_kSpacePirates_Ilist_F270: Consts_B2 = 62064;
pub const addr_kSpacePirates_Ilist_F22E: Consts_B2 = 61998;
pub const addr_kSpacePirates_Ilist_F1C4: Consts_B2 = 61892;
pub const addr_kSpacePirates_Ilist_F15C: Consts_B2 = 61788;
pub const addr_kSpacePirates_Ilist_EDF6: Consts_B2 = 60918;
pub const addr_off_B2EDAC: Consts_B2 = 60844;
pub const addr_kSpacePirates_Ilist_EDA4: Consts_B2 = 60836;
pub const addr_kSpacePirates_Ilist_ED80: Consts_B2 = 60800;
pub const addr_kSpacePirates_Ilist_ED36: Consts_B2 = 60726;
pub const addr_off_B2ECEC: Consts_B2 = 60652;
pub const addr_kSpacePirates_Ilist_ECE4: Consts_B2 = 60644;
pub const addr_kSpacePirates_Ilist_ECC0: Consts_B2 = 60608;
pub use crate::types::Consts_B4;
enemy_layout! {
    Enemy_SpacePirates {
        base: EnemyBase,
        words: [
            sps_var_A, sps_var_B, sps_var_C, sps_var_D, sps_var_E, sps_var_F,
            sps_parameter_1, sps_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            sps_var_00, sps_var_01, sps_var_02, sps_var_03, sps_var_04, sps_var_05,
            sps_var_06, sps_var_07, sps_var_08, sps_var_09, sps_var_0A, sps_var_0B,
            sps_var_0C, sps_var_0D, sps_var_0E, sps_var_0F, sps_var_10, sps_var_11,
            sps_var_12, sps_var_13, sps_var_14, sps_var_15, sps_var_16, sps_var_17,
            sps_var_18, sps_var_19, sps_var_1A, sps_var_1B, sps_var_1C, sps_var_1D,
            sps_var_1E, sps_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            sps_var_20, sps_var_21, sps_var_22, sps_var_23, sps_var_24, sps_var_25,
            sps_var_26, sps_var_27, sps_var_28, sps_var_29, sps_var_2A, sps_var_2B,
            sps_var_2C, sps_var_2D, sps_var_2E, sps_var_2F, sps_var_30, sps_var_31,
            sps_var_32, sps_var_33, sps_var_34, sps_var_35, sps_var_36, sps_var_37,
            sps_var_38, sps_var_39, sps_var_3A, sps_var_3B, sps_var_3C, sps_var_3D,
            sps_var_3E, sps_var_3F,
        ],
    }
}

#[inline]
unsafe fn RomPtr_A0(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn get_EnemyDef_A2(a: uint16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_Vulnerability(a: uint16, game: &mut Game) -> *mut Vulnerability {
    RomPtr((0xb40000u32) | ((a) as u32), game) as *mut Vulnerability
}
pub const fnnullsub_169_B2: ::core::ffi::c_int = 0xb2804b;
pub const fnSpacePirates_Func_1: ::core::ffi::c_int = 0xb2f034;
pub const fnnullsub_279: uint32 = 11726927;
pub const fnSpacePirates_Func_2: uint32 = 11726928;
pub const fnSpacePirates_Func_3: ::core::ffi::c_int = 0xb2f0c8;
pub const fnnullsub_281: uint32 = 11727075;
pub const fnSpacePirates_Func_4: uint32 = 11727076;
pub const fnSpacePirates_Func_5: uint32 = 11728553;
pub const fnSpacePirates_Func_6: uint32 = 11728612;
pub const fnSpacePirates_Func_11: uint32 = 11728919;
pub const fnSpacePirates_Func_12: ::core::ffi::c_int = 0xb2f84c;
pub const fnSpacePirates_Func_13: uint32 = 11729040;
pub const fnSpacePirates_Func_14: ::core::ffi::c_int = 0xb2f8c5;
pub const fnSpacePirates_Func_15: uint32 = 11729161;
pub const fnSpacePirates_F985: uint32 = 11729285;
pub const fnSpacePirates_F9C1: ::core::ffi::c_int = 0xb2f9c1;
pub const fnSpacePirates_FA15: ::core::ffi::c_int = 0xb2fa15;
pub const fnSpacePirates_FA59: uint32 = 11729497;
pub const fnSpacePirates_FA95: ::core::ffi::c_int = 0xb2fa95;
pub const fnSpacePirates_FAE9: ::core::ffi::c_int = 0xb2fae9;
pub const fnWalkingSpacePirates_FD44: uint32 = 11730244;
pub const fnWalkingSpacePirates_FDCE: uint32 = 11730382;
pub const fnnullsub_282: uint32 = 11730506;
#[inline]
unsafe fn gEnemyData(o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_SpacePirates(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_SpacePirates {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SpacePirates
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_B2(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_B2(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn WalkingSpacePirates_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn WalkingSpacePirates_Touch(game: &mut Game) {
    if (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .frozen_timer
        == 0
    {
        NormalEnemyTouchAi(game);
    }
}
pub unsafe fn WalkingSpacePirates_Shot(game: &mut Game) {
    if (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        == addr_kEnemyDef_F593 as ::core::ffi::c_int
    {
        WalkingSpacePirates_8789(game);
    } else {
        NormalEnemyShotAi(game);
    };
}
pub unsafe fn WalkingSpacePirates_8789(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*E).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*E).base.y_pos;
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    if (*E).base.health == 0 {
        let v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
        if (*E).base.enemy_ptr as ::core::ffi::c_int == addr_kEnemyDef_F593 as ::core::ffi::c_int {
            (*E).sps_var_B = 0 as uint16;
            EnemyDeathAnimation(v1, 4 as uint16, game);
            Enemy_ItemDrop_LowerNorfairSpacePirate(v1, game);
        } else {
            (*E).sps_var_B = 0 as uint16;
            EnemyDeathAnimation(v1, 4 as uint16, game);
        }
    }
}
pub unsafe fn WalkingSpacePirates_87C8(game: &mut Game) {
    let mut r18: uint16 = 0;
    let mut enemy_ptr: uint16 = 0;
    let mut vulnerability_ptr: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut current_block: u64;
    let mut v2: *mut Vulnerability = ::core::ptr::null_mut::<Vulnerability>();
    let mut v3: int16 = 0;
    let mut v: *mut Vulnerability = ::core::ptr::null_mut::<Vulnerability>();
    if ((*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        == addr_kEnemyDef_F593 as ::core::ffi::c_int)
    {
        r18 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize);
        if (r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int)
            - kProjectileType_PowerBomb as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            enemy_ptr = 0;
            vulnerability_ptr = 0;
            enemy_ptr = (*Get_SpacePirates(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .base
            .enemy_ptr;
            vulnerability_ptr = (*get_EnemyDef_A2(enemy_ptr, game)).vulnerability_ptr;
            if vulnerability_ptr == 0 {
                vulnerability_ptr = addr_stru_B4EC1C as uint16;
            }
            r20 = vulnerability_ptr;
            if (r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int) != 0i32
            {
                if r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
                    != kProjectileType_Missile as ::core::ffi::c_int
                    && r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
                        != kProjectileType_SuperMissile as ::core::ffi::c_int
                {
                    current_block = 1528265894032410192;
                } else {
                    v3 = (((r18) & ((kProjectileType_TypeMask) as u16)) as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int) as int16;
                    v = get_Vulnerability((r20).wrapping_add((v3) as u16), game);
                    if ((*v).plasma_ice_wave as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                        != 0i32
                        && ((*v).plasma_ice_wave as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                            != 15i32
                    {
                        current_block = 1528265894032410192;
                    } else {
                        current_block = 17407779659766490442;
                    }
                }
            } else {
                v2 = get_Vulnerability(
                    (r20 as ::core::ffi::c_int
                        + (r18 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
                        as uint16,
                    game,
                );
                if ((*v2).power as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) != 0i32
                    && ((*v2).power as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) != 15i32
                {
                    current_block = 1528265894032410192;
                } else {
                    current_block = 17407779659766490442;
                }
            }
            match current_block {
                1528265894032410192 => {}
                _ => {
                    WalkingSpacePirates_883E(game);
                    current_block = 15976848397966268834;
                }
            }
        } else {
            current_block = 15976848397966268834;
        }
        match current_block {
            1528265894032410192 => {}
            _ => {
                return;
            }
        }
    }
    WalkingSpacePirates_8789(game);
}
pub unsafe fn WalkingSpacePirates_883E(game: &mut Game) {
    if (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .enemy_ptr as ::core::ffi::c_int
        != addr_kEnemyDef_F593 as ::core::ffi::c_int
    {
        WalkingSpacePirates_8789(game);
        return;
    }
    let v0: uint16 =
        (2u16).wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16));
    let v1: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as ::core::ffi::c_int;
    let r18: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize);
    if r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int
        == kProjectileType_SuperMissile as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16).offset(v1 as isize)
            == 0
        {
            return;
        }
    } else if (r18 as ::core::ffi::c_int & kProjectileType_TypeMask as ::core::ffi::c_int)
        - kProjectileType_PowerBomb as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return;
    }
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .invincibility_timer = 10 as uint16;
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 7i32
    {
        v2 = 1 as uint16;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 2i32
    {
        v2 = 8 as uint16;
    } else {
        v2 = 5 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize) = v2;
    ProjectileReflection(v0, game);
    QueueSfx2_Max6(0x66 as uint16, game);
}
pub unsafe fn SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*jp.offset(0) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        return jp.offset(1);
    }
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).sps_var_C = ((*E).sps_var_C) ^ (1u16);
    if (*E).sps_var_C != 0 {
        return addr_off_B2ECEC as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    addr_kSpacePirates_Ilist_ED36 as ::core::ffi::c_int as uintptr_t as *const uint16
}
pub unsafe fn SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*jp.offset(0) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        return jp.offset(1);
    }
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).sps_var_C = ((*E).sps_var_C) ^ (1u16);
    if (*E).sps_var_C != 0 {
        return addr_kSpacePirates_Ilist_EDF6 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    addr_off_B2EDAC as ::core::ffi::c_int as uintptr_t as *const uint16
}
pub unsafe fn SpacePirates_Instr_RandomNewDirFaceR(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let result: uint16 = addr_kSpacePirates_Ilist_ED36 as uint16;
    let v4: uint16 = (NextRandom(game)) & (1u16);
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sps_var_C = v4;
    if v4 != 0 {
        return addr_off_B2ECEC as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    result as uintptr_t as *const uint16
}
pub unsafe fn SpacePirates_Instr_RandomNewDirFaceL(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let result: uint16 = addr_off_B2EDAC as uint16;
    let v4: uint16 = (NextRandom(game)) & (1u16);
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sps_var_C = v4;
    if v4 != 0 {
        return addr_kSpacePirates_Ilist_EDF6 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    result as uintptr_t as *const uint16
}
pub unsafe fn SpacePirates_Instr_PrepareWallJumpR(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).sps_var_B = ((*E).base.x_pos).wrapping_add((*E).sps_parameter_2);
    (*E).sps_var_D = ((*E).base.x_pos as ::core::ffi::c_int
        + ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
        as uint16;
    (*E).sps_var_E = (*E).base.y_pos;
    (*E).sps_var_F = 64 as uint16;
    jp
}
pub unsafe fn SpacePirates_Instr_PrepareWallJumpL(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).sps_var_B = ((*E).base.x_pos).wrapping_sub((*E).sps_parameter_2);
    (*E).sps_var_D = ((*E).base.x_pos as ::core::ffi::c_int
        - ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int))
        as uint16;
    (*E).sps_var_E = (*E).base.y_pos;
    (*E).sps_var_F = 192 as uint16;
    jp
}
pub unsafe fn SpacePirates_Instr_FireLaserL(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x1993) as *mut uint16) =
        *(RomPtr_A0((*E).base.enemy_ptr, game) as *mut uint16).offset(3);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_sub(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    game.eproj_spawn_r22 = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86A17B as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn SpacePirates_Instr_FireLaserR(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(16u16)),
    };
    game.eproj_spawn_r22 = 1 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86A17B as uint16, 1 as uint16, game);
    jp
}
pub unsafe fn SpacePirates_Instr_SetEnemyFunc(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sps_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn SpacePirates_Instr_PlaySfx(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x66 as uint16, game);
    jp
}
pub unsafe fn WallSpacePirates_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kSpacePirates_Ilist_ED36 as uint16;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_off_B2EDAC as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).sps_var_20 = 190 as uint16;
    (*E).sps_var_21 = 66 as uint16;
    (*E).sps_var_22 = 2 as uint16;
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        (*E).sps_var_20 = ((*E).sps_var_20).wrapping_add(2u16);
        (*E).sps_var_21 = ((*E).sps_var_21).wrapping_sub(2u16);
        (*E).sps_var_22 = ((*E).sps_var_22).wrapping_add(2u16);
    }
    let mut v2: uint16 = ((0xb2f034i32) as u16);
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        v2 = ((0xb2f0c8i32) as u16);
    }
    (*E).sps_var_A = v2;
    let mut v3: uint16 = 0;
    if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        - 11 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v3 = ((*E).base.x_pos) & (0xfff8u16);
    } else {
        v3 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.x_pos = v3;
}
pub unsafe fn CallSpacePiratesEnemyFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11698251 => (),
        11726900 => {
            SpacePirates_Func_1(k, game);
        }
        11726927 => (),
        11726928 => {
            SpacePirates_Func_2(k, game);
        }
        11727048 => {
            SpacePirates_Func_3(k, game);
        }
        11727075 => (),
        11730506 => (),
        11727076 => {
            SpacePirates_Func_4(k, game);
        }
        11728553 => {
            SpacePirates_Func_5(k, game);
        }
        11728612 => {
            SpacePirates_Func_6(k, game);
        }
        11728919 => {
            SpacePirates_Func_11(k, game);
        }
        11728972 => {
            SpacePirates_Func_12(k, game);
        }
        11729040 => {
            SpacePirates_Func_13(k, game);
        }
        11729093 => {
            SpacePirates_Func_14(k, game);
        }
        11729161 => {
            SpacePirates_Func_15(k, game);
        }
        11729285 => {
            SpacePirates_F985(k, game);
        }
        11729345 => {
            SpacePirates_F9C1(k, game);
        }
        11729429 => {
            SpacePirates_FA15(k, game);
        }
        11729497 => {
            SpacePirates_FA59(k, game);
        }
        11729557 => {
            SpacePirates_FA95(k, game);
        }
        11729641 => {
            SpacePirates_FAE9(k, game);
        }
        11730244 => {
            WalkingSpacePirates_FD44(game);
        }
        11730382 => {
            WalkingSpacePirates_FDCE(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn WallSpacePirates_Main(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSpacePiratesEnemyFunc(
        (((*v0).ai_var_A) as u32) | (0xb20000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn SpacePirates_Func_1(k: uint16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x20 as uint16,
        game,
    ) != 0
    {
        let E: *mut Enemy_SpacePirates = Get_SpacePirates(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_ED80 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).sps_var_D).wrapping_add(SineMult8bit(
        (*E).sps_var_F,
        ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    ));
    (*E).base.y_pos = ((*E).sps_var_E).wrapping_sub(CosineMult8bit(
        (*E).sps_var_F,
        ((*E).sps_parameter_2 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
        game,
    ));
    let v1: uint16 = ((*(&raw mut (*E).sps_var_F as *mut uint8).offset(0))
        .wrapping_sub(*(&raw mut (*E).sps_var_22 as *mut uint8).offset(0)))
        as uint16;
    (*E).sps_var_F = v1;
    if v1 == (*E).sps_var_20 {
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_EDA4 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v2: uint16 = 0;
        if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = ((*E).base.x_pos) & (0xfff8u16);
        } else {
            v2 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
                + 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v2;
    }
}
pub unsafe fn SpacePirates_Func_3(k: uint16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x20 as uint16,
        game,
    ) != 0
    {
        let E: *mut Enemy_SpacePirates = Get_SpacePirates(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_ECC0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.x_pos = ((*E).sps_var_D).wrapping_add(SineMult8bit(
        (*E).sps_var_F,
        ((*E).sps_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    ));
    (*E).base.y_pos = ((*E).sps_var_E).wrapping_sub(CosineMult8bit(
        (*E).sps_var_F,
        ((*E).sps_parameter_2 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16,
        game,
    ));
    let v1: uint16 = ((*(&raw mut (*E).sps_var_22 as *mut uint8).offset(0))
        .wrapping_add(*(&raw mut (*E).sps_var_F as *mut uint8).offset(0)))
        as uint16;
    (*E).sps_var_F = v1;
    if v1 == (*E).sps_var_21 {
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_ECE4 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v2: uint16 = 0;
        if ((*E).base.x_pos as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            - 11 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = ((*E).base.x_pos) & (0xfff8u16);
        } else {
            v2 = (((*E).base.x_pos as ::core::ffi::c_int & 0xfff0 as ::core::ffi::c_int)
                + 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v2;
    }
}
pub unsafe fn SpacePirates_Instr_20(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .palette_index = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn SpacePirates_Instr_16(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(*jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn sub_B2F554(game: &mut Game) {
    (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .instruction_timer = 1 as uint16;
}
pub unsafe fn SpacePirates_Instr_15(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_rect = Rect16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
        w: *jp.offset(1),
        h: *jp.offset(2),
    };
    SpawnEprojWithGfx(
        *jp.offset(0),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_PirateClaw as uint16,
        game,
    );
    jp.offset(3)
}
pub unsafe fn SpacePirates_Instr_18(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).sps_var_C = (if ((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        addr_kSpacePirates_Ilist_F420 as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F22E as ::core::ffi::c_int
    }) as uint16;
    (*E).sps_var_C as uintptr_t as *const uint16
}
pub unsafe fn sub_B2F5B3(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).instruction_timer = 1 as uint16;
    (*v0).ai_var_C = (if ((*v0).x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        addr_kSpacePirates_Ilist_F51A as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F32E as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn SpacePirates_Instr_17(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_SpacePirates(k as ::core::ffi::c_int, game)).sps_var_00 = 0 as uint16;
    jp
}
pub unsafe fn NinjaSpacePirates_Init(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction =
        (if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            addr_kSpacePirates_Ilist_F4CC as ::core::ffi::c_int
        } else {
            addr_kSpacePirates_Ilist_F2DA as ::core::ffi::c_int
        }) as uint16;
    (*E).sps_var_C = (*E).base.current_instruction;
    let x_pos: uint16 = (*E).base.x_pos;
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        (*E).sps_var_E = x_pos;
        (*E).sps_var_F = ((*E).sps_parameter_2).wrapping_add(x_pos);
    } else {
        (*E).sps_var_F = x_pos;
        (*E).sps_var_E = (x_pos).wrapping_sub((*E).sps_parameter_2);
    }
    let mut r20: uint16 = ((((*E).sps_var_F).wrapping_sub((*E).sps_var_E)) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_D = ((*E).sps_var_E).wrapping_add(r20);
    let mut r18: uint16 = 0 as uint16;
    let mut R22: uint16 = 0 as uint16;
    r20 = ((r20 as uint8 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    loop {
        r18 = (r18).wrapping_add(32u16);
        R22 = (R22).wrapping_add(r18);
        if (R22 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0)
        {
            break;
        }
    }
    (*E).sps_var_B = r18;
    let R24: uint16 = ((R22 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int) as uint16;
    (*E).sps_var_F = ((*E).sps_var_D).wrapping_add(R24);
    (*E).sps_var_E = ((*E).sps_var_D).wrapping_sub(R24);
    let mut sps_var_E: uint16 = (*E).sps_var_E;
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        sps_var_E = (*E).sps_var_F;
    }
    (*E).base.x_pos = sps_var_E;
    (*E).sps_var_A = ((0xb2804bi32) as u16);
    (*E).sps_var_08 = (*E).base.y_pos;
    let mut v6: uint16 = 0 as uint16;
    let mut v7: uint16 = 0 as uint16;
    let mut n: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16).offset(
            ((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 240 as ::core::ffi::c_int)
                as isize,
        ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb28727)) as *const uint8 as *mut uint16)
            .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        v6 = (v6).wrapping_add(2u16);
        v7 = (v7).wrapping_add(2u16);
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn NinjaSpacePirates_Main(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSpacePiratesEnemyFunc(
        (((*E).sps_var_A) as u32) | (0xb20000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn SpacePirates_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        ((*E).base.x_pos)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    )) as i16)
        .wrapping_sub(128i16)
        >= 0i16
    {
        SpacePirates_Func_8(game);
    } else {
        (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_kSpacePirates_Ilist_F420 as ::core::ffi::c_int
        } else {
            addr_kSpacePirates_Ilist_F22E as ::core::ffi::c_int
        }) as uint16;
        (*E).sps_var_C = (*E).base.current_instruction;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn SpacePirates_Func_6(k: uint16, game: &mut Game) {
    if SpacePirates_Func_8(game) == 0
        && SpacePirates_Func_10(game) == 0
        && SpacePirates_Func_9(game) == 0
    {
        SpacePirates_Func_7(k, game);
    }
}
pub unsafe fn SpacePirates_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    if ((*E).base.frame_counter as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) == 0i32 {
        if (*E).base.x_pos == (*E).sps_var_E {
            if (((*E).base.x_pos) as i16).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
            ) < 0i16
            {
                return;
            }
            (*E).base.current_instruction = addr_kSpacePirates_Ilist_F15C as uint16;
        } else {
            if (((*E).base.x_pos) as i16).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
            ) >= 0i16
            {
                return;
            }
            (*E).base.current_instruction = addr_kSpacePirates_Ilist_F34A as uint16;
        }
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Func_8(game: &mut Game) -> uint16 {
    let mut v0: uint16 = 8 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 = (v0).wrapping_sub(2u16);
        if (v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return 0 as uint16;
        }
    }
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize))
            .wrapping_sub((*E).base.x_pos),
    )) as i16)
        .wrapping_sub(32i16)
        >= 0i16
        || ((abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize))
            .wrapping_sub((*E).base.y_pos),
        )) as i16)
            .wrapping_sub(32i16)
            >= 0i16
    {
        return 0 as uint16;
    }
    (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        addr_kSpacePirates_Ilist_F462 as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F270 as ::core::ffi::c_int
    }) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    1 as uint16
}
pub unsafe fn SpacePirates_Func_9(game: &mut Game) -> uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        ((*E).sps_var_D)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    )) as i16)
        .wrapping_sub(32i16)
        >= 0i16
    {
        return 0 as uint16;
    }
    let mut v1: uint16 = addr_kSpacePirates_Ilist_F1C4 as uint16;
    if (*E).base.x_pos == (*E).sps_var_E {
        v1 = addr_kSpacePirates_Ilist_F3B2 as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    1 as uint16
}
pub unsafe fn SpacePirates_Func_10(game: &mut Game) -> uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_sub((*E).base.x_pos),
    )) as i16)
        .wrapping_sub(40i16)
        >= 0i16
        || ((abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub((*E).base.y_pos),
        )) as i16)
            .wrapping_sub(40i16)
            >= 0i16
    {
        return 0 as uint16;
    }
    (*E).base.current_instruction = (if ((*E).base.x_pos as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        addr_kSpacePirates_Ilist_F51A as ::core::ffi::c_int
    } else {
        addr_kSpacePirates_Ilist_F32E as ::core::ffi::c_int
    }) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    1 as uint16
}
pub unsafe fn SpacePirates_Func_11(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).sps_var_00 = ((*E).sps_var_00).wrapping_add(32u16);
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sps_var_D) as i16) < 0i16 {
        (*E).sps_var_A = ((0xb2f84ci32) as u16);
    }
}
pub unsafe fn SpacePirates_Func_12(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let v2: uint16 = ((*E).sps_var_00).wrapping_sub(32u16);
    (*E).sps_var_00 = v2;
    if v2 == 0 {
        (*E).sps_var_A = ((0xb2804bi32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F2F8 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = (*E).sps_var_E;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_Func_13(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).sps_var_00 = ((*E).sps_var_00).wrapping_add(32u16);
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sps_var_D) as i16) >= 0i16 {
        (*E).sps_var_A = ((0xb2f8c5i32) as u16);
    }
}
pub unsafe fn SpacePirates_Func_14(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let v2: uint16 = ((*E).sps_var_00).wrapping_sub(32u16);
    (*E).sps_var_00 = v2;
    if v2 == 0 {
        (*E).sps_var_A = ((0xb2804bi32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F4EA as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = (*E).sps_var_F;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_Func_15(k: uint16, game: &mut Game) {
    if SpacePirates_Func_8(game) == 0 && SpacePirates_Func_10(game) == 0 {
        SpacePirates_Func_16(game);
    }
}
pub unsafe fn SpacePirates_Func_16(game: &mut Game) {
    let mut v3: int16 = 0;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        ((*E).sps_var_D)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    )) as i16)
        .wrapping_sub(32i16)
        < 0i16
    {
        let mut v2: uint16 = 0;
        loop {
            v2 = (NextRandom(game)) & (3u16);
            if (v2 != 0) {
                break;
            }
        }
        v3 = 0 as int16;
        if (*E).base.x_pos == (*E).sps_var_E {
            v3 = 4 as int16;
        }
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb2f959))
            as *const uint8 as *mut uint16)
            .offset((v2 as ::core::ffi::c_int + v3 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn SpacePirates_Instr_19(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).sps_var_00 = 1536 as uint16;
    (*E).sps_var_03 = ((*E).sps_var_D as ::core::ffi::c_int
        + ((((*E).sps_var_F).wrapping_sub((*E).sps_var_D)) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn SpacePirates_F985(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    v2 = (((*E).sps_var_00) as i16).wrapping_sub(64i16);
    (*E).sps_var_00 = v2 as uint16;
    if v2 < 0i16 {
        (*E).sps_var_A = ((0xb2f9c1i32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F2A0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_00 = 1536 as uint16;
    }
}
pub unsafe fn SpacePirates_F9C1(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(5u16);
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint32)
            << 16 as ::core::ffi::c_int
            | *(&raw mut (*E).sps_var_00 as *mut uint8).offset(0) as uint16 as uint32)
            as int32,
        game,
    ) != 0u8
        || {
            v2 = (((*E).sps_var_00) as i16).wrapping_sub(64i16);
            (*E).sps_var_00 = v2 as uint16;
            v2 < 0i16
        }
        || (v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) == 0i32
    {
        (*E).sps_var_A = ((0xb2fa15i32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F2B2 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.y_pos = (*E).sps_var_08;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_FA15(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).base.x_pos).wrapping_sub(2u16);
    (*E).base.x_pos = v2;
    if ((v2) as i16).wrapping_sub(((*E).sps_var_E) as i16) < 0i16 {
        (*E).base.x_pos = (*E).sps_var_E;
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F2F8 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_A = ((0xb2804bi32) as u16);
    }
}
pub unsafe fn SpacePirates_Instr_21(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).sps_var_00 = 1536 as uint16;
    (*E).sps_var_03 = ((*E).sps_var_E as ::core::ffi::c_int
        + ((((*E).sps_var_D).wrapping_sub((*E).sps_var_E)) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn SpacePirates_FA59(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(
        (*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    v2 = (((*E).sps_var_00) as i16).wrapping_sub(64i16);
    (*E).sps_var_00 = v2 as uint16;
    if v2 < 0i16 {
        (*E).sps_var_A = ((0xb2fa95i32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F492 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_00 = 1536 as uint16;
    }
}
pub unsafe fn SpacePirates_FA95(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(5u16);
    if Enemy_MoveDown(
        k,
        ((*(&raw mut (*E).sps_var_00 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint32)
            << 16 as ::core::ffi::c_int
            | *(&raw mut (*E).sps_var_00 as *mut uint8).offset(0) as uint16 as uint32)
            as int32,
        game,
    ) != 0u8
        || {
            v2 = (((*E).sps_var_00) as i16).wrapping_sub(64i16);
            (*E).sps_var_00 = v2 as uint16;
            v2 < 0i16
        }
        || (v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) == 0i32
    {
        (*E).sps_var_A = ((0xb2fae9i32) as u16);
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F4A4 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.y_pos = (*E).sps_var_08;
        SpacePirates_FB11(k, game);
    }
}
pub unsafe fn SpacePirates_FAE9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).base.x_pos).wrapping_add(2u16);
    (*E).base.x_pos = v2;
    if ((v2) as i16).wrapping_sub(((*E).sps_var_F) as i16) >= 0i16 {
        (*E).base.x_pos = (*E).sps_var_F;
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_F4EA as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).sps_var_A = ((0xb2804bi32) as u16);
    }
}
pub unsafe fn SpacePirates_FB11(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(k as ::core::ffi::c_int, game);
    CreateSpriteAtPos(
        ((*E).base.x_pos).wrapping_sub(8u16),
        ((*E).base.y_pos).wrapping_add(28u16),
        10 as uint16,
        0 as uint16,
        game,
    );
    CreateSpriteAtPos(
        ((*E).base.x_pos).wrapping_add(8u16),
        ((*E).base.y_pos).wrapping_add(28u16),
        10 as uint16,
        0 as uint16,
        game,
    );
}
pub unsafe fn SpacePirates_Instr_12(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_sub(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(*jp.offset(0))),
    };
    game.eproj_spawn_r22 = 0 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86A17B as uint16, 0 as uint16, game);
    jp.offset(1)
}
pub unsafe fn SpacePirates_Instr_14(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(24u16)),
        y: (((*E).base.y_pos).wrapping_sub(*jp.offset(0))),
    };
    game.eproj_spawn_r22 = 1 as uint16;
    SpawnEprojWithRoomGfx(addr_stru_86A17B as uint16, 1 as uint16, game);
    jp.offset(1)
}
pub unsafe fn SpacePirates_Instr_11(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .sps_var_A = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn SpacePirates_Instr_13(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x10 as uint16,
        game,
    ) != 0
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(
                ((*Get_SpacePirates(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .x_pos) as i16,
            )
            >= 0i16
        {
            return addr_kSpacePirates_Ilist_FC0E as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
        addr_kSpacePirates_Ilist_FB8C as ::core::ffi::c_int as uintptr_t as *const uint16
    } else {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(
                ((*Get_SpacePirates(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .x_pos) as i16,
            )
            >= 0i16
        {
            return addr_kSpacePirates_Ilist_FB64 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
        addr_kSpacePirates_Ilist_FBE6 as ::core::ffi::c_int as uintptr_t as *const uint16
    }
}
pub unsafe fn WalkingSpacePirates_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kSpacePirates_Ilist_FB64 as uint16;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_kSpacePirates_Ilist_FBE6 as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).sps_var_A = ((0xb2804bi32) as u16);
    (*E).sps_var_F = ((*E).sps_parameter_2).wrapping_add((*E).base.x_pos);
    (*E).sps_var_E = ((*E).base.x_pos).wrapping_sub((*E).sps_parameter_2);
}
pub unsafe fn WalkingSpacePirates_Main(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSpacePiratesEnemyFunc(
        (((*E).sps_var_A) as u32) | (0xb20000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    if ((*E).sps_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        WalkingSpacePirates_FE4B(game);
    }
}
pub unsafe fn WalkingSpacePirates_FD44(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x10 as uint16,
        game,
    ) != 0
    {
        let mut v1: uint16 = addr_kSpacePirates_Ilist_FB8C as uint16;
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            >= 0i16
        {
            v1 = addr_kSpacePirates_Ilist_FC0E as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
    } else if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (1 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).sps_var_00 = x_pos;
        (*E).base.x_pos = (x_pos).wrapping_sub(17u16);
        let result: uint8 = Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
        (*E).base.x_pos = (*E).sps_var_00;
        if result != 0 {
            EnemyFunc_BBBF(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                (-(9 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
                game,
            );
            if Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                -(14337 as int32),
                game,
            ) == 0
            {
                if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sps_var_E) as i16) >= 0i16 {
                    return;
                }
            }
        }
        (*E).base.current_instruction = addr_kSpacePirates_Ilist_FBC6 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn WalkingSpacePirates_FDCE(game: &mut Game) {
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x10 as uint16,
        game,
    ) != 0
    {
        let mut v1: uint16 = addr_kSpacePirates_Ilist_FB8C as uint16;
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            >= 0i16
        {
            v1 = addr_kSpacePirates_Ilist_FC0E as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
    } else if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (1 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).sps_var_00 = x_pos;
        (*E).base.x_pos = (x_pos).wrapping_add(16u16);
        let result: uint8 = Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
        (*E).base.x_pos = (*E).sps_var_00;
        if result == 0
            || Enemy_MoveRight_IgnoreSlopes(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                14336 as int32,
                game,
            ) != 0u8
            || (((*E).base.x_pos) as i16).wrapping_sub(((*E).sps_var_F) as i16) >= 0i16
        {
            (*E).base.current_instruction = addr_kSpacePirates_Ilist_FC48 as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    }
}
pub unsafe fn WalkingSpacePirates_FE4B(game: &mut Game) {
    let mut v0: uint16 = 8 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 0
    {
        v0 = (v0).wrapping_sub(2u16);
        if (v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return;
        }
    }
    let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let E: *mut Enemy_SpacePirates = Get_SpacePirates(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize))
            .wrapping_sub((*E).base.x_pos),
    )) as i16)
        .wrapping_sub(32i16)
        < 0i16
        && ((abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize))
            .wrapping_sub((*E).base.y_pos),
        )) as i16)
            .wrapping_sub(32i16)
            < 0i16
    {
        (*E).base.current_instruction = (if (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            addr_kSpacePirates_Ilist_FB58 as ::core::ffi::c_int
        } else {
            addr_kSpacePirates_Ilist_FB4C as ::core::ffi::c_int
        }) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
