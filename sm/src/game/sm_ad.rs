use crate::c_library::printf;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_a9::WriteColorsToPalette;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_word;
use crate::ram_map::set_ram_words;
use crate::rtl::RomPtr;
pub use crate::types::Consts_86;
pub use crate::types::Consts_AD;
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, Func_V, Point16U, VoidP};
use ::core::ffi::c_char;
crate::game::constants::typed_constants! { Consts_AD;
    addr_kTurnOffLightsForShitroidDeath = 61999,
}
pub use crate::types::{Enemy_MotherBrain, Func_CalcHdma, Ram8000_Default, SpawnHardcodedPlmArgs};
const RomPtr_AD: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xad>;
crate::game::constants::typed_constants! { i32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 0xa9881d,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gRam8000_Default(o: i32) -> Ram8000_Default = [0x8000, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
}
pub fn nullsub_341(game: &mut Game) {}
static funcs_BE56D: [Option<Func_V>; 16] = callback_table! { unsafe Func_V;
    MotherBrain_CalcHdma_Down, MotherBrain_CalcHdma_BeamAimedRight2, None, None, None,
    MotherBrain_CalcHdma_BeamAimedUp, MotherBrain_CalcHdma_BeamAimedUp, None, None, None,
    MotherBrain_CalcHdma_BeamAimedUp, nullsub_341, MotherBrain_CalcHdma_Down, None, None,
    MotherBrain_CalcHdma_Down,
};
pub unsafe fn MotherBrain_CalcHdma(game: &mut Game) {
    let E = Get_MotherBrain(0i32, game);
    let E1 = Get_MotherBrain(0x40i32, game);
    let mut r18: u16 = (*(&raw mut (*E).var_33 as *mut u8).offset(1isize) >> 1) as u16;
    (*E).var_3A = ((*E).var_31).wrapping_sub(r18);
    (*E).var_3B = (r18).wrapping_add((*E).var_31);
    let v2: u16 = (*((&raw mut (*E1).base.enemy_ptr as *mut u8).offset(1) as *mut u16) as i32
        + 3584i32
        & 0xff00i32) as u16;
    (*E).var_3C = v2;
    (*E).var_3E = v2;
    let v3: u16 = ((*E1).base.y_pos).wrapping_add(5u16);
    (*E).var_3D = v3;
    (*E).var_3F = v3;
    r18 = (4i32 * ((*E).var_3A as i32 & 0xc0i32)) as u16;
    let i: i32 = (r18 as i32 | (*E).var_3B as i32 & 0xc0i32) >> 6i32;
    funcs_BE56D[i as usize].expect("non-null function pointer")(game);
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight2(game: &mut Game) {
    let E = Get_MotherBrain(0i32, game);
    MotherBrain_CalcHdma_BeamAimedRight((*E).var_3C, (*E).var_3E, game);
    *(ram_ptr!(game, u16; 0x9c00)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = -25600i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 3)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 4)) = -25597i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 6)) = 240u16;
    *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 9)) = 244u16;
    *(ram_ptr!(game, u16; 0x9c00, 10)) = -25108i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 12)) = 0u16;
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_words! { game, 255u16; hdma_table_2[0], hdma_table_2[1] }
    let E = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).var_3A as *mut u8).offset(0) as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).var_3B as *mut u8).offset(0) as isize
    );
    let mut mbn_var_3F: u16 = (*E).var_3F;
    let mut v2 = (ram_ptr!(game, u16; 0x9d00))
        .offset((mbn_var_3F as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255u16;
    *v2.offset(2) = 255u16;
    let v8 = v2;
    loop {
        let mut v4: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            break;
        }
        r24 = (r24).wrapping_add(r20);
        crate::update_place! { previous fresh11 = v2 => v2.offset(-1); }
        *fresh11 = ((*(&raw mut v4 as *mut u8).offset(1isize)) as u16) | (0xff00u16);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    loop {
        crate::update_place! { previous fresh12 = v2 => v2.offset(-1); }
        *fresh12 = 255u16;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    let mut v5 = v8.offset(1);
    let mut var_3F: u16 = (*gRam8000_Default(0i32, game)).var_3F;
    loop {
        let mut v7: u16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            break;
        }
        r22 = (r22).wrapping_add(r18);
        crate::update_place! { previous fresh13 = v5 => v5.offset(1); }
        *fresh13 = ((*(&raw mut v7 as *mut u8).offset(1isize)) as u16) | (0xff00u16);
        var_3F = var_3F.wrapping_add(1);
        if var_3F == 232u16 {
            return;
        }
    }
    loop {
        crate::update_place! { previous fresh14 = v5 => v5.offset(1); }
        *fresh14 = 255u16;
        var_3F = var_3F.wrapping_add(1);
        if (var_3F == 232u16) {
            break;
        }
    }
}
static off_ADE024: [Option<Func_CalcHdma>; 4] = callback_table! { unsafe Func_CalcHdma;
    MotherBrain_CalcHdma_Up_UpRight, MotherBrain_CalcHdma_Up_Up, None,
    MotherBrain_CalcHdma_Up_UpLeft,
};
pub unsafe fn MotherBrain_CalcHdma_BeamAimedUp(game: &mut Game) {
    let E = Get_MotherBrain(0i32, game);
    off_ADE024[((*E).var_3A as i32 >> 6i32 & 2i32 | (*E).var_3B as i32 >> 7i32 & 1i32) as usize]
        .expect("non-null function pointer")((*E).var_3C, (*E).var_3E, game);
    *(ram_ptr!(game, u16; 0x9c00)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = -25344i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 3)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 4)) = -25342i32 as u16;
    let v2: u16 = ((*E).var_3D).wrapping_sub(32u16);
    printf(b"Carry crap!\n\0" as *const u8 as *const c_char);
    if v2 >= 0x80u16 {
        *(ram_ptr!(game, u16; 0x9c00, 6)) = (v2 as i32 - 127i32 | 0x80i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 9)) = 255u16;
        *(ram_ptr!(game, u16; 0x9c00, 10)) = (2i32 * (v2 as i32 - 127i32) - 25340i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 12)) = 127u16;
        *(ram_ptr!(game, u16; 0x9c00, 13)) =
            (2i32 * (v2 as i32 - 127i32) - 25340i32 + 254i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 15)) = 0u16;
    } else {
        *(ram_ptr!(game, u16; 0x9c00, 6)) = (v2) | (0x80u16);
        *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 9)) = 127u16;
        *(ram_ptr!(game, u16; 0x9c00, 10)) = (2i32 * v2 as i32 - 25340i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 12)) = 0u16;
    };
}
#[derive(Clone, Copy)]
enum MotherBrainBeamDirection {
    UpRight,
    Up,
    UpLeft,
    DownRight,
    Down,
    DownLeft,
}

/// Rasterize the two saturated beam edges in their original RAM order.
unsafe fn mother_brain_beam_edges(
    direction: MotherBrainBeamDirection,
    mut left: u16,
    mut right: u16,
    game: &mut Game,
) {
    use MotherBrainBeamDirection::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    let zero_sentinel = matches!(direction, DownLeft);
    set_ram_words! { game, 255u16; hdma_table_2[0], hdma_table_2[1] }
    let enemy = Get_MotherBrain(0, game);
    let left_step = crate::rtl::rom_table_word!(game, 0x91c9d4, {
        let angle = if down {
            *(&raw mut (*enemy).var_3A as *mut u8).offset(0)
        } else {
            *(&raw mut (*enemy).var_3B as *mut u8).offset(0)
        };
        (if add_left {
            angle
        } else {
            (-(angle as i32)) as u8
        }) as isize
    });
    let right_step = crate::rtl::rom_table_word!(game, 0x91c9d4, {
        let angle = if down {
            *(&raw mut (*enemy).var_3B as *mut u8).offset(0)
        } else {
            *(&raw mut (*enemy).var_3A as *mut u8).offset(0)
        };
        (if add_right {
            angle
        } else {
            (-(angle as i32)) as u8
        }) as isize
    });
    let mut row = (*enemy).var_3D;
    let mut dst = if down {
        let mut remaining = row.wrapping_sub(32);
        let mut dst = ram_ptr!(game, u16; 0x9d00).offset(2);
        loop {
            crate::update_place! { previous current = dst => dst.offset(1); }
            *current = 255;
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }
        // Preserve the original post-fill read of the starting row.
        row = (*Get_MotherBrain(0, game)).var_3D;
        dst
    } else {
        let dst = ram_ptr!(game, u16; 0x9d00)
            .offset((row as i32 - 32) as isize)
            .offset(1);
        *dst.offset(1) = 255;
        *dst.offset(2) = 255;
        dst
    };
    loop {
        left = if add_left {
            left_step.saturating_add(left)
        } else {
            left.saturating_sub(left_step)
        };
        let low = left >> 8;
        right = if add_right {
            right_step.saturating_add(right)
        } else {
            right.saturating_sub(right_step)
        };
        let mut packed = low | (right & 0xff00);
        if packed == if zero_sentinel { 0 } else { 0xffff } {
            packed = 255;
        }
        crate::update_place! { previous current = dst => dst.offset(if down { 1 } else { -1 }); }
        *current = packed;
        row = if down {
            row.wrapping_add(1)
        } else {
            row.wrapping_sub(1)
        };
        if row == if down { 232 } else { 32 } {
            break;
        }
    }
}

crate::forward_calls! { pub unsafe fn (r22: u16, r24: u16, game: &mut Game);
    MotherBrain_CalcHdma_Up_UpRight =>
        mother_brain_beam_edges(MotherBrainBeamDirection::UpRight, r22, r24, game);
    MotherBrain_CalcHdma_Up_Up =>
        mother_brain_beam_edges(MotherBrainBeamDirection::Up, r22, r24, game);
    MotherBrain_CalcHdma_Up_UpLeft =>
        mother_brain_beam_edges(MotherBrainBeamDirection::UpLeft, r22, r24, game);
}
static g_off_ADE20E: [Option<Func_CalcHdma>; 4] = callback_table! { unsafe Func_CalcHdma;
    MotherBrain_CalcHdma_Down_DownRight, None, MotherBrain_CalcHdma_Down_Down,
    MotherBrain_CalcHdma_Down_DownLeft,
};
pub unsafe fn MotherBrain_CalcHdma_Down(game: &mut Game) {
    let E = Get_MotherBrain(0i32, game);
    g_off_ADE20E[((*E).var_3A as i32 >> 6i32 & 2i32 | (*E).var_3B as i32 >> 7i32 & 1i32) as usize]
        .expect("non-null function pointer")((*E).var_3C, (*E).var_3E, game);
    *(ram_ptr!(game, u16; 0x9c00)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = -25600i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 3)) = 16u16;
    *(ram_ptr!(game, u16; 0x9c00, 4)) = -25597i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 6)) = 240u16;
    *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 9)) = 244u16;
    *(ram_ptr!(game, u16; 0x9c00, 10)) = -25114i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 12)) = 0u16;
}
crate::forward_calls! { pub unsafe fn (r22: u16, r24: u16, game: &mut Game);
    MotherBrain_CalcHdma_Down_DownRight =>
        mother_brain_beam_edges(MotherBrainBeamDirection::DownRight, r22, r24, game);
    MotherBrain_CalcHdma_Down_Down =>
        mother_brain_beam_edges(MotherBrainBeamDirection::Down, r22, r24, game);
    MotherBrain_CalcHdma_Down_DownLeft =>
        mother_brain_beam_edges(MotherBrainBeamDirection::DownLeft, r22, r24, game);
}
pub unsafe fn MotherBrain_SealWall(game: &mut Game) {
    game.eproj_spawn_pt = Point16U {
        x: 248u16,
        y: 72u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 248u16,
        y: 152u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x4u8, 0xb673 as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x9u8, 0xb673 as VoidP),
        game,
    );
    (*Get_MotherBrain(0i32, game)).var_A = ((0xa9881di32) as u16);
}
pub unsafe fn MotherBrain_HealthBasedPaletteHandling(game: &mut Game) {
    if (*Get_MotherBrain(0i32, game)).var_1F >= 2u16 {
        let mut v0: u16 = 0u16;
        let health: u16 = (*Get_MotherBrain(0x40i32, game)).base.health;
        if health < 0x2328u16 {
            v0 = 2u16;
            if health < 0x1518u16 {
                v0 = 4u16;
                if health < 0x708u16 {
                    v0 = 6u16;
                }
            }
        }
        WriteColorsToPalette(
            0x82u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 >> 1) as isize),
            0xfu16,
            game,
        );
        WriteColorsToPalette(
            0x122u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 >> 1) as isize),
            0xfu16,
            game,
        );
        WriteColorsToPalette(
            0x162u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade742, (v0 >> 1) as isize),
            0xfu16,
            game,
        );
    }
}
pub unsafe fn MotherBrain_FadePalToBlack(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 14u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 14u16, game);
    WriteColorsToPalette(0x162u16, 0xadu8, (j).wrapping_add(28u16), 14u16, game);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded8a, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1u8
    }
}
pub unsafe fn MotherBrain_FadeFromGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1u8
    }
}
pub unsafe fn MotherBrain_EEF6(a: u16, game: &mut Game) -> u8 {
    WriteColorsToPalette(0x122u16, 0xadu8, a, 3u16, game);
    0u8
}
pub unsafe fn MotherBrain_FadeFromGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 13u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 13u16, game);
    WriteColorsToPalette(0x168u16, 0xadu8, (j).wrapping_add(26u16), 5u16, game);
    *(ram_ptr!(game, u16; 0x17c)) = *(RomPtr_AD((j).wrapping_add(36u16), game) as *mut u16);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x168u16, 0xadu8, (j).wrapping_add(30u16), 5u16, game);
    *(ram_ptr!(game, u16; 0x17c)) = *(RomPtr_AD((j).wrapping_add(40u16), game) as *mut u16);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_RealDeath(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize) == 0 {
        return 1u8;
    }
    WriteColorsToPalette(
        0x1e2u16,
        0xadu8,
        crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize),
        0xfu16,
        game,
    );
    0u8
}
pub unsafe fn TurnOffLightsForShitroidDeath(game: &mut Game) {
    WriteColorsToPalette(
        0x62u16,
        0xadu8,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14u16,
        game,
    );
    WriteColorsToPalette(
        0xa2u16,
        0xadu8,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14u16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_TurnLightsBackOn(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize) == 0 {
        return 1u8;
    }
    let v3: u16 = crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize);
    WriteColorsToPalette(0x62u16, 0xadu8, v3, 14u16, game);
    WriteColorsToPalette(0xa2u16, 0xadu8, (v3).wrapping_add(28u16), 14u16, game);
    0u8
}
pub unsafe fn EnableEarthquakeAframes(a: u16, game: &mut Game) {
    set_ram_word!(game.earthquake_type = a);
    set_ram_word!(game.earthquake_timer = 20u16);
    QueueSfx2_Max6(0x25u16, game);
}
pub unsafe fn HandleMotherBrainBodyFlickering(game: &mut Game) {
    let v0 = gEnemyData(0i32, game);
    if (*v0).frame_counter & 1 != 0 {
        crate::update_place! { reference fresh0 = *(ram_ptr!(game, u16; 0x69)) => (*fresh0) | (2u16); }
        (*v0).properties = ((*v0).properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    } else {
        crate::update_place! { reference fresh1 = *(ram_ptr!(game, u16; 0x69)) => (*fresh1 & !(2)); }
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_Invisible) as u16);
    };
}
