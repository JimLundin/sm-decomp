use crate::c_library::printf;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::actors::reflec;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::actor_sine;
use crate::game_state::Game;
use std::ffi::c_char;

pub use crate::types::Consts_A3;
crate::game::constants::typed_constants! { Consts_A3;
    addr_kMaridiaFish_Ilist_9072 = 36978,
    addr_kMaridiaFish_Ilist_9060 = 36960, addr_kMaridiaFish_Ilist_903C = 36924,
    addr_kMaridiaFish_Ilist_902A = 36906,
}

crate::type_aliases! {
    pub Enemy_MaridiaFish = crate::game::enemy_layouts::EnemyLayout40;
}

crate::ram_map::ram_pointer_functions! {
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
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
