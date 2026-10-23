use crate::game::enemy_combat::{
    apply_contact, apply_shot, contact_without_death, flash_on_shot, power_bomb_without_death,
    reduce_suit_damage, select_frozen_ai, shot_reaction,
};
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_scheduling::update_frozen_enemy;
use crate::game::enemy_script::{EnemyScript, bank_base};
use crate::game::minimap;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::rtl::rom_record;
use core::mem::offset_of;

pub use crate::game::constants::bank_86::*;

use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_88::{SpawnBG3ScrollHdmaObject, sub_88DF34};
use crate::game::sm_90::{
    CallSomeSamusCode, Samus_ReleaseFromDraygon_, Samus_SetGrabbedByDraygonPose,
};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, ConvertAngleToXy, CosineMult8bit, EnemyFunc_B7A1, EnemyFunc_B691,
    EnemyFunc_Divide, SignExtend8, SineMult8bit,
};
use crate::game::sprite_effect_runtime::{self as sprite_effects, create_sprite};

pub use crate::types::Consts_A5;

crate::game::constants::typed_constants! { Consts_A5;
    addr_kDraygon_Ilist_E77D = 59261, addr_stru_A5E729 = 59177,
    addr_kDraygon_Ilist_E6D5 = 59093, addr_kDraygon_Ilist_E6C7 = 59079,
    addr_kDraygon_Ilist_E6B9 = 59065, addr_kDraygon_BigSprmap_C11B = 49435,
    addr_kDraygon_BigSprmap_C08F = 49295, addr_word_A5A297 = 41623,
    addr_kDraygon_MorePalettes2 = 41591, addr_kDraygon_Palette = 41463,
    addr_kDraygon_Ilist_9F15 = 40725, addr_kDraygon_Ilist_9EA1 = 40609,
    addr_kDraygon_Ilist_9E21 = 40481, addr_kDraygon_Ilist_9D9E = 40350,
    addr_kDraygon_Ilist_9D62 = 40290, addr_kDraygon_Ilist_9D5C = 40284,
    addr_kDraygon_Ilist_9D56 = 40278, addr_kDraygon_Ilist_9D50 = 40272,
    addr_kDraygon_Ilist_9D3E = 40254, addr_kDraygon_Ilist_9D1C = 40220,
    addr_kDraygon_Ilist_9CB4 = 40116, addr_kDraygon_Ilist_9C90 = 40080,
    addr_kDraygon_Ilist_9C5A = 40026, addr_kDraygon_Ilist_9C38 = 39992,
    addr_kDraygon_Ilist_9C18 = 39960, addr_kDraygon_Ilist_9C06 = 39942,
    addr_kDraygon_Ilist_9BDA = 39898, addr_kDraygon_Ilist_9B5A = 39770,
    addr_kDraygon_Ilist_9AE8 = 39656, addr_kDraygon_Ilist_9A68 = 39528,
    addr_kDraygon_Ilist_99FC = 39420, addr_kDraygon_Ilist_99C0 = 39360,
    addr_kDraygon_Ilist_99BA = 39354, addr_kDraygon_Ilist_99B4 = 39348,
    addr_kDraygon_Ilist_99AE = 39342, addr_kDraygon_Ilist_999C = 39324,
    addr_kDraygon_Ilist_997A = 39290, addr_kDraygon_Ilist_9944 = 39236,
    addr_kDraygon_Ilist_9922 = 39202, addr_kDraygon_Ilist_98FE = 39166,
    addr_kDraygon_Ilist_98ED = 39149, addr_kDraygon_Ilist_9889 = 39049,
    addr_kDraygon_Ilist_9867 = 39015, addr_kDraygon_Ilist_9845 = 38981,
    addr_kDraygon_Ilist_9825 = 38949, addr_kDraygon_Ilist_9813 = 38931,
    addr_kDraygon_Ilist_97E7 = 38887, addr_kDraygon_Ilist_97D1 = 38865,
    addr_kDraygon_Ilist_97BB = 38843, addr_kDraygon_Ilist_97B9 = 38841,
}

crate::type_aliases! {
    pub Enemy_Draygon, Enemy_SporeSpawn = crate::game::enemy_layouts::EnemyLayout104;
}

const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
const RomPtr_A5: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa5>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_ScrollFinishedHook_SporeSpawnFight = 0x909589, fnGrappleBeam_Func2 = 0x9bc8c5,
    fnDraygon_Func_36 = 0xa59342,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Draygon(o: i32) -> Enemy_Draygon = [0xf78, o as isize];
    Get_SporeSpawn(o: i32) -> Enemy_SporeSpawn = [0xf78, o as isize];
}
pub unsafe fn Draygon_Func_3(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 0x3f) == 0 {
            let mut Random: i8 = NextRandom(game) as i8;
            let result: u16 = (2 * ((Random as i32 & 3) + 2)) as u16;
            if (*Get_Draygon(result as i32, game)).var_40 == 0 {
                let v2: i32 = (4 * ((Random as i32 & 3) + 2)) as u16 as i32 >> 1;
                game.eproj_spawn_pt = Point16U {
                    x: crate::rtl::rom_table_word!(game, 0xa587dc, v2 as isize),
                    y: crate::rtl::rom_table_word!(game, 0xa587dc, (v2 + 1) as isize),
                };
                SpawnEprojWithGfx(3, result, addr_stru_868E5E as u16, game);
            }
        }
    }
}
macro_rules! advance_draygon_scripted_y {
    ($game:ident, $enemy:ident, $instruction:ident) => {
        Draygon_Func_3($game);
        Draygon_Func_35($game);
        let $enemy = Get_Draygon(0, $game);
        let draygon_var_B: u16 = (*$enemy).var_B;
        if draygon_var_B == 104 {
            let E3 = Get_Draygon(0xc0, $game);
            enemy_instruction! {[(*E3).base]; instruction = $instruction as u16, timer = 1}
        }
        (*Get_Draygon(word!($game.cur_enemy_index) as i32, $game))
            .base
            .y_pos = *(ram_ptr!($game, u16; 0x9002, draygon_var_B as i32 as isize));
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallDraygonFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10848027] Draygon_Func_1 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        if (*E).var_B as i32 - 256i32 & 0x8000 != 0 {
            if (*E).var_B == 0 {
                let mut v2: *mut VramWriteEntry =
                    gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
                (*v2).size = 1536;
                (*v2).src.addr = -27648i32 as VoidP;
                *(&raw mut (*v2).src.bank as *mut u16) = 177;
                (*v2).vram_dst = 27904;
                word!(game.vram_write_queue_tail += 7);
                Draygon_Func_45(game);
            }
            (*E).var_B = (*E).var_B.wrapping_add(1);
        } else {
            (*E).var_A = ((0xa5878bi32) as u16);
            (*E).var_B = 0;
            let x_pos: u16 = (*E).base.x_pos;
            (*E).var_00 = x_pos;
            (*E).var_02 = (x_pos).wrapping_add(672);
            (*E).var_01 = (*E).base.y_pos;
            (*E).base.x_pos = (*E).var_00;
            (*E).base.y_pos = (*E).var_01;
            (*E).var_0F = 24;
        };
    };
    [10848139] Draygon_Func_2 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        if (*E).var_B as i32 - 1232i32 & 0x8000 != 0 {
            Draygon_Func_46(game);
            (*E).var_B = (*E).var_B.wrapping_add(1);
        } else {
            (*E).var_A = ((0xa587f4i32) as u16);
            (*E).var_B = 0;
        };
    };
    [10848244] Draygon_Func_4 [] {
        Draygon_Func_3(game);
        Draygon_Func_5(game);
        let E = Get_Draygon(0, game);
        (*E).var_A = ((0xa588b1i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_97D1 as u16, timer = 1}
        (*E).var_20 = 1;
    };
    [10848433] Draygon_Func_6 [] {
        advance_draygon_scripted_y!(game, E, addr_kDraygon_Ilist_9C06);
        (*E).var_B = ((*E).var_B).wrapping_sub(4);
        if (*E).var_B != 0 {
            words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_E));
        } else {
            (*E).var_A = ((0xa58922i32) as u16);
        };
    };
    [10848546] Draygon_Func_8 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        let t: u32 = EnemyFunc_Divide(
            ((*E).var_C >> 2) as u32,
            ((672 - (*E).base.x_pos as i32) << 16) as u32,
        );
        words!((*E), var_D, var_E = t);
        (*E).var_A = ((0xa58951i32) as u16);
    };
    [10848593] Draygon_Func_9 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        let draygon_var_B: u16 = (*E).var_B;
        if draygon_var_B == 104 {
            let E3 = Get_Draygon(0xc0, game);
            enemy_instruction! {[(*E3).base]; instruction = addr_kDraygon_Ilist_9BDA as u16, timer = 1}
        }
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .y_pos = *(ram_ptr!(game, u16; 0x9002, draygon_var_B as i32 as isize));
        (*E).var_B = ((*E).var_B).wrapping_add(4);
        if (*E).var_B == (*E).var_C {
            (*E).var_B = 0;
            if (word!(game.random_number) & 1) != 0 {
                (*E).var_A = ((0xa589b3i32) as u16);
            } else {
                (*E).var_A = ((0xa58c8ei32) as u16);
            }
        } else {
            words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_E));
        };
    };
    [10848691] Draygon_Func_10 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        printf(b"Wtf is varE26?\n\0" as *const u8 as *const c_char);
        let varE26: u16 = 0;
        let t: u32 = EnemyFunc_Divide(
            (varE26 >> 2) as u32,
            ((Abs16(((*E).var_02).wrapping_sub(word!(game.samus_x_pos))) as i32) << 16) as u32,
        );
        words!((*E), var_D, var_E = t);
        (*E).var_A = ((0xa58a00i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_97BB as u16, timer = 1}
        (*E).var_20 = 0;
        (*E).var_B = (*E).var_C;
    };
    [10848768] Draygon_Func_11 [] {
        advance_draygon_scripted_y!(game, E, addr_kDraygon_Ilist_9813);
        (*E).var_B = ((*E).var_B).wrapping_sub(4);
        if (*E).var_B != 0 {
            words!(
                (*E).base,
                x_pos,
                x_subpos += -(words!(i32, (*E), var_D, var_E)) as u32
            );
        } else {
            (*E).var_A = ((0xa58a50i32) as u16);
        };
    };
    [10848848] Draygon_Func_12 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        let t: u32 = EnemyFunc_Divide(
            ((*E).var_C >> 2) as u32,
            (((*E).base.x_pos as i32 - (*E).var_00 as i32) << 16) as u32,
        );
        (*E).var_D = (t >> 16) as u16;
        (*E).var_E = t as u16;
        (*E).var_A = ((0xa58a90i32) as u16);
    };
    [10848912] Draygon_Func_13 [] {
        advance_draygon_scripted_y!(game, E, addr_kDraygon_Ilist_97E7);
        (*E).var_B = ((*E).var_B).wrapping_add(4);
        if (*E).var_B == (*E).var_C {
            if (word!(game.random_number) & 1) != 0 {
                (*E).var_A = ((0xa587f4i32) as u16);
                (*E).var_B = 0;
                (*E).base.x_pos = (*E).var_00;
                (*E).base.y_pos = (*E).var_01;
            } else {
                (*E).var_A = ((0xa58b0ai32) as u16);
                (*E).base.x_pos = (*E).var_00;
            }
        } else {
            words!(
                (*E).base,
                x_pos,
                x_subpos += -(words!(i32, (*E), var_D, var_E)) as u32
            );
        };
    };
    [10849034] Draygon_Func_14 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        (*E).base.x_pos = -80i32 as u16;
        (*E).base.y_pos = 384;
        (*E).var_E = 0;
        (*E).var_D = 1;
        (*E).var_0D = 0;
        let E3 = Get_Draygon(0xc0, game);
        enemy_instruction! {[(*E3).base]; instruction = addr_kDraygon_Ilist_9C06 as u16, timer = 1}
        (*E).var_A = ((0xa58b52i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_97D1 as u16, timer = 1}
        (*E).var_20 = 1;
    };
    [10849106] Draygon_Func_15 [] {
        Draygon_Func_3(game);
        Draygon_Func_35(game);
        let E = Get_Draygon(0, game);
        let v2: u16 = Abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
        if v2 as i32 - 208i32 & 0x8000 != 0 {
            (*E).var_A = ((0xa58baei32) as u16);
            (*E).var_03 = 16;
        } else {
            (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
            (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
            words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_E));
        };
    };
    [10849492] Draygon_Func_19 [] {
        Draygon_Func_3(game);
        Draygon_Func_35(game);
        let E = Get_Draygon(0, game);
        let v2: u16 = Abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
        if v2 as i32 - 208i32 & 0x8000 != 0 {
            (*E).var_A = ((0xa58d30i32) as u16);
            (*E).var_03 = 16;
        } else {
            (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
            (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
            words!(
                (*E).base,
                x_pos,
                x_subpos += -(words!(i32, (*E), var_D, var_E)) as u32
            );
        };
    };
    [10850064] Draygon_Func_23 [] {
        word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
        (*Get_Draygon(0i32, game)).var_A = ((0xa59128i32) as u16);
    };
    [10850600] Draygon_Func_29 [] {
        Samus_ReleaseFromDraygon_(game);
        word!(game.samus_grapple_flags = 0);
        let E = Get_Draygon(0, game);
        (*E).var_A = ((0xa59154i32) as u16);
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
        let mut v1: u16 = addr_kDraygon_Ilist_9B5A as u16;
        if (*E).var_20 != 0 {
            v1 = addr_kDraygon_Ilist_9F15 as u16;
        }
        let E2 = Get_Draygon(0x80, game);
        enemy_instruction! {[(*E2).base]; instruction = v1, timer = 1}
    };
    [10850644] Draygon_Func_30 [] {
        Draygon_Func_3(game);
        let E = Get_Draygon(0, game);
        let mut v1: i16 = (((*E).base.y_pos) as i16).wrapping_sub(4);
        (*E).base.y_pos = v1 as u16;
        if v1 < 0 {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            (*E).var_A = ((0xa587f4i32) as u16);
            (*E).var_B = 0;
            (*E).base.x_pos = (*E).var_00;
            (*E).base.y_pos = (*E).var_01;
        }
    };
    [10850964] Draygon_Func_32 [] {
        Draygon_Func_34(game);
        let E = Get_Draygon(0, game);
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if (*E).var_B != 0 {
            Draygon_Func_42(0, game);
        } else {
            (*E).var_A = ((0xa592abi32) as u16);
        };
    };
    [10850987] Draygon_Func_33 [] {
        Draygon_Func_34(game);
        Draygon_Func_42(0, game);
        let E = Get_Draygon(0, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        if (*E).base.y_pos as i32 - 576i32 & 0x8000 == 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_98ED as u16, timer = 1}
            let v1: u16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            (*E).base.properties = v1;
            (*Get_Draygon(0x40i32, game)).base.properties = v1;
            let v2: u16 = word!(game.area_index);
            crate::update_place! { unaligned_word fresh3 = &raw mut (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize))) => (u16::from_le(fresh3.read_unaligned())) | (1u16); }
            scatter(DropSource::Draygon, v2, 0, game);
            Draygon_Func_44(game);
        }
    };
    [10873627] SporeSpawn_Func_1 [] {
        crate::game::enemy_projectiles::update_spore_stem(game);
        let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
        let v1: u16 = ((*E).base.y_pos).wrapping_add(1);
        (*E).base.y_pos = v1;
        if v1 as i32 - 624i32 & 0x8000 == 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_E6D5 as u16, timer = 1}
        }
        let E0 = Get_SporeSpawn(0, game);
        (*E0).var_0B = 48;
        (*E0).var_0C = 1;
        (*E0).var_0A = 192;
    };
    [10873755] SporeSpawn_Func_3 [] {
        let E0 = Get_SporeSpawn(0, game);
        (*E0).var_43 = ((64u8).wrapping_sub(
            (CalculateAngleFromXY(
                (128u16).wrapping_sub((*E0).base.x_pos),
                (624u16).wrapping_sub((*E0).base.y_pos),
                game,
            )) as u8,
        )) as u16;
        let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
        let pt: Point32 = ConvertAngleToXy(byte!((*E0).var_43; 0) as u16, 1, game);
        (*E).var_28 = (pt.x >> 16) as u16;
        (*E).var_29 = pt.x as u16;
        (*E).var_2A = (pt.y >> 16) as u16;
        (*E).var_2B = pt.y as u16;
    };
    [10873838] SporeSpawn_Func_4 [] {
        let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
        let E0 = Get_SporeSpawn(0, game);
        let varE20: u16 = byte!((*E0).var_43; 0) as u16;
        EnemyFunc_B691(
            varE20,
            Point32 {
                x: words!((*E), var_28, var_29),
                y: words!((*E), var_2A, var_2B),
            },
            game,
        );
        let v2: u16 = Abs16(((*E0).base.x_pos).wrapping_sub(128));
        if v2 as i32 - 8i32 & 0x8000 != 0 {
            let v3: u16 = Abs16(((*E0).base.y_pos).wrapping_sub(624));
            if v3 as i32 - 8i32 & 0x8000 != 0 {
                (*E0).var_A = ((0xa5eb1ai32) as u16);
            }
        }
        crate::game::enemy_projectiles::update_spore_stem(game);
        sub_A5E9F5(game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Draygon(0, game); };
    Draygon_Func_5 [] {
        let mut varE24: u16 = 384;
        let mut varE22: u16 = 0;
        let mut varE26: u16 = 0;
        loop {
            varE22 = (varE22).wrapping_add((*E).var_0F);
            let v1: u16 = (varE24).wrapping_sub((byte!(varE22; 1)) as u16);
            if ((v1) as i16).wrapping_sub(((*E).var_01) as i16) < 0 {
                break;
            }
            varE24 = (varE24).wrapping_sub((byte!(varE22; 1)) as u16);
            crate::update_place! { previous fresh4 = varE26 => varE26.wrapping_add(1); }
            *(ram_ptr!(game, u16; 0x9002, fresh4 as i32 as isize)) = v1;
            varE26 = (varE26).wrapping_add(2);
            varE26 = varE26.wrapping_add(1);
            if varE26 as i32 - 2048i32 & 0x8000 == 0 {
                Unreachable(game);
            }
        }
        let t: u32 = EnemyFunc_Divide(
            (varE26 >> 2) as u32,
            ((Abs16(((*E).var_00).wrapping_sub(word!(game.samus_x_pos))) as i32) << 16) as u32,
        );
        words!((*E), var_D, var_E = t);
        let v6: u16 = varE26;
        *(ram_ptr!(game, u16; 0x9002, varE26 as i32 as isize)) =
            (*E).base.y_pos;
        (*E).var_B = v6;
        (*E).var_C = v6;
    };
    [10849198] Draygon_Func_16 [] {
        Draygon_Func_3(game);
        if word!(game.samus_x_speed_divisor) == 0 {
            'advance: {
                if (word!(game.random_number) & 0xf) == 0 {
                    let v1: u16 = ((*E).var_03).wrapping_sub(1);
                    (*E).var_03 = v1;
                    if v1 == 0 {
                        break 'advance;
                    }
                    enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_9C90 as u16, timer = 1}
                }
                (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
                (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
                words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_E));
                if !(((*E).base.x_pos as i16) >= 0 && (*E).base.x_pos as i32 - 672i32 & 0x8000 == 0) {
                    return;
                }
            }
            if word!(game.samus_x_speed_divisor) == 0 {
                let E3 = Get_Draygon(0xc0, game);
                enemy_instruction! {[(*E3).base]; instruction = addr_kDraygon_Ilist_9BDA as u16, timer = 1}
                (*E).var_A = ((0xa58c33i32) as u16);
                return;
            }
        }
        (*E).var_A = ((0xa58e19i32) as u16);
    };
    [10849422] Draygon_Func_18 [] {
        (*E).base.x_pos = (*E).var_02;
        (*E).base.y_pos = 384;
        (*E).var_E = 0;
        (*E).var_D = 1;
        (*E).var_0D = 0;
        let E3 = Get_Draygon(0xc0, game);
        enemy_instruction! {[(*E3).base]; instruction = addr_kDraygon_Ilist_9C06 as u16, timer = 1}
        (*E).var_A = ((0xa58cd4i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_97BB as u16, timer = 1}
        (*E).var_20 = 0;
    };
    [10849584] Draygon_Func_20 [] {
        if word!(game.samus_x_speed_divisor) == 0 {
            'advance: {
                if (word!(game.random_number) & 0xf) == 0 {
                    let v1: u16 = ((*E).var_03).wrapping_sub(1);
                    (*E).var_03 = v1;
                    if v1 == 0 {
                        break 'advance;
                    }
                    enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_98FE as u16, timer = 1}
                }
                (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
                (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
                words!(
                    (*E).base,
                    x_pos,
                    x_subpos += -(words!(i32, (*E), var_D, var_E)) as u32
                );
                if !((*E).base.x_pos < 0 && (*E).base.x_pos as i32 + 80 & 0x8000 != 0) {
                    return;
                }
            }
            if word!(game.samus_x_speed_divisor) == 0 {
                let E3 = Get_Draygon(0xc0, game);
                enemy_instruction! {[(*E3).base]; instruction = addr_kDraygon_Ilist_97E7 as u16, timer = 1}
                (*Get_Draygon(0i32, game)).var_A = ((0xa58db2i32) as u16);
                return;
            }
        }
        (*Get_Draygon(0i32, game)).var_A = ((0xa58e19i32) as u16);
    };
    [10849817] Draygon_Func_22 [] {
        Draygon_Func_3(game);
        if word!(game.samus_x_speed_divisor) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            let mut v2: i16 = 8;
            if (*E).var_20 == 0 {
                v2 = -8i16;
            }
            let v3: u16 =
                Abs16(((*E).base.x_pos as i32 + v2 as i32 - word!(game.samus_x_pos) as i32) as u16);
            if v3 as i32 - 8i32 & 0x8000 != 0 && {
                let mut v4: u16 = Abs16(((*E).base.y_pos).wrapping_sub(word!(game.samus_y_pos)));
                v4 as i32 - 8i32 & 0x8000 != 0
            } {
                let mut v5: u16 = addr_kDraygon_Ilist_9845 as u16;
                if (*E).var_20 != 0 {
                    v5 = addr_kDraygon_Ilist_9C38 as u16;
                }
                let E3 = Get_Draygon(0xc0, game);
                enemy_instruction! {[(*E3).base]; instruction = v5, timer = 1}
                if CallSomeSamusCode(0xd, game) != 0 {
                    (*E).var_A = ((0xa58f10i32) as u16);
                } else {
                    Samus_SetGrabbedByDraygonPose(((*E).var_20 != 0) as u16, game);
                    (*E).var_06 = 256;
                    (*E).var_07 = 384;
                    set_fields! {[(*E)], 0; var_05, var_09}
                    (*E).var_08 = 192;
                    (*E).var_0B = 2048;
                    (*E).var_A = ((0xa58f1ei32) as u16);
                }
            } else {
                let varE20: u16 = ((64u8).wrapping_sub(
                    (CalculateAngleFromXY(
                        (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos),
                        (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos),
                        game,
                    )) as u8,
                )) as u16;
                let pt: Point32 = ConvertAngleToXy(varE20, 2, game);
                EnemyFunc_B691(varE20, pt, game);
            }
        } else {
            (*E).var_A = ((0xa59154i32) as u16);
        };
    };
    [10850693] Draygon_Func_31 [] {
        if ((*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .frame_counter
            & 0xf)
            == 0
        {
            let x: u16 = (if (*E).var_20 != 0 {
                (*E).base.x_pos as i32 + 32
            } else {
                (*E).base.x_pos as i32 - 32i32
            }) as u16;
            let y: u16 = ((*E).base.y_pos).wrapping_sub(16);
            create_sprite(x, y, 61, 0, game);
        }
        (*E).var_43 = ((64u8).wrapping_sub(
            (CalculateAngleFromXY(
                (64 - ((*E).base.x_pos as i32 >> 2)) as u16,
                (120 - ((*E).base.y_pos as i32 >> 2)) as u16,
                game,
            )) as u8,
        )) as u16;
        let pt: Point32 = ConvertAngleToXy(byte!((*E).var_43; 0) as u16, 1, game);
        EnemyFunc_B691(byte!((*E).var_43; 0) as u16, pt, game);
        let v3: u16 = Abs16(((*E).base.x_pos).wrapping_sub(256));
        if v3 as i32 - 4i32 & 0x8000 != 0 {
            let v4: u16 = Abs16(((*E).base.y_pos).wrapping_sub(480));
            if v4 as i32 - 4i32 & 0x8000 != 0 {
                Draygon_Func_43(game);
                (*E).var_A = ((0xa59294i32) as u16);
                QueueMusic_Delayed8(3, game);
                (*E).var_B = 416;
                enemy_instruction! {[(*E).base]; instruction = addr_kDraygon_Ilist_97B9 as u16, timer = 1}
                let v5: u16 = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
                (*Get_Draygon(0x80i32, game)).base.properties = v5;
                (*Get_Draygon(0xc0i32, game)).base.properties = v5;
                let mut v6: u16 = addr_kDraygon_Ilist_999C as u16;
                if (*E).var_20 != 0 {
                    v6 = addr_kDraygon_Ilist_9D3E as u16;
                }
                let E1 = Get_Draygon(0x40, game);
                enemy_instruction! {[(*E1).base]; instruction = v6, timer = 1}
                (*E1).var_A = ((0xa5804bi32) as u16);
            }
        }
    };
    Draygon_Func_36 [] {
        word! { game;
            reg_BG2HOFS = ((*E).var_5E as i32 + word!(game.layer1_x_pos) as i32
                - (*E).base.x_pos as i32
                - 450i32) as u16;
            reg_BG2VOFS = ((*E).var_5F as i32 + word!(game.layer1_y_pos) as i32
                - (*E).base.y_pos as i32
                - 192i32) as u16;
        }
    };
    Draygon_Shot [] {
        if (*E).var_0F as i32 - 152i32 & 0x8000 != 0 {
            (*E).var_0F = ((*E).var_0F).wrapping_add(8);
        }
        shot_reaction(game);
        sub_A5960D(game);
    };
    Draygon_Func_41 [] {
        let mut i: i32 = 0;
        while ((((*E).base.health) as i16)
            .wrapping_sub((crate::rtl::rom_table_word!(game, 0xa596ef, (i >> 1) as isize)) as i16)
            < 0)
        {
            i += 2;
        }
        if i != (*E).var_0E as i32 {
            (*E).var_0E = i as u16;
            let mut v2: u16 = (4u16).wrapping_mul((*E).var_0E);
            for j in (0u16..8).step_by(2) {
                *(ram_ptr!(game, u16; 0xc000)).offset(((j as i32 >> 1) + 89) as isize) =
                    crate::rtl::rom_table_word!(game, 0xa596af, (v2 >> 1) as isize);
                v2 = (v2).wrapping_add(2);
            }
        }
    };
    @external {
        10846283 => [()];
        10849331 => [Draygon_Func_17(k, game)];
        10849714 => [Draygon_Func_21(k, game)];
        10850078 => [Draygon_Func_24(k, game)];
        10850262 => [Draygon_Func_25(k, game)]; 10850516 => [Draygon_Func_26(k, game)];
        10850565 => [Draygon_Func_27(k, game)]; 10850596 => [Draygon_Func_28(k, game)];
        10863757 => [Draygon_Func_48(k, game)]; 10863891 => [Draygon_Func_49(k, game)];
        10873626 => [()]; 10873682 => [SporeSpawn_Func_2(k, game)];
    }
}
pub unsafe fn Draygon_Func_17(k: u16, game: &mut Game) {
    unsafe {
        Draygon_Func_35(game);
        let E = Get_Draygon(0, game);
        (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
        (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
        words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_E));
        if ((*E).base.x_pos as i16) >= 0 && (*E).base.x_pos as i32 - 672i32 & 0x8000 == 0 {
            (*E).var_A = ((0xa589b3i32) as u16);
            (*E).base.x_pos = 592;
            (*E).base.y_pos = -80i32 as u16;
            (*E).var_B = (*E).var_C;
        }
    }
}
pub unsafe fn Draygon_Func_21(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Draygon(0, game);
        Draygon_Func_35(game);
        if word!(game.samus_x_speed_divisor) != 0 {
            (*E).var_A = ((0xa58e19i32) as u16);
        } else {
            (*E).base.y_pos = (CosineMult8bit((*E).var_0D, 32u16, game)).wrapping_add(384);
            (*E).var_0D = ((((*E).var_0D) as u8).wrapping_add(1)) as u16;
            words!(
                (*E).base,
                x_pos,
                x_subpos += -(words!(i32, (*E), var_D, var_E)) as u32
            );
            if ((*E).base.x_pos as i16) < 0 && (*E).base.x_pos as i32 + 80 & 0x8000 != 0 {
                (*E).var_A = ((0xa587f4i32) as u16);
                (*E).base.x_pos = -80i32 as u16;
                (*E).base.y_pos = -80i32 as u16;
                (*E).var_B = (*E).var_C;
            }
        };
    }
}
pub unsafe fn Draygon_Func_24(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Draygon(0, game);
        if (word!(game.samus_grapple_flags) & 1) != 0 {
            (*E).base.flash_timer = ((rom_record(0xa0, (*E).base.enemy_ptr, game)
                .byte(offset_of!(EnemyDef, hurt_ai_time)))
                as u16)
                .wrapping_add(8);
            (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
        } else {
            let v4: u16 = Abs16(((*E).base.x_pos).wrapping_sub(256));
            if v4 as i32 - 2i32 & 0x8000 != 0 && {
                let mut v5: u16 = Abs16(((*E).base.y_pos).wrapping_sub(384));
                v5 as i32 - 2i32 & 0x8000 != 0
            } {
                (*E).var_A = ((0xa58fd6i32) as u16);
                let mut v6: u16 = addr_kDraygon_Ilist_9922 as u16;
                if (*E).var_20 != 0 {
                    v6 = addr_kDraygon_Ilist_9CB4 as u16;
                }
                let EK = Get_Draygon(k as i32, game);
                enemy_instruction! {[(*EK).base]; instruction = v6, timer = 1}
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            } else {
                let varE20: u16 = ((64u8).wrapping_sub(
                    (CalculateAngleFromXY(
                        (256u16).wrapping_sub((*E).base.x_pos),
                        (384u16).wrapping_sub((*E).base.y_pos),
                        game,
                    )) as u8,
                )) as u16;
                let pt: Point32 = ConvertAngleToXy(varE20, 2, game);
                EnemyFunc_B691(varE20, pt, game);
                Draygon_Func_40(k, game);
            }
        };
    }
}
pub unsafe fn Draygon_Func_25(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Draygon(0, game);
        if (word!(game.samus_grapple_flags) & 1) != 0 {
            (*E).base.flash_timer = ((rom_record(0xa0, (*E).base.enemy_ptr, game)
                .byte(offset_of!(EnemyDef, hurt_ai_time)))
                as u16)
                .wrapping_add(8);
            (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
        } else if word!(game.random_number) as u8 != 0 {
            (*E).base.x_pos =
                ((*E).var_06).wrapping_add(CosineMult8bit((*E).var_08, (*E).var_05, game));
            (*E).base.y_pos =
                ((*E).var_07).wrapping_add(SineMult8bit((*E).var_08, ((*E).var_05 >> 2), game));
            if ((*Get_Draygon(k as i32, game)).base.frame_counter & 7) == 0 {
                let x: u16 = (if (*E).var_20 != 0 {
                    (*E).base.x_pos as i32 + 32
                } else {
                    (*E).base.x_pos as i32 - 32i32
                }) as u16;
                let y: u16 = ((*E).base.y_pos).wrapping_sub(16);
                create_sprite(x, y, 61, 0, game);
            }
            words!((*E), var_05, var_09 += 0x2000);
            if (*E).var_05 as i32 - 160i32 & 0x8000 == 0
                || {
                    let mut v6: u16 = ((*E).var_0B).wrapping_sub(1);
                    (*E).var_0B = v6;
                    (*E).var_08 = ((byte!((*E).var_08; 0)).wrapping_add(byte!(v6; 1))) as u16;
                    crate::update_place! { previous mut draygon_var_0A: u16 = (*E).var_0A => (draygon_var_0A).wrapping_sub(0x4000); }
                    let mut v8: u16 =
                        ((words!(((*E).var_07, draygon_var_0A))).wrapping_sub(0x4000) >> 16) as u16;
                    (*E).var_07 = v8;
                    v8 as i32 - 64i32 & 0x8000 != 0
                }
            {
                (*E).var_A = ((0xa59105i32) as u16);
            } else {
                Draygon_Func_40(k, game);
            }
        } else {
            (*E).var_0C = 64;
            (*E).var_A = ((0xa590d4i32) as u16);
        };
    }
}
pub unsafe fn Draygon_Func_26(k: u16, game: &mut Game) {
    unsafe {
        let v0: u16 = k;
        Draygon_Func_40(v0, game);
        let E = Get_Draygon(0, game);
        let v2: u16 = ((*E).var_0C).wrapping_sub(1);
        (*E).var_0C = v2;
        if v2 != 0 {
            if v2 == 63 {
                let mut v3: u16 = addr_kDraygon_Ilist_9AE8 as u16;
                if (*E).var_20 != 0 {
                    v3 = addr_kDraygon_Ilist_9EA1 as u16;
                }
                let E2 = Get_Draygon(0x80, game);
                enemy_instruction! {[(*E2).base]; instruction = v3, timer = 1}
            }
        } else {
            (*E).var_A = ((0xa58fd6i32) as u16);
        };
    }
}
pub unsafe fn Draygon_Func_27(k: u16, game: &mut Game) {
    unsafe {
        Draygon_Func_40(k, game);
        let mut v1: u16 = addr_kDraygon_Ilist_9A68 as u16;
        if (*Get_Draygon(k as i32, game)).var_20 != 0 {
            v1 = addr_kDraygon_Ilist_9E21 as u16;
        }
        let E = Get_Draygon(0x80, game);
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        (*Get_Draygon(0i32, game)).var_A = ((0xa59124i32) as u16);
    }
}
pub unsafe fn Draygon_Func_28(k: u16, game: &mut Game) {
    unsafe {
        Draygon_Func_40(k, game);
    }
}
pub unsafe fn Draygon_Func_34(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            let x: u16 = ((NextRandom(game) as i32 & 0x7f) + 192) as u16;
            let y: u16 = ((((word!(game.random_number)) & (0x3f00u16)) as i32 >> 8) + 400) as u16;
            create_sprite(x, y, 21, 0, game);
        }
    }
}
pub unsafe fn Draygon_Func_35(game: &mut Game) {
    unsafe {
        let mut Draygon = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        if ((*Draygon).base.frame_counter & 0x7f) == 0 {
            Draygon = Get_Draygon(0, game);
            create_sprite(
                ((*Draygon).base.x_pos).wrapping_sub(16),
                ((*Draygon).base.y_pos).wrapping_sub(16),
                24,
                0,
                game,
            );
        }
    }
}
pub unsafe fn Draygon_Func_40(k: u16, game: &mut Game) {
    unsafe {
        let mut v1: i16 = 8;
        if (*Get_Draygon(k as i32, game)).var_20 == 0 {
            v1 = -8i16;
        }
        let E = Get_Draygon(0, game);
        word!(game.samus_x_pos = ((*E).base.x_pos).wrapping_add((v1) as u16));
        word!(game.samus_y_pos = ((*E).base.y_pos).wrapping_add(40));
        EnemyFunc_B7A1(game);
        if (word!(game.samus_grapple_flags) & 2) != 0 {
            word!(game.samus_grapple_flags = 0);
            (*E).var_A = ((0xa59154i32) as u16);
        }
    }
}
macro_rules! copy_draygon_hurt_palette {
    ($game:ident, $palette:ident, $destination:ident, $row_offset:literal) => {
        if ((*Get_Draygon(word!($game.cur_enemy_index) as i32, $game))
            .base
            .flash_timer
            & 2)
            != 0
        {
            $palette = addr_word_A5A297 as u16;
        }
        memcpy(
            ram_ptr!($game, c_void; $destination as i32 as isize, $row_offset),
            RomPtr_A5($palette, $game) as *const c_void,
            32 as size_t,
        );
    };
}

pub unsafe fn sub_A5960D(game: &mut Game) {
    unsafe {
        if (*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .health
            != 0
        {
            Draygon_Func_41(game);
        } else {
            word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
            let mut v0: u16 = addr_kDraygon_Ilist_9867 as u16;
            let E = Get_Draygon(0, game);
            if (*E).var_20 != 0 {
                v0 = addr_kDraygon_Ilist_9C5A as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v0, timer = 1}
            let mut v3: u16 = addr_kDraygon_Ilist_997A as u16;
            if (*E).var_20 != 0 {
                v3 = addr_kDraygon_Ilist_9D1C as u16;
            }
            let E1 = Get_Draygon(0x40, game);
            enemy_instruction! {[(*E1).base]; instruction = v3, timer = 1}
            (*E1).var_A = ((0xa5804bi32) as u16);
            (*E).var_A = ((0xa59185i32) as u16);
            Samus_ReleaseFromDraygon_(game);
            word!(game.samus_grapple_flags = 0);
            (*E).var_43 = ((64u8).wrapping_sub(
                (CalculateAngleFromXY(
                    (256u16).wrapping_sub((*E).base.x_pos),
                    (480u16).wrapping_sub((*E).base.y_pos),
                    game,
                )) as u8,
            )) as u16;
            let pt: Point32 = ConvertAngleToXy(byte!((*E).var_43; 0) as u16, 1, game);
            (*E).var_28 = (pt.x >> 16) as u16;
            (*E).var_29 = pt.x as u16;
            (*E).var_2A = (pt.y >> 16) as u16;
            (*E).var_2B = pt.y as u16;
        };
    }
}
unsafe fn Draygon_Func_47(game: &mut Game) -> Point16U {
    unsafe {
        NextRandom(game);
        let E = Get_Draygon(0, game);
        let r18: u16 = ((word!(game.random_number) as i32 & 0x7f) - 64i32) as u16;
        let r20: u16 = (((word!(game.random_number) as i32 & 0x7f00) >> 8) - 64i32) as u16;
        Point16U {
            x: ((r18).wrapping_add((*E).base.x_pos)),
            y: ((r20).wrapping_add((*E).base.y_pos)),
        }
    }
}
pub unsafe fn Draygon_Func_42(varE24: u16, game: &mut Game) {
    sprite_effects::draygon_motion(varE24, game);
    crate::verify_probe::hit("sprite.draygon.42");
}
pub unsafe fn Draygon_Func_43(game: &mut Game) {
    sprite_effects::draygon_spawn_ring(game);
    crate::verify_probe::hit("sprite.draygon.43");
}
pub unsafe fn Draygon_Func_44(game: &mut Game) {
    sprite_effects::retire_all(game);
    crate::verify_probe::hit("sprite.draygon.44");
}
pub unsafe fn Draygon_Func_45(game: &mut Game) {
    sprite_effects::draygon_spawn_trails(game);
    crate::verify_probe::hit("sprite.draygon.45");
}
pub unsafe fn Draygon_Func_46(game: &mut Game) {
    unsafe {
        let cursor = (*Get_Draygon(0, game)).var_46;
        let next = sprite_effects::draygon_trajectory(cursor, game);
        (*Get_Draygon(0, game)).var_46 = next;
    }
}
pub unsafe fn Draygon_Func_48(k: u16, game: &mut Game) {
    unsafe {
        if ((*Get_Draygon(k as i32, game)).base.frame_counter & 0x7f) == 0 {
            let E1 = Get_Draygon(0x40, game);
            game.eproj_spawn_pt = Point16U {
                x: (((*E1).base.x_pos).wrapping_sub(24)),
                y: (((*E1).base.y_pos).wrapping_sub(32)),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x18, game);
        }
        let E0 = Get_Draygon(0, game);
        let v4: u16 = CalculateAngleFromXY(
            (word!(game.samus_x_pos) as i32 - ((*E0).base.x_pos as i32 - 24i32)) as u16,
            (word!(game.samus_y_pos) as i32 - ((*E0).base.y_pos as i32 - 32i32)) as u16,
            game,
        );
        let E = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        if v4 != (*E).var_F {
            let mut v6: u16 = addr_kDraygon_Ilist_99BA as u16;
            if v4 as i32 - 32i32 & 0x8000 == 0 {
                v6 = addr_kDraygon_Ilist_99B4 as u16;
                if v4 as i32 - 96i32 & 0x8000 == 0 {
                    v6 = addr_kDraygon_Ilist_99C0 as u16;
                    if v4 as i32 - 160i32 & 0x8000 == 0 {
                        v6 = addr_kDraygon_Ilist_99AE as u16;
                        if v4 as i32 - 224i32 & 0x8000 == 0 {
                            v6 = addr_kDraygon_Ilist_99BA as u16;
                        }
                    }
                }
            }
            (*E).var_F = v4;
            enemy_instruction! {[(*E).base]; instruction = v6, timer = 1}
            (*E).base.timer = 0;
        }
    }
}
pub unsafe fn Draygon_Func_49(k: u16, game: &mut Game) {
    unsafe {
        let E0 = Get_Draygon(0, game);
        if ((*Get_Draygon(k as i32, game)).base.frame_counter & 0x7f) == 0 {
            game.eproj_spawn_pt = Point16U {
                x: (((*E0).base.x_pos).wrapping_add(24)),
                y: (((*E0).base.y_pos).wrapping_sub(32)),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x18, game);
        }
        let v4: u16 = CalculateAngleFromXY(
            (word!(game.samus_x_pos) as i32 - ((*E0).base.x_pos as i32 + 24)) as u16,
            (word!(game.samus_y_pos) as i32 - ((*E0).base.y_pos as i32 - 32i32)) as u16,
            game,
        );
        let E = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        if v4 != (*E).var_F {
            let mut v6: u16 = addr_kDraygon_Ilist_9D5C as u16;
            if v4 as i32 - 32i32 & 0x8000 == 0 {
                v6 = addr_kDraygon_Ilist_9D50 as u16;
                if v4 as i32 - 96i32 & 0x8000 == 0 {
                    v6 = addr_kDraygon_Ilist_9D62 as u16;
                    if v4 as i32 - 160i32 & 0x8000 == 0 {
                        v6 = addr_kDraygon_Ilist_9D56 as u16;
                        if v4 as i32 - 224i32 & 0x8000 == 0 {
                            v6 = addr_kDraygon_Ilist_9D5C as u16;
                        }
                    }
                }
            }
            (*E).var_F = v4;
            enemy_instruction! {[(*E).base]; instruction = v6, timer = 1}
            (*E).base.timer = 0;
        }
    }
}
pub unsafe fn sub_A5E9F5(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 0xf) == 0 {
            NextRandom(game);
            let x: u16 = ((word!(game.random_number) as i32 & 0x3f) + 96) as u16;
            let y: u16 = ((((word!(game.random_number)) & (0xf00u16)) as i32 >> 8) + 480) as u16;
            create_sprite(x, y, 21, 0, game);
        }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallSporeSpawnFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10873626 => [()]; 10873627 => SporeSpawn_Func_1;
        10873682 => [{
                    SporeSpawn_Func_2(word!(game.cur_enemy_index), game);
                }];
        10873755 => SporeSpawn_Func_3; 10873838 => SporeSpawn_Func_4;
    }
}
pub unsafe fn SporeSpawn_Func_2(k: u16, game: &mut Game) {
    unsafe {
        crate::game::enemy_projectiles::update_spore_stem(game);
        let E0 = Get_SporeSpawn(0, game);
        let E = Get_SporeSpawn(k as i32, game);
        (*E).base.x_pos =
            ((*E).var_C).wrapping_add(CosineMult8bit((*E0).var_0A, (*E0).var_0B, game));
        (*E).base.y_pos = ((*E).var_D).wrapping_add(SineMult8bit(
            (2 * ((*E0).var_0A as i32 - 64i32)) as u16,
            ((*E0).var_0B).wrapping_sub(16),
            game,
        ));
        (*E0).var_0A = ((byte!((*E0).var_0C; 0)).wrapping_add(((*E0).var_0A) as u8)) as u16;
    }
}

pub unsafe fn SporeSpawn_Func_6(game: &mut Game) {
    unsafe {
        if (*Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game))
            .base
            .health
            == 0
        {
            let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
            let E0 = Get_SporeSpawn(0, game);
            (*E0).var_0E = 0;
            set_fields! {[(*E).base], 0; invincibility_timer, flash_timer, ai_handler_bits}
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            word_transfers!(game, descending, 14; native(0x19b1) <- value(0));
            enemy_instruction! {[(*E0).base]; instruction = addr_kDraygon_Ilist_E77D as u16, timer = 1}
            crate::update_place! { unaligned_word fresh5 = &raw mut (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize))) => (u16::from_le(fresh5.read_unaligned())) | (2u16); }
            word!(game.scrolling_finished_hook = 0);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x1e, 0xb78f as VoidP), game);
        }
    }
}
pub unsafe fn SporeSpawn_Func_7(a: u16, game: &mut Game) {
    unsafe {
        let mut v1: u16 = a;
        for i in (0i32..32).step_by(2) {
            word!(
                game.palette_buffer[((i >> 1) + 144)] =
                    crate::rtl::rom_table_word!(game, 0xa5e379, (v1 >> 1) as isize)
            );
            v1 = (v1).wrapping_add(2);
        }
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10847879] Draygon_Init [] {
        word_transfers!(game, descending, 25; le(target_palettes, 168) <- rom(0xa5a217, 24));
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .palette_index = 0xe00;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 165;
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa59342i32) as u16);
        word!(game.flag_process_all_enemies = -1i32 as u16);
        (*Get_Draygon(0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9889 as u16;
        (*Get_Draygon(0x40i32, game)).base.current_instruction = addr_kDraygon_Ilist_9944 as u16;
        (*Get_Draygon(0x80i32, game)).base.current_instruction = addr_kDraygon_Ilist_99FC as u16;
        (*Get_Draygon(0xc0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9813 as u16;
        word!(game.enemy_bg2_tilemap_size = 1024);
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game)).var_A = ((0xa5871bi32) as u16);
        sub_88DF34(game);
        word!(game.room_loading_irq_handler = 12);
        SpawnBG3ScrollHdmaObject(game);
        (*Get_Draygon(0i32, game)).var_45 = 1;
        minimap::disable_for_boss(game);
    };
    [10847996] Draygon_Main [] {
        let EK = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        CallDraygonFunc(
            (((*EK).var_A) as u32) | (0xa50000u32),
            word!(game.cur_enemy_index),
            game,
        );
        let E0 = Get_Draygon(0, game);
        let x_pos: u16 = (*E0).base.x_pos;
        let E1 = Get_Draygon(0x40, game);
        (*E1).base.x_pos = x_pos;
        let E2 = Get_Draygon(0x80, game);
        (*E2).base.x_pos = x_pos;
        let E3 = Get_Draygon(0xc0, game);
        (*E3).base.x_pos = x_pos;
        let y_pos: u16 = (*E0).base.y_pos;
        (*E1).base.y_pos = y_pos;
        (*E2).base.y_pos = y_pos;
        (*E3).base.y_pos = y_pos;
    };
    [10851661] Draygon_Hurt [] {
        let mut v0: u16 = addr_kDraygon_MorePalettes2 as u16;
        copy_draygon_hurt_palette!(game, v0, addr_kDraygon_BigSprmap_C08F, 17);
        if ((*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .flash_timer
            & 2)
            == 0
        {
            let mut v3: u16 = (4u16).wrapping_mul((*Get_Draygon(0i32, game)).var_0E);
            for i in (0i32..8).step_by(2) {
                word!(
                    game.palette_buffer[((i >> 1) + 89)] =
                        crate::rtl::rom_table_word!(game, 0xa596af, (v3 >> 1) as isize)
                );
                v3 = (v3).wrapping_add(2);
            }
        }
        let mut v5: u16 = addr_kDraygon_Palette as u16;
        copy_draygon_hurt_palette!(game, v5, addr_kDraygon_BigSprmap_C11B, 197);
        if (word!(game.samus_grapple_flags) & 1) != 0 {
            let E = Get_Draygon(0, game);
            if ((*E).base.frame_counter & 7) == 0 {
                let mut v9: i16 = (((*E).base.health) as i16).wrapping_sub(256);
                if v9 < 0 {
                    v9 = 0;
                }
                (*E).base.health = v9 as u16;
                sub_A5960D(game);
            }
        }
    };
    [10851818] Draygon_Touch [] {
        contact_without_death(game);
        sub_A5960D(game);
    };
    [10851847] Draygon_Powerbomb [] {
        power_bomb_without_death(game);
        sub_A5960D(game);
    };
    [10863723] DraygonsEye_Init [] {
        let E = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        (*E).base.current_instruction = addr_kDraygon_Ilist_9944 as u16;
        (*E).var_A = ((0xa5804bi32) as u16);
    };
    [10863750] DraygonsEye_Main [] {
        let E = Get_Draygon(word!(game.cur_enemy_index) as i32, game);
        CallDraygonFunc(
            (((*E).var_A) as u32) | (0xa50000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [10864025] DraygonsTail_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).current_instruction = addr_kDraygon_Ilist_99FC as u16;
        (*v0).palette_index = 0xe00;
    };
    [10864045] DraygonsArms_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).current_instruction = addr_kDraygon_Ilist_97E7 as u16;
        (*v0).palette_index = 0xe00;
        (*v0).layer = 2;
    };
    [10873386] SporeSpawn_Init [] {
        word_transfers!(game, ascending, 16; le(target_palettes, 240) <- rom(0xa5e359, 0));
        SpawnEprojWithGfx(
            0,
            word!(game.cur_enemy_index),
            addr_kEproj_SporeSpawnsStalk as u16,
            game,
        );
        SpawnEprojWithGfx(
            1,
            word!(game.cur_enemy_index),
            addr_kEproj_SporeSpawnsStalk as u16,
            game,
        );
        SpawnEprojWithGfx(
            2,
            word!(game.cur_enemy_index),
            addr_kEproj_SporeSpawnsStalk as u16,
            game,
        );
        SpawnEprojWithGfx(
            3,
            word!(game.cur_enemy_index),
            addr_kEproj_SporeSpawnsStalk as u16,
            game,
        );
        let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
        let E0 = Get_SporeSpawn(0, game);
        set_fields! {[(*E)]; var_04 = (*E).base.x_pos, var_05 = ((*E).base.y_pos).wrapping_sub(72),
        var_C = (*E).base.x_pos, var_D = (*E).base.y_pos, var_F = 0, };
        if (*ram_ptr!(game; 0xd828, word!(game.area_index) as isize) & 2) != 0 {
            (*E0).base.current_instruction = addr_kDraygon_Ilist_E6B9 as u16;
            (*E0).var_A = ((0xa5eb1ai32) as u16);
            (*E0).base.properties = ((*E0).base.properties) | (0x8000u16);
            crate::game::enemy_projectiles::update_spore_stem(game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x1e, 0xb793 as VoidP), game);
            word!(game.scrolling_finished_hook = 0);
        } else {
            (*E0).base.current_instruction = addr_kDraygon_Ilist_E6C7 as u16;
            word!(game.flag_process_all_enemies = -1i32 as u16);
            let v5: u16 = word!(game.cur_enemy_index);
            (*E).var_A = ((0xa5eb1ai32) as u16);
            word!(game.scrolling_finished_hook = ((0x909589i32) as u16));
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(128);
            SpawnEprojWithGfx(0, v5, addr_kEproj_SporeSpawners as u16, game);
            SpawnEprojWithGfx(1, v5, addr_kEproj_SporeSpawners as u16, game);
            SpawnEprojWithGfx(2, v5, addr_kEproj_SporeSpawners as u16, game);
            SpawnEprojWithGfx(3, v5, addr_kEproj_SporeSpawners as u16, game);
            crate::game::enemy_projectiles::update_spore_stem(game);
        };
    };
    [10873619] SporeSpawn_Main [] {
        let E = Get_SporeSpawn(word!(game.cur_enemy_index) as i32, game);
        CallSporeSpawnFunc((((*E).var_A) as u32) | (0xa50000u32), game);
    };
    [10874202] SporeSpawn_Shot [] {
        let mut v0: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
        let v1: u16 = word!(game.projectile_type[word!(game.collision_detection_index)]);
        if (v1 & 0x700) != 0 || (v1 & 0x10) != 0 {
            flash_on_shot(game);
            v0 = word!(game.cur_enemy_index);
            let EK = Get_SporeSpawn(v0 as i32, game);
            if (*EK).base.flash_timer != 0 {
                let E = Get_SporeSpawn(0, game);
                (*EK).var_A = ((0xa5eb52i32) as u16);
                let mut v4: i16 = 2;
                if (*EK).base.health as i32 - 400i32 & 0x8000 != 0 {
                    if ((*E).var_0C & 0x8000) != 0 {
                        v4 = -2i16;
                    }
                    (*E).var_0C = v4 as u16;
                }
                if (*E).var_2F == 0 {
                    (*E).var_0C = -((*E).var_0C as i32) as u16;
                    (*E).var_2F = 1;
                    enemy_instruction! {[(*EK).base]; instruction = addr_stru_A5E729 as u16, timer = 1}
                    let mut v7: u16 = 96;
                    let health: u16 = (*EK).base.health;
                    if health as i32 - 70i32 & 0x8000 == 0 {
                        v7 = 64;
                        if health as i32 - 410i32 & 0x8000 == 0 {
                            v7 = 32;
                            if health as i32 - 770i32 & 0x8000 == 0 {
                                v7 = 0;
                            }
                        }
                    }
                    if health != (*E).var_40 {
                        (*E).var_40 = health;
                        SporeSpawn_Func_7(v7, game);
                    }
                }
            }
            SporeSpawn_Func_6(game);
        }
    };
    [10874348] SporeSpawn_Touch [] {
        contact_without_death(game);
        SporeSpawn_Func_6(game);
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [10846223] Enemy_GrappleReact_CancelBeam_A5 => select_frozen_ai(game);
    [10846243] Enemy_NormalTouchAI_A5 => apply_contact(game);
    [10846253] Enemy_NormalShotAI_A5 => apply_shot(game);
    [10846273] Enemy_NormalFrozenAI_A5 => update_frozen_enemy(game);
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10851549] Draygon_Instr_1 [] {
        (*Get_Draygon(0i32, game)).base.current_instruction = jp.word(0);
        (*Get_Draygon(0x40i32, game)).base.current_instruction = jp.word(2);
        (*Get_Draygon(0x80i32, game)).base.current_instruction = jp.word(4);
        (*Get_Draygon(0xc0i32, game)).base.current_instruction = jp.word(6);
        (*Get_Draygon(0i32, game)).base.instruction_timer = 1;
        (*Get_Draygon(0x40i32, game)).base.instruction_timer = 1;
        (*Get_Draygon(0x80i32, game)).base.instruction_timer = 1;
        (*Get_Draygon(0xc0i32, game)).base.instruction_timer = 1;
        jp.offset(4)
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10852150] Draygon_Instr_13 [jp.offset(1)] { (*Get_Draygon(k as i32, game)).var_A = jp.word(0); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let pt: Point16U = Draygon_Func_47(game); };
    [10852159] Draygon_Instr_8 [jp] {  create_sprite(pt.x, pt.y, 21, 0, game); };
    [10852178] Draygon_Instr_7 [jp] {  create_sprite(pt.x, pt.y, 3, 0, game); };
    [10852197] Draygon_Instr_6 [jp] {  create_sprite(pt.x, pt.y, 29, 0, game); };
    [10852216] Draygon_Instr_9 [jp] {  create_sprite(pt.x, pt.y, 24, 0, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10852501] Draygon_Instr_2 [jp] { word!(game.room_loading_irq_handler = 12); };
    [10852563] Draygon_Instr_11 [jp] {
        let E2 = Get_Draygon(0x80, game);
        enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kDraygon_Ilist_97B9 as u16}
        let E3 = Get_Draygon(0xc0, game);
        enemy_instruction! {[(*E3).base]; timer = 1, instruction = addr_kDraygon_Ilist_97B9 as u16}
    };
    [10852591] Draygon_Instr_5 [jp] {
        let E = Get_Draygon(0, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
    [10853274] Draygon_Instr_15 [jp] {
        let E = Get_Draygon(0, game);
        (*E).var_0F = 24;
        let v3 = RomPtr_A0((*E).base.enemy_ptr, game);
        Samus_DealDamage(reduce_suit_damage(*(v3.offset(6) as *mut u16), game), game);
        word! {game; earthquake_timer = 32; earthquake_type = 7; }
        create_sprite(
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_add(16),
            21,
            0,
            game,
        );
    };
    [10853514] Draygon_Instr_17 [jp] { word!(game.room_loading_irq_handler = 12); };
    [10853898] Draygon_Instr_14 [jp.offset(2)] {
        let E = Get_Draygon(0, game);
        (*E).var_5E = jp.word(0);
        (*E).var_5F = jp.word(2);
    };
    [10854231] Draygon_Instr_16 [jp.offset(1)] { (*Get_Draygon(0, game)).var_A = jp.word(0); };
    [10854240] Draygon_Instr_10 [jp.offset(1)] { QueueSfx2_Max6(jp.word(0), game); };
    [10854254] Draygon_Instr_4 [jp.offset(1)] { QueueSfx3_Max6(jp.word(0), game); };
    [10854268] Draygon_Instr_12 [jp] {
        let E = Get_Draygon(0, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_sub(28)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        word!(game.cinematic_var9 = ((NextRandom(game) as i32 & 0x3f) + 128) as u16);
        SpawnEprojWithGfx(
            2,
            word!(game.cur_enemy_index),
            addr_stru_868E50 as u16,
            game,
        );
    };
    [10854318] Draygon_Instr_18 [jp] {
        let E = Get_Draygon(0, game);
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(24)),
            y: (((*E).base.y_pos).wrapping_sub(16)),
        };
        word!(game.cinematic_var9 = ((NextRandom(game) as i32 & 0x3f) + 192) as u16);
        SpawnEprojWithGfx(
            2,
            word!(game.cur_enemy_index),
            addr_stru_868E50 as u16,
            game,
        );
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10863739] Draygon_Instr_3 [jp.offset(1)] { (*Get_Draygon(0x40, game)).var_A = jp.word(0); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10872671] Draygon_Instr_25 [jp] {
        let E = Get_Draygon(0, game);
        if (*E).var_0B as i32 - 40i32 & 0x8000 != 0 {
            (*E).var_0B = ((*E).var_0B).wrapping_add(8);
        }
    };
    [10872689] Draygon_Instr_24 [jp] { (*Get_Draygon(0, game)).var_2F = 0; };
    [10872877] Draygon_Instr_21 [jp.offset(2)] {
        let E = Get_Draygon(0, game);
        (*E).var_0B = jp.word(0);
        (*E).var_0C = jp.word(2);
    };
    [10872946] Draygon_Instr_22 [jp.offset(1)] { word!(game.kraid_unk9000 = jp.word(0)); };
    [10872956] Draygon_Instr_27 [jp] {
        let E = Get_Draygon(0, game);
        set_fields! {[(*E).base]; x_pos = 128, y_pos = 624,
            properties = ((*E).base.properties & 0x5bff | 0xa000), };
    };
    [10872981] Draygon_Instr_23 [jp.offset(1)] { QueueSfx2_Max6(jp.word(0), game); };
    [10873009] Draygon_Instr_30 [jp] { scatter(DropSource::SporeSpawn, k, 0, game); };
    [10873018] Draygon_Instr_20 [jp.offset(1)] {
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game)).var_A = jp.word(0);
    };
    [10873034] Draygon_Instr_29 [jp.offset(1)] {
        let v2: u16 = jp.word(0);
        word_transfers!(game, ascending, 16;
            le(palette_buffer, 144) <- rom(0xa5e3f9, (v2 as i32 >> 1)),
        );
        word_transfers!(game, ascending, 16;
            le(palette_buffer, 64) <- rom(0xa5e4f9, (v2 as i32 >> 1)),
        );
        word_transfers!(game, ascending, 16;
            le(palette_buffer, 112) <- rom(0xa5e5d9, (v2 as i32 >> 1)),
        );
    };
    [10873116] Draygon_Instr_19 [jp.offset(1)] {
        let v2: u16 = jp.word(0);
        word_transfers!(game, ascending, 16;
            le(target_palettes, 144) <- rom(0xa5e3f9, (v2 as i32 >> 1)),
        );
        word_transfers!(game, ascending, 16;
            le(target_palettes, 64) <- rom(0xa5e4f9, (v2 as i32 >> 1)),
        );
        word_transfers!(game, ascending, 16;
            le(target_palettes, 112) <- rom(0xa5e5d9, (v2 as i32 >> 1)),
        );
    };
    [10873198] Draygon_Instr_28 [jp] {
        NextRandom(game);
        let mut r18: u16 = ((word!(game.random_number) as i32 & 0x7f) - 64i32) as u16;
        let E = Get_Draygon(0, game);
        r18 = (r18).wrapping_add((*E).base.x_pos);
        let mut r20: u16 =
            ((((word!(game.random_number)) & (0x7f00u16)) as i32 >> 8) - 64i32) as u16;
        r20 = (r20).wrapping_add((*E).base.y_pos);
        game.eproj_spawn_pt = Point16U { x: r18, y: r20 };
        SpawnEprojWithRoomGfx(0xe509, 0x15, game);
        QueueSfx2_Max6(0x29, game);
    };
    [10873265] Draygon_Instr_26 [jp] {
        NextRandom(game);
        let E = Get_Draygon(0, game);
        let x: u16 =
            ((*E).base.x_pos as i32 + (word!(game.random_number) as i32 & 0x7f) - 64i32) as u16;
        let y: u16 = ((*E).base.y_pos as i32
            + (((word!(game.random_number)) & (0x3f00u16)) as i32 >> 8)
            - 32i32) as u16;
        create_sprite(x, y, 3, 0, game);
        QueueSfx2_Max6(0x25, game);
    };
}
