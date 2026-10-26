use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::actors::botwoon;
pub use crate::game::constants::bank_86::*;
use crate::game::enemy_layouts::EnemyLayout104 as Boss;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_runtime;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::ram_map::slot;

use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::music::QueueSfx2_Max6;
use crate::game::room_mechanisms::spawn_hardcoded;

pub use crate::types::Consts_B3;

crate::game::constants::typed_constants! { Consts_B3;
    addr_kEscapeDachora_Ilist_E964 = 59748, addr_kEscapeEtecoon_Ilist_E5DA = 58842,
    addr_kEscapeEtecoon_Ilist_E5AE = 58798, addr_kEscapeEtecoon_Ilist_E582 = 58754,
    addr_kEscapeEtecoon_Ilist_E556 = 58710,
    addr_kUnusedSpinningTurtleEye_Ilist_86A7 = 34471, addr_kSpritemap_Nothing_B3 = 32845,
}
crate::type_aliases! {
    pub Enemy_EscapeDachora, Enemy_EscapeEtecoon = crate::game::enemy_layouts::EnemyLayout8;
}

crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_EscapeDachora(o: i32) -> Enemy_EscapeDachora = [0xf78, o as isize];
    Get_EscapeEtecoon(o: i32) -> Enemy_EscapeEtecoon = [0xf78, o as isize];
}
pub fn UnusedSpinningTurtleEye_Main() {}
crate::game::callback_dispatch! {
    #[allow(unused_unsafe)]
    pub unsafe fn CallBugInstr(ea: u32, _k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @external {
        11766733 => actors::norfair_pipe_bug::await_group;
        11766783 => actors::norfair_pipe_bug::await_player;
        11766950 => actors::norfair_pipe_bug::rise;
        11767039 => actors::norfair_pipe_bug::settle;
        11767052 => [actors::norfair_pipe_bug::move_to_limit(-16, game)];
        11767118 => [actors::norfair_pipe_bug::move_to_limit(-32, game)];
        11767184 => [actors::norfair_pipe_bug::move_to_limit(16, game)];
        11767250 => [actors::norfair_pipe_bug::move_to_limit(32, game)];
        11767316 => [actors::norfair_pipe_bug::fly(2, game)];
        11767349 => [actors::norfair_pipe_bug::fly(0, game)];
        11767386 => actors::norfair_pipe_bug::start_flight;
        11767733 => actors::yellow_pipe_bug::detect_player;
        11767797 => [actors::yellow_pipe_bug::launch_delay(word!(game.cur_enemy_index), game)];
        11767848 => [actors::yellow_pipe_bug::fly(true, game)];
        11767997 => [actors::yellow_pipe_bug::fly(false, game)];
        11768154 => [actors::yellow_pipe_bug::turn(true, game)];
        11768280 => [actors::yellow_pipe_bug::turn(false, game)];
    }
}
// Room creation and narrow numeric dispatch retain their unsafe bank contracts.
crate::game::callback_dispatch! {
    pub unsafe fn CallBotwoonFuncVarD(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    Botwoon_Init [] {
        let e = i32::from(word!(game.cur_enemy_index));
        if gameplay_flag_runtime::boss_byte_at(4, game) & 2 != 0 {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb797 as VoidP), game);
            crate::game::room_mechanisms::open_passage(game);
            slot!(game, Boss[e].base.current_instruction = 0x9389);
            slot!(game, Boss[e].base.properties, |old| old | kEnemyProps_Deleted as u16);
        } else {
            botwoon::initialize(e, game);
        }
    };
    [11770013] Botwoon_Chase [] {
        let e = i32::from(word!(game.cur_enemy_index));
        if !botwoon::select_attack(e, game) {
            step_botwoon_body(e, game);
            botwoon::detect_portal(word!(game.cur_enemy_index), game);
        }
    };
    [11770276] Botwoon_FollowPath [] {
        let e = i32::from(word!(game.cur_enemy_index));
        if !botwoon::resume_chase(e, game) {
            step_botwoon_body(e, game);
        }
    };
    [11770340] Botwoon_AwaitSpit [] {
        let e = i32::from(word!(game.cur_enemy_index));
        if botwoon::await_spit(e, game) {
            actor_species_pre_instruction(0xb30000 | u32::from(slot!(game, Boss[e].var_F)), game);
        }
    };
    [11770570] Botwoon_AwaitBody [] {
        let e = i32::from(word!(game.cur_enemy_index));
        if slot!(game, Boss[e].var_5F) != 0 {
            slot!(game, Boss[e].var_D = 0x9af9);
            let k = word!(game.cur_enemy_index);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb79b as VoidP), game);
            scatter(DropSource::Botwoon, k, game);
            slot!(game, Boss[i32::from(k)]; var_23 = 0, var_24 = 0, var_25 = 0);
        }
    };
    @functions [unsafe fn (e: i32, game: &mut Game)];
    step_botwoon_body [] {
        CallBotwoonFuncVarE(0xb30000 | u32::from(slot!(game, Boss[e].var_E)), game);
        crate::game::enemy_projectiles::record_botwoon_head(game);
        crate::game::enemy_projectiles::follow_botwoon_history(game);
        actor_species_pre_instruction(0xb30000 | u32::from(slot!(game, Boss[e].var_F)), game);
        crate::game::enemy_projectiles::orient_botwoon_body(game);
        crate::game::enemy_projectiles::advance_botwoon_history(game);
    };
    @external {
        11769976 => botwoon::wait_entry;
        11770438 => botwoon::wait_collapse;
        11770462 => botwoon::fall;
        11770617 => [botwoon::rubble(k, game)];
    }
}
crate::game::callback_dispatch! {
    #[allow(unused_unsafe)]
    pub unsafe fn CallBotwoonFuncVarE(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @external {
        11770807 => botwoon::seek_hole;
        11788940 => botwoon::advance_path;
        11788880 => botwoon::load_path;
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11765499] UnusedSpinningTurtleEye_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*v0).current_instruction = addr_kUnusedSpinningTurtleEye_Ilist_86A7 as u16;
    };
    [11769448] Botwoon_Main [] {
        botwoon::arm_death(game);
        let e = i32::from(word!(game.cur_enemy_index));
        CallBotwoonFuncVarD(
            0xb30000 | u32::from(slot!(game, Boss[e].var_D)),
            word!(game.cur_enemy_index),
            game,
        );
        botwoon::health_palette(game);
    };
    [11771903] Botwoon_Touch [] { botwoon::contact(false, game); };
    [11771969] Botwoon_Powerbomb [] { botwoon::contact(true, game); };
    [11789909] EscapeEtecoon_Main [] {
        let E = Get_EscapeEtecoon(word!(game.cur_enemy_index) as i32, game);
        actor_species_pre_instruction((((*E).var_F) as u32) | (0xb30000u32), game);
    };
    [11790027] EscapeEtecoon_Init [] {
        let E = Get_EscapeEtecoon(word!(game.cur_enemy_index) as i32, game);
        if CheckEventHappened(0xf, game) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32 | 0x8000))
                as u16;
            (*E).base.instruction_timer = 1;
            set_fields! {[(*E).base], 0; timer, palette_index}
            let v3: i32 = (*E).parameter_1 as i32 >> 1;
            (*E).base.x_pos = crate::rtl::rom_table_word!(game, 0xb3e718, v3 as isize);
            (*E).base.y_pos = crate::rtl::rom_table_word!(game, 0xb3e71e, v3 as isize);
            (*E).var_F = crate::rtl::rom_table_word!(game, 0xb3e724, v3 as isize);
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xb3e72a, v3 as isize);
            (*E).var_A = crate::rtl::rom_table_word!(game, 0xb3e730, v3 as isize);
        };
    };
    [11791077] EscapeDachora_Init [] {
        let E = Get_EscapeDachora(word!(game.cur_enemy_index) as i32, game);
        if CheckEventHappened(0xf, game) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
            (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_B3 as u16;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            (*E).base.current_instruction = addr_kEscapeDachora_Ilist_E964 as u16;
        };
    };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11789637] EscapeEtecoon_Instr_1 [] {
        if word!(game.lava_acid_y_pos) >= 0xce {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [11789840] EscapeEtecoon_Instr_2 [jp.offset(1)] {
        let E = Get_EscapeEtecoon(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(jp.word(0));
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11791016] EscapeDachora_Instr_2 [] {
        if word!(game.lava_acid_y_pos) >= 0xce {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [11791032] EscapeDachora_Instr_3 [] {
        if CheckEventHappened(0xf, game) != 0 {
            EnemyScript::jump(jp.word(0))
        } else {
            jp.offset(1)
        }
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E = Get_EscapeDachora(word!(game.cur_enemy_index) as i32, game); };
    [11791049] EscapeDachora_Instr_1 [jp] {  (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(6); };
    [11791063] EscapeDachora_Instr_4 [jp] {  (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(6); };
    @external {
        11769031 | 11769095 | 11769111 | 11769175 => [{ botwoon::set_hitbox(8, 16, game); j }];
        11769047 | 11769079 | 11769127 | 11769159 => [{ botwoon::set_hitbox(12, 12, game); j }];
        11769063 | 11769143 => [{ botwoon::set_hitbox(16, 8, game); j }];
        11769191 => [{ botwoon::set_spitting(game); j }];
        11769202 => [{ QueueSfx2_Max6(0x7c, game); j }];
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11789916] EscapeEtecoon_E65C [] {
        let E = Get_EscapeEtecoon(k as i32, game);
        words!((*E).base, x_pos, x_subpos += 0x38000);
    };
    [11789936] EscapeEtecoon_E670 [] {
        if CheckEventHappened(0xf, game) != 0 {
            (*Get_EscapeEtecoon(k as i32, game))
                .base
                .current_instruction = addr_kEscapeEtecoon_Ilist_E5DA as u16;
        }
    };
    [11789952] EscapeEtecoon_E680 [] {
        let E = Get_EscapeEtecoon(k as i32, game);
        if ({ let (actor, amount) = (k, ((*E).var_A as i16 as i32) << 8); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            (*E).base.instruction_timer = 1;
            let v3: bool = (-((*E).var_A as i32) & 0x8000) != 0;
            (*E).var_A = -((*E).var_A as i32) as u16;
            (*E).base.current_instruction = (if v3 {
                addr_kEscapeEtecoon_Ilist_E556 as i32
            } else {
                addr_kEscapeEtecoon_Ilist_E582 as i32
            }) as u16;
            if CheckEventHappened(0xf, game) != 0 {
                (*E).base.current_instruction = addr_kEscapeEtecoon_Ilist_E5AE as u16;
            }
        }
        ({ let (actor, amount) = (k, (1i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
    };
    @external {
        11771328 => [botwoon::animate_motion(game)];
        11771517 => [{ let address = botwoon::aim_spit(game); actor_species_pre_instruction(address, game); }];
        11771616 => [botwoon::spit(true, game)];
        11771700 => [botwoon::spit(false, game)];
        11771770 => [botwoon::wait_spit(k, game)];
    }
}
