use crate::c_library::printf;
use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::actors::reflec;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::{actor_cosine, actor_sine};
use crate::game::visual_runtime::darken_fireflea_room;
use crate::game_state::Game;
use std::ffi::c_char;

pub use crate::types::Consts_A3;
crate::game::constants::typed_constants! { Consts_A3;
    addr_kMaridiaFish_Ilist_9072 = 36978,
    addr_kMaridiaFish_Ilist_9060 = 36960, addr_kMaridiaFish_Ilist_903C = 36924,
    addr_kMaridiaFish_Ilist_902A = 36906, addr_kFireflea_Ilist_8C2F = 35887,
}

crate::type_aliases! {
    pub Enemy_MaridiaFish,
        Enemy_FireFlea = crate::game::enemy_layouts::EnemyLayout40;
}

crate::ram_map::ram_pointer_functions! {
    Get_FireFlea(o: i32) -> Enemy_FireFlea = [0xf78, o as isize];
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
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
    @forward [pub unsafe fn (game: &mut Game)];
    Enemy_NormalTouchAI_A3 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    Enemy_NormalShotAI_A3 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    Enemy_NormalPowerBombAI_A3 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
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
}

pub(crate) unsafe fn Reflec_Shot(game: &mut Game) {
    unsafe {
        if let Some(target) = reflec::prepare_shot(game) {
            printf(b"Possible bug. What is X?\n\0" as *const u8 as *const c_char);
            reflec::absorb(target, game);
        }
    }
}
