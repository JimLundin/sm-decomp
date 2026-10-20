pub use crate::game::constants::bank_86::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, QueueMusic_Delayed8, QueueSfx1_Max6,
    QueueSfx2_Max6, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::Consts_86;
const RomPtr_87: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x87>;
pub unsafe fn EnableAnimtiles(game: &mut Game) {
    update_ram_word!(game.animtiles_enable_flag, |fresh9| (fresh9) | (0x8000u16));
}
pub unsafe fn DisableAnimtiles(game: &mut Game) {
    update_ram_word!(game.animtiles_enable_flag, |fresh8| (fresh8 & !(0x8000)));
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
    while ram_word!(game, animtiles_ids[(v1 >> 1)]) != 0 {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 & 0x8000) != 0 {
            return;
        }
    }
    let v2: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.animtiles_ids[v2] = v0);
    set_ram_word!(game.animtiles_timers[v2] = 0u16);
    let v3 = RomPtr_87(v0, game) as *const u16;
    set_ram_word!(game.animtiles_instr_list_ptrs[v2] = *v3);
    set_ram_word!(game.animtiles_src_ptr[v2] = 0u16);
    set_ram_word!(game.animtiles_sizes[v2] = *v3.offset(1));
    set_ram_word!(game.animtiles_vram_ptr[v2] = *v3.offset(2));
    set_ram_word!(game.animtiles_instr_timers[v2] = 1u16);
}
pub unsafe fn AnimtilesHandler(game: &mut Game) {
    if (ram_word!(game, animtiles_enable_flag) & 0x8000) != 0 {
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
crate::game::address_dispatch! {
    pub unsafe fn CallAnimtilesInstr(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 => ea;
        call (j, k, game); default Unreachable(game) as u16;
    {
        8880306 => AnimtilesInstr_Delete; 8880311 => AnimtilesInstr_Goto;
        8880316 => AnimtilesInstr_GotoRel; 8880340 => AnimtilesInstr_DecrementTimerAndGoto;
        8880348 => AnimtilesInstr_DecrementTimerAndGotoRel; 8880355 => AnimtilesInstr_SetTimer;
        8880368 => AnimtilesInstr_QueueMusic; 8880380 => AnimtilesInstr_QueueSfx1;
        8880392 => AnimtilesInstr_QueueSfx2; 8880404 => AnimtilesInstr_QueueSfx3;
        8880416 => AnimtilesInstr_GotoIfBossBitSet; 8880435 => AnimtilesInstr_SetBossBit;
        8880447 => AnimtilesInstr_GotoIfEventHappened; 8880464 => AnimtilesInstr_SetEventHappened;
        8880570 => AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet;
        8880899 => AnimtilesInstr_GotoIfBossBitSetInArea;
        8880928 => AnimtilesInstr_SpawnTourianStatueEyeGlow;
        8880943 => AnimtilesInstr_SpawnTourianStatueSoul;
        8880958 => AnimtilesInstr_GotoIfTourianStatueBusy;
        8880969 => AnimtilesInstr_TourianStatueSetState;
        8880978 => AnimtilesInstr_TourianStatueClearState;
        8880987 => AnimtilesInstr_Clear3PaletteColors; 8881010 => AnimtilesInstr_SpawnPalfxObj;
        8881023 => AnimtilesInstr_Write8PaletteColors;
    }
}
pub unsafe fn ProcessAnimtilesObject(game: &mut Game) {
    let v0: u16 = ram_word!(game, animtiles_object_index);
    let v1: i32 = ram_word!(game, animtiles_object_index) as i32 >> 1i32;
    crate::update_place! { reference fresh6 [fresh7] = *(ram_ptr!(game, u16; 0x1f19)).offset(v1 as isize) => (*fresh6).wrapping_sub(1); }
    if fresh7 == 1u16 {
        let mut v3: u16 = ram_word!(game, animtiles_instr_list_ptrs[v1]);
        let mut v5: u16 = 0;
        loop {
            let v4 = RomPtr_87(v3, game) as *const u16;
            v5 = *v4;
            if (*v4 & 0x8000) == 0 {
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
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    AnimtilesInstr_Delete [0u16] { set_ram_word!(game.animtiles_ids[(k >> 1)] = 0u16); };
    AnimtilesInstr_Goto [*(RomPtr_87(j, game) as *mut u16)] { };
    AnimtilesInstr_GotoRel [(j).wrapping_add((*RomPtr_87(j, game) as i8) as u16)] {
        set_ram_word!(game.animtiles_instruction = j);
    };
    AnimtilesInstr_DecrementTimerAndGoto [] {
        let v2: i32 = k as i32 >> 1i32;
        crate::update_place! { reference fresh0 [fresh1] = *(ram_ptr!(game, u16; 0x1f01)).offset(v2 as isize) => (*fresh0).wrapping_sub(1); }
        if fresh1 == 1u16 {
            (j).wrapping_add(2u16)
        } else {
            AnimtilesInstr_Goto(k, j, game)
        }
    };
    AnimtilesInstr_DecrementTimerAndGotoRel [] {
        let v2: i32 = k as i32 >> 1i32;
        crate::update_place! { reference fresh2 [fresh3] = *(ram_ptr!(game, u16; 0x1f01)).offset(v2 as isize) => (*fresh2).wrapping_sub(1); }
        if fresh3 == 1u16 {
            (j).wrapping_add(1u16)
        } else {
            AnimtilesInstr_GotoRel(k, j, game)
        }
    };
    AnimtilesInstr_SetTimer [(j).wrapping_add(1u16)] {
        *(ram_ptr!(game, u16; 0x1f01) as *mut u8).offset(k as i32 as isize) = *RomPtr_87(j, game);
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    @prefix { let v2 = RomPtr_87(j, game); };
    AnimtilesInstr_QueueMusic [(j).wrapping_add(1u16)] {  QueueMusic_Delayed8(*v2 as u16, game); };
    AnimtilesInstr_QueueSfx1 [(j).wrapping_add(1u16)] {  QueueSfx1_Max6(*v2 as u16, game); };
    AnimtilesInstr_QueueSfx2 [(j).wrapping_add(1u16)] {  QueueSfx2_Max6(*v2 as u16, game); };
    AnimtilesInstr_QueueSfx3 [(j).wrapping_add(1u16)] {  QueueSfx3_Max6(*v2 as u16, game); };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    AnimtilesInstr_GotoIfBossBitSet [] {
        let v2 = RomPtr_87(j, game);
        let v3: u16 = (j).wrapping_add(1u16);
        if CheckBossBitForCurArea(*(v2 as *mut u16) as u8 as u16, game) & 1 != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2u16)
        }
    };
    AnimtilesInstr_SetBossBit [(j).wrapping_add(1u16)] {
        let v2 = RomPtr_87(j, game);
        SetBossBitForCurArea(*v2 as u16, game);
    };
    AnimtilesInstr_GotoIfEventHappened [] {
        let v2 = RomPtr_87(j, game) as *const u16;
        let v3: u16 = (j).wrapping_add(2u16);
        if CheckEventHappened(*v2, game) & 1 != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2u16)
        }
    };
    AnimtilesInstr_SetEventHappened [(j).wrapping_add(2u16)] {
        let v2 = RomPtr_87(j, game) as *const u16;
        SetEventHappened(*v2, game);
    };
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    UNUSED_sub_87815A => CallSomeSamusCode(0u16, game);
    UNUSED_sub_878162 => CallSomeSamusCode(1u16, game);
}
pub unsafe fn AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    if CheckBossBitForCurArea(1u16, game) & 1 == 0 {
        set_ram_word!(game.animtiles_instr_timers[(k >> 1)] = 1u16);
        return 0u16;
    }
    j
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16;
    AnimtilesInstr_GotoIfBossBitSetInArea [] {
        let v2 = RomPtr_87(j, game);
        let v3: u16 = (j).wrapping_add(2u16);
        if (*v2 & *ram_ptr!(game; 0xd828, *v2.offset(1) as isize)) != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2u16)
        }
    };
    AnimtilesInstr_SpawnTourianStatueEyeGlow [(j).wrapping_add(2u16)] {
        let v2 = RomPtr_87(j, game) as *const u16;
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueEyeGlow as u16, *v2, game);
    };
    AnimtilesInstr_SpawnTourianStatueSoul [(j).wrapping_add(2u16)] {
        let v2 = RomPtr_87(j, game) as *const u16;
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueSoul as u16, *v2, game);
    };
    AnimtilesInstr_GotoIfTourianStatueBusy [] {
        if (ram_word!(game, tourian_entrance_statue_animstate) & 0x8000) == 0 {
            (j).wrapping_add(2u16)
        } else {
            AnimtilesInstr_Goto(k, j, game)
        }
    };
    AnimtilesInstr_TourianStatueSetState [(j).wrapping_add(2u16)] {
        update_ram_word!(game.tourian_entrance_statue_animstate, |fresh5| (fresh5)
            | (*(RomPtr_87(j, game) as *mut u16)));
    };
    AnimtilesInstr_TourianStatueClearState [(j).wrapping_add(2u16)] {
        update_ram_word!(game.tourian_entrance_statue_animstate, |fresh4| (fresh4
            & !(*(RomPtr_87(j, game) as *mut u16))));
    };
    AnimtilesInstr_Clear3PaletteColors [] {
        let v2: i32 = *(RomPtr_87(j, game) as *mut u16) as i32 >> 1i32;
        set_ram_words! { game, 0u16;
            palette_buffer[v2], palette_buffer[(v2 + 1i32)], palette_buffer[(v2 + 2i32)],
        }
        (j).wrapping_add(2u16)
    };
    AnimtilesInstr_SpawnPalfxObj [(j).wrapping_add(2u16)] {
        let v2 = RomPtr_87(j, game) as *const u16;
        SpawnPalfxObject(*v2, game);
    };
    AnimtilesInstr_Write8PaletteColors [(j).wrapping_add(2u16)] {
        static kAnimtilesInstr_Write8PaletteColors: [u16; 8] = [
            0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
        ];
        let mut v2: u16 = *(RomPtr_87(j, game) as *mut u16);
        let mut i: i32 = 0i32;
        while i != 16i32 {
            set_ram_word!(
                game.target_palettes[(v2 >> 1)] =
                    kAnimtilesInstr_Write8PaletteColors[(i >> 1i32) as usize]
            );
            v2 = (v2).wrapping_add(2u16);
            i += 2i32;
        }
    };
}
