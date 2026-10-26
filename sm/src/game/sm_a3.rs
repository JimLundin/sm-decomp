use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::game::session_runtime::{actor_cosine, actor_sine};
use crate::game::visual_runtime::darken_fireflea_room;

use crate::cpu_infra::Call;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_b4::*;

use crate::game::music::{QueueSfx2_Max3, QueueSfx2_Max6};

pub use crate::game::waver::{Waver_Init, Waver_Instr_1, Waver_Main};

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MaridiaSnailData2 {
    pub field_0: VoidP,
    pub field_2: u16,
    pub field_4: VoidP,
    pub field_6: u16,
}
pub use crate::types::Consts_A3;

crate::game::constants::typed_constants! { Consts_A3;
    addr_stru_A3CD3A = 52538,
    addr_stru_A3CD32 = 52530, addr_stru_A3CD2A = 52522,
    addr_stru_A3CD22 = 52514, addr_stru_A3CD1A = 52506,
    addr_stru_A3CD12 = 52498, addr_stru_A3CD0A = 52490,
    addr_stru_A3CD02 = 52482, addr_stru_A3CCFA = 52474,
    addr_stru_A3CCF2 = 52466, addr_stru_A3CCEA = 52458,
    addr_stru_A3CCE2 = 52450,
    addr_kBang_Ilist_B75E = 46942,
    addr_kMaridiaFish_Ilist_9072 = 36978,
    addr_kMaridiaFish_Ilist_9060 = 36960, addr_kMaridiaFish_Ilist_903C = 36924,
    addr_kMaridiaFish_Ilist_902A = 36906, addr_kFireflea_Ilist_8C2F = 35887,
    addr_kWaver_Ilist_86A7 = 34471,
    addr_kSpritemap_Nothing_A3 = 32845,
}

crate::type_aliases! {
    pub Enemy_Bang, Enemy_MaridiaSnail =
        crate::game::enemy_layouts::EnemyLayout72;
}

crate::type_aliases! {
    pub Enemy_MaridiaFish,
        Enemy_FireFlea = crate::game::enemy_layouts::EnemyLayout40;
}

crate::type_aliases! {
    pub Enemy_Reflec =
        crate::game::enemy_layouts::EnemyLayout8;
}

const RomPtr_A3: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa3>;
crate::game::constants::typed_constants! { i32;
    fnBang_Func_6 = 0xa3bca5,
    fnBang_Func_7 = 0xa3bcc1, fnBang_Func_8 = 0xa3bcc5,
    fnBang_Func_10 = 0xa3bd1c, fnBang_Func_11 = 0xa3bd2c,
    fnnullsub_215 = 0xa3cf5f, fnMaridiaSnail_Func_15 = 0xa3d1b3,
    fnReflec_Func_1 = 0xa3db0c, fnnullsub_304 = 0xa3e6c1,
}
crate::ram_map::ram_pointer_functions! {
    Get_Bang(o: i32) -> Enemy_Bang = [0xf78, o as isize];
    Get_FireFlea(o: i32) -> Enemy_FireFlea = [0xf78, o as isize];
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
    Get_MaridiaSnail(o: i32) -> Enemy_MaridiaSnail = [0xf78, o as isize];
    Get_Reflec(o: i32) -> Enemy_Reflec = [0xf78, o as isize];
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_FireFlea(k as i32, game); };
    Fireflea_Func_1 [] {
        (*E).var_E = (*E).base.x_pos;
        (*E).var_F = (*E).base.y_pos;
    };
    Fireflea_Func_2 [] {
        (*E).var_D = ((byte!((*E).parameter_1; 1) as i32) << 8) as u16;
    };
    Fireflea_Func_3 [] {
        let mut v2: u16 = (8u16).wrapping_mul((byte!((*E).parameter_2; 0)) as u16);
        if ((*E).parameter_1 & 1) == 0 {
            v2 = (v2).wrapping_add(4);
        }
        (*E).var_02 = v2;
        let v3: i32 = v2 as i32 >> 1;
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
    };
    Fireflea_Func_4 [] {
        (*E).var_C = byte!(@pointer (rom_ptr!(game, u16; 0xa38d1d))
            .offset(*(&raw mut (*E).parameter_2 as *mut u8).offset(1isize) as isize); 0)
            as u16;
    };
    Fireflea_Func_5 [] {
        (*E).base.x_pos = ((*E).var_E).wrapping_add(actor_cosine((*E).var_D, (*E).var_C, game));
        (*E).base.y_pos = ((*E).var_F).wrapping_add(actor_sine((*E).var_D, (*E).var_C, game));
    };
    Fireflea_Func_6 [] {
        (*E).var_00 = ((*E).base.y_pos).wrapping_sub((*E).var_C);
        (*E).var_01 = ((*E).var_C).wrapping_add((*E).base.y_pos);
    };
}
pub unsafe fn Fireflea_Touch(k: u16, game: &mut Game) {
    unsafe {
        let v1: u16 = 0;
        Enemy_NormalTouchAI_A3(game);
        ({
            let (_ignored_key, animation) = (k, v1);
            actors::combat::die(
                &mut Runtime::new(game),
                actors::api::DeathKind::Normal { animation },
            )
        });
        darken_fireflea_room(game);
    }
}
pub unsafe fn Fireflea_Common(game: &mut Game) {
    unsafe {
        if (*Get_FireFlea(word!(game.cur_enemy_index) as i32, game))
            .base
            .health
            == 0
        {
            darken_fireflea_room(game);
        }
    }
}
macro_rules! advance_maridia_fish_sine_motion {
    ($enemy:ident, $game:ident, $collision_state:literal, $collision_instruction:ident) => {
        let v2: u16 = actor_sine((*$enemy).var_F, (*$enemy).var_00, $game);
        (*$enemy).var_04 = v2;
        if ({ let (actor, amount) = (word!($game.cur_enemy_index), ((((v2) as i16).wrapping_sub(((*$enemy).var_03) as i16)) as i32) << 16); (actor_api::move_vertical(&mut Runtime::new($game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*$enemy).var_A = (($collision_state) as u16);
            (*$enemy).base.instruction_timer = 1;
            (*$enemy).base.timer = 0;
            (*$enemy).base.current_instruction = $collision_instruction as u16;
        } else {
            (*$enemy).var_F = ((byte!((*$enemy).var_02; 0))
                .wrapping_add(((*$enemy).var_F) as u8))
                as u16;
        }
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallMaridiaFishFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game); };
    MaridiaFish_Init [] {
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as u16;
        (*E).var_A = (0xa39132i32) as u16;
        if byte!((*E).parameter_1; 1) == 0 {
            (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as u16;
            (*E).var_A = (0xa391abi32) as u16;
        }
        let v1: i32 = 8 * byte!((*E).parameter_1; 0) as i32 >> 1;
        (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize);
        (*E).var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize);
        (*E).var_00 = byte!((*E).parameter_2; 0) as u16;
        (*E).var_02 = byte!((*E).parameter_2; 1) as u16;
        set_fields! {[(*E)], 0; var_F, var_01}
        (*E).var_03 = actor_sine((*E).var_F, (*E).var_00, game);
    };
    [10719538] MaridiaFish_Func_1 [] {
        if ({ let (actor, amount) = (word!(game.cur_enemy_index), (words!((*E), var_E, var_D)) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*E).var_A = (0xa39224i32) as u16;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kMaridiaFish_Ilist_903C as u16, };
        } else {
            advance_maridia_fish_sine_motion!(E, game, 0xa39224i32, addr_kMaridiaFish_Ilist_903C);
        }
        (*E).var_03 = (*E).var_04;
    };
    [10719659] MaridiaFish_Func_2 [] {
        if ({ let (actor, amount) = (word!(game.cur_enemy_index), (words!((*E), var_C, var_B)) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*E).var_A = (0xa39256i32) as u16;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kMaridiaFish_Ilist_9072 as u16, };
        } else {
            advance_maridia_fish_sine_motion!(E, game, 0xa39256i32, addr_kMaridiaFish_Ilist_9072);
        }
        (*E).var_03 = (*E).var_04;
    };
    [10719780] MaridiaFish_Func_3 [] {
        if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            (*E).var_A = (0xa391abi32) as u16;
            (*E).var_02 = -((*E).var_02 as i32) as u16;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kMaridiaFish_Ilist_9060 as u16, };
        }
    };
    [10719830] MaridiaFish_Func_4 [] {
        if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            (*E).var_A = (0xa39132i32) as u16;
            (*E).var_02 = -((*E).var_02 as i32) as u16;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kMaridiaFish_Ilist_902A as u16, };
        }
    };
    MaridiaFish_Main [] {
        CallMaridiaFishFunc((((*E).var_A) as u32) | (0xa30000u32), game);
    };
}
pub fn Bang_Func_7(k: u16) {}
pub unsafe fn Bang_Func_9(game: &mut Game) {
    unsafe {
        let v1: u8 = (({
            let actor = word!(game.cur_enemy_index);
            actor_api::angle_to_player(&Runtime::new(game), Key::from_legacy(actor))
        }) as u8)
            .wrapping_sub(64);
        let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
        (*E).var_02 = v1 as u16;
        let v3: u16 = sign_extend8(((v1) as u16).wrapping_sub((*E).var_01));
        let v4: u16 = abs16(v3);
        if v4 as i32 - 48i32 & 0x8000 == 0 {
            (*E).var_F = (0xa3bd2ci32) as u16;
        }
    }
}
crate::game::countdown_callbacks! {
    pub unsafe fn (game: &mut Game);
    before {
        let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
        let v1: i16 = (((*E).var_0C) as i16).wrapping_sub(1);
        (*E).var_0C = v1 as u16;
    } when v1 < 0;
    Bang_Func_12 {
        (*E).var_0C = (*E).var_0D;
        if (((*E).var_0A) as i16).wrapping_sub(((*E).var_0B) as i16) < 0 {
            (*E).var_09 = ((*E).var_09).wrapping_add(22);
            (*E).var_0A = ((*E).var_0A).wrapping_add((*E).var_09);
        }
    };
    Bang_Func_13 {
        (*E).var_0C = (*E).var_0E;
        (*E).var_09 = ((*E).var_09).wrapping_sub(22);
        (*E).var_0A = ((*E).var_0A).wrapping_sub((*E).var_09);
    };
}
pub unsafe fn MaridiaSnail_Func_2(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let v2: i32 = j as i32 >> 1;
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, v2 as isize);
        let r20: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 1) as isize);
        let r22: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 2) as isize);
        let r24: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 3) as isize);
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).var_A = (r20 as i32
            + (r18 as i32
                ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).parameter_1 as isize) as i32))
            as u16;
        (*E).var_B = (r24 as i32
            + (r22 as i32
                ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).parameter_1 as isize) as i32))
            as u16;
    }
}
pub unsafe fn MaridiaSnail_Func_4(k: u16, game: &mut Game) {
    unsafe {
        let msl_var_08: u16 = (*Get_MaridiaSnail(k as i32, game)).var_08;
        if msl_var_08 != 3
            && msl_var_08 != 4
            && msl_var_08 != 5
            && word!(game.earthquake_timer) == 30
            && word!(game.earthquake_type) == 20
        {
            MaridiaSnail_Func_14(k, game);
        }
    }
}
fn Shift8AddMagn(a: i16, s: i32) -> u32 {
    let mut r: i32 = (a as i32) << 8;
    r = r + if r < 0 { -(s << 16i32) } else { s << 16 };
    r as u32
}
pub unsafe fn MaridiaSnail_D002(k: u16, j: *const u8, game: &mut Game) {
    unsafe {
        MaridiaSnail_Func_10(k, j, game);
        let E = Get_MaridiaSnail(k as i32, game);
        let rv: u8 = {
            let (actor, amount) = (k, Shift8AddMagn((*E).var_A as i16, 1) as i32);
            (actor_api::move_horizontal(
                &mut Runtime::new(game),
                Key::from_legacy(actor),
                amount,
                actor_api::SlopePolicy::Follow,
            ) == actor_api::MovementResult::Blocked) as u8
        };
        MaridiaSnail_Func_11(k, j, game);
        if rv != 0 {
            (*E).var_E = 0;
            let carry: u8 = {
                let actor = k;
                actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8
            };
            MaridiaSnail_Func_12(k, carry as u16, game);
            if {
                let (actor, amount) = (k, ((*E).var_B as i16 as i32) << 8);
                (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                    == actor_api::MovementResult::Blocked) as u8
            } != 0
            {
                (*E).var_A = -((*E).var_A as i32) as u16;
                MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
            }
        } else {
            let v7: u16 = ((*E).var_E).wrapping_add(1);
            (*E).var_E = v7;
            if v7 as i32 - 4i32 & 0x8000 != 0 {
                (*E).var_B = -((*E).var_B as i32) as u16;
                MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
            } else {
                MaridiaSnail_Func_14(k, game);
            }
        };
    }
}
pub unsafe fn MaridiaSnail_D07E(k: u16, j: *const u8, game: &mut Game) {
    unsafe {
        MaridiaSnail_Func_10(k, j, game);
        let E = Get_MaridiaSnail(k as i32, game);
        let rv: u8 = {
            let (actor, amount) = (k, Shift8AddMagn((*E).var_B as i16, 1) as i32);
            (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                == actor_api::MovementResult::Blocked) as u8
        };
        MaridiaSnail_Func_11(k, j, game);
        if rv != 0 {
            (*E).var_E = 0;
            if {
                let (actor, amount) = (k, ((*E).var_A as i16 as i32) << 8);
                (actor_api::move_horizontal(
                    &mut Runtime::new(game),
                    Key::from_legacy(actor),
                    amount,
                    actor_api::SlopePolicy::Follow,
                ) == actor_api::MovementResult::Blocked) as u8
            } != 0
            {
                (*E).var_B = -((*E).var_B as i32) as u16;
                MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
            } else {
                let carry: u8 = {
                    let actor = k;
                    actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor))
                        as u8
                };
                MaridiaSnail_Func_12(k, carry as u16, game);
            }
        } else {
            let v7: u16 = ((*E).var_E).wrapping_add(1);
            (*E).var_E = v7;
            if v7 as i32 - 4i32 & 0x8000 != 0 {
                (*E).var_A = -((*E).var_A as i32) as u16;
                MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
            } else {
                MaridiaSnail_Func_14(k, game);
            }
        };
    }
}
pub unsafe fn MaridiaSnail_Func_10(k: u16, j: *const u8, game: &mut Game) {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*(j as *mut u16));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*(j.offset(2) as *mut u16));
    }
}
pub unsafe fn MaridiaSnail_Func_11(k: u16, j: *const u8, game: &mut Game) {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(*(j as *mut u16));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(*(j.offset(2) as *mut u16));
    }
}
pub unsafe fn MaridiaSnail_Func_12(k: u16, carry: u16, game: &mut Game) {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        if carry == 0 {
            if ((*E).var_04).wrapping_add(1) >= 0x10 {
                (*E).var_05 = 0;
            } else {
                (*E).var_04 = (*E).var_04.wrapping_add(1);
            }
        } else {
            (*E).var_05 = 1;
            (*E).var_04 = 0;
        };
    }
}
pub unsafe fn MaridiaSnail_Func_13(k: u16, a: u16, game: &mut Game) {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).var_05 = 1;
        (*E).var_04 = 0;
    }
}
pub unsafe fn MaridiaSnail_Func_18(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        if (*E).var_C != 0 {
            (if (*E).base.x_pos >= word!(game.samus_x_pos) {
                MaridiaSnail_Func_19(k, game) as i32
            } else {
                0
            }) as u8
        } else {
            (if (*E).base.x_pos < word!(game.samus_x_pos) {
                MaridiaSnail_Func_19(k, game) as i32
            } else {
                0
            }) as u8
        }
    }
}
pub unsafe fn MaridiaSnail_Func_19(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_MaridiaSnail(k as i32, game);
        if (*E).var_08 == 2 || (*E).var_F == 0xcf5f {
            return 0;
        }
        let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3cdc2, (*E).var_07 as isize);
        (*E).var_07 = v2;
        (*E).base.current_instruction =
            *((&raw mut (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))).field_0 as *mut u8)
                .offset((8 * v2 as i32) as isize) as *mut VoidP);
        (*E).base.properties = *((&raw mut (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))).field_2
            as *mut u8)
            .offset((8 * v2 as i32) as isize) as *mut u16)
            | (*E).base.properties & !(3);
        (*E).var_D = *((&raw mut (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))).field_4
            as *mut u8)
            .offset((8 * v2 as i32) as isize) as *mut VoidP);
        (*E).var_C = *((&raw mut (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))).field_6
            as *mut u8)
            .offset((8 * v2 as i32) as isize) as *mut u16);
        MaridiaSnail_Func_2(k, (8u16).wrapping_mul(v2), game);
        MaridiaSnail_Func_3(k, game);
        (*E).var_05 = 1;
        (*E).var_04 = 0;
        1
    }
}
pub unsafe fn MaridiaSnail_Func_20(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let r20: u16 = ((*Get_MaridiaSnail(k as i32, game)).var_C) & (1u16);
        if (((word!(game.joypad1_lastkeys)) & (0x300u16)) as i32 >> 8) == 1 {
            if r20 != 0 {
                return 0;
            }
        } else if r20 == 0 {
            return 0;
        }
        1
    }
}
pub unsafe fn Reflec_Func_1(game: &mut Game) {
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
                    crate::rtl::rom_table_word!(game, 0xa3dabc, (v1 >> 1) as isize);
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
    [10718509] Fireflea_Init [] {
        let E = Get_FireFlea(word!(game.cur_enemy_index) as i32, game);
        (*E).base.current_instruction = addr_kFireflea_Ilist_8C2F as u16;
        if ((*E).parameter_1 & 2) != 0 {
            Fireflea_Func_1(word!(game.cur_enemy_index), game);
            Fireflea_Func_2(word!(game.cur_enemy_index), game);
            Fireflea_Func_3(word!(game.cur_enemy_index), game);
            Fireflea_Func_4(word!(game.cur_enemy_index), game);
            Fireflea_Func_5(word!(game.cur_enemy_index), game);
        } else {
            Fireflea_Func_3(word!(game.cur_enemy_index), game);
            Fireflea_Func_4(word!(game.cur_enemy_index), game);
            Fireflea_Func_6(word!(game.cur_enemy_index), game);
        };
    };
    [10718702] Fireflea_Main [] {
        let E = Get_FireFlea(word!(game.cur_enemy_index) as i32, game);
        if ((*E).parameter_1 & 2) != 0 {
            (*E).base.x_pos = ((*E).var_E).wrapping_add(actor_cosine(
                byte!((*E).var_D; 1) as u16,
                (*E).var_C,
                game,
            ));
            (*E).base.y_pos = ((*E).var_F).wrapping_add(actor_sine(
                byte!((*E).var_D; 1) as u16,
                (*E).var_C,
                game,
            ));
            (*E).var_D = ((*E).var_D)
                .wrapping_add(*((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16));
        } else {
            let v2: i32 = (*E).var_02 as i32 >> 1;
            words!(
                (*E).base,
                y_pos,
                y_subpos += words!((
                    crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize),
                    crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1) as isize)
                ))
            );
            if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_00) as i16) < 0
                || (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_01) as i16) >= 0
            {
                (*E).var_02 = ((*E).var_02) ^ (4u16);
            }
        };
    };
    [10718851] Fireflea_Powerbomb [] {
        Enemy_NormalPowerBombAI_A3(game);
        Fireflea_Common(game);
    };
    [10718857] Fireflea_Shot [] {
        Enemy_NormalShotAI_A3(game);
        Fireflea_Common(game);
    };
    [10735074] MaridiaSnail_Init [] {
        let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
        (*E).var_F = (0xa3cf5fi32) as u16;
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
        (*E).base.instruction_timer = 1;
        crate::update_place! { previous current_instruction: u16 = (*E).base.current_instruction => (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))
            .offset(current_instruction as isize))
        .field_0; }
        (*E).base.properties = ((*E).base.properties)
            | ((*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))
                .offset(current_instruction as isize))
            .field_2);
        (*E).var_D = (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))
            .offset(current_instruction as isize))
        .field_4;
        (*E).var_C = (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))
            .offset(current_instruction as isize))
        .field_6;
        (*E).var_08 = 0;
        (*E).var_06 = (*E).parameter_1;
        MaridiaSnail_Func_2(
            word!(game.cur_enemy_index),
            (current_instruction).wrapping_mul(8),
            game,
        );
    };
    [10735204] MaridiaSnail_Main [] {
        MaridiaSnail_Func_4(word!(game.cur_enemy_index), game);
        MaridiaSnail_Func_5(word!(game.cur_enemy_index), game);
        MaridiaSnail_Func_6(word!(game.cur_enemy_index), game);
        let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
        actor_species_pre_instruction((((*E).var_F) as u32) | (0xa30000u32), game);
    };
    [10736560] MaridiaSnail_Touch [] {
        let E = Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game);
        if ((*E).var_08 != 1
            || (*E).var_F == 0xcf5f
            || MaridiaSnail_Func_20(word!(game.cur_enemy_index), game) & 1 == 0)
            && ((*E).var_F == 0xd1b3 || word!(game.samus_has_momentum_flag) != 0)
        {
            MaridiaSnail_Func_22(word!(game.cur_enemy_index), game);
            if (*E).var_F == 0xd1b3 {
                QueueSfx2_Max3(0x70, game);
            }
        } else if (*E).var_F != 0xcf5f && (*E).var_08 != 4 && (*E).var_08 != 3 {
            Enemy_NormalTouchAI_A3(game);
            (*E).parameter_1 = (*E).var_06;
            if (*E).var_08 != 0 {
                MaridiaSnail_Func_19(word!(game.cur_enemy_index), game);
            }
            (*E).var_08 = 0;
        }
    };
    [10736745] MaridiaSnail_Shot [] {
        let v0: u16 =
            (word!(game.projectile_type[word!(game.collision_detection_index)])) & (0xff00u16);
        if v0 == 768 || v0 == 1280 {
            Enemy_NormalShotAI_A3(game);
        } else {
            let msl_var_08: u16 =
                (*Get_MaridiaSnail(word!(game.cur_enemy_index) as i32, game)).var_08;
            if msl_var_08 != 3 && msl_var_08 != 4 {
                MaridiaSnail_Func_23(word!(game.cur_enemy_index), game);
            }
            QueueSfx2_Max3(0x70, game);
        };
    };
    [10738643] Reflec_Init [] {
        let E = Get_Reflec(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa3dc0b, (*E).parameter_1 as isize);
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = (0xa3db0ci32) as u16;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 163;
        word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((16 * (*E).base.palette_index as i32 & 0xff00) >> 8) as u16
        );
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16);
    };
    [10738716] Reflec_Shot [] {
        let v0: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
        let EK = Get_Reflec(word!(game.cur_enemy_index) as i32, game);
        (*EK).base.invincibility_timer = 10;
        let v2: u16 = (32 * (*EK).parameter_2 as i32
            + 2 * (word!(game.projectile_dir[(v0 >> 1)]) as i32 & 0xf))
            as u16;
        let varE32: u16 = v2;
        let v3: i32 = v2 as i32 >> 1;
        if crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) == 0x8000 {
            word!(game.projectile_dir[(v0 >> 1)] |= (0x10u16));
            printf(b"Possible bug. What is X?\n\0" as *const u8 as *const c_char);
            let ET = Get_Reflec(v2 as i32, game);
            if (*ET).base.health != 0 {
                Enemy_NormalShotAI_SkipSomeParts_A3(game);
                if (*ET).base.health == 0 {
                    enemy_instruction! {[(*ET).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3dca6, (*ET).parameter_2 as isize), timer = 1}
                    (*ET).base.timer = 0;
                }
            }
        } else {
            let mut v4: i16 = crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) as i16;
            if v4 < 0 {
                v4 = -(crate::rtl::rom_table_word!(game, 0xa3dcae, (varE32 >> 1) as isize) as i32)
                    as i16;
            }
            let v5: i32 = v0 as i32 >> 1;
            word!(game.projectile_dir[v5] = v4 as u16);
            word!(game.projectile_type[v5] &= !(0x8000));
            crate::game::player_projectiles::reflect(v0, game);
            QueueSfx2_Max6(0x57, game);
        };
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Bang(word!(game.cur_enemy_index) as i32, game); };
    [10730163] Bang_Init [] {
        (*E).var_B = (*E).base.palette_index;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
        (*E).var_F = (0xa3bca5i32) as u16;
        (*E).var_00 = 16;
        set_fields! {[(*E)], 0; var_01, var_02, var_20, var_21, var_22}
        (*E).var_0B = crate::rtl::rom_table_word!(
            game,
            0xa3ba84,
            *(&raw mut (*E).parameter_2 as *mut u8) as isize
        );
        let v1: i32 = 2 * byte!((*E).parameter_2; 1) as i32;
        let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3bc6a, v1 as isize);
        (*E).var_0C = v2;
        (*E).var_0D = v2;
        (*E).var_0E = crate::rtl::rom_table_word!(game, 0xa3bc6a, (v1 + 1) as isize);
        if (*E).base.current_instruction == 0 {
            (*E).var_F = (0xa3bcc1i32) as u16;
        }
        (*E).base.current_instruction = addr_kBang_Ilist_B75E as u16;
    };
    [10730277] Bang_Main [] {
        Call((((*E).parameter_1) as u32) | (0xa30000u32), game);
    };
    Bang_Func_14 [] {
        let r18: u16 = crate::rtl::rom_table_word!(
            game,
            0xa0b1c3,
            ((((*E).var_01) as u8).wrapping_add(64)) as isize
        );
        if (r18 & 0x8000) != 0 {
            (*E).var_07 = 1;
        }
        let mut t: u32 = ((abs16(r18) as i32 >> 8) * (*E).var_0A as i32) as u32;
        if (*E).var_07 != 0 {
            t = -(t as i32) as u32;
        }
        t = t.wrapping_add((*E).base.x_subpos as u32);
        (*E).base.x_subpos = t as u16;
        (*E).base.x_pos = (*E).base.x_pos.wrapping_add((t >> 16) as u16);
    };
    Bang_Func_15 [] {
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa0b1c3, (*E).var_01 as u8 as isize);
        if (r18 & 0x8000) != 0 {
            (*E).var_08 = 1;
        }
        let mut t: u32 = ((abs16(r18) as i32 >> 8) * (*E).var_0A as i32) as u32;
        if (*E).var_08 != 0 {
            t = -(t as i32) as u32;
        }
        t = t.wrapping_add((*E).base.y_subpos as u32);
        (*E).base.y_subpos = t as u16;
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add((t >> 16) as u16);
    };
    Bang_Func_18 [] {
        let bang_var_20: u16 = (*E).var_20;
        if bang_var_20 != (*E).var_21 {
            (*E).var_21 = bang_var_20;
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa3b722, bang_var_20 as isize), timer = 1}
            (*E).base.timer = 0;
        }
    };
    [10731261] Bang_Shot [] {
        if (*E).var_F != 0xbcc1 {
            (*E).var_01 = crate::rtl::rom_table_word!(
                game,
                0xa3ba94,
                (word!(game.projectile_dir[word!(game.collision_detection_index)]) as i32 & 0xf)
                    as isize,
            );
            (*E).var_F = (0xa3bd2ci32) as u16;
            set_fields! {[(*E)], 0; var_07, var_08}
            (*E).var_09 = 256;
            (*E).var_0A = 1536;
        }
        if (*E).var_20 != 9 {
            (*E).var_20 = (*E).var_20.wrapping_add(1);
            Bang_Func_18(game);
            let v1: i32 = word!(game.collision_detection_index) as i32;
            (*E).var_E = ((*E).var_E).wrapping_add(word!(game.projectile_damage[v1]));
            word!(game.projectile_dir[v1] |= (0x10u16));
            if (*E).var_20 == 9 {
                (*E).var_D = 1;
            }
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [10715146] Enemy_GrappleReact_KillEnemy_A3 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [10715151] Enemy_GrappleReact_CancelBeam_A3 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10715171] Enemy_NormalTouchAI_A3 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10715181] Enemy_NormalShotAI_A3 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    Enemy_NormalShotAI_SkipSomeParts_A3 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
    Enemy_NormalPowerBombAI_A3 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10715201] Enemy_NormalFrozenAI_A3 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10719382] MaridiaFish_Instr_3 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game))
            .base
            .layer = 6;
    };
    [10719392] MaridiaFish_Instr_1 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game))
            .base
            .layer = 2;
    };
    [10719402] MaridiaFish_Instr_2 [jp] {
        (*Get_MaridiaFish(word!(game.cur_enemy_index) as i32, game)).var_01 = 1;
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10730104] Bang_Instr_1 [jp] { QueueSfx2_Max6(0x56, game); };
    [10730152]
    Bang_Instr_2 [jp] { (*Get_Bang(word!(game.cur_enemy_index) as i32, game)).var_22 = 1; };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10734646]
    MaridiaSnail_Instr_1 [jp.offset(1)] { (*Get_MaridiaSnail(k as i32, game)).var_F = jp.word(0); };
    [10734655]
    MaridiaSnail_Instr_2 [jp.offset(1)] { (*Get_MaridiaSnail(k as i32, game)).var_D = jp.word(0); };
    [10734664] MaridiaSnail_Instr_4 [jp.offset(1)] {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).var_07 = jp.word(0);
        (*E).var_C = *((&raw mut (*(rom_ptr!(game, MaridiaSnailData2; 0xa3cd42))
            )
        .field_6 as *mut u8)
            .offset((8 * (*E).var_07 as i32) as isize) as *mut u16);
    };
    [10734687] MaridiaSnail_Instr_3 [jp.offset(2)] {
        let E = Get_MaridiaSnail(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(jp.word(0));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(jp.word(2));
    };
    @functions [pub unsafe fn (k: u16, mut jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10734712] MaridiaSnail_Instr_5 [] {
        if (*Get_MaridiaSnail(k as i32, game)).var_08 == 2 || (game.next_random() & 1) != 0 {
            jp = jp.offset(-(3i32 as isize));
        }
        jp
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10738632] Reflec_Instr_1 [jp.offset(1)] { (*Get_Reflec(k as i32, game)).parameter_2 = jp.word(0); };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10730661] Bang_Func_6 [] {
        let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
        let v1: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v1;
        if v1 == 0 {
            (*E).var_00 = 16;
            (*E).var_F = (0xa3bcc5i32) as u16;
        }
    };
    [10730693] Bang_Func_8 [] {
        let v1: u8 = (({ let actor = word!(game.cur_enemy_index); actor_api::angle_to_player(&Runtime::new(game), Key::from_legacy(actor)) }) as u8)
            .wrapping_sub(64);
        let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
        (*E).var_01 = v1 as u16;
        (*E).var_F = (0xa3bd1ci32) as u16;
        set_fields! {[(*E)], 0; var_07, var_08, var_09, var_0A}
    };
    [10730780] Bang_Func_10 [] {
        Bang_Func_14(game);
        Bang_Func_15(game);
        Bang_Func_12(game);
        Bang_Func_9(game);
    };
    [10730796] Bang_Func_11 [] {
        Bang_Func_14(game);
        Bang_Func_15(game);
        Bang_Func_13(game);
        let E = Get_Bang(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_0A != 0 || ((*E).var_09 as i16) <= 0 {
            (*E).var_F = (0xa3bca5i32) as u16;
        }
    };
    [10735526] MaridiaSnail_Func_9 [] {
        let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
            addr_stru_A3CD22 as i32
        } else {
            addr_stru_A3CCE2 as i32
        }) as u16;
        MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
    };
    [10735543] MaridiaSnail_CFB7 [] {
        MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCEA as u16, game), game);
    };
    [10735549] MaridiaSnail_CFBD [] {
        let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
            addr_stru_A3CD2A as i32
        } else {
            addr_stru_A3CCF2 as i32
        }) as u16;
        MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
    };
    [10735566] MaridiaSnail_CFCE [] {
        MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCFA as u16, game), game);
    };
    [10735572] MaridiaSnail_CFD4 [] {
        let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
            addr_stru_A3CD32 as i32
        } else {
            addr_stru_A3CD02 as i32
        }) as u16;
        MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
    };
    [10735589] MaridiaSnail_CFE5 [] {
        MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD0A as u16, game), game);
    };
    [10735595] MaridiaSnail_CFEB [] {
        let mut v1: u16 = if (*Get_MaridiaSnail(k as i32, game)).var_05 != 0 {
            addr_stru_A3CD3A as u16
        } else {
            addr_stru_A3CD12 as u16
        };
        MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
    };
    [10735612] MaridiaSnail_CFFC [] {
        MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD1A as u16, game), game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_MaridiaSnail(k as i32, game); };
    MaridiaSnail_Func_3 [] {
        (*E).var_F = crate::rtl::rom_table_word!(game, 0xa3cdd2, (*E).var_07 as isize);
    };
    MaridiaSnail_Func_5 [] {
        let msl_var_08: u16 = (*E).var_08;
        if msl_var_08 != 1 && msl_var_08 != 3 && msl_var_08 != 4 && msl_var_08 != 5 {
            if (*E).base.y_pos as i32 - word!(game.samus_y_pos) as i32 + 96 & 0x8000 == 0 {
                let skip_instruction_change: bool =
                    if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
                        *ram_ptr!(game; 0xa1e) != 4
                    } else {
                        *ram_ptr!(game; 0xa1e) != 8
                    };
                if skip_instruction_change {
                } else {
                    if (*E).var_08 == 2 {
                        return;
                    }
                    if (*E).var_D as i32 != 0xa3cf5f & 0xffff {
                        set_fields! {[(*E).base]; current_instruction = (*E).var_D,
                            spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16,
                            instruction_timer = 1, timer = 0, };
                        (*E).var_08 = 2;
                        return;
                    }
                }
            }
            (*E).var_08 = 0;
        }
    };
    MaridiaSnail_Func_6 [] {
        if (*E).var_F as i32 != 0xa3d1b3 & 0xffff {
            let keep_properties: bool = if (*E).var_08 == 1 {
                word!(game.samus_has_momentum_flag) != 0 || (*E).var_F as i32 != 0xa3cf5f & 0xffff
            } else {
                !(word!(game.samus_has_momentum_flag) == 0)
                    || ({
                        let msl_var_08: u16 = (*E).var_08;
                        !(msl_var_08 == 2 || msl_var_08 != 3 && msl_var_08 != 5)
                    })
            };
            if keep_properties {
            } else {
                (*E).base.properties = ((*E).base.properties) | (0x8000u16);
                return;
            }
        }
        (*E).base.properties = ((*E).base.properties) & (0x7fffu16);
    };
    [10735456] MaridiaSnail_Func_7 [] {
        if (*E).var_07 >= 4 {
            if ({ let (actor, amount) = (k, Shift8AddMagn((*E).var_B as i16, 7) as i32); (actor_api::probe_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
                return;
            }
        } else if ({ let (actor, amount) = (k, Shift8AddMagn((*E).var_A as i16, 7) as i32); (actor_api::probe_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            return;
        }
        MaridiaSnail_Func_14(k, game);
    };
    MaridiaSnail_Func_14 [] {
        if (*E).var_08 != 3 {
            (*E).var_08 = 3;
            (*E).var_F = (0xa3d1b3i32) as u16;
            let v2: i32 = 2 * (*E).var_C as i32;
            (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d1ab, v2 as isize);
            (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d1ab, (v2 + 1) as isize);
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            set_fields! {[(*E)], 0; var_02, var_03, var_00, var_01}
        }
    };
    [10736051] MaridiaSnail_Func_15 [] {
        if (*E).var_08 != 3 {
            if ({ let (actor, amount) = (k, (words!((*E), var_03, var_02)) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
                (*E).var_02 = -((*E).var_02 as i32) as u16;
                (*E).var_03 = -((*E).var_03 as i32) as u16;
                (*E).var_20 = 1;
                QueueSfx2_Max3(0x70, game);
            } else {
                let mut delta: u32 = if ((*E).var_03 & 0x8000) != 0 {
                    4096
                } else {
                    -4096i32 as u32
                };
                let t: u32 = (words!((*E), var_03, var_02)).wrapping_add(delta);
                (*E).var_02 = t as u16;
                if t >> 16 != 0 {
                    (*E).var_03 = (t >> 16) as u16;
                }
            }
        }
        if ({ let (actor, amount) = (k, (words!((*E), var_01, var_00)) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            let msl_var_01: i16 = (*E).var_01 as i16;
            if msl_var_01 >= 0 && msl_var_01 as i32 - 3i32 & 0x8000 != 0 {
                set_fields! {[(*E)], 0; var_03, var_02, var_01, var_00, var_E, var_04}
                (*E).var_05 = 1;
                if (*E).var_08 == 3 {
                    (*E).var_08 = 0;
                } else {
                    (*E).var_08 = 1;
                    (*E).parameter_1 = 8;
                    MaridiaSnail_Func_18(k, game);
                    MaridiaSnail_Func_16(k, game);
                }
                (*E).base.properties = (*E).var_C | (*E).base.properties & !(3);
                actors::wall_crawlers::prepare(k, game);
                (*E).var_F = (0xa3cf5fi32) as u16;
                enemy_instruction! {[(*E).base]; instruction = (*E).var_D, timer = 1}
                (*E).base.timer = 0;
            } else {
                (*E).var_00 = -((*E).var_00 as i32) as u16;
                (*E).var_01 = -((*E).var_01 as i32) as u16;
                (*E).var_20 = 0;
            }
        } else {
            let t_0: u32 = (words!((*E), var_01, var_00)).wrapping_add(0x2000);
            (*E).var_00 = t_0 as u16;
            if (t_0 >> 16i32).wrapping_sub(4) & 0x8000 != 0 {
                (*E).var_01 = (t_0 >> 16) as u16;
            }
        };
    };
    MaridiaSnail_Func_16 [] {
        let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d30d, v2 as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d30d, (v2 + 1) as isize);
    };
    MaridiaSnail_Func_22 [] {
        (*E).var_08 = 4;
        (*E).var_F = (0xa3d1b3i32) as u16;
        let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d50f, v2 as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d50f, (v2 + 1) as isize);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_02 = word!(game.absolute_moved_last_frame_x_fract);
        let mut v3: u16 = word!(game.absolute_moved_last_frame_x);
        (*E).var_03 = word!(game.absolute_moved_last_frame_x);
        if v3 >= 0x10 {
            v3 = 15;
        }
        let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
        (*E).var_00 = crate::rtl::rom_table_word!(game, 0xa3d517, v4 as isize);
        (*E).var_01 = crate::rtl::rom_table_word!(game, 0xa3d517, (v4 + 1) as isize);
        if (*ram_ptr!(game; 0xa1e) & 4) != 0 {
            (*E).var_02 = -((*E).var_02 as i32) as u16;
            (*E).var_03 = -((*E).var_03 as i32) as u16;
        }
    };
    MaridiaSnail_Func_23 [] {
        (*E).var_08 = 5;
        (*E).var_F = (0xa3d1b3i32) as u16;
        let v2: i32 = ((4u16).wrapping_mul((*E).var_C)) as i32 >> 1;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d5a4, v2 as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa3d5a4, (v2 + 1) as isize);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_01 = -1i32 as u16;
        (*E).var_03 = (if *ram_ptr!(game; 0xa1e) == 4 {
            -1i32
        } else {
            1
        }) as u16;
    };
}
