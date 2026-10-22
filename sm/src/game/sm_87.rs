pub use crate::game::constants::bank_86::*;
pub use crate::game::prelude::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, QueueMusic_Delayed8, QueueSfx1_Max6,
    QueueSfx2_Max6, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;

const animtiles_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x87>;
pub unsafe fn EnableAnimtiles(game: &mut Game) {
    word!(game.animtiles_enable_flag |= (0x8000u16));
}
pub unsafe fn DisableAnimtiles(game: &mut Game) {
    word!(game.animtiles_enable_flag &= !(0x8000));
}
pub unsafe fn ClearAnimtiles(game: &mut Game) {
    word_transfers!(game, descending, 6; le(animtiles_ids, 5) <- value(0));
}
pub unsafe fn SpawnAnimtiles(j: u16, game: &mut Game) {
    let v0: u16 = j;
    let mut v1: u16 = 10;
    while word!(game.animtiles_ids[(v1 >> 1)]) != 0 {
        v1 = (v1).wrapping_sub(2);
        if (v1 & 0x8000) != 0 {
            return;
        }
    }
    let v2: i32 = v1 as i32 >> 1;
    word! {game; animtiles_ids[v2] = v0; animtiles_timers[v2] = 0; }
    let v3 = animtiles_record(v0, game);
    word! {game; animtiles_instr_list_ptrs[v2] = v3.word(0); animtiles_src_ptr[v2] = 0;
    animtiles_sizes[v2] = v3.word(2); animtiles_vram_ptr[v2] = v3.word(4);
    animtiles_instr_timers[v2] = 1; }
}
pub unsafe fn AnimtilesHandler(game: &mut Game) {
    if (word!(game.animtiles_enable_flag) & 0x8000) != 0 {
        let mut i: i32 = 10;
        while i >= 0 {
            word!(game.animtiles_object_index = i as u16);
            if word!(game.animtiles_ids[(i >> 1)]) != 0 {
                ProcessAnimtilesObject(game);
                i = word!(game.animtiles_object_index) as i32;
            }
            i -= 2;
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallAnimtilesInstr(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 => ea;
    call (j, k, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [8880306] AnimtilesInstr_Delete [0] { word!(game.animtiles_ids[(k >> 1)] = 0); };
    [8880311] AnimtilesInstr_Goto [animtiles_record(j, game).word(0)] { };
    [8880316] AnimtilesInstr_GotoRel [(j).wrapping_add((animtiles_record(j, game).byte(0) as i8) as u16)] {
        word!(game.animtiles_instruction = j);
    };
    [8880340] AnimtilesInstr_DecrementTimerAndGoto [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh0 [fresh1] = &raw mut (*(ram_ptr!(game, u16; 0x1f01)).offset(v2 as isize)) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
        if fresh1 == 1 {
            (j).wrapping_add(2)
        } else {
            AnimtilesInstr_Goto(k, j, game)
        }
    };
    [8880348] AnimtilesInstr_DecrementTimerAndGotoRel [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh2 [fresh3] = &raw mut (*(ram_ptr!(game, u16; 0x1f01)).offset(v2 as isize)) => (u16::from_le(fresh2.read_unaligned())).wrapping_sub(1); }
        if fresh3 == 1 {
            (j).wrapping_add(1)
        } else {
            AnimtilesInstr_GotoRel(k, j, game)
        }
    };
    [8880355] AnimtilesInstr_SetTimer [(j).wrapping_add(1)] {
        *ram_ptr!(game; 0x1f01, k as i32 as isize) = animtiles_record(j, game).byte(0);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    @prefix { let v2 = animtiles_record(j, game); };
    [8880368]
    AnimtilesInstr_QueueMusic [(j).wrapping_add(1)] {  QueueMusic_Delayed8(u16::from(v2.byte(0)), game); };
    [8880380] AnimtilesInstr_QueueSfx1 [(j).wrapping_add(1)] {  QueueSfx1_Max6(u16::from(v2.byte(0)), game); };
    [8880392] AnimtilesInstr_QueueSfx2 [(j).wrapping_add(1)] {  QueueSfx2_Max6(u16::from(v2.byte(0)), game); };
    [8880404] AnimtilesInstr_QueueSfx3 [(j).wrapping_add(1)] {  QueueSfx3_Max6(u16::from(v2.byte(0)), game); };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [8880416] AnimtilesInstr_GotoIfBossBitSet [] {
        let v2 = animtiles_record(j, game);
        let v3: u16 = (j).wrapping_add(1);
        if CheckBossBitForCurArea(v2.word(0) as u8 as u16, game) & 1 != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2)
        }
    };
    [8880435] AnimtilesInstr_SetBossBit [(j).wrapping_add(1)] {
        let v2 = animtiles_record(j, game);
        SetBossBitForCurArea(u16::from(v2.byte(0)), game);
    };
    [8880447] AnimtilesInstr_GotoIfEventHappened [] {
        let v2 = animtiles_record(j, game);
        let v3: u16 = (j).wrapping_add(2);
        if CheckEventHappened(v2.word(0), game) & 1 != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2)
        }
    };
    [8880464] AnimtilesInstr_SetEventHappened [(j).wrapping_add(2)] {
        let v2 = animtiles_record(j, game);
        SetEventHappened(v2.word(0), game);
    };
    @functions [pub unsafe fn (
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16];
    [8880570] AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet [] {
        if CheckBossBitForCurArea(1, game) & 1 == 0 {
            word!(game.animtiles_instr_timers[(k >> 1)] = 1);
            return 0;
        }
        j
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> u16];
    [8880899] AnimtilesInstr_GotoIfBossBitSetInArea [] {
        let v2 = animtiles_record(j, game);
        let v3: u16 = (j).wrapping_add(2);
        if (v2.byte(0) & *ram_ptr!(game; 0xd828, v2.byte(1) as isize)) != 0 {
            AnimtilesInstr_Goto(k, v3, game)
        } else {
            (v3).wrapping_add(2)
        }
    };
    [8880928] AnimtilesInstr_SpawnTourianStatueEyeGlow [(j).wrapping_add(2)] {
        let v2 = animtiles_record(j, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueEyeGlow as u16, v2.word(0), game);
    };
    [8880943] AnimtilesInstr_SpawnTourianStatueSoul [(j).wrapping_add(2)] {
        let v2 = animtiles_record(j, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueSoul as u16, v2.word(0), game);
    };
    [8880958] AnimtilesInstr_GotoIfTourianStatueBusy [] {
        if (word!(game.tourian_entrance_statue_animstate) & 0x8000) == 0 {
            (j).wrapping_add(2)
        } else {
            AnimtilesInstr_Goto(k, j, game)
        }
    };
    [8880969] AnimtilesInstr_TourianStatueSetState [(j).wrapping_add(2)] {
        word!(game.tourian_entrance_statue_animstate |= (animtiles_record(j, game).word(0)));
    };
    [8880978] AnimtilesInstr_TourianStatueClearState [(j).wrapping_add(2)] {
        word!(game.tourian_entrance_statue_animstate &= !(animtiles_record(j, game).word(0)));
    };
    [8880987] AnimtilesInstr_Clear3PaletteColors [] {
        let v2: i32 = animtiles_record(j, game).word(0) as i32 >> 1;
        word! { game, 0;
            palette_buffer[v2], palette_buffer[(v2 + 1)], palette_buffer[(v2 + 2)],
        }
        (j).wrapping_add(2)
    };
    [8881010] AnimtilesInstr_SpawnPalfxObj [(j).wrapping_add(2)] {
        let v2 = animtiles_record(j, game);
        SpawnPalfxObject(v2.word(0), game);
    };
    [8881023] AnimtilesInstr_Write8PaletteColors [(j).wrapping_add(2)] {
        static kAnimtilesInstr_Write8PaletteColors: [u16; 8] = [
            0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
        ];
        let mut v2: u16 = animtiles_record(j, game).word(0);
        for i in (0i32..16).step_by(2) {
            word!(
                game.target_palettes[(v2 >> 1)] =
                    kAnimtilesInstr_Write8PaletteColors[(i >> 1) as usize]
            );
            v2 = (v2).wrapping_add(2);
        }
    };
}
pub unsafe fn ProcessAnimtilesObject(game: &mut Game) {
    let v0: u16 = word!(game.animtiles_object_index);
    let v1: i32 = word!(game.animtiles_object_index) as i32 >> 1;
    crate::update_place! { unaligned_word fresh6 [fresh7] = &raw mut (*(ram_ptr!(game, u16; 0x1f19)).offset(v1 as isize)) => (u16::from_le(fresh6.read_unaligned())).wrapping_sub(1); }
    if fresh7 == 1 {
        let mut v3: u16 = word!(game.animtiles_instr_list_ptrs[v1]);
        let mut v5: u16 = 0;
        loop {
            let v4 = animtiles_record(v3, game);
            v5 = v4.word(0);
            if (v4.word(0) & 0x8000) == 0 {
                break;
            }
            word!(game.animtiles_instruction = v4.word(0));
            v3 = CallAnimtilesInstr(
                ((v5) as u32) | (0x870000u32),
                v0,
                (v3).wrapping_add(2),
                game,
            );
            if v3 == 0 {
                return;
            }
        }
        let v6: i32 = v0 as i32 >> 1;
        word!(game.animtiles_instr_timers[v6] = v5);
        word!(game.animtiles_src_ptr[v6] = animtiles_record(v3, game).word(2));
        word!(game.animtiles_instr_list_ptrs[v6] = (v3).wrapping_add(4));
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    UNUSED_sub_87815A => CallSomeSamusCode(0, game);
    UNUSED_sub_878162 => CallSomeSamusCode(1, game);
}
