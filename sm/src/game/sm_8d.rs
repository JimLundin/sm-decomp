pub use crate::game::constants::bank_8d::*;
pub use crate::game::prelude::*;
use crate::game::sm_80::{QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct PalFxDef {
        pub func, instrs: VoidP;
    }
}

const RomPtr_8D: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8d>;
#[inline]
unsafe fn get_PalFxDef(a: u16, game: &mut Game) -> *mut PalFxDef {
    RomPtr((0x8d0000u32) | ((a) as u32), game) as *mut PalFxDef
}
crate::game::constants::typed_constants! { i32;
    fnCinematicFunction_Intro_Page2 = 0x8bb35f,
}
pub unsafe fn EnablePaletteFx(game: &mut Game) {
    word!(game.flag_for_palette_fx_objects |= (0x8000u16));
}
pub unsafe fn DisablePaletteFx(game: &mut Game) {
    word!(game.flag_for_palette_fx_objects &= !(0x8000));
}
pub unsafe fn ClearPaletteFXObjects(game: &mut Game) {
    word_transfers!(game, descending, 8; le(palettefx_ids, 7) <- value(0));
}
pub unsafe fn SpawnPalfxObject(j: u16, game: &mut Game) {
    let mut v1: u16 = 14;
    while word!(game.palettefx_ids[(v1 >> 1)]) != 0 {
        v1 = (v1).wrapping_sub(2);
        if (v1 & 0x8000) != 0 {
            return;
        }
    }
    let v2: i32 = v1 as i32 >> 1;
    word! {game; palettefx_ids[v2] = j; palettefx_color_indexes[v2] = 0;
    palettefx_pre_instr[v2] = ((0x8dc526i32) as u16);
    palettefx_instr_list_ptrs[v2] = (*get_PalFxDef(j, game)).instrs;
    palettefx_instr_timers[v2] = 1; palettefx_timers[v2] = 0; }
    let mut PalFxDef = get_PalFxDef(j, game);
    CallPalfxFunc(
        (((*PalFxDef).func) as u32) | (0x8d0000u32),
        -1i32 as u16,
        v1,
        game,
    );
}
pub fn PalPreInstr_nullsub_129(k: u16) {}
pub unsafe fn PaletteFxHandler(game: &mut Game) {
    if (word!(game.flag_for_palette_fx_objects) & 0x8000) != 0 {
        let mut i: i32 = 14;
        while i >= 0 {
            word!(game.palettefx_index = i as u16);
            if word!(game.palettefx_ids[(i >> 1)]) != 0 {
                PalFx_ProcessOne(i as u16, game);
                i = word!(game.palettefx_index) as i32;
            }
            i -= 2;
        }
    }
}
pub unsafe fn PalFx_ProcessOne(k: u16, game: &mut Game) {
    CallPalFxPreInstr(
        ((word!(game.palettefx_pre_instr[(k >> 1)])) as u32) | (0x8d0000u32),
        k,
        game,
    );
    let mut v1: u16 = word!(game.palettefx_index);
    crate::update_place! { unaligned_word fresh2 [fresh3] = &raw mut (*(ram_ptr!(game, u16; 0x1ecd)).offset((v1 >> 1) as isize)) => (u16::from_le(fresh2.read_unaligned())).wrapping_sub(1); }
    if fresh3 != 1 {
        return;
    }
    let mut j: u16 = word!(game.palettefx_instr_list_ptrs[(v1 >> 1)]);
    let mut v6: u16 = 0;
    loop {
        let p = RomPtr_8D(j, game) as *const u16;
        v6 = *p;
        if (*p & 0x8000) == 0 {
            break;
        }
        let v7: PairU16 =
            CallPalInstr(((v6) as u32) | (0x8d0000u32), v1, (j).wrapping_add(2), game);
        v1 = v7.k;
        j = v7.j;
        if v7.j == 0 {
            return;
        }
    }
    let v8: i32 = v1 as i32 >> 1;
    word!(game.palettefx_instr_timers[v8] = v6);
    let mut color_idx: u16 = word!(game.palettefx_color_indexes[v8]);
    let mut v11: u16 = 0;
    do_until! { (j == 0);
        loop {
            let mut v10 = RomPtr_8D(j, game);
            v11 = *(v10.offset(2) as *mut u16);
            if v11 & 0x8000 != 0 {
                break;
            }
            word!(game.palette_buffer[(color_idx >> 1)] = v11);
            color_idx = (color_idx).wrapping_add(2);
            j = (j).wrapping_add(2);
        }
        let v12: PairU16 = CallPalInstr(((v11) as u32) | (0x8d0000u32), color_idx, j, game);
        color_idx = v12.k;
        j = v12.j;
    }
}
crate::forward_calls! { pub unsafe fn (k: u16, j: u16) -> PairU16;
    PalInstr_ColorPlus2 => MakePairU16((k).wrapping_add(4), (j).wrapping_add(2));
    PalInstr_ColorPlus3 => MakePairU16((k).wrapping_add(6), (j).wrapping_add(2));
    PalInstr_ColorPlus4 => MakePairU16((k).wrapping_add(8), (j).wrapping_add(2));
    PalInstr_ColorPlus8 => MakePairU16((k).wrapping_add(16), (j).wrapping_add(2));
    PalInstr_ColorPlus9 => MakePairU16((k).wrapping_add(18), (j).wrapping_add(2));
    PalInstr_ColorPlus15 => MakePairU16((k).wrapping_add(30), (j).wrapping_add(2));
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPalFxPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [9298443] PalPreInstr_E1BC [] {
        if word!(game.cinematic_function) as i32 == 0x8bb35f & 0xffff {
            let v1: i32 = k as i32 >> 1;
            word!(game.palettefx_instr_list_ptrs[v1] = addr_off_8DE192 as u16);
            word!(game.palettefx_instr_timers[v1] = 1);
        }
    };
    [9298656] PalPreInstr_CheckEnemy0Health [] {
        if (*(ram_ptr!(game, EnemyData; 0xf78))).health == 0 {
            word!(game.palettefx_ids[(k >> 1)] = 0);
        }
    };
    [9298809] PalPreInstr_SamusInHeat [] {
        if (word!(game.equipped_items) & 0x21) == 0 {
            words!(
                game.samus_periodic_damage,
                samus_periodic_subdamage += 0x4000
            );
            if (word!(game.nmi_frame_counter_word) & 7) == 0 && word!(game.samus_health) > 0x46 {
                QueueSfx3_Max6(0x2d, game);
            }
        }
        if word!(game.samus_in_heat_palfx_index) != word!(game.samus_in_heat_palettefx_prev_index) {
            word!(game.samus_in_heat_palettefx_prev_index = word!(game.samus_in_heat_palfx_index));
            let v2: u16 = (2u16).wrapping_mul(word!(game.samus_in_heat_palfx_index));
            let v3: i32 = k as i32 >> 1;
            word!(game.palettefx_instr_timers[v3] = 1);
            let mut v4: u16 = if (word!(game.equipped_items) & 0x20) != 0 {
                addr_off_8DE3E0 as u16
            } else if (word!(game.equipped_items) & 1) != 0 {
                addr_off_8DE400 as u16
            } else {
                addr_off_8DE420 as u16
            };
            word!(
                game.palettefx_instr_list_ptrs[v3] =
                    *((RomPtr_8D as unsafe fn(u16, &mut Game) -> *const u8)(v4, game)
                        .offset(v2 as isize) as *mut u16)
            );
        }
    };
    [9301081] PalPreInstr_SwitchIfYpos [] {
        if word!(game.samus_y_pos) < 0x380 {
            let v2: i32 = k as i32 >> 1;
            word!(game.palettefx_instr_timers[v2] = 1);
            word!(game.palettefx_instr_list_ptrs[v2] = addr_word_8DEB43 as u16);
        }
    };
    [9301380] PalPreInstr_SwitchIfYpos2 [] {
        if word!(game.samus_y_pos) < 0x380 {
            let v2: i32 = k as i32 >> 1;
            word!(game.palettefx_instr_timers[v2] = 1);
            word!(game.palettefx_instr_list_ptrs[v2] = addr_word_8DEC76 as u16);
        }
    };
    [9301701] PalPreInstr_DeletePalfxIfMinibossDead [] {
        if (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize)) & 2) != 0 {
            word!(game.palettefx_ids[(k >> 1)] = 0);
        }
    };
    [9303585] PalPreInstr_F621 [] {
        if *((ram_ptr!(game, u16; 0x1e79) as *mut u8).offset(k as i32 as isize) as *mut u16) != 0 {
            word!(game.palettefx_ids[(k >> 1)] = 0);
        }
    };
    @external {
        9291046 => [PalPreInstr_nullsub_129(k)];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPalInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> PairU16 => ea;
    call (k, j, game); default {
            Unreachable(game);
            PairU16 { k: 0, j: 0 }
        };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> PairU16];
    [9291157] PalInstr_Wait [] {
        *(ram_ptr!(game, u16; 0x1ebd)).offset((word!(game.palettefx_index) >> 1) as isize) =
            (j).wrapping_add(4);
        MakePairU16(0, 0)
    };
    [9302470] PalInstr_SetPalfxIndex [] {
        word!(game.samus_in_heat_palfx_index = *RomPtr_8D(j, game) as u16);
        MakePairU16(k, (j).wrapping_add(1))
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> PairU16];
    [9291215] PalInstr_Delete [MakePairU16(k, 0)] { word!(game.palettefx_ids[(k >> 1)] = 0); };
    [9291220] PalInstr_SetPreInstr [MakePairU16(k, (j).wrapping_add(2))] {
        word!(game.palettefx_pre_instr[(k >> 1)] = *(RomPtr_8D(j, game) as *mut u16));
    };
    [9291229] PalInstr_ClearPreInstr [MakePairU16(k, j)] {
        word!(game.palettefx_pre_instr[(k >> 1)] = addr_locret_8DC5E3 as u16);
    };
    [9291294]
    PalInstr_Goto [MakePairU16(k, v2)] { let v2: u16 = *(RomPtr_8D(j, game) as *mut u16); };
    [9291321] PalInstr_DecTimerGoto [MakePairU16(v4.k, v4.j)] {
        let mut v4: PairU16 = PairU16 { k: 0, j: 0 };
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { unaligned_word fresh4 [fresh5] = &raw mut (*(ram_ptr!(game, u16; 0x1edd)).offset(v2 as isize)) => (u16::from_le(fresh4.read_unaligned())).wrapping_sub(1); }
        if fresh5 == 1 {
            return MakePairU16(k, (j).wrapping_add(2));
        }
        v4 = PalInstr_Goto(k, j, game);
    };
    [9291336] PalInstr_SetTimer [MakePairU16(k, (j).wrapping_add(1))] {
        *(ram_ptr!(game, u16; 0x1edd) as *mut u8).offset(k as i32 as isize) = *RomPtr_8D(j, game);
    };
    [9291349] PalInstr_SetColorIndex [MakePairU16(k, (j).wrapping_add(2))] {
        word!(game.palettefx_color_indexes[(k >> 1)] = *(RomPtr_8D(j, game) as *mut u16));
    };
    [9291358] PalInstr_QueueMusic [MakePairU16(k, (j).wrapping_add(1))] {
        let v2 = RomPtr_8D(j, game);
        QueueMusic_Delayed8(*v2 as u16, game);
    };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game) -> PairU16];
    @prefix { let v2 = RomPtr_8D(j, game) as *const u16; };
    [9291370]
    PalInstr_QueueSfx1 [MakePairU16(k, (j).wrapping_add(1))] {  QueueSfx1_Max6(*v2, game); };
    [9291379]
    PalInstr_QueueSfx2 [MakePairU16(k, (j).wrapping_add(1))] {  QueueSfx2_Max6(*v2, game); };
    [9291388]
    PalInstr_QueueSfx3 [MakePairU16(k, (j).wrapping_add(1))] {  QueueSfx3_Max6(*v2, game); };
    @external {
        9291161 => [PalInstr_ColorPlus2(k, j)];
        9291170 => [PalInstr_ColorPlus3(k, j)]; 9291179 => [PalInstr_ColorPlus4(k, j)];
        9291188 => [PalInstr_ColorPlus8(k, j)]; 9291197 => [PalInstr_ColorPlus9(k, j)];
        9291206 => [PalInstr_ColorPlus15(k, j)];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPalfxFunc(ea: u32, k: u16, j: u16, game: &mut Game) => ea;
    call (k, j, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, j: u16, game: &mut Game)];
    [9298436] PalInit_E1BC [] {
        word!(game.palettefx_pre_instr[(j >> 1)] = ((0x8de20bi32) as u16));
    };
    [9299008] PalInit_F761_Norfair1 [] {
        let mut v2: u16 = if (word!(game.equipped_items) & 0x20) != 0 {
            addr_off_8DE8B6 as u16
        } else if (word!(game.equipped_items) & 1) != 0 {
            addr_off_8DE68A as u16
        } else {
            addr_kPalfxInstrList_E45E as u16
        };
        word!(game.palettefx_instr_list_ptrs[(j >> 1)] = v2);
    };
    [9303856] PalInit_F779_Brinstar8 [] {
        if (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize)) & 2) != 0 {
            word!(game.palettefx_ids[(j >> 1)] = 0);
        }
    };
    @external {
        9291397 => [()];
    }
}
