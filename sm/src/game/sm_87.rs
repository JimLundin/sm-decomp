pub use crate::game::constants::bank_86::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, QueueMusic_Delayed8, QueueSfx1_Max6,
    QueueSfx2_Max6, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::Consts_86;
const RomPtr_87: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x87>;
pub const fnAnimtilesInstr_Delete: u32 = 8880306;
pub const fnAnimtilesInstr_Goto: u32 = 8880311;
pub const fnAnimtilesInstr_GotoRel: u32 = 8880316;
pub const fnAnimtilesInstr_DecrementTimerAndGoto: u32 = 8880340;
pub const fnAnimtilesInstr_DecrementTimerAndGotoRel: u32 = 8880348;
pub const fnAnimtilesInstr_SetTimer: u32 = 8880355;
pub const fnAnimtilesInstr_QueueMusic: u32 = 8880368;
pub const fnAnimtilesInstr_QueueSfx1: u32 = 8880380;
pub const fnAnimtilesInstr_QueueSfx2: u32 = 8880392;
pub const fnAnimtilesInstr_QueueSfx3: u32 = 8880404;
pub const fnAnimtilesInstr_GotoIfBossBitSet: u32 = 8880416;
pub const fnAnimtilesInstr_SetBossBit: u32 = 8880435;
pub const fnAnimtilesInstr_GotoIfEventHappened: u32 = 8880447;
pub const fnAnimtilesInstr_SetEventHappened: u32 = 8880464;
pub const fnAnimtilesInstr_WaitUntilAreaBossDead_DoubleRet: u32 = 8880570;
pub const fnAnimtilesInstr_GotoIfBossBitSetInArea: u32 = 8880899;
pub const fnAnimtilesInstr_SpawnTourianStatueEyeGlow: u32 = 8880928;
pub const fnAnimtilesInstr_SpawnTourianStatueSoul: u32 = 8880943;
pub const fnAnimtilesInstr_GotoIfTourianStatueBusy: u32 = 8880958;
pub const fnAnimtilesInstr_TourianStatueSetState: u32 = 8880969;
pub const fnAnimtilesInstr_TourianStatueClearState: u32 = 8880978;
pub const fnAnimtilesInstr_Clear3PaletteColors: u32 = 8880987;
pub const fnAnimtilesInstr_SpawnPalfxObj: u32 = 8881010;
pub const fnAnimtilesInstr_Write8PaletteColors: u32 = 8881023;
pub unsafe fn EnableAnimtiles(game: &mut Game) {
    update_ram_word!(game.animtiles_enable_flag, |fresh9| (fresh9) | (0x8000u16));
}
pub unsafe fn DisableAnimtiles(game: &mut Game) {
    update_ram_word!(
        game.animtiles_enable_flag,
        |fresh8| (fresh8 as i32 & !(0x8000i32)) as u16
    );
}
pub unsafe fn ClearAnimtiles(game: &mut Game) {
    let mut i: i32 = 10i32;
    while i >= 0i32 {
        set_ram_word!(game.animtiles_ids[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
}
pub unsafe fn SpawnAnimtiles(j: u16, game: &mut Game) {
    let v0: u16 = j;
    let mut v1: u16 = 10u16;
    while ram_word!(game, animtiles_ids[(v1 as i32 >> 1i32)]) != 0 {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let v2: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.animtiles_ids[v2] = v0);
    set_ram_word!(game.animtiles_timers[v2] = 0u16);
    let v3: *const u16 = RomPtr_87(v0, game) as *const u16;
    set_ram_word!(game.animtiles_instr_list_ptrs[v2] = *v3);
    set_ram_word!(game.animtiles_src_ptr[v2] = 0u16);
    set_ram_word!(game.animtiles_sizes[v2] = *v3.offset(1));
    set_ram_word!(game.animtiles_vram_ptr[v2] = *v3.offset(2));
    set_ram_word!(game.animtiles_instr_timers[v2] = 1u16);
}
pub unsafe fn AnimtilesHandler(game: &mut Game) {
    if (ram_word!(game, animtiles_enable_flag) as i32 & 0x8000i32) != 0i32 {
        let mut i: i32 = 10i32;
        while i >= 0i32 {
            set_ram_word!(game.animtiles_object_index = i as u16);
            if ram_word!(game, animtiles_ids[(i >> 1i32)]) != 0 {
                ProcessAnimtilesObject(game);
                i = ram_word!(game, animtiles_object_index) as i32;
            }
            i -= 2i32;
        }
    }
}
pub unsafe fn CallAnimtilesInstr(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 {
    match ea {
        8880306 => AnimtilesInstr_Delete(j, k, game),
        8880311 => AnimtilesInstr_Goto(j, k, game),
        8880316 => AnimtilesInstr_GotoRel(j, k, game),
        8880340 => AnimtilesInstr_DecrementTimerAndGoto(j, k, game),
        8880348 => AnimtilesInstr_DecrementTimerAndGotoRel(j, k, game),
        8880355 => AnimtilesInstr_SetTimer(j, k, game),
        8880368 => AnimtilesInstr_QueueMusic(j, k, game),
        8880380 => AnimtilesInstr_QueueSfx1(j, k, game),
        8880392 => AnimtilesInstr_QueueSfx2(j, k, game),
        8880404 => AnimtilesInstr_QueueSfx3(j, k, game),
        8880416 => AnimtilesInstr_GotoIfBossBitSet(j, k, game),
        8880435 => AnimtilesInstr_SetBossBit(j, k, game),
        8880447 => AnimtilesInstr_GotoIfEventHappened(j, k, game),
        8880464 => AnimtilesInstr_SetEventHappened(j, k, game),
        8880570 => AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(j, k, game),
        8880899 => AnimtilesInstr_GotoIfBossBitSetInArea(j, k, game),
        8880928 => AnimtilesInstr_SpawnTourianStatueEyeGlow(j, k, game),
        8880943 => AnimtilesInstr_SpawnTourianStatueSoul(j, k, game),
        8880958 => AnimtilesInstr_GotoIfTourianStatueBusy(j, k, game),
        8880969 => AnimtilesInstr_TourianStatueSetState(j, k, game),
        8880978 => AnimtilesInstr_TourianStatueClearState(j, k, game),
        8880987 => AnimtilesInstr_Clear3PaletteColors(j, k, game),
        8881010 => AnimtilesInstr_SpawnPalfxObj(j, k, game),
        8881023 => AnimtilesInstr_Write8PaletteColors(j, k, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn ProcessAnimtilesObject(game: &mut Game) {
    let v0: u16 = ram_word!(game, animtiles_object_index);
    let v1: i32 = ram_word!(game, animtiles_object_index) as i32 >> 1i32;
    let ref mut fresh6 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1f19) as *mut u16).offset(v1 as isize);
    let fresh7 = *fresh6;
    *fresh6 = (*fresh6).wrapping_sub(1);
    if fresh7 == 1u16 {
        let mut v3: u16 = ram_word!(game, animtiles_instr_list_ptrs[v1]);
        let mut v5: u16 = 0;
        loop {
            let v4: *const u16 = RomPtr_87(v3, game) as *const u16;
            v5 = *v4;
            if (*v4 as i32 & 0x8000i32) == 0i32 {
                break;
            }
            set_ram_word!(game.animtiles_instruction = *v4);
            v3 = CallAnimtilesInstr(
                ((v5) as u32) | (0x870000u32),
                v0,
                (v3).wrapping_add(2u16),
                game,
            );
            if v3 == 0 {
                return;
            }
        }
        let v6: i32 = v0 as i32 >> 1i32;
        set_ram_word!(game.animtiles_instr_timers[v6] = v5);
        set_ram_word!(game.animtiles_src_ptr[v6] = *(RomPtr_87(v3, game) as *mut u16).offset(1));
        set_ram_word!(game.animtiles_instr_list_ptrs[v6] = (v3).wrapping_add(4u16));
    }
}
pub unsafe fn AnimtilesInstr_Delete(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.animtiles_ids[(k as i32 >> 1i32)] = 0u16);
    0u16
}
pub unsafe fn AnimtilesInstr_Goto(k: u16, j: u16, game: &mut Game) -> u16 {
    *(RomPtr_87(j, game) as *mut u16)
}
pub unsafe fn AnimtilesInstr_GotoRel(k: u16, j: u16, game: &mut Game) -> u16 {
    set_ram_word!(game.animtiles_instruction = j);
    (j).wrapping_add((*RomPtr_87(j, game) as i8) as u16)
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGoto(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh0 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1f01) as *mut u16).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 == 1u16 {
        (j).wrapping_add(2u16)
    } else {
        AnimtilesInstr_Goto(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGotoRel(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh2 =
        *((&raw mut game.g_ram as *mut u8).offset(0x1f01) as *mut u16).offset(v2 as isize);
    let fresh3 = *fresh2;
    *fresh2 = (*fresh2).wrapping_sub(1);
    if fresh3 == 1u16 {
        (j).wrapping_add(1u16)
    } else {
        AnimtilesInstr_GotoRel(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_SetTimer(k: u16, j: u16, game: &mut Game) -> u16 {
    *((&raw mut game.g_ram as *mut u8).offset(0x1f01) as *mut u16 as *mut u8)
        .offset(k as i32 as isize) = *RomPtr_87(j, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueMusic(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    QueueMusic_Delayed8(*v2 as u16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx1(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    QueueSfx1_Max6(*v2 as u16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx2(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    QueueSfx2_Max6(*v2 as u16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx3(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    QueueSfx3_Max6(*v2 as u16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSet(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    let v3: u16 = (j).wrapping_add(1u16);
    if CheckBossBitForCurArea(*(v2 as *mut u16) as u8 as u16, game) as i32 & 1i32 != 0 {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SetBossBit(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    SetBossBitForCurArea(*v2 as u16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_GotoIfEventHappened(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_87(j, game) as *const u16;
    let v3: u16 = (j).wrapping_add(2u16);
    if CheckEventHappened(*v2, game) as i32 & 1i32 != 0 {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SetEventHappened(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_87(j, game) as *const u16;
    SetEventHappened(*v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn UNUSED_sub_87815A(game: &mut Game) {
    CallSomeSamusCode(0u16, game);
}
pub unsafe fn UNUSED_sub_878162(game: &mut Game) {
    CallSomeSamusCode(1u16, game);
}
pub unsafe fn AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    if CheckBossBitForCurArea(1u16, game) as i32 & 1i32 == 0 {
        set_ram_word!(game.animtiles_instr_timers[(k as i32 >> 1i32)] = 1u16);
        return 0u16;
    }
    j
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSetInArea(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u8 = RomPtr_87(j, game);
    let v3: u16 = (j).wrapping_add(2u16);
    if (*v2 as i32
        & *(&raw mut game.g_ram as *mut u8)
            .offset(0xd828)
            .offset(*v2.offset(1) as isize) as i32)
        != 0i32
    {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueEyeGlow(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_87(j, game) as *const u16;
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueEyeGlow as u16, *v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueSoul(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_87(j, game) as *const u16;
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueSoul as u16, *v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_GotoIfTourianStatueBusy(k: u16, j: u16, game: &mut Game) -> u16 {
    if (ram_word!(game, tourian_entrance_statue_animstate) as i32 & 0x8000i32) == 0i32 {
        (j).wrapping_add(2u16)
    } else {
        AnimtilesInstr_Goto(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_TourianStatueSetState(k: u16, j: u16, game: &mut Game) -> u16 {
    update_ram_word!(game.tourian_entrance_statue_animstate, |fresh5| (fresh5)
        | (*(RomPtr_87(j, game) as *mut u16)));
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_TourianStatueClearState(k: u16, j: u16, game: &mut Game) -> u16 {
    update_ram_word!(
        game.tourian_entrance_statue_animstate,
        |fresh4| (fresh4 as i32 & !(*(RomPtr_87(j, game) as *mut u16) as i32)) as u16
    );
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_Clear3PaletteColors(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: i32 = *(RomPtr_87(j, game) as *mut u16) as i32 >> 1i32;
    set_ram_word!(game.palette_buffer[v2] = 0u16);
    set_ram_word!(game.palette_buffer[(v2 + 1i32)] = 0u16);
    set_ram_word!(game.palette_buffer[(v2 + 2i32)] = 0u16);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_SpawnPalfxObj(k: u16, j: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_87(j, game) as *const u16;
    SpawnPalfxObject(*v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_Write8PaletteColors(k: u16, j: u16, game: &mut Game) -> u16 {
    static kAnimtilesInstr_Write8PaletteColors: [u16; 8] = [
        0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
    ];
    let mut v2: u16 = *(RomPtr_87(j, game) as *mut u16);
    let mut i: i32 = 0i32;
    while i != 16i32 {
        set_ram_word!(
            game.target_palettes[(v2 as i32 >> 1i32)] =
                kAnimtilesInstr_Write8PaletteColors[(i >> 1i32) as usize]
        );
        v2 = (v2).wrapping_add(2u16);
        i += 2i32;
    }
    (j).wrapping_add(2u16)
}
