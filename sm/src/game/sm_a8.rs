use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_runtime;
pub use crate::game::prelude::*;
use crate::game::session_runtime::correct_previous_position;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::ram_map::slot;

pub use crate::game::constants::bank_86::*;

use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::music::QueueSfx2_Max6;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::sprite_effect_runtime::{self as sprite_effects, create_sprite};
use crate::game::visual_runtime::initialize_morph_eye_beam;

pub use crate::types::Consts_A8;

crate::game::constants::typed_constants! { Consts_A8;
    addr_kBlueBrinstarFaceBlock_Ilist_E828 = 59432, addr_kBlueBrinstarFaceBlock_Ilist_E81A = 59418,
    addr_kBlueBrinstarFaceBlock_Ilist_E80C = 59404, addr_kWreckedShipSpark_Ilist_E5E5 = 58853,
    addr_kWreckedShipSpark_Ilist_E5A7 = 58791,
    addr_kNorfairLavaMan_Ilist_AE0C = 44556, addr_kNorfairLavaMan_Ilist_ADDC = 44508,
    addr_kNorfairLavaMan_Ilist_ADAC = 44460, addr_kNorfairLavaMan_Ilist_AD7E = 44414,
    addr_kNorfairLavaMan_Ilist_AD50 = 44368, addr_kNorfairLavaMan_Ilist_AD3C = 44348,
    addr_kNorfairLavaMan_Ilist_AD0C = 44300, addr_kNorfairLavaMan_Ilist_ACDE = 44254,
    addr_kNorfairLavaMan_Ilist_ACB0 = 44208, addr_kNorfairLavaMan_Ilist_AC9C = 44188,
    addr_kYappingMaw_Ilist_A079 = 41081, addr_off_A8A061 = 41057,
    addr_kYappingMaw_Ilist_A05B = 41051, addr_kYappingMaw_Ilist_A03D = 41021,
    addr_off_A8A025 = 40997, addr_kYappingMaw_Ilist_A01F = 40991,
    addr_kYappingMaw_Ilist_9FC7 = 40903, addr_kYappingMaw_Ilist_9F6F = 40815,
    addr_kMorphBallEye_Ilist_9026 = 36902, addr_kMorphBallEye_Ilist_9014 = 36884,
    addr_stru_A8900E = 36878, addr_kMorphBallEye_Ilist_9002 = 36866,
    addr_stru_A88FFC = 36860, addr_kMorphBallEye_Ilist_8FF0 = 36848,
    addr_kSpritemap_Nothing_A8 = 32845,
}
crate::type_aliases! {
    pub Enemy_NorfairLavaMan,
        Enemy_WreckedShipOrbs = crate::game::enemy_layouts::EnemyLayout40;
}

crate::type_aliases! {
    pub Enemy_WreckedShipSpark, Enemy_MorphBallEye =
        crate::game::enemy_layouts::EnemyLayout8;
}

pub type Enemy_YappingMaw = crate::game::enemy_layouts::EnemyLayout104;

const RomPtr_A8: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa8>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_152 = 0x90e90e, fnSamus_InputHandler_E913 = 0x90e913,
    fnSamus_InputHandler_E91D = 0x90e91d, fnnullsub_170_A8 = 0xa8804c,
    fnMorphBallEye_Func_1 = 0xa890f1, fnMorphBallEye_Func_2 = 0xa8912e,
    fnMorphBallEye_Func_3 = 0xa89160, fnMorphBallEye_Func_4 = 0xa891ce,
    fnnullsub_244 = 0xa891dc,
    fnNorfairLavaMan_Func_6 = 0xa8b0b2,
    fnNorfairLavaMan_Func_7 = 0xa8b11a, fnNorfairLavaMan_Func_8 = 0xa8b175,
    fnsub_A8B193 = 0xa8b193, fnNorfairLavaMan_Func_9 = 0xa8b1b8,
    fnNorfairLavaMan_Func_10 = 0xa8b1dd, fnNorfairLavaMan_Func_11 = 0xa8b204,
    fnsub_A8B291 = 0xa8b291, fnNorfairLavaMan_Func_13 = 0xa8b295,
    fnNorfairLavaMan_Func_15 = 0xa8b30d, fnNorfairLavaMan_Func_16 = 0xa8b31f,
    fnNorfairLavaMan_Func_17 = 0xa8b356, fnNorfairLavaMan_Func_18 = 0xa8b3a7,
    fnBlueBrinstarFaceBlock_Func_1 = 0xa8e86e,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    Get_YappingMaw(o: i32) -> Enemy_YappingMaw = [0xf78, o as isize];
}
type YappingMaw = Enemy_YappingMaw;
type WreckedShipSpark = Enemy_WreckedShipSpark;
type WreckedShipOrbs = Enemy_WreckedShipOrbs;
type NorfairLavaMan = Enemy_NorfairLavaMan;
type MorphBallEye = Enemy_MorphBallEye;
static g_word_A890CA: [i16; 4] = [-8, 8, 0, 0];
static g_word_A890D2: [i16; 4] = [0, 0, -8, 8];
static g_word_A89050: u16 = 0x80;
static g_word_A89052: u16 = 0xb0;
static g_word_A89054: u16 = 0x80;
static g_word_A89056: u16 = 0x80;
crate::game::callback_dispatch! {
    pub unsafe fn CallYappingMawFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_YappingMaw(word!(game.cur_enemy_index) as i32, game); };
    YappingMaw_Init [] {
        let sprite_owner = word!(game.cur_enemy_index);
        (*E).var_0C = (*E).base.x_pos;
        (*E).var_0D = (*E).base.y_pos;
        set_fields! {[(*E)], 0; var_00, var_01, var_02, var_03, var_04, var_05, var_06, var_07, var_30}
        (*E).var_F = (*E).parameter_1;
        (*E).var_E = 64;
        (*E).base.current_instruction = addr_kYappingMaw_Ilist_9F6F as u16;
        let mut R36: u16 = 57;
        let mut R34: u16 = 8;
        if (*E).parameter_2 == 0 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_9FC7 as u16;
            R36 = 56;
            R34 = -8i32 as u16;
        }
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_A = (0xa8a235i32) as u16;
        (*E).var_45 = ((*E).base.palette_index) & (0xe00u16);
        let mut v1: i16 = 3;
        (*E).var_44 = 3;
        do_until! { v1 < 0 ;
    SpawnEprojWithGfx(
        v1 as u16,
        word!(game.cur_enemy_index),
        addr_kEproj_YappingMawsBody as u16,
        game,
    );
    v1 = (((*E).var_44) as i16).wrapping_sub(1);
    (*E).var_44 = v1 as u16;
}
        let v3: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).var_47 = v3;
        let sprite = create_sprite(
            (*E).base.x_pos,
            (R34).wrapping_add((*E).base.y_pos),
            R36,
            v3,
            game,
        );
        slot!(game, YappingMaw[sprite_owner as i32].var_46 = sprite);
    };
    YappingMaw_Main [] {
        (*E).var_35 = (*E).var_35.wrapping_sub(1);
        (*E).var_36 = u16::from(actor_api::current_outside_screen(&Runtime::new(game)));
        CallYappingMawFunc((((*E).var_A) as u32) | (0xa80000u32), game);
        crate::game::enemy_projectiles::update_maw_segment(3, game);
        crate::game::enemy_projectiles::update_maw_segment(2, game);
        crate::game::enemy_projectiles::update_maw_segment(1, game);
        crate::game::enemy_projectiles::update_maw_segment(0, game);
    };
    [11051573] YappingMaw_Func_1 [] {
        let pair: PairU16 = { let (origin, target) = (Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            }, Point16U {
                x: word!(game.samus_x_pos),
                y: word!(game.samus_y_pos),
            }); { let result = actor_api::distance_and_direction(&mut Runtime::new(game), crate::game::session::Point { x: origin.x, y: origin.y }, crate::game::session::Point { x: target.x, y: target.y }); crate::types::PairU16 { k: result.distance, j: result.direction } } };
        let v2: u16 = abs16(pair.k);
        (*E).var_08 = v2;
        if v2 as i32 - 32i32 & 0x8000 != 0 {
            (*E).var_35 = 48;
        } else if ((v2) as i16).wrapping_sub(((*E).var_F) as i16) < 0 {
            if (*E).var_08 as i32 - 64i32 & 0x8000 == 0 {
                (*E).var_08 = 64;
            }
            (*E).var_0A = pair.j;
            (*E).var_A = (0xa8a28ci32) as u16;
        }
    };
    [11051660] YappingMaw_Func_2 [] {
        set_fields! {[(*E)], 0; var_B, var_C, var_D}
        (*E).var_09 = (*E).var_08 >> 1;
        let v1: u16 = ((64u8).wrapping_sub(((*E).var_0A) as u8)) as u16;
        (*E).var_0B = v1;
        if v1 as i32 - 128i32 & 0x8000 != 0 {
            (*E).var_2F = 0;
        } else {
            (*E).var_2F = 1;
        }
        YappingMaw_Func_3(game);
        let v2: u16 = (2 * (((((*E).var_0A) as u8).wrapping_add(16)) as i32 >> 5)) as u16;
        (*E).var_34 = v2;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa8a097, (v2 >> 1) as isize), timer = 1}
        (*E).base.timer = 0;
        (*E).var_32 = *((rom_ptr!(game, u16; 0xa8a0a7) as *mut u8)
            .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
        (*E).var_33 = *((rom_ptr!(game, u16; 0xa8a0a9) as *mut u8)
            .offset(((2u16).wrapping_mul(v2)) as i32 as isize) as *mut u16);
        (*E).var_A = (0xa8a445i32) as u16;
    };
    YappingMaw_Func_3 [] {
        (*E).var_0E = YappingMaw_Func_16(0x80, (*E).var_09, game);
        (*E).var_0F = YappingMaw_Func_17(0x80, (*E).var_09 >> 1, game);
    };
    YappingMaw_Func_4 [] {
        let varE32: u16 = (*E).var_00;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_27 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_28 = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_5 [] {
        let varE32: u16 = (*E).var_02;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_29 = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2A = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_6 [] {
        let varE32: u16 = (*E).var_04;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_2B = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2C = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    YappingMaw_Func_7 [] {
        let varE32: u16 = (*E).var_06;
        (*E).var_10 = YappingMaw_Func_16(0, varE32, game);
        (*E).var_11 = YappingMaw_Func_17(0, varE32, game);
        (*E).var_2D = (YappingMaw_Func_16((*E).var_0B, varE32, game)).wrapping_sub((*E).var_10);
        (*E).var_2E = (YappingMaw_Func_17((*E).var_0B, varE32, game)).wrapping_sub((*E).var_11);
    };
    [11052101] YappingMaw_Func_8 [] {
        let r20: u16 = (*E).var_B >> 2;
        if (*E).var_2F != 0 {
            (*E).var_21 = (128u16).wrapping_add(r20);
            (*E).var_22 = (128 + r20 as i32 * 2) as u16;
            (*E).var_23 = (128 + r20 as i32 * 3) as u16;
            (*E).var_24 = (128 + r20 as i32 * 4) as u16;
        } else {
            (*E).var_21 = (128u16).wrapping_sub(r20);
            (*E).var_22 = (128 - r20 as i32 * 2) as u16;
            (*E).var_23 = (128 - r20 as i32 * 3) as u16;
            (*E).var_24 = (128 - r20 as i32 * 4) as u16;
        }
        let mut varE32: u16 = (*E).var_09;
        (*E).var_00 = (YappingMaw_Func_16((*E).var_21, varE32, game)).wrapping_sub((*E).var_0E);
        (*E).var_02 = (YappingMaw_Func_16((*E).var_22, varE32, game)).wrapping_sub((*E).var_0E);
        (*E).var_04 = (YappingMaw_Func_16((*E).var_23, varE32, game)).wrapping_sub((*E).var_0E);
        (*E).var_06 = (YappingMaw_Func_16((*E).var_24, varE32, game)).wrapping_sub((*E).var_0E);
        varE32 = (*E).var_09 >> 1;
        (*E).var_01 = (YappingMaw_Func_17((*E).var_21, varE32, game)).wrapping_sub((*E).var_0F);
        (*E).var_03 = (YappingMaw_Func_17((*E).var_22, varE32, game)).wrapping_sub((*E).var_0F);
        (*E).var_05 = (YappingMaw_Func_17((*E).var_23, varE32, game)).wrapping_sub((*E).var_0F);
        (*E).var_07 = (YappingMaw_Func_17((*E).var_24, varE32, game)).wrapping_sub((*E).var_0F);
        YappingMaw_Func_4(game);
        YappingMaw_Func_5(game);
        YappingMaw_Func_6(game);
        YappingMaw_Func_7(game);
        (*E).var_00 = ((*E).var_00).wrapping_add((*E).var_27);
        (*E).var_01 = ((*E).var_01).wrapping_add((*E).var_28);
        (*E).var_02 = ((*E).var_02).wrapping_add((*E).var_29);
        (*E).var_03 = ((*E).var_03).wrapping_add((*E).var_2A);
        (*E).var_04 = ((*E).var_04).wrapping_add((*E).var_2B);
        (*E).var_05 = ((*E).var_05).wrapping_add((*E).var_2C);
        (*E).var_06 = ((*E).var_06).wrapping_add((*E).var_2D);
        (*E).var_07 = ((*E).var_07).wrapping_add((*E).var_2E);
        (*E).base.x_pos = ((*E).var_06).wrapping_add((*E).var_0C);
        (*E).base.y_pos = ((*E).var_07).wrapping_add((*E).var_0D);
        YappingMaw_Func_9(word!(game.cur_enemy_index), game);
        if (*E).var_B & 0x8000 == 0 {
            if (*E).var_B as i32 - 128i32 & 0x8000 == 0 {
                (*E).var_B = 128;
                (*E).var_C = 0;
                (*E).var_D = ((*E).var_D).wrapping_add(4);
            }
        } else {
            (*E).var_A = (0xa8a68ai32) as u16;
            (*E).var_35 = 48;
            if (*E).var_34 == 4 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A01F as u16;
            } else if (*E).var_34 == 12 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A03D as u16;
            } else {
                (*E).base.current_instruction = addr_off_A8A025 as u16;
            }
            if (*E).parameter_2 == 0 {
                if (*E).var_34 == 4 {
                    (*E).base.current_instruction = addr_kYappingMaw_Ilist_A05B as u16;
                } else {
                    if (*E).var_34 != 12 {
                        (*E).base.current_instruction = addr_off_A8A061 as u16;
                        return;
                    }
                    (*E).base.current_instruction = addr_kYappingMaw_Ilist_A079 as u16;
                }
            } else {
                return;
            }
        }
        if (*E).var_30 != 0 {
            YappingMaw_Func_10(game);
        }
    };
    [11052682] YappingMaw_Func_11 [] {
        if (*E).var_30 != 0 {
            YappingMaw_Func_10(game);
        }
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        if ((*E).var_E & 0x8000) != 0 && word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
            word!(game.samus_input_handler = (0x90e913i32) as u16);
            (*E).var_30 = 0;
            (*E).var_35 = 48;
            (*E).var_E = 64;
            (*E).var_A = (0xa8a235i32) as u16;
        }
    };




    YappingMaw_Touch [] {
        if ((*E).var_35 & 0x8000) != 0 && (*E).var_30 == 0 {
            (*E).var_35 = 0;
            (*E).var_30 = 1;
            word!(game.samus_input_handler = (0x90e90ei32) as u16);
        }
    };
    YappingMaw_Func_19 [] {
        let enemy = word!(game.cur_enemy_index);
        let appearance = (*E).var_45;
        let handle = (*E).var_46;
        sprite_effects::maw_appearance(handle, appearance, enemy, game);
        crate::verify_probe::hit("sprite.maw.appearance");
    };
}
pub unsafe fn YappingMaw_Func_9(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    let v2: i32 = slot!(game, YappingMaw[E].var_D) as i32 >> 1;
    slot!(
        game,
        YappingMaw[E](var_B, var_C) += words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize)
        ))
    );
    slot!(game, YappingMaw[E].var_D, |old| (old).wrapping_add(8));
}
pub unsafe fn YappingMaw_Func_10(game: &mut Game) {
    CallSomeSamusCode(3, game);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    word!(
        game.samus_x_pos =
            (slot!(game, YappingMaw[E].var_32)).wrapping_add(slot!(game, YappingMaw[E].base.x_pos))
    );
    word!(
        game.samus_y_pos =
            (slot!(game, YappingMaw[E].var_33)).wrapping_add(slot!(game, YappingMaw[E].base.y_pos))
    );
    correct_previous_position(game);
}
pub unsafe fn YappingMaw_Func_16(a: u16, varE32: u16, game: &mut Game) -> u16 {
    unsafe { YappingMaw_Func_17((a).wrapping_sub(64), varE32, game) }
}
pub unsafe fn YappingMaw_Func_17(a: u16, varE32: u16, game: &mut Game) -> u16 {
    unsafe {
        let mut r20: u16 = 0;
        let mut r22: u16 = 0;
        let mut v1: i16 =
            crate::rtl::rom_table_word!(game, 0xa0b1c3, -(a as i32) as u8 as isize) as i16;
        if v1 < 0 {
            v1 = -(v1 as i32) as i16;
            r20 = r20.wrapping_add(1);
        }
        WriteReg(WRMPYA as u16, byte!(v1; 1), game);
        if varE32 as u8 != 0 {
            let prod: u16 = Mult8x8(byte!(v1; 1), varE32 as u8);
            r22 = (2 * ((prod as i32 & 0xff00) >> 8)) as u16;
            if r20 != 0 {
                r22 = (2 * (-(prod as i32) as u16 as i32 >> 8) | 0xff00) as u16;
            }
        }
        r22
    }
}

static g_off_A8AF4F: [Func_V; 3] = callback_table! { nonnull Func_V;
    NorfairLavaMan_Func_1, NorfairLavaMan_Func_2, NorfairLavaMan_Func_3,
};
pub unsafe fn NorfairLavaMan_Func_5(k: u16, game: &mut Game) {
    unsafe {
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = (0xa8b0b2i32) as u16;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168;
        word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((((slot!(game, NorfairLavaMan[k as i32].base.palette_index)) & (0xe00u16)) as i32
                    >> 4)
                    + 256) as u16
        );
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
        word!(game.variables_for_enemy_graphics_drawn_hook[1] = 0);
    }
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    {
        if word!(game.door_transition_flag_enemies) == 0 && {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) == 0
        } {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
            word!(game.variables_for_enemy_graphics_drawn_hook[1] += 1);
            let v0: i32 = (32 * (word!(game.variables_for_enemy_graphics_drawn_hook[1]) as i32 & 3))
                as u16 as i32
                >> 1;
            let r18: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 9) as isize);
            let r20: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 10) as isize);
            let r22: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 11) as isize);
            let r24: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 12) as isize);
            let v1: i32 = word!(game.variables_for_enemy_graphics_drawn_hook[0]) as i32 >> 1;
            word! {game; palette_buffer[(v1 + 9)] = r18; palette_buffer[(v1 + 10)] = r20;
            palette_buffer[(v1 + 11)] = r22; palette_buffer[(v1 + 12)] = r24; }
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_12(k: u16, game: &mut Game) {
    unsafe {
        let mut v3: i32 = 0;
        let E: i32 = k as i32;
        if -(slot!(game, NorfairLavaMan[E].var_E) as i32) - 108i32 & 0x8000 == 0 {
            printf(
                c"Y undefined %d!\n".as_ptr(),
                -(slot!(game, NorfairLavaMan[E].var_E) as i16 as i32),
            );
        }
        let nlmn_var_B: u16 = slot!(game, NorfairLavaMan[E].var_B);
        let advance_lava_phase: bool =
            if -(slot!(game, NorfairLavaMan[E].var_E) as i32) - 108i32 & 0x8000 == 0 || {
                v3 = nlmn_var_B as i32 >> 1;
                ((slot!(game, NorfairLavaMan[E].base.y_pos) as i32
                    - crate::rtl::rom_table_word!(game, 0xa8af79, v3 as isize) as i32
                    - word!(game.samus_y_pos) as i32) as i16)
                    < 0
            } {
                slot!(game, NorfairLavaMan[E].var_F = (0xa8b291i32) as u16);
                slot!(game, NorfairLavaMan[E].var_0C = 1);
                if ((-(slot!(game, NorfairLavaMan[E].var_E) as i32)
                    - crate::rtl::rom_table_word!(game, 0xa8af55, (nlmn_var_B >> 1) as isize)
                        as i32) as i16)
                    < 0
                {
                    return;
                }
                true
            } else {
                ((-(slot!(game, NorfairLavaMan[E].var_E) as i32)
                    - crate::rtl::rom_table_word!(game, 0xa8af55, v3 as isize) as i32)
                    as i16)
                    >= 0
            };
        if advance_lava_phase {
            slot!(game, NorfairLavaMan[E].var_B, |old| old.wrapping_add(1));
            slot!(game, NorfairLavaMan[E].var_B, |old| old.wrapping_add(1));
            slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old)
                .wrapping_add(8));
            slot!(
                game,
                NorfairLavaMan[E].var_D = crate::rtl::rom_table_word!(
                    game,
                    0xa8af67,
                    (slot!(game, NorfairLavaMan[E].var_B) >> 1) as isize
                )
            );
            NorfairLavaMan_Func_20(game);
        };
    }
}
pub fn sub_A8B291(k: u16) {}
pub unsafe fn NorfairLavaMan_Func_14(k: u16, game: &mut Game) {
    unsafe {
        let E: i32 = k as i32;
        let mut nlmn_var_E: i16 = slot!(game, NorfairLavaMan[E].var_E) as i16;
        if nlmn_var_E >= 0 {
            slot!(game, NorfairLavaMan[E].var_0D = 1);
            (*gEnemySpawnData(k as i32, game).offset(31)).field_4 = 0;
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b1ddi32) as u16);
        } else if ((-(nlmn_var_E as i32)
            - crate::rtl::rom_table_word!(
                game,
                0xa8af55,
                (((slot!(game, NorfairLavaMan[E].var_B)).wrapping_sub(2)) >> 1) as isize
            ) as i32) as i16)
            < 0
        {
            slot!(game, NorfairLavaMan[E].var_B, |old| (old).wrapping_sub(2));
            slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old)
                .wrapping_sub(8));
            slot!(
                game,
                NorfairLavaMan[E].var_D = crate::rtl::rom_table_word!(
                    game,
                    0xa8af67,
                    (slot!(game, NorfairLavaMan[E].var_B) >> 1) as isize
                )
            );
            NorfairLavaMan_Func_20(game);
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_19(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (word!(game.enemy_drawing_queue[(v1 + 61)]) as i32
        - slot!(game, NorfairLavaMan[k as i32].base.y_pos) as i32
        + 2) as u16;
    word!(game.enemy_drawing_queue[(v1 + 64)] = v2);
    if v2 as i32 - 8i32 & 0x8000 != 0 {
        word!(game.enemy_drawing_queue[(v1 + 64)] = 8);
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallWreckedShipOrbsA(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    WreckedShipOrbs_Init [] {
        slot!(game, WreckedShipOrbs[E].base.instruction_timer = 1);
        slot!(game, WreckedShipOrbs[E].base.timer = 0);
        slot!(game, WreckedShipOrbs[E].base.current_instruction = crate::rtl::rom_table_word!(game, 0xa8e380, slot!(game, WreckedShipOrbs[E].parameter_1) as isize))
;
        let v1: i32 = 8 * slot!(game, WreckedShipOrbs[E].parameter_2) as i32 >> 1;
        slot!(game, WreckedShipOrbs[E].var_01 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize));
        slot!(game, WreckedShipOrbs[E].var_00 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize));
        slot!(game, WreckedShipOrbs[E].var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize));
        slot!(game, WreckedShipOrbs[E].var_02 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize));
    };
    WreckedShipOrbs_Func_1 [] {
        slot!(game, WreckedShipOrbs[E].var_A = (0xa8e405i32) as u16);
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y) }) & 0x8000) == 0 {
            slot!(game, WreckedShipOrbs[E].var_A = (0xa8e424i32) as u16);
        }
    };
    WreckedShipOrbs_Func_2 [] {
        slot!(game, WreckedShipOrbs[E].var_B = (0xa8e443i32) as u16);
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
            slot!(game, WreckedShipOrbs[E].var_B = (0xa8e462i32) as u16);
        }
    };
    [11068421] WreckedShipOrbs_Func_3 [] {
        slot!(game, WreckedShipOrbs[E].base(y_pos, y_subpos) += crate::rtl::words!((slot!(game, WreckedShipOrbs[E].var_03), slot!(game, WreckedShipOrbs[E].var_02))));
    };
    [11068452] WreckedShipOrbs_Func_4 [] {
        slot!(game, WreckedShipOrbs[E].base(y_pos, y_subpos) += crate::rtl::words!((slot!(game, WreckedShipOrbs[E].var_01), slot!(game, WreckedShipOrbs[E].var_00))));
    };
    WreckedShipOrbs_Func_5 [] {
        slot!(game, WreckedShipOrbs[E].base(x_pos, x_subpos) += crate::rtl::words!((slot!(game, WreckedShipOrbs[E].var_03), slot!(game, WreckedShipOrbs[E].var_02))));
    };
    WreckedShipOrbs_Func_6 [] {
        slot!(game, WreckedShipOrbs[E].base(x_pos, x_subpos) += crate::rtl::words!((slot!(game, WreckedShipOrbs[E].var_01), slot!(game, WreckedShipOrbs[E].var_00))));
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallWreckedShipOrbsB(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        11068483 => WreckedShipOrbs_Func_5; 11068514 => WreckedShipOrbs_Func_6;
    }
}
unsafe fn WreckedShipSpark_Func_4(k: u16, r18: u16, game: &mut Game) {
    {
        let E: i32 = k as i32;
        let mut wssk_var_E: i16 = slot!(game, WreckedShipSpark[E].var_E) as i16;
        if wssk_var_E < 0 {
            wssk_var_E = ((game.next_random() as i32 & 0x3f) + 4) as i16;
        }
        slot!(
            game,
            WreckedShipSpark[E].var_F = (r18).wrapping_add((wssk_var_E) as u16)
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallWreckedShipSpark(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
    }
    when slot!(game, WreckedShipSpark[E].var_F) == 1;
    [11069077] WreckedShipSpark_Func_1 {
        slot!(game, WreckedShipSpark[E].var_B = (0xa8e6b7i32) as u16);
        slot!(game, WreckedShipSpark[E].base; current_instruction = addr_kWreckedShipSpark_Ilist_E5A7 as u16, instruction_timer = 1);
        WreckedShipSpark_Func_4(k, 0, game);
    } else {
        slot!(game, WreckedShipSpark[E].var_F, |old| old.wrapping_sub(1));
    } after { ; };
    [11069111] WreckedShipSpark_Func_2 {
        slot!(game, WreckedShipSpark[E].var_B = (0xa8e695i32) as u16);
        slot!(game, WreckedShipSpark[E].base; current_instruction = addr_kWreckedShipSpark_Ilist_E5E5 as u16, instruction_timer = 1);
        WreckedShipSpark_Func_4(k, 8, game);
    } else {
        slot!(game, WreckedShipSpark[E].var_F, |old| old.wrapping_sub(1));
    } after { ; };
    [11069148] WreckedShipSpark_Func_3 {
        let v2: u16 = word!(game.cur_enemy_index);
        SpawnEprojWithGfx(
            0,
            word!(game.cur_enemy_index),
            addr_kEproj_Sparks as u16,
            game,
        );
        WreckedShipSpark_Func_4(v2, 0, game);
    } else {
        slot!(game, WreckedShipSpark[E].var_F, |old| old.wrapping_sub(1));
    } after { ; };
    @external {
        11069076 => [()];
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Func_1(game: &mut Game) {
    unsafe {
        if word!(game.door_transition_flag_enemies) == 0 && {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) == 0
        } {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
            let mut v0: u16 = word!(game.variables_for_enemy_graphics_drawn_hook[0]);
            let mut v1: u16 =
                (8u16).wrapping_mul(word!(game.variables_for_enemy_graphics_drawn_hook[1]));
            for _ in 0..4 {
                *(ram_ptr!(game, u16; 0xc000)).offset(((v0 as i32 >> 1) + 137) as isize) =
                    crate::rtl::rom_table_word!(game, 0xa8e7cc, (v1 >> 1) as isize);
                v1 = (v1).wrapping_add(2);
                v0 = (v0).wrapping_add(2);
            }
            word!(
                game.variables_for_enemy_graphics_drawn_hook[1] =
                    (*((ram_ptr!(game, u16; 0x1794)).offset(1) as *mut u8) as i32 + 1 & 7) as u16
            );
        }
    }
}
crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11047000] MorphBallEye_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, MorphBallEye[E].base.properties, |old| (old) | ((kEnemyProps_DisableSamusColl) as u16));
        slot!(game, MorphBallEye[E].base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as u16);
        slot!(game, MorphBallEye[E].base.instruction_timer = 1);
        slot!(game, MorphBallEye[E].base.timer = 0);
        if (slot!(game, MorphBallEye[E].parameter_2) & 0x8000) != 0 {
            let v1: i32 = slot!(game, MorphBallEye[E].parameter_2) as i32 & 0xf;
            slot!(game, MorphBallEye[E].base.x_pos, |old| (old).wrapping_add((g_word_A890CA[v1 as usize]) as u16));
            slot!(game, MorphBallEye[E].base.y_pos, |old| (old).wrapping_add((g_word_A890D2[v1 as usize]) as u16));
            slot!(game, MorphBallEye[E].var_F = (0xa891dci32) as u16);
            slot!(game, MorphBallEye[E].base.current_instruction = crate::rtl::rom_table_word!(game, 0xa890da, v1 as isize))
;
            word_transfers!(game, descending, 256; native(0x92fe) <- value(255));
        } else {
            slot!(game, MorphBallEye[E].var_F = (0xa890f1i32) as u16);
            if (slot!(game, MorphBallEye[E].parameter_1) & 1) != 0 {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_stru_A8900E as u16);
            } else {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_stru_A88FFC as u16);
            }
        };
    };
    [11047138] MorphBallEye_Main [] {
        if (word!(game.collected_items) & 4) != 0 {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            actor_species_pre_instruction(((slot!(game, MorphBallEye[E].var_F)) as u32) | (0xa80000u32), game);
        }
    };
    [11052989] YappingMaw_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, YappingMaw[E].base.health) != 0 {
            if slot!(game, YappingMaw[E].base.frozen_timer) != 0 {
                if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                    word!(game.samus_input_handler = (0x90e913i32) as u16);
                }
                slot!(game, YappingMaw[E].var_30 = 0);
            }
        } else {
            *(ram_ptr!(game, u16; 0x1997, slot!(game, YappingMaw[E].var_40) as i32 as isize)) = 0;
            *(ram_ptr!(game, u16; 0x1997, slot!(game, YappingMaw[E].var_41) as i32 as isize)) = 0;
            *(ram_ptr!(game, u16; 0x1997, slot!(game, YappingMaw[E].var_42) as i32 as isize)) = 0;
            *(ram_ptr!(game, u16; 0x1997, slot!(game, YappingMaw[E].var_43) as i32 as isize)) = 0;
            let sprite = slot!(game, YappingMaw[E].var_46);
            sprite_effects::retire(sprite, game);
            if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                word!(game.samus_input_handler = (0x90e913i32) as u16);
            }
            slot!(game, YappingMaw[word!(game.cur_enemy_index) as i32].var_30 = 0);
        };
    };
    [11053109] YappingMaw_Frozen [] {
        Enemy_NormalFrozenAI_A8(game);
        let E: i32 = word!(game.cur_enemy_index) as i32;
        crate::game::enemy_projectiles::update_maw_palette(slot!(game, YappingMaw[E].var_40), game);
        crate::game::enemy_projectiles::update_maw_palette(slot!(game, YappingMaw[E].var_41), game);
        crate::game::enemy_projectiles::update_maw_palette(slot!(game, YappingMaw[E].var_42), game);
        crate::game::enemy_projectiles::update_maw_palette(slot!(game, YappingMaw[E].var_43), game);
        YappingMaw_Func_19(game);
    };
    [11054987] NorfairLavaMan_Init [] {
        let v0: u16 = (2u16).wrapping_mul(
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].parameter_1),
        );
        g_off_A8AF4F[(v0 >> 1) as usize](game);
        NorfairLavaMan_Func_4(game);
        NorfairLavaMan_Func_5(word!(game.cur_enemy_index), game);
    };
    [11056128] NorfairLavaMan_Powerbomb [] {
        Enemy_NormalPowerBombAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056134] NorfairLavaMan_Touch [] {
        Enemy_NormalTouchAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056140] NorfairLavaMan_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        NorfairLavaMan_Common(game);
    };
    [11068355] WreckedShipOrbs_Main [] {
        WreckedShipOrbs_Func_1(game);
        WreckedShipOrbs_Func_2(game);
        let E: i32 = word!(game.cur_enemy_index) as i32;
        CallWreckedShipOrbsA(((slot!(game, WreckedShipOrbs[E].var_A)) as u32) | (0xa80000u32), game);
        CallWreckedShipOrbsB(((slot!(game, WreckedShipOrbs[E].var_B)) as u32) | (0xa80000u32), game);
    };
    [11068983] WreckedShipSpark_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v2: i32 = slot!(game, WreckedShipSpark[E].parameter_1) as i32 & 3;
        slot!(game, WreckedShipSpark[E].var_B = crate::rtl::rom_table_word!(game, 0xa8e688, v2 as isize));
        slot!(game, WreckedShipSpark[E].var_E = slot!(game, WreckedShipSpark[E].parameter_2));
        WreckedShipSpark_Func_4(word!(game.cur_enemy_index), 0, game);
        slot!(game, WreckedShipSpark[E].base; instruction_timer = 1, current_instruction = crate::rtl::rom_table_word!(game, 0xa8e682, v2 as isize));
        slot!(game, WreckedShipSpark[E].base.timer = 0);
        slot!(game, WreckedShipSpark[E].base.instruction_timer = 1);
        if (gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) & 1) == 0 {
            slot!(game, WreckedShipSpark[E].base.properties, |old| (old)
                | (*((&raw mut (*(gVramWriteEntry
                    as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                    0i32, game
                )
                .offset(6))
                .vram_dst as *mut u8)
                    .offset(1) as *mut u16)));
        }
    };
    [11069070] WreckedShipSpark_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        CallWreckedShipSpark(
            ((slot!(game, WreckedShipSpark[E].var_B)) as u32) | (0xa80000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11069198] WreckedShipSpark_Shot [] {
        word!(game.projectile_dir[word!(game.collision_detection_index)] &= !(0x10));
    };
    [11069486] BlueBrinstarFaceBlock_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).current_instruction = addr_kBlueBrinstarFaceBlock_Ilist_E828 as u16;
        let mut v1: u16 = (0xa8e86ei32) as u16;
        if (word!(game.collected_items) & 4) == 0 {
            v1 = (0xa8804ci32) as u16;
        }
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = v1;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168;
        word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((16 * (*v0).palette_index as i32 & 0xff00) >> 8) as u16
        );
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
        (*v0).parameter_2 =
            (((*v0).parameter_2 as i32 & 1) >> 2 | ((*v0).parameter_2 as i32 & 1) << 15) as u16;
    };
    [11069614] BlueBrinstarFaceBlock_Main [] {
        if (word!(game.collected_items) & 4) != 0 {
            (*(ram_ptr!(game, LongPtr; 0x178c))).addr = (0xa8e86ei32) as u16;
            let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
            word!(
                game.variables_for_enemy_graphics_drawn_hook[0] =
                    ((16 * (*v1).palette_index as i32 & 0xff00) >> 8) as u16
            );
            if (*v1).ai_var_A == 0 {
                let SamusEnemyDelta_Y: u16 =
                    { let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y) };
                if ((abs16(SamusEnemyDelta_Y)) as i16).wrapping_sub(((*v1).parameter_1) as i16) < 0
                {
                    let SamusEnemyDelta_X: u16 =
                        { let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) };
                    (*v1).ai_var_B = SamusEnemyDelta_X;
                    if ((abs16(SamusEnemyDelta_X)) as i16).wrapping_sub(((*v1).parameter_1) as i16)
                        < 0
                        && (*v1).ai_var_B & 0x8000 != (*v1).parameter_2
                    {
                        let mut v4: u16 = addr_kBlueBrinstarFaceBlock_Ilist_E80C as u16;
                        if ((*v1).ai_var_B & 0x8000) == 0 {
                            v4 = addr_kBlueBrinstarFaceBlock_Ilist_E81A as u16;
                        }
                        enemy_instruction! {[(*v1)]; instruction = v4, timer = 1}
                        (*v1).ai_var_A = 1;
                        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
                    }
                }
            }
        }
    };
    [11069725] BlueBrinstarFaceBlock_Shot [] {
        word!(game.projectile_dir[word!(game.collision_detection_index)] &= !(0x10));
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    NorfairLavaMan_Func_1 [] {
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_00 = 0, var_02 = 0);
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
            slot!(game, NorfairLavaMan[E].var_00 = 1);
        }
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
        if slot!(game, NorfairLavaMan[E].var_00) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
        }
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b11ai32) as u16);
    };
    NorfairLavaMan_Func_2 [] {
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_0C = 0);
        slot!(game, NorfairLavaMan[E].var_0D = 1);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ADDC as u16);
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old).wrapping_add(32));
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b1ddi32) as u16);
        slot!(game, NorfairLavaMan[E].base.properties, |old| (old) | ((kEnemyProps_Invisible) as u16));
    };
    NorfairLavaMan_Func_3 [] {
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        slot!(game, NorfairLavaMan[E].var_11 = slot!(game, NorfairLavaMan[E].base.x_pos));
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_04 = 0);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16);
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old).wrapping_add(32));
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b30di32) as u16);
        slot!(game, NorfairLavaMan[E].base.properties, |old| (old) | ((kEnemyProps_Invisible) as u16));
    };
    NorfairLavaMan_Func_4 [] {
        let v1: i32 = 8 * slot!(byte game, NorfairLavaMan[E].parameter_2[1]) as i32 >> 1;
        slot!(game, NorfairLavaMan[E].var_08 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize));
        slot!(game, NorfairLavaMan[E].var_07 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize));
        slot!(game, NorfairLavaMan[E].var_0A = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize));
        slot!(game, NorfairLavaMan[E].var_09 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize));
    };
    [11055370] NorfairLavaMan_Main [] {
        slot!(game, NorfairLavaMan[E].var_04, |old| old.wrapping_sub(1));
        actor_species_pre_instruction(((slot!(game, NorfairLavaMan[E].var_F)) as u32) | (0xa80000u32), game);
    };
    NorfairLavaMan_Func_13 [] {
        let mut v2 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, slot!(game, NorfairLavaMan[E].base.y_subpos)).1;
        slot!(game, NorfairLavaMan[E].base.y_subpos = ((*v2.offset(31)).field_E).wrapping_add(slot!(game, NorfairLavaMan[E].base.y_subpos)));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16);
        slot!(game, NorfairLavaMan[E].var_E, |old| (old as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16);
        NorfairLavaMan_Func_14(word!(game.cur_enemy_index), game);
    };
    NorfairLavaMan_Func_20 [] {
        let nlmn_var_D: u16 = slot!(game, NorfairLavaMan[E].var_D);
        if nlmn_var_D != slot!(game, NorfairLavaMan[E].var_C) {
            slot!(game, NorfairLavaMan[E].base.current_instruction = nlmn_var_D);
            slot!(game, NorfairLavaMan[E].var_C = nlmn_var_D);
            slot!(game, NorfairLavaMan[E].base.instruction_timer = 1);
            slot!(game, NorfairLavaMan[E].base.timer = 0);
        }
    };
    NorfairLavaMan_Common [] {
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        if slot!(game, NorfairLavaMan[E].base.health) == 0 {
            slot!(game, NorfairLavaMan[E1].base.properties, |old| (old) | (0x200u16));
            slot!(game, NorfairLavaMan[E2].base.properties, |old| (old) | (0x200u16));
        }
        let frozen_timer: u16 = slot!(game, NorfairLavaMan[E].base.frozen_timer);
        if frozen_timer != 0 {
            slot!(game, NorfairLavaMan[E1].base.frozen_timer = frozen_timer);
            slot!(game, NorfairLavaMan[E1].base.ai_handler_bits, |old| (old) | (4u16));
            slot!(game, NorfairLavaMan[E2].base.frozen_timer = frozen_timer);
            slot!(game, NorfairLavaMan[E2].base.ai_handler_bits, |old| (old) | (4u16));
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11042826] Enemy_GrappleReact_KillEnemy_A8 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [11042831] Enemy_GrappleReact_CancelBeam_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042846] Enemy_GrappleReact_HurtSamus_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042851] Enemy_NormalTouchAI_A8 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [11042861] Enemy_NormalShotAI_A8 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    Enemy_NormalPowerBombAI_A8 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [11042881] Enemy_NormalFrozenAI_A8 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11051207] YappingMaw_Instr_2 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0ab, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0ad, 0));
    };
    [11051225] YappingMaw_Instr_4 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0c3, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0c5, 0));
    };
    [11051243] YappingMaw_Instr_5 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b3, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b5, 0));
    };
    [11051261] YappingMaw_Instr_7 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0bb, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0bd, 0));
    };
    [11051279] YappingMaw_Instr_3 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0a7, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0a9, 0));
    };
    [11051297] YappingMaw_Instr_6 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, YappingMaw[E].var_32 = crate::rtl::rom_table_word!(game, 0xa8a0b7, 0));
        slot!(game, YappingMaw[E].var_33 = crate::rtl::rom_table_word!(game, 0xa8a0b9, 0));
    };
    [11051315] YappingMaw_Instr_1 [jp] {
        if slot!(game, YappingMaw[word!(game.cur_enemy_index) as i32].var_36) == 0 {
            QueueSfx2_Max6(0x2f, game);
        }
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11054610] NorfairLavaMan_Instr_1 [jp.offset(1)] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) == 0 {
            QueueSfx2_Max6(jp.word(0), game);
        }
    };
    [11054630] NorfairLavaMan_Instr_8 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
    };
    [11054640] NorfairLavaMan_Instr_14 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
    };
    [11054650] NorfairLavaMan_Instr_2 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_01 = 1);
    };
    [11054661] NorfairLavaMan_Instr_7 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_01 = 0);
    };
    [11054672] NorfairLavaMan_Instr_10 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos, |old| old.wrapping_sub(1));
    };
    [11054682] NorfairLavaMan_Instr_12 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos, |old| old.wrapping_add(1));
    };
    [11054692] NorfairLavaMan_Instr_9 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v4: u16 = (slot!(game, NorfairLavaMan[E].var_03)).wrapping_add(24);
        slot!(game, NorfairLavaMan[E].base.y_pos = v4);
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos = v4);
        slot!(game, NorfairLavaMan[E1].base.properties, |old| old & !(0x100));
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        slot!(game, NorfairLavaMan[E2].base.properties, |old| old & !(0x100));
    };
    [11054728] NorfairLavaMan_Instr_11 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let nlmn_var_03: u16 = slot!(game, NorfairLavaMan[E].var_03);
        slot!(game, NorfairLavaMan[E].base.y_pos = nlmn_var_03);
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32 + 64].base.y_pos = nlmn_var_03)

;
    };
    [11054742] NorfairLavaMan_Instr_13 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v4: u16 = (slot!(game, NorfairLavaMan[E].var_03)).wrapping_add(4);
        slot!(game, NorfairLavaMan[E].base.y_pos = v4);
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos = v4);
        slot!(game, NorfairLavaMan[E1].base.properties, |old| (old) | (0x100u16));
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        slot!(game, NorfairLavaMan[E2].base.properties, |old| (old) | (0x100u16));
    };
    [11054778] NorfairLavaMan_Instr_5 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        SpawnEprojWithGfx(
            slot!(game, NorfairLavaMan[E].var_B),
            word!(game.cur_enemy_index),
            addr_kEproj_LavaThrownByLavaman as u16,
            game,
        );
    };
    [11054794] NorfairLavaMan_Instr_15 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_add(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054820] NorfairLavaMan_Instr_4 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_sub(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054846] NorfairLavaMan_Instr_16 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_add(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(8));
    };
    [11054872] NorfairLavaMan_Instr_6 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_sub(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054916] NorfairLavaMan_Instr_3 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_04 = 256);
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11068957] WreckedShipSpark_Instr_2 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, WreckedShipSpark[E].base.properties, |old| (old) | ((kEnemyProps_Tangible) as u16));
    };
    [11068970] WreckedShipSpark_Instr_1 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, WreckedShipSpark[E].base.properties, |old| (old as i32 & !(kEnemyProps_Tangible as i32)) as u16);
    };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11047153] MorphBallEye_Func_1 [] {
        if ({ let (actor, distance) = (k, g_word_A89054); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (k, g_word_A89050); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            let E: i32 = k as i32;
            slot!(game, MorphBallEye[E].var_E = 32);
            slot!(game, MorphBallEye[E].base.instruction_timer = 1);
            if (slot!(game, MorphBallEye[E].parameter_1) & 1) != 0 {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_kMorphBallEye_Ilist_9026 as u16);
            } else {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_kMorphBallEye_Ilist_9014 as u16);
            }
            slot!(game, MorphBallEye[E].var_F = (0xa8912ei32) as u16);
        }
    };
    [11047264] MorphBallEye_Func_3 [] {
        let E: i32 = k as i32;
        if ({ let (actor, distance) = (k, g_word_A89056); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (k, g_word_A89052); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            let v3: u16 = actor_api::angle_from_delta((word!(game.samus_x_pos)).wrapping_sub(slot!(game, MorphBallEye[E].base.x_pos)), (word!(game.samus_y_pos)).wrapping_sub(slot!(game, MorphBallEye[E].base.y_pos)));
            slot!(game, MorphBallEye[E].var_D = v3);
            slot!(game, MorphBallEye[E].base.current_instruction = (((v3 as i32 & 0xf0) >> 2) - 28756i32) as u16);
        } else {
            QueueSfx2_Max6(0x71, game);
            slot!(game, MorphBallEye[E].var_C = 0);
            slot!(game, MorphBallEye[E].var_E = 32);
            if (slot!(game, MorphBallEye[E].parameter_1) & 1) != 0 {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_kMorphBallEye_Ilist_9002 as u16);
            } else {
                slot!(game, MorphBallEye[E].base.current_instruction = addr_kMorphBallEye_Ilist_8FF0 as u16);
            }
            slot!(game, MorphBallEye[E].var_F = (0xa891cei32) as u16);
        }
        slot!(game, MorphBallEye[E].base.instruction_timer = 1);
    };
    [11055885] NorfairLavaMan_Func_15 [] {
        if *(ram_ptr!(game, u16; 0xe84))
            .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 87) as isize) as i32
            == 0xa8b175 & 0xffff
        {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_F = (0xa8b356i32) as u16)
;
        }
    };
    [11055958] NorfairLavaMan_Func_17 [] {
        if (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 != 0 {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ACB0 as u16);
            slot!(game, NorfairLavaMan[E].var_B = 0);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD50 as u16);
                slot!(game, NorfairLavaMan[E].var_B = 1);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b31fi32) as u16);
            slot!(game, NorfairLavaMan[E].var_12 = slot!(game, NorfairLavaMan[E].base.x_pos));
            slot!(game, NorfairLavaMan[E].var_13 = slot!(game, NorfairLavaMan[E].base.y_pos));
        } else {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].base.y_pos = (*(ram_ptr!(game, u16; 0xe84))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1i32) + 93i32) as isize))
            .wrapping_sub(crate::rtl::rom_table_word!(
                game,
                0xa8af79,
                (*(ram_ptr!(game, u16; 0xf68))
                    .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 1) as isize)
                    >> 1) as isize,
            )))

;
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    [11056039] NorfairLavaMan_Func_18 [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 87)]) as i32 == 0xa8b11a & 0xffff {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_F = (0xa8b30di32) as u16)
;
        } else {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].base.y_pos = (word!(game.enemy_drawing_queue[(v1 + 93)])).wrapping_sub(
                crate::rtl::rom_table_word!(
                    game,
                    0xa8af79,
                    (word!(game.enemy_drawing_queue_sizes[(v1 + 1)]) >> 1) as isize,
                ),
            ))

;
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    [11055386] NorfairLavaMan_Func_7 [] {
        slot!(game, NorfairLavaMan[E].var_00 = 0);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
            slot!(game, NorfairLavaMan[E].var_00 = 1);
        }
        NorfairLavaMan_Func_20(game);
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        if (slot!(game, NorfairLavaMan[E2].var_04) & 0x8000) != 0 {
            slot!(game, NorfairLavaMan[E2].var_04 = 0);
            if ({ let (actor, distance) = (word!(game.cur_enemy_index), slot!(byte game, NorfairLavaMan[E].parameter_2[0]) as u16); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ACDE as u16);
                if slot!(game, NorfairLavaMan[E].var_00) != 0 {
                    slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD7E as u16);
                }
                NorfairLavaMan_Func_20(game);
                slot!(game, NorfairLavaMan[E].var_F = (0xa8b175i32) as u16);
            }
        }
    };
    [11055477] NorfairLavaMan_Func_8 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_02 = 1);
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32 + 64i32].var_0D = 0);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b193i32) as u16);
        }
    };
    [11055507] sub_A8B193 [] {
        if slot!(game, NorfairLavaMan[E].var_02) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD0C as u16);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ADAC as u16);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b1b8i32) as u16);
        }
    };
    [11055544] NorfairLavaMan_Func_9 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b11ai32) as u16);
        }
    };
    [11055581] NorfairLavaMan_Func_10 [] {
        if slot!(game, NorfairLavaMan[E].var_0D) == 0 {
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b204i32) as u16);
            slot!(game, NorfairLavaMan[E]; var_E = 0, var_0D = 0);
            slot!(game, NorfairLavaMan[E].var_B = 2);
            slot!(game, NorfairLavaMan[E].base.y_pos = slot!(game, NorfairLavaMan[E].var_03));
        }
    };
    [11055620] NorfairLavaMan_Func_11 [] {
        let v3 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v: u32 = (((*v3.offset(31)).ypos2) as u32).wrapping_add((slot!(game, NorfairLavaMan[E].base.y_subpos)) as u32);
        slot!(game, NorfairLavaMan[E].base.y_subpos = v as u16);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16);
        slot!(game, NorfairLavaMan[E].var_E, |old| (old as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16);
        NorfairLavaMan_Func_12(word!(game.cur_enemy_index), game);
    };
    [11055903] NorfairLavaMan_Func_16 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16);
            NorfairLavaMan_Func_20(game);
            *(ram_ptr!(game, u16; 0xf68))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 5) as isize) =
                (0xa8b295i32) as u16;
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b3a7i32) as u16);
            (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 = 0;
            slot!(game, NorfairLavaMan[E].base.x_pos = slot!(game, NorfairLavaMan[E].var_12));
            slot!(game, NorfairLavaMan[E].base.y_pos = slot!(game, NorfairLavaMan[E].var_13));
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
        let v2: bool = slot!(game, MorphBallEye[E].var_E) == 1;
        slot!(game, MorphBallEye[E].var_E, |old| old.wrapping_sub(1));
        let v3: bool = (slot!(game, MorphBallEye[E].var_E) & 0x8000) != 0;
    }
    when v2 || v3;
    [11047214] MorphBallEye_Func_2 {
        QueueSfx2_Max6(0x17, game);
        initialize_morph_eye_beam(game);
        slot!(game, MorphBallEye[E].var_F = (0xa89160i32) as u16);
        slot!(game, MorphBallEye[E].var_D = actor_api::angle_from_delta((word!(game.samus_x_pos)).wrapping_sub(slot!(game, MorphBallEye[E].base.x_pos)), (word!(game.samus_y_pos)).wrapping_sub(slot!(game, MorphBallEye[E].base.y_pos))));
    };
    [11047374] MorphBallEye_Func_4 {
        slot!(game, MorphBallEye[E].var_F = (0xa890f1i32) as u16);
    };
}
