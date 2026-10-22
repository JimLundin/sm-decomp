pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::prelude::*;

use crate::game::sm_80::{NextRandom, QueueSfx2_Max6};
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_90::ProjectileReflection;
use crate::game::sm_a0::{
    CosineMult8bit, EnemyDeathAnimation, EnemyFunc_BBBF, Enemy_ItemDrop_LowerNorfairSpacePirate,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SineMult8bit,
};
use crate::game::sprite_objects::create_sprite;

pub use crate::types::Consts_B2;

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
pub unsafe fn WalkingSpacePirates_87C8(game: &mut Game) {
    if ((*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
        .base
        .enemy_ptr as i32
        == addr_kEnemyDef_F593 as i32)
    {
        let r18: u16 = word!(game.projectile_type[word!(game.collision_detection_index)]);
        if (r18 as i32 & kProjectileType_TypeMask as i32) - kProjectileType_PowerBomb as i32
            & 0x8000
            == 0
        {
            return;
        }
        let enemy_ptr = (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
            .base
            .enemy_ptr;
        let mut vulnerability_ptr = (*get_EnemyDef_A2(enemy_ptr, game)).vulnerability_ptr;
        if vulnerability_ptr == 0 {
            vulnerability_ptr = addr_stru_B4EC1C as u16;
        }
        let r20 = vulnerability_ptr;
        let vulnerable = if (r18 as i32 & kProjectileType_TypeMask as i32) != 0 {
            if r18 as i32 & kProjectileType_TypeMask as i32 != kProjectileType_Missile as i32
                && r18 as i32 & kProjectileType_TypeMask as i32
                    != kProjectileType_SuperMissile as i32
            {
                true
            } else {
                let v3: i16 = (((r18) & ((kProjectileType_TypeMask) as u16)) >> 8) as i16;
                let v = get_Vulnerability((r20).wrapping_add((v3) as u16), game);
                ((*v).plasma_ice_wave & 0xf) != 0 && ((*v).plasma_ice_wave & 0xf) != 15
            }
        } else {
            let v2 = get_Vulnerability((r20 as i32 + (r18 as i32 & 0xf)) as u16, game);
            ((*v2).power & 0xf) != 0 && ((*v2).power & 0xf) != 15
        };
        if !vulnerable {
            WalkingSpacePirates_883E(game);
            return;
        }
    }
    WalkingSpacePirates_8789(game);
}
pub unsafe fn WalkingSpacePirates_883E(game: &mut Game) {
    if (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
        .base
        .enemy_ptr as i32
        != addr_kEnemyDef_F593 as i32
    {
        WalkingSpacePirates_8789(game);
        return;
    }
    let v0: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
    let v1: i32 = word!(game.collision_detection_index) as i32;
    let r18: u16 = word!(game.projectile_type[v1]);
    if r18 as i32 & kProjectileType_TypeMask as i32 == kProjectileType_SuperMissile as i32 {
        if word!(game.projectile_variables[v1]) == 0 {
            return;
        }
    } else if (r18 as i32 & kProjectileType_TypeMask as i32) - kProjectileType_PowerBomb as i32
        & 0x8000
        == 0
    {
        return;
    }
    (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
        .base
        .invincibility_timer = 10;
    let mut v2: u16 = if (word!(game.projectile_dir[v1]) & 0xf) == 7 {
        1
    } else if (word!(game.projectile_dir[v1]) & 0xf) == 2 {
        8
    } else {
        5
    };
    word!(game.projectile_dir[v1] = v2);
    ProjectileReflection(v0, game);
    QueueSfx2_Max6(0x66, game);
}
pub unsafe fn SpacePirates_Func_8(game: &mut Game) -> u16 {
    let mut v0: u16 = 8;
    while word!(game.projectile_type[(v0 >> 1)]) == 0 {
        v0 = (v0).wrapping_sub(2);
        if (v0 & 0x8000) != 0 {
            return 0;
        }
    }
    let v1: i32 = v0 as i32 >> 1;
    let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
    if ((abs16((word!(game.projectile_x_pos[v1])).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub(32)
        >= 0
        || ((abs16((word!(game.projectile_y_pos[v1])).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(32)
            >= 0
    {
        return 0;
    }
    enemy_instruction! {[(*E).base]; instruction = (if (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 & 0x8000 != 0 {
        addr_kSpacePirates_Ilist_F462 as i32
    } else {
        addr_kSpacePirates_Ilist_F270 as i32
    }) as u16, timer = 1}
    1
}
pub unsafe fn SpacePirates_Func_9(game: &mut Game) -> u16 {
    let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
    if ((abs16(((*E).var_D).wrapping_sub(word!(game.samus_x_pos)))) as i16).wrapping_sub(32) >= 0 {
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
    let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
    if ((abs16((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos))) as i16).wrapping_sub(40)
        >= 0
        || ((abs16((word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(40)
            >= 0
    {
        return 0;
    }
    enemy_instruction! {[(*E).base]; instruction = (if ((*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 & 0x8000) != 0 {
        addr_kSpacePirates_Ilist_F51A as i32
    } else {
        addr_kSpacePirates_Ilist_F32E as i32
    }) as u16, timer = 1}
    1
}
macro_rules! finish_space_pirate_vertical_jump {
    (
        $enemy_index:ident, $enemy:ident, $game:ident, $velocity:ident, $next_state:literal,
        $instruction:ident $(,)?
    ) => {
        if Enemy_MoveDown(
            $enemy_index,
            (words!((
                byte!((*$enemy).var_00; 1),
                byte!((*$enemy).var_00; 0) as u16
            ))) as i32,
            $game,
        ) != 0
            || {
                $velocity = (((*$enemy).var_00) as i16).wrapping_sub(64);
                (*$enemy).var_00 = $velocity as u16;
                $velocity < 0
            }
            || ($velocity as u16 & 0xff00) == 0
        {
            (*$enemy).var_A = (($next_state) as u16);
            enemy_instruction! {[(*$enemy).base]; instruction = $instruction as u16, timer = 1}
            (*$enemy).base.y_pos = (*$enemy).var_08;
            SpacePirates_FB11($enemy_index, $game);
        }
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallSpacePiratesEnemyFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11726900] SpacePirates_Func_1 [] {
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x20, game) != 0 {
            let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ED80 as u16, timer = 1}
        }
    };
    [11726928] SpacePirates_Func_2 [] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos =
            ((*E).var_D).wrapping_add(SineMult8bit((*E).var_F, ((*E).parameter_2 >> 1), game));
        (*E).base.y_pos =
            ((*E).var_E).wrapping_sub(CosineMult8bit((*E).var_F, ((*E).parameter_2 >> 2), game));
        let v1: u16 = ((byte!((*E).var_F; 0)).wrapping_sub(byte!((*E).var_22; 0))) as u16;
        (*E).var_F = v1;
        if v1 == (*E).var_20 {
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_EDA4 as u16, timer = 1}
            let mut v2: u16 = if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
                ((*E).base.x_pos) & (0xfff8u16)
            } else {
                (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16
            };
            (*E).base.x_pos = v2;
        }
    };
    [11727048] SpacePirates_Func_3 [] {
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x20, game) != 0 {
            let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ECC0 as u16, timer = 1}
        }
    };
    [11727076] SpacePirates_Func_4 [] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).base.x_pos =
            ((*E).var_D).wrapping_add(SineMult8bit((*E).var_F, ((*E).parameter_2 >> 1), game));
        (*E).base.y_pos =
            ((*E).var_E).wrapping_sub(CosineMult8bit((*E).var_F, ((*E).parameter_2 >> 2), game));
        let v1: u16 = ((byte!((*E).var_22; 0)).wrapping_add(byte!((*E).var_F; 0))) as u16;
        (*E).var_F = v1;
        if v1 == (*E).var_21 {
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_ECE4 as u16, timer = 1}
            let mut v2: u16 = if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
                ((*E).base.x_pos) & (0xfff8u16)
            } else {
                (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16
            };
            (*E).base.x_pos = v2;
        }
    };
    [11728553] SpacePirates_Func_5 [] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        if ((abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos)))) as i16)
            .wrapping_sub(128)
            >= 0
        {
            SpacePirates_Func_8(game);
        } else {
            (*E).base.current_instruction =
                (if (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 & 0x8000 != 0 {
                    addr_kSpacePirates_Ilist_F420 as i32
                } else {
                    addr_kSpacePirates_Ilist_F22E as i32
                }) as u16;
            (*E).var_C = (*E).base.current_instruction;
            (*E).base.instruction_timer = 1;
        };
    };
    [11728612] SpacePirates_Func_6 [] {
        if SpacePirates_Func_8(game) == 0
            && SpacePirates_Func_10(game) == 0
            && SpacePirates_Func_9(game) == 0
        {
            SpacePirates_Func_7(k, game);
        }
    };
    [11729161] SpacePirates_Func_15 [] {
        if SpacePirates_Func_8(game) == 0 && SpacePirates_Func_10(game) == 0 {
            SpacePirates_Func_16(game);
        }
    };
    [11729345] SpacePirates_F9C1 [] {
        let mut v2: i16 = 0;
        let E = Get_SpacePirates(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(5);
        finish_space_pirate_vertical_jump!(
            k,
            E,
            game,
            v2,
            0xb2fa15i32,
            addr_kSpacePirates_Ilist_F2B2
        );
    };
    [11729557] SpacePirates_FA95 [] {
        let mut v2: i16 = 0;
        let E = Get_SpacePirates(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(5);
        finish_space_pirate_vertical_jump!(
            k,
            E,
            game,
            v2,
            0xb2fae9i32,
            addr_kSpacePirates_Ilist_F4A4
        );
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_SpacePirates(k as i32, game); };
    SpacePirates_Func_7 [] {
        if ((*E).base.frame_counter & 0x3f) == 0 {
            if (*E).base.x_pos == (*E).var_E {
                if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
                    return;
                }
                (*E).base.current_instruction = addr_kSpacePirates_Ilist_F15C as u16;
            } else {
                if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
                    return;
                }
                (*E).base.current_instruction = addr_kSpacePirates_Ilist_F34A as u16;
            }
            (*E).base.instruction_timer = 1;
        }
    };
    [11728919] SpacePirates_Func_11 [] {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub((byte!((*E).var_00; 1)) as u16);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).var_00 = ((*E).var_00).wrapping_add(32);
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_D) as i16) < 0 {
            (*E).var_A = ((0xb2f84ci32) as u16);
        }
    };
    [11728972] SpacePirates_Func_12 [] {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub((byte!((*E).var_00; 1)) as u16);
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
    };
    [11729040] SpacePirates_Func_13 [] {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((byte!((*E).var_00; 1)) as u16);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).var_00 = ((*E).var_00).wrapping_add(32);
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_D) as i16) >= 0 {
            (*E).var_A = ((0xb2f8c5i32) as u16);
        }
    };
    [11729093] SpacePirates_Func_14 [] {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((byte!((*E).var_00; 1)) as u16);
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
    };
    [11729285] SpacePirates_F985 [] {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((byte!((*E).var_00; 1)) as u16);
        let mut v2: i16 = (((*E).var_00) as i16).wrapping_sub(64);
        (*E).var_00 = v2 as u16;
        if v2 < 0 {
            (*E).var_A = ((0xb2f9c1i32) as u16);
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2A0 as u16, timer = 1}
            (*E).var_00 = 1536;
        }
    };
    [11729429] SpacePirates_FA15 [] {
        let v2: u16 = ((*E).base.x_pos).wrapping_sub(2);
        (*E).base.x_pos = v2;
        if ((v2) as i16).wrapping_sub(((*E).var_E) as i16) < 0 {
            (*E).base.x_pos = (*E).var_E;
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F2F8 as u16, timer = 1}
            (*E).var_A = ((0xb2804bi32) as u16);
        }
    };
    [11729497] SpacePirates_FA59 [] {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((byte!((*E).var_00; 1)) as u16);
        let mut v2: i16 = (((*E).var_00) as i16).wrapping_sub(64);
        (*E).var_00 = v2 as u16;
        if v2 < 0 {
            (*E).var_A = ((0xb2fa95i32) as u16);
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F492 as u16, timer = 1}
            (*E).var_00 = 1536;
        }
    };
    [11729641] SpacePirates_FAE9 [] {
        let v2: u16 = ((*E).base.x_pos).wrapping_add(2);
        (*E).base.x_pos = v2;
        if ((v2) as i16).wrapping_sub(((*E).var_F) as i16) >= 0 {
            (*E).base.x_pos = (*E).var_F;
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_F4EA as u16, timer = 1}
            (*E).var_A = ((0xb2804bi32) as u16);
        }
    };
    SpacePirates_FB11 [] {
        create_sprite(
            ((*E).base.x_pos).wrapping_sub(8),
            ((*E).base.y_pos).wrapping_add(28),
            10,
            0,
            game,
        );
        create_sprite(
            ((*E).base.x_pos).wrapping_add(8),
            ((*E).base.y_pos).wrapping_add(28),
            10,
            0,
            game,
        );
    };
    @external {
        11698251 => [()]; 11726927 => [()];
        11727075 => [()];
        11730506 => [()];
        11730244 => [WalkingSpacePirates_FD44(game)]; 11730382 => [WalkingSpacePirates_FDCE(game)];
    }
}
pub unsafe fn WalkingSpacePirates_FE4B(game: &mut Game) {
    let mut v0: u16 = 8;
    while word!(game.projectile_type[(v0 >> 1)]) == 0 {
        v0 = (v0).wrapping_sub(2);
        if (v0 & 0x8000) != 0 {
            return;
        }
    }
    let v1: i32 = v0 as i32 >> 1;
    let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
    if ((abs16((word!(game.projectile_x_pos[v1])).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub(32)
        < 0
        && ((abs16((word!(game.projectile_y_pos[v1])).wrapping_sub((*E).base.y_pos))) as i16)
            .wrapping_sub(32)
            < 0
    {
        enemy_instruction! {[(*E).base]; instruction = (if (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 & 0x8000 != 0 {
            addr_kSpacePirates_Ilist_FB58 as i32
        } else {
            addr_kSpacePirates_Ilist_FB4C as i32
        }) as u16, timer = 1}
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11700076] WalkingSpacePirates_Touch [] {
        if (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
            .base
            .frozen_timer
            == 0
        {
            NormalEnemyTouchAi(game);
        }
    };
    [11700089] WalkingSpacePirates_Shot [] {
        if (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
            .base
            .enemy_ptr as i32
            == addr_kEnemyDef_F593 as i32
        {
            WalkingSpacePirates_8789(game);
        } else {
            NormalEnemyShotAi(game);
        };
    };
    [11726751] WallSpacePirates_Init [] {
        let mut v0: u16 = addr_kSpacePirates_Ilist_ED36 as u16;
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
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
        let mut v3: u16 = if ((*E).base.x_pos as i32 & 0xf) - 11i32 & 0x8000 != 0 {
            ((*E).base.x_pos) & (0xfff8u16)
        } else {
            (((*E).base.x_pos as i32 & 0xfff0) + 16) as u16
        };
        (*E).base.x_pos = v3;
    };
    [11726893] WallSpacePirates_Main [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        CallSpacePiratesEnemyFunc(
            (((*v0).ai_var_A) as u32) | (0xb20000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11730178] WalkingSpacePirates_Init [] {
        let mut v0: u16 = addr_kSpacePirates_Ilist_FB64 as u16;
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        if ((*E).parameter_1 & 1) != 0 {
            v0 = addr_kSpacePirates_Ilist_FBE6 as u16;
        }
        (*E).base.current_instruction = v0;
        (*E).var_A = ((0xb2804bi32) as u16);
        (*E).var_F = ((*E).parameter_2).wrapping_add((*E).base.x_pos);
        (*E).var_E = ((*E).base.x_pos).wrapping_sub((*E).parameter_2);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game); };
    WalkingSpacePirates_8789 [] {
        word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
        word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        if (*E).base.health == 0 {
            let v1: u16 = word!(game.cur_enemy_index);
            if (*E).base.enemy_ptr as i32 == addr_kEnemyDef_F593 as i32 {
                (*E).var_B = 0;
                EnemyDeathAnimation(v1, 4, game);
                Enemy_ItemDrop_LowerNorfairSpacePirate(v1, game);
            } else {
                (*E).var_B = 0;
                EnemyDeathAnimation(v1, 4, game);
            }
        }
    };
    [11728350] NinjaSpacePirates_Init [] {
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
        do_until! { (R22 as i32 - r20 as i32 & 0x8000 == 0);
    r18 = (r18).wrapping_add(32);
    R22 = (R22).wrapping_add(r18);
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
        word_transfers!(game, ascending, 16; native(0xc3e0) <- rom(0xb28727, 0));
    };
    [11728546] NinjaSpacePirates_Main [] {
        CallSpacePiratesEnemyFunc(
            (((*E).var_A) as u32) | (0xb20000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    SpacePirates_Func_16 [] {
        if ((abs16(((*E).var_D).wrapping_sub(word!(game.samus_x_pos)))) as i16).wrapping_sub(32) < 0 {
            let mut v2: u16 = 0;
            do_until! { (v2 != 0);
    v2 = (NextRandom(game)) & (3u16);
}
            let mut v3: i16 = 0;
            if (*E).base.x_pos == (*E).var_E {
                v3 = 4;
            }
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xb2f959, (v2 as i32 + v3 as i32) as isize), timer = 1}
        }
    };
    [11730226] WalkingSpacePirates_Main [] {
        CallSpacePiratesEnemyFunc(
            (((*E).var_A) as u32) | (0xb20000u32),
            word!(game.cur_enemy_index),
            game,
        );
        if ((*E).parameter_1 & 0x8000) != 0 {
            WalkingSpacePirates_FE4B(game);
        }
    };
    WalkingSpacePirates_FD44 [] {
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x10, game) != 0 {
            let mut v1: u16 = addr_kSpacePirates_Ilist_FB8C as u16;
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0 {
                v1 = addr_kSpacePirates_Ilist_FC0E as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        } else if Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game) != 0 {
            let x_pos: u16 = (*E).base.x_pos;
            (*E).var_00 = x_pos;
            (*E).base.x_pos = (x_pos).wrapping_sub(17);
            let result: u8 = Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game);
            (*E).base.x_pos = (*E).var_00;
            if result != 0 {
                EnemyFunc_BBBF(word!(game.cur_enemy_index), (-9i16 as i32) << 16, game);
                if Enemy_MoveRight_IgnoreSlopes(word!(game.cur_enemy_index), -14337i32, game) == 0
                    && (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_E) as i16) >= 0
                {
                    return;
                }
            }
            enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_FBC6 as u16, timer = 1}
        }
    };
    WalkingSpacePirates_FDCE [] {
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x10, game) != 0 {
            let mut v1: u16 = addr_kSpacePirates_Ilist_FB8C as u16;
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0 {
                v1 = addr_kSpacePirates_Ilist_FC0E as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        } else if Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game) != 0 {
            let x_pos: u16 = (*E).base.x_pos;
            (*E).var_00 = x_pos;
            (*E).base.x_pos = (x_pos).wrapping_add(16);
            let result: u8 = Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game);
            (*E).base.x_pos = (*E).var_00;
            if result == 0
                || Enemy_MoveRight_IgnoreSlopes(word!(game.cur_enemy_index), 14336, game) != 0
                || (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_F) as i16) >= 0
            {
                enemy_instruction! {[(*E).base]; instruction = addr_kSpacePirates_Ilist_FC48 as u16, timer = 1}
            }
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11698191] Enemy_GrappleReact_CancelBeam_B2 => Enemy_SwitchToFrozenAi(game);
    [11698241] Enemy_NormalFrozenAI_B2 => NormalEnemyFrozenAI(game);
    [11700071] WalkingSpacePirates_Powerbomb => NormalEnemyPowerBombAi(game);
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: *const u16, game: &mut Game) -> *const u16 => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); core::ptr::null::<u16>() };
    @functions [pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16];
    @prefix {
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            (*jp as i16 as i32) << 16,
            game,
        ) == 0
        {
            return jp.offset(1);
        }
    };
    [11726400]
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceRight [addr_kSpacePirates_Ilist_ED36 as i32 as usize as *const u16] {

        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).var_C = ((*E).var_C) ^ (1u16);
        if (*E).var_C != 0 {
            return addr_off_B2ECEC as i32 as usize as *const u16;
        }
    };
    [11726450]
    SpacePirates_Instr_MovePixelsDownAndChangeDirFaceLeft [addr_off_B2EDAC as i32 as usize as *const u16] {

        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).var_C = ((*E).var_C) ^ (1u16);
        if (*E).var_C != 0 {
            return addr_kSpacePirates_Ilist_EDF6 as i32 as usize as *const u16;
        }
    };
    @functions [pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16];
    [11726500] SpacePirates_Instr_RandomNewDirFaceR [result as usize as *const u16] {
        let result: u16 = addr_kSpacePirates_Ilist_ED36 as u16;
        let v4: u16 = (NextRandom(game)) & (1u16);
        (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game)).var_C = v4;
        if v4 != 0 {
            return addr_off_B2ECEC as i32 as usize as *const u16;
        }
    };
    [11726524] SpacePirates_Instr_RandomNewDirFaceL [result as usize as *const u16] {
        let result: u16 = addr_off_B2EDAC as u16;
        let v4: u16 = (NextRandom(game)) & (1u16);
        (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game)).var_C = v4;
        if v4 != 0 {
            return addr_kSpacePirates_Ilist_EDF6 as i32 as usize as *const u16;
        }
    };
    [11726548] SpacePirates_Instr_PrepareWallJumpR [jp] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).var_B = ((*E).base.x_pos).wrapping_add((*E).parameter_2);
        (*E).var_D = ((*E).base.x_pos as i32 + ((*E).parameter_2 as i32 >> 1)) as u16;
        (*E).var_E = (*E).base.y_pos;
        (*E).var_F = 64;
    };
    [11726589] SpacePirates_Instr_PrepareWallJumpL [jp] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).var_B = ((*E).base.x_pos).wrapping_sub((*E).parameter_2);
        (*E).var_D = ((*E).base.x_pos as i32 - ((*E).parameter_2 as i32 >> 1)) as u16;
        (*E).var_E = (*E).base.y_pos;
        (*E).var_F = 192;
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11726634] SpacePirates_Instr_FireLaserL [jp] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        word!(
            game.cinematic_var8 = *(RomPtr_A0((*E).base.enemy_ptr, game) as *mut u16).offset(3)
        );
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_sub(24)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        game.eproj_spawn_r22 = 0;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 0, game);
    };
    [11726685] SpacePirates_Instr_FireLaserR [jp] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(24)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        game.eproj_spawn_r22 = 1;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
    @functions [pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16];
    [11726723] SpacePirates_Instr_SetEnemyFunc [] {
        (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game)).var_A = *jp;
        jp.offset(1)
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11726739] SpacePirates_Instr_PlaySfx [jp] { QueueSfx2_Max6(0x66, game); };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11728182] SpacePirates_Instr_20 [jp.offset(1)] {
        (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
            .base
            .palette_index = *jp;
    };
    [11728198] SpacePirates_Instr_16 [jp.offset(1)] { QueueSfx2_Max6(*jp, game); };
    [11728228] SpacePirates_Instr_15 [jp.offset(3)] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        game.eproj_spawn_rect = Rect16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
            w: *jp.offset(1),
            h: *jp.offset(2),
        };
        SpawnEprojWithGfx(
            *jp,
            word!(game.cur_enemy_index),
            addr_kEproj_PirateClaw as u16,
            game,
        );
    };
    [11728272] SpacePirates_Instr_18 [(*E).var_C as usize as *const u16] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        (*E).base.instruction_timer = 1;
        (*E).var_C =
            (if ((*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 & 0x8000) != 0 {
                addr_kSpacePirates_Ilist_F420 as i32
            } else {
                addr_kSpacePirates_Ilist_F22E as i32
            }) as u16;
    };
    [11728342] SpacePirates_Instr_17 [jp] { (*Get_SpacePirates(k as i32, game)).var_00 = 0; };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11729257] SpacePirates_Instr_19 [] {
        let E = Get_SpacePirates(k as i32, game);
        (*E).var_00 = 1536;
        (*E).var_03 =
            ((*E).var_D as i32 + ((((*E).var_F).wrapping_sub((*E).var_D)) as i32 >> 1)) as u16;
        jp
    };
    [11729469] SpacePirates_Instr_21 [] {
        let E = Get_SpacePirates(k as i32, game);
        (*E).var_00 = 1536;
        (*E).var_03 =
            ((*E).var_E as i32 + ((((*E).var_D).wrapping_sub((*E).var_E)) as i32 >> 1)) as u16;
        jp
    };
    @functions [pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16];
    [11730024] SpacePirates_Instr_12 [jp.offset(1)] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_sub(24)),
            y: (((*E).base.y_pos).wrapping_sub(*jp)),
        };
        game.eproj_spawn_r22 = 0;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 0, game);
    };
    [11730064] SpacePirates_Instr_14 [jp.offset(1)] {
        let E = Get_SpacePirates(word!(game.cur_enemy_index) as i32, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(24)),
            y: (((*E).base.y_pos).wrapping_sub(*jp)),
        };
        game.eproj_spawn_r22 = 1;
        SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1, game);
    };
    [11730104] SpacePirates_Instr_11 [jp.offset(1)] {
        (*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game)).var_A = *jp;
    };
    [11730120] SpacePirates_Instr_13 [] {
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x10, game) != 0 {
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(
                ((*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
                    .base
                    .x_pos) as i16,
            ) >= 0
            {
                return addr_kSpacePirates_Ilist_FC0E as i32 as usize as *const u16;
            }
            addr_kSpacePirates_Ilist_FB8C as i32 as usize as *const u16
        } else {
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(
                ((*Get_SpacePirates(word!(game.cur_enemy_index) as i32, game))
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
