use crate::game::actors;
use crate::game::actors::botwoon;
pub use crate::game::constants::bank_86::*;
use crate::game::enemy_layouts::EnemyLayout104 as Boss;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::gameplay_flag_runtime;
pub use crate::game::prelude::*;
use crate::game::session_runtime::actor_species_pre_instruction;
use crate::ram_map::slot;

use crate::game::room_mechanisms::spawn_hardcoded;

pub use crate::types::Consts_B3;

crate::game::constants::typed_constants! { Consts_B3;
    addr_kUnusedSpinningTurtleEye_Ilist_86A7 = 34471,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
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

}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @external {
        11771328 => [botwoon::animate_motion(game)];
        11771517 => [{ let address = botwoon::aim_spit(game); actor_species_pre_instruction(address, game); }];
        11771616 => [botwoon::spit(true, game)];
        11771700 => [botwoon::spit(false, game)];
        11771770 => [botwoon::wait_spit(k, game)];
    }
}
