pub use crate::game::constants::bank_8d::*;
use crate::game::sm_80::{QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::AddToHiLo;
use crate::rtl::{MakePairU16, RomPtr, Unreachable};
use ::core::ptr::{null, null_mut};

pub use crate::types::{EnemyData, PairU16, VoidP};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PalFxDef {
    pub func: VoidP,
    pub instrs: VoidP,
}
pub use crate::types::Consts_8D;
const RomPtr_8D: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8d>;
#[inline]
unsafe fn get_PalFxDef(a: u16, game: &mut Game) -> *mut PalFxDef {
    RomPtr((0x8d0000u32) | ((a) as u32), game) as *mut PalFxDef
}
crate::game::constants::typed_constants! { i32;
    fnCinematicFunction_Intro_Page2 = 0x8bb35f, fnPalPreInstr_nullsub_129 = 0x8dc526,
}
crate::game::constants::typed_constants! { u32;
    fnPalInstr_Wait = 9291157, fnPalInstr_ColorPlus2 = 9291161,
    fnPalInstr_ColorPlus3 = 9291170, fnPalInstr_ColorPlus4 = 9291179,
    fnPalInstr_ColorPlus8 = 9291188, fnPalInstr_ColorPlus9 = 9291197,
    fnPalInstr_ColorPlus15 = 9291206, fnPalInstr_Delete = 9291215,
    fnPalInstr_SetPreInstr = 9291220, fnPalInstr_ClearPreInstr = 9291229,
    fnPalInstr_Goto = 9291294, fnPalInstr_DecTimerGoto = 9291321,
    fnPalInstr_SetTimer = 9291336, fnPalInstr_SetColorIndex = 9291349,
    fnPalInstr_QueueMusic = 9291358, fnPalInstr_QueueSfx1 = 9291370,
    fnPalInstr_QueueSfx2 = 9291379, fnPalInstr_QueueSfx3 = 9291388,
    fnnullsub_131 = 9291397, fnPalInit_E1BC = 9298436,
}
crate::game::constants::typed_constants! { i32;
    fnPalPreInstr_E1BC = 0x8de20b,
}
crate::game::constants::typed_constants! { u32;
    fnPalPreInstr_CheckEnemy0Health = 9298656, fnPalPreInstr_SamusInHeat = 9298809,
    fnPalInit_F761_Norfair1 = 9299008, fnPalPreInstr_SwitchIfYpos = 9301081,
    fnPalPreInstr_SwitchIfYpos2 = 9301380, fnPalPreInstr_DeletePalfxIfMinibossDead = 9301701,
    fnPalInstr_SetPalfxIndex = 9302470, fnPalPreInstr_F621 = 9303585,
    fnPalInit_F779_Brinstar8 = 9303856,
}
pub unsafe fn EnablePaletteFx(game: &mut Game) {
    update_ram_word!(game.flag_for_palette_fx_objects, |fresh1| (fresh1)
        | (0x8000u16));
}
pub unsafe fn DisablePaletteFx(game: &mut Game) {
    update_ram_word!(
        game.flag_for_palette_fx_objects,
        |fresh0| (fresh0 as i32 & !(0x8000i32)) as u16
    );
}
pub unsafe fn ClearPaletteFXObjects(game: &mut Game) {
    let mut i: i32 = 14i32;
    while i >= 0i32 {
        set_ram_word!(game.palettefx_ids[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
}
pub unsafe fn SpawnPalfxObject(j: u16, game: &mut Game) {
    let mut PalFxDef: *mut PalFxDef = null_mut::<PalFxDef>();
    let mut v1: u16 = 14u16;
    while ram_word!(game, palettefx_ids[(v1 as i32 >> 1i32)]) != 0 {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            return;
        }
    }
    let v2: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.palettefx_ids[v2] = j);
    set_ram_word!(game.palettefx_color_indexes[v2] = 0u16);
    set_ram_word!(game.palettefx_pre_instr[v2] = ((0x8dc526i32) as u16));
    set_ram_word!(game.palettefx_instr_list_ptrs[v2] = (*get_PalFxDef(j, game)).instrs);
    set_ram_word!(game.palettefx_instr_timers[v2] = 1u16);
    set_ram_word!(game.palettefx_timers[v2] = 0u16);
    PalFxDef = get_PalFxDef(j, game);
    CallPalfxFunc(
        (((*PalFxDef).func) as u32) | (0x8d0000u32),
        -1i32 as u16,
        v1,
        game,
    );
}
pub fn PalPreInstr_nullsub_129(k: u16) {}
pub unsafe fn PaletteFxHandler(game: &mut Game) {
    if (ram_word!(game, flag_for_palette_fx_objects) as i32 & 0x8000i32) != 0i32 {
        let mut i: i32 = 14i32;
        while i >= 0i32 {
            set_ram_word!(game.palettefx_index = i as u16);
            if ram_word!(game, palettefx_ids[(i >> 1i32)]) != 0 {
                PalFx_ProcessOne(i as u16, game);
                i = ram_word!(game, palettefx_index) as i32;
            }
            i -= 2i32;
        }
    }
}
pub unsafe fn PalFx_ProcessOne(k: u16, game: &mut Game) {
    CallPalFxPreInstr(
        ((ram_word!(game, palettefx_pre_instr[(k as i32 >> 1i32)])) as u32) | (0x8d0000u32),
        k,
        game,
    );
    let mut v1: u16 = ram_word!(game, palettefx_index);
    let ref mut fresh2 = *(ram_ptr!(game, u16; 0x1ecd)).offset((v1 as i32 >> 1i32) as isize);
    let fresh3 = *fresh2;
    *fresh2 = (*fresh2).wrapping_sub(1);
    if fresh3 != 1u16 {
        return;
    }
    let mut j: u16 = ram_word!(game, palettefx_instr_list_ptrs[(v1 as i32 >> 1i32)]);
    let mut v6: u16 = 0;
    loop {
        let p: *const u16 = RomPtr_8D(j, game) as *const u16;
        v6 = *p;
        if (*p as i32 & 0x8000i32) == 0i32 {
            break;
        }
        let v7: PairU16 = CallPalInstr(
            ((v6) as u32) | (0x8d0000u32),
            v1,
            (j).wrapping_add(2u16),
            game,
        );
        v1 = v7.k;
        j = v7.j;
        if v7.j == 0 {
            return;
        }
    }
    let v8: i32 = v1 as i32 >> 1i32;
    set_ram_word!(game.palettefx_instr_timers[v8] = v6);
    let mut color_idx: u16 = ram_word!(game, palettefx_color_indexes[v8]);
    let mut v10: *const u8 = null::<u8>();
    let mut v11: u16 = 0;
    loop {
        loop {
            v10 = RomPtr_8D(j, game);
            v11 = *(v10.offset(2) as *mut u16);
            if v11 as i32 & 0x8000i32 != 0 {
                break;
            }
            set_ram_word!(game.palette_buffer[(color_idx as i32 >> 1i32)] = v11);
            color_idx = (color_idx).wrapping_add(2u16);
            j = (j).wrapping_add(2u16);
        }
        let v12: PairU16 = CallPalInstr(((v11) as u32) | (0x8d0000u32), color_idx, j, game);
        color_idx = v12.k;
        j = v12.j;
        if (j == 0) {
            break;
        }
    }
}
pub unsafe fn PalInstr_Wait(k: u16, j: u16, game: &mut Game) -> PairU16 {
    *(ram_ptr!(game, u16; 0x1ebd))
        .offset((ram_word!(game, palettefx_index) as i32 >> 1i32) as isize) =
        (j).wrapping_add(4u16);
    MakePairU16(0u16, 0u16)
}
pub unsafe fn PalInstr_ColorPlus2(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(4u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ColorPlus3(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(6u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ColorPlus4(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(8u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ColorPlus8(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(16u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ColorPlus9(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(18u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ColorPlus15(k: u16, j: u16) -> PairU16 {
    MakePairU16((k).wrapping_add(30u16), (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_Delete(k: u16, j: u16, game: &mut Game) -> PairU16 {
    set_ram_word!(game.palettefx_ids[(k as i32 >> 1i32)] = 0u16);
    MakePairU16(k, 0u16)
}
pub unsafe fn PalInstr_SetPreInstr(k: u16, j: u16, game: &mut Game) -> PairU16 {
    set_ram_word!(game.palettefx_pre_instr[(k as i32 >> 1i32)] = *(RomPtr_8D(j, game) as *mut u16));
    MakePairU16(k, (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_ClearPreInstr(k: u16, j: u16, game: &mut Game) -> PairU16 {
    set_ram_word!(game.palettefx_pre_instr[(k as i32 >> 1i32)] = addr_locret_8DC5E3 as u16);
    MakePairU16(k, j)
}
pub unsafe fn PalInstr_Goto(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let v2: u16 = *(RomPtr_8D(j, game) as *mut u16);
    MakePairU16(k, v2)
}
pub unsafe fn PalInstr_DecTimerGoto(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let mut v4: PairU16 = PairU16 { k: 0, j: 0 };
    let v2: i32 = k as i32 >> 1i32;
    let ref mut fresh4 = *(ram_ptr!(game, u16; 0x1edd)).offset(v2 as isize);
    let fresh5 = *fresh4;
    *fresh4 = (*fresh4).wrapping_sub(1);
    if fresh5 == 1u16 {
        return MakePairU16(k, (j).wrapping_add(2u16));
    }
    v4 = PalInstr_Goto(k, j, game);
    MakePairU16(v4.k, v4.j)
}
pub unsafe fn PalInstr_SetTimer(k: u16, j: u16, game: &mut Game) -> PairU16 {
    *(ram_ptr!(game, u16; 0x1edd) as *mut u8).offset(k as i32 as isize) = *RomPtr_8D(j, game);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalInstr_SetColorIndex(k: u16, j: u16, game: &mut Game) -> PairU16 {
    set_ram_word!(
        game.palettefx_color_indexes[(k as i32 >> 1i32)] = *(RomPtr_8D(j, game) as *mut u16)
    );
    MakePairU16(k, (j).wrapping_add(2u16))
}
pub unsafe fn PalInstr_QueueMusic(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let v2: *const u8 = RomPtr_8D(j, game);
    QueueMusic_Delayed8(*v2 as u16, game);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalInstr_QueueSfx1(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let v2: *const u16 = RomPtr_8D(j, game) as *const u16;
    QueueSfx1_Max6(*v2, game);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalInstr_QueueSfx2(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let v2: *const u16 = RomPtr_8D(j, game) as *const u16;
    QueueSfx2_Max6(*v2, game);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalInstr_QueueSfx3(k: u16, j: u16, game: &mut Game) -> PairU16 {
    let v2: *const u16 = RomPtr_8D(j, game) as *const u16;
    QueueSfx3_Max6(*v2, game);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalInit_E1BC(k: u16, j: u16, game: &mut Game) {
    set_ram_word!(game.palettefx_pre_instr[(j as i32 >> 1i32)] = ((0x8de20bi32) as u16));
}
pub unsafe fn PalPreInstr_E1BC(k: u16, game: &mut Game) {
    if ram_word!(game, cinematic_function) as i32 == 0x8bb35fi32 & 0xffffi32 {
        let v1: i32 = k as i32 >> 1i32;
        set_ram_word!(game.palettefx_instr_list_ptrs[v1] = addr_off_8DE192 as u16);
        set_ram_word!(game.palettefx_instr_timers[v1] = 1u16);
    }
}
pub unsafe fn PalPreInstr_CheckEnemy0Health(k: u16, game: &mut Game) {
    if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(0)).health == 0 {
        set_ram_word!(game.palettefx_ids[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn PalPreInstr_SamusInHeat(k: u16, game: &mut Game) {
    if (ram_word!(game, equipped_items) as i32 & 0x21i32) == 0i32 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xa50),
            ram_ptr!(game, u16; 0xa4e),
            0x4000u32,
        );
        if (ram_word!(game, nmi_frame_counter_word) as i32 & 7i32) == 0i32
            && ram_word!(game, samus_health) > 0x46u16
        {
            QueueSfx3_Max6(0x2du16, game);
        }
    }
    if ram_word!(game, samus_in_heat_palfx_index)
        != ram_word!(game, samus_in_heat_palettefx_prev_index)
    {
        set_ram_word!(
            game.samus_in_heat_palettefx_prev_index = ram_word!(game, samus_in_heat_palfx_index)
        );
        let v2: u16 = (2u16).wrapping_mul(ram_word!(game, samus_in_heat_palfx_index));
        let mut v4: u16 = 0;
        let v3: i32 = k as i32 >> 1i32;
        set_ram_word!(game.palettefx_instr_timers[v3] = 1u16);
        if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
            v4 = addr_off_8DE3E0 as u16;
        } else if (ram_word!(game, equipped_items) as i32 & 1i32) != 0i32 {
            v4 = addr_off_8DE400 as u16;
        } else {
            v4 = addr_off_8DE420 as u16;
        }
        set_ram_word!(
            game.palettefx_instr_list_ptrs[v3] =
                *((RomPtr_8D as unsafe fn(u16, &mut Game) -> *const u8)(v4, game)
                    .offset(v2 as isize) as *mut u16)
        );
    }
}
pub unsafe fn PalInit_F761_Norfair1(k: u16, j: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        v2 = addr_off_8DE8B6 as u16;
    } else if (ram_word!(game, equipped_items) as i32 & 1i32) != 0i32 {
        v2 = addr_off_8DE68A as u16;
    } else {
        v2 = addr_kPalfxInstrList_E45E as u16;
    }
    set_ram_word!(game.palettefx_instr_list_ptrs[(j as i32 >> 1i32)] = v2);
}
pub unsafe fn PalPreInstr_SwitchIfYpos(k: u16, game: &mut Game) {
    if ram_word!(game, samus_y_pos) < 0x380u16 {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.palettefx_instr_timers[v2] = 1u16);
        set_ram_word!(game.palettefx_instr_list_ptrs[v2] = addr_word_8DEB43 as u16);
    }
}
pub unsafe fn PalPreInstr_SwitchIfYpos2(k: u16, game: &mut Game) {
    if ram_word!(game, samus_y_pos) < 0x380u16 {
        let v2: i32 = k as i32 >> 1i32;
        set_ram_word!(game.palettefx_instr_timers[v2] = 1u16);
        set_ram_word!(game.palettefx_instr_list_ptrs[v2] = addr_word_8DEC76 as u16);
    }
}
pub unsafe fn PalPreInstr_DeletePalfxIfMinibossDead(k: u16, game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) as i32 & 2i32) != 0i32
    {
        set_ram_word!(game.palettefx_ids[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn PalInstr_SetPalfxIndex(k: u16, j: u16, game: &mut Game) -> PairU16 {
    set_ram_word!(game.samus_in_heat_palfx_index = *RomPtr_8D(j, game) as u16);
    MakePairU16(k, (j).wrapping_add(1u16))
}
pub unsafe fn PalPreInstr_F621(k: u16, game: &mut Game) {
    if *((ram_ptr!(game, u16; 0x1e79) as *mut u8).offset(k as i32 as isize) as *mut u16) != 0 {
        set_ram_word!(game.palettefx_ids[(k as i32 >> 1i32)] = 0u16);
    }
}
pub unsafe fn PalInit_F779_Brinstar8(k: u16, j: u16, game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) as i32 & 2i32) != 0i32
    {
        set_ram_word!(game.palettefx_ids[(j as i32 >> 1i32)] = 0u16);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallPalFxPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        9291046 => [PalPreInstr_nullsub_129(k)]; 9298443 => PalPreInstr_E1BC;
        9298656 => PalPreInstr_CheckEnemy0Health; 9298809 => PalPreInstr_SamusInHeat;
        9301701 => PalPreInstr_DeletePalfxIfMinibossDead; 9301081 => PalPreInstr_SwitchIfYpos;
        9301380 => PalPreInstr_SwitchIfYpos2; 9303585 => PalPreInstr_F621;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallPalInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> PairU16 => ea;
        call (k, j, game); default {
            Unreachable(game);
            PairU16 { k: 0u16, j: 0u16 }
        };
    {
        9291157 => PalInstr_Wait; 9291161 => [PalInstr_ColorPlus2(k, j)];
        9291170 => [PalInstr_ColorPlus3(k, j)]; 9291179 => [PalInstr_ColorPlus4(k, j)];
        9291188 => [PalInstr_ColorPlus8(k, j)]; 9291197 => [PalInstr_ColorPlus9(k, j)];
        9291206 => [PalInstr_ColorPlus15(k, j)]; 9291215 => PalInstr_Delete;
        9291220 => PalInstr_SetPreInstr; 9291229 => PalInstr_ClearPreInstr;
        9291294 => PalInstr_Goto; 9291321 => PalInstr_DecTimerGoto; 9291336 => PalInstr_SetTimer;
        9291349 => PalInstr_SetColorIndex; 9291358 => PalInstr_QueueMusic;
        9291370 => PalInstr_QueueSfx1; 9291379 => PalInstr_QueueSfx2; 9291388 => PalInstr_QueueSfx3;
        9302470 => PalInstr_SetPalfxIndex;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallPalfxFunc(ea: u32, k: u16, j: u16, game: &mut Game) => ea;
        call (k, j, game); default { Unreachable(game); };
    {
        9291397 => [()]; 9298436 => PalInit_E1BC; 9299008 => PalInit_F761_Norfair1;
        9303856 => PalInit_F779_Brinstar8;
    }
}
