use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::{actor_cosine, actor_sine};
use crate::game::visual_runtime::darken_fireflea_room;

use crate::cpu_infra::Call;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_b4::*;

use crate::game::music::QueueSfx2_Max6;

pub use crate::game::waver::{Waver_Init, Waver_Instr_1, Waver_Main};

pub use crate::types::Consts_A3;

crate::game::constants::typed_constants! { Consts_A3;
    addr_kBang_Ilist_B75E = 46942,
    addr_kMaridiaFish_Ilist_9072 = 36978,
    addr_kMaridiaFish_Ilist_9060 = 36960, addr_kMaridiaFish_Ilist_903C = 36924,
    addr_kMaridiaFish_Ilist_902A = 36906, addr_kFireflea_Ilist_8C2F = 35887,
    addr_kWaver_Ilist_86A7 = 34471,
}

crate::type_aliases! {
    pub Enemy_Bang =
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

crate::game::constants::typed_constants! { i32;
    fnBang_Func_6 = 0xa3bca5,
    fnBang_Func_7 = 0xa3bcc1, fnBang_Func_8 = 0xa3bcc5,
    fnBang_Func_10 = 0xa3bd1c, fnBang_Func_11 = 0xa3bd2c,
    fnReflec_Func_1 = 0xa3db0c, fnnullsub_304 = 0xa3e6c1,
}
crate::ram_map::ram_pointer_functions! {
    Get_Bang(o: i32) -> Enemy_Bang = [0xf78, o as isize];
    Get_FireFlea(o: i32) -> Enemy_FireFlea = [0xf78, o as isize];
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
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
}
