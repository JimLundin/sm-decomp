use crate::game::actors;
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::rippers::{self, Kind};
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;

use crate::game::music::QueueSfx2_Max6;

pub use crate::types::Consts_A2;

unsafe fn update_randomized_pre_instruction(game: &mut Game) {
    unsafe {
        game.next_random();
        let phase = {
            let runtime = Runtime::new(game);
            let key = Key::from_legacy(runtime.word(0xe54));
            runtime
                .resolve_actor(key, actors::access::AccessMode::Native)
                .pre_instruction
        };
        actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
    }
}

crate::game::constants::typed_constants! { Consts_A2;
    addr_kGripper_Sprmap_E44B = 58443, addr_kGripper_Sprmap_E43F = 58431,
    addr_kMaridiaLargeSnail_Ilist_CA4B = 51787,
    addr_kMaridiaSpikeyShell_Ilist_A3BD = 41917, addr_kMaridiaSpikeyShell_Ilist_A3AB = 41899,
    addr_kSpritemap_Nothing_A2 = 32845,
}

crate::type_aliases! {
    pub Enemy_MaridiaLargeSnail, Enemy_MaridiaSpikeyShell =
        crate::game::enemy_layouts::EnemyLayout40;
}

crate::game::constants::typed_constants! { i32;
    fnSamus_FrameHandlerAlfa_Func11 = 0x90e695, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnnullsub_182 = 0xa2a01b, fnnullsub_187 = 0xa2a7d7,
    fnMaridiaLargeSnail_Func_7 = 0xa2cf66,
    fnMaridiaLargeSnail_Func_8 = 0xa2cfa9,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    Get_MaridiaLargeSnail(o: i32) -> Enemy_MaridiaLargeSnail = [0xf78, o as isize];
    Get_MaridiaSpikeyShell(o: i32) -> Enemy_MaridiaSpikeyShell = [0xf78, o as isize];
}
pub unsafe fn ThinHoppingBlobs_Main(game: &mut Game) {
    if !crate::game::hopping_blobs::advance(game) {
        unsafe {
            Unreachable(game);
        }
    }
}
static off_A2A3D3: [Func_V; 7] = callback_table! { nonnull Func_V;
    MaridiaSpikeyShell_1, MaridiaSpikeyShell_2, MaridiaSpikeyShell_3, MaridiaSpikeyShell_4,
    MaridiaSpikeyShell_5, MaridiaSpikeyShell_6, MaridiaSpikeyShell_7,
};
pub unsafe fn MaridiaSpikeyShell_8(k: u16, game: &mut Game) {
    unsafe {
        game.next_random();
        let E = Get_MaridiaSpikeyShell(k as i32, game);
        if ((byte!((*E).base.frame_counter; 0)).wrapping_add((word!(game.random_number)) as u8))
            as i32
            - 6i32
            & 0x8000
            != 0
        {
            (*E).var_E = 3;
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMaridiaLargeSnailFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game); };
    MaridiaLargeSnail_Init [] {
        set_fields! {[(*E)], 0; var_B, var_D, var_00, var_01, var_02, var_03, var_0A}
        (*E).var_E = 3;
        (*E).var_C = 128;
        (*E).base.current_instruction = addr_kMaridiaLargeSnail_Ilist_CA4B as u16;
        (*E).var_A = (0xa2cde6i32) as u16;
        (*E).var_F = (0xa2cf66i32) as u16;
    };
    [10669542] MaridiaLargeSnail_Func_4 [] {
        if (*E).var_E != 0 {
            actor_species_pre_instruction((((*E).var_F) as u32) | (0xa20000u32), game);
        } else {
            (*E).var_00 = 0;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                set_fields! {[(*E)], 1; var_D, var_00}
            }
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x18); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0 {
                (*E).var_00 = ((*E).var_00) | (2u16);
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = (0xa2ce2bi32) as u16;
            }
        };
    };
    [10669611] MaridiaLargeSnail_Func_5 [] {
        if (*E).var_E != 0 {
            actor_species_pre_instruction((((*E).var_F) as u32) | (0xa20000u32), game);
        } else if ({ let (actor, amount) = (word!(game.cur_enemy_index), (1i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) == 0 {
            (*E).var_B = 0;
            (*E).var_F = (0xa2cf66i32) as u16;
            (*E).var_E = 3;
        }
        if (*E).var_0B == 0 {
            let mut R36: u16 = 0;
            (*E).var_C = (*E).var_C.wrapping_sub(1);
            let v4: bool = ((*E).var_C & 0x8000) != 0;
            if v4
                && {
                    (*E).var_C = 0;
                    ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x20); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
                }
                && (*E).var_03 != 0
                && (*E).var_E == 0
            {
                set_fields! {[(*E)], 0; var_00, var_D}
                if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                    set_fields! {[(*E)], 1; var_00, var_D}
                }
                (*E).var_C = 128;
                (*E).var_00 = (*E).var_00 & 1 | 4;
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = (0xa2cf40i32) as u16;
            } else {
                let mut v5: u16 = 128;
                if (*E).var_D != 0 {
                    v5 = 132;
                }
                let v6: i32 = v5 as i32 >> 1;
                let r18: u16 = crate::rtl::rom_table_word!(game, 0xa28187, (v6 + 1) as isize);
                let mut r20: u16 = crate::rtl::rom_table_word!(game, 0xa28187, v6 as isize);
                if (*E).var_04 != 0 && (*E).var_D == (*E).var_05 {
                    if (*E).var_D != 0 {
                        r20 = (r20).wrapping_sub(16);
                    } else {
                        r20 = (r20).wrapping_add(16);
                    }
                }
                if ({ let (actor, amount) = (word!(game.cur_enemy_index), (words!((r20, r18))) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Solid) == actor_api::MovementResult::Blocked) as u8 }) != 0
                {
                    R36 = R36.wrapping_add(1);
                }
                if (*E).var_04 != 0 && (*E).var_D == (*E).var_05 {
                    let mut v7: u16 = if (*E).var_D != 0 {
                        ((*E).base.x_pos).wrapping_add(16)
                    } else {
                        ((*E).base.x_pos).wrapping_sub(16)
                    };
                    (*E).base.x_pos = v7;
                }
                if R36 != 0 {
                    (*E).var_D = ((*E).var_D) ^ (1u16);
                    (*E).var_00 = ((*E).var_00) ^ (4u16);
                    MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                }
            }
        }
    };
    [10669888] MaridiaLargeSnail_Func_6 [] {
        if (*E).var_02 != 0 {
            (*E).var_02 = 0;
            (*E).var_00 = ((*E).var_00).wrapping_sub(2);
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            (*E).var_A = (0xa2ce2bi32) as u16;
        }
    };
    MaridiaLargeSnail_Func_11 [] {
        set_fields! {[(*E)], 0; var_04, var_05}
        if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x20); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x18); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            (*E).var_04 = 1;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                (*E).var_05 = 1;
            }
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_1(k: u16, game: &mut Game) {
    unsafe {
        MaridiaLargeSnail_Func_11(game);
        let E = Get_MaridiaLargeSnail(k as i32, game);
        (*E).var_06 = (*E).base.x_pos;
        (*E).var_08 = (*E).base.y_pos;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    unsafe {
        Enemy_NormalTouchAI_A2(game);
        MaridiaLargeSnail_Touch(game);
    }
}
fn MaridiaLargeSnail_Func_10(k: u16, game: &mut Game) {
    actors::animation::restart_from_native_cache(&mut Runtime::new(game), usize::from(k), 0xa2cb77);
}
crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10652144] MiniCrocomire_Main [] {
        let index = game.cur_enemy_index();
        let preinstruction = Runtime::new(game)
            .resolve_actor(Key::from_legacy(index), Le)
            .pre_instruction;
        actor_species_pre_instruction(u32::from(preinstruction) | 0xa20000, game);
    };
    [10653138] MaridiaBeybladeTurtle_Main [] { if !crate::game::maridia_turtles::advance(false, game) { Unreachable(game); } };
    [10653998] MiniMaridiaBeybladeTurtle_Main [] { if !crate::game::maridia_turtles::advance(true, game) { Unreachable(game); } };
    [10658942] MaridiaSpikeyShell_Main [] {
        let e = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game);
        off_A2A3D3[((*e).var_E as i16 as i32 + 2i32) as usize](game);
    };
    @functions [pub fn (game: &mut Game)];
    [10659396] GunshipTop_Init [] { crate::game::gunship::initialize_top(game); };
    [10659538] GunshipBottom_Init [] { crate::game::gunship::initialize_bottom(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10659673] GunshipTop_Main [] {
        let phase = crate::game::gunship::advance(game);
        actor_species_pre_instruction(phase, game);
    };
    @functions [pub fn (game: &mut Game)];
    [10663426] Rinka_Init [] { crate::game::rinka::initialize(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10663876] Rinka_Main [] {
        if !crate::game::rinka::advance(game) {
            Unreachable(game);
        }
    };
    @functions [pub fn (game: &mut Game)];
    [10664263] Rinka_Touch [] { crate::game::rinka::contact(game); };
    [10664269] Rinka_Shot [] { crate::game::rinka::shot(game); };
    @functions [pub fn (game: &mut Game)];
    [10664909] Rio_Init [] {
        crate::game::rio::initialize(game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [10664931] Rio_Main [] {
        game.next_random();
        if !crate::game::rio::advance(game) {
            Unreachable(game);
        }
    };
    [10669331] MaridiaLargeSnail_Main [] {
        MaridiaLargeSnail_Func_1(word!(game.cur_enemy_index), game);
        let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game);
        CallMaridiaLargeSnailFunc((((*E).var_A) as u32) | (0xa20000u32), game);
        MaridiaLargeSnail_Func_2(word!(game.cur_enemy_index), game);
        MaridiaLargeSnail_Func_3(word!(game.cur_enemy_index), game);
    };
    [10670988] MaridiaLargeSnail_Touch [] {
        let k: u16 = word!(game.cur_enemy_index);
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) == 0 {
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(
                ((*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game))
                    .base
                    .x_pos) as i16,
            ) < 0
            {
                word!(game.extra_samus_x_displacement -= 4);
            } else {
                word!(game.extra_samus_x_displacement += 4);
            }
        }
    };
    [10671028] MaridiaLargeSnail_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        QueueSfx2_Max6(0x57, game);
    };
    @functions [pub fn (game: &mut Game)];
    [10674643] Gripper_Init [] {
        rippers::initialize(Kind::Gripper, game);
    };
    [10674721] Gripper_Main [] {
        rippers::advance(Kind::Gripper, game);
    };
    [10674968] JetPowerRipper_Init [] {
        rippers::initialize(Kind::JetPowered, game);
    };
    [10675027] JetPowerRipper_Main [] {
        rippers::advance(Kind::JetPowered, game);
    };
    [10675113] JetPowerRipper_Shot [] {
        rippers::jet_shot(game);
    };
    [10675359] Ripper_Init [] {
        rippers::initialize(Kind::Ripper, game);
    };
    [10675418] Ripper_Main [] {
        rippers::advance(Kind::Ripper, game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [10657715] SpikeShootingPlant_Main [] {
        let phase = {
            let runtime = Runtime::new(game);
            runtime.resolve_actor(Key::from_legacy(runtime.word(0xe54)), actors::access::AccessMode::Native).pre_instruction
        };
        actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game); };
    [10658809] MaridiaSpikeyShell_Init [] {
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
        let mssl_parameter_1_low: u16 = byte!((*E).parameter_1; 0) as u16;
        (*E).var_E = mssl_parameter_1_low;
        off_A2A3D3[(mssl_parameter_1_low as i32 & 1i32) as usize](
            game,
        );
        let v3: i32 = 4 * byte!((*E).parameter_2; 0) as i32;
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
        (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
        let v4: i32 = byte!((*E).parameter_2; 1) as i32;
        (*E).var_02 =
            (crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize)).wrapping_add((*E).base.x_pos);
        (*E).var_01 =
            ((*E).base.x_pos).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize));
        (*E).var_00 = crate::rtl::rom_table_word!(
            game,
            0xa2a3ed,
            *(&raw mut (*E).parameter_1 as *mut u8).offset(1) as isize
        );
        (*E).var_F = 0;
        if (*E).var_E == 2 {
            (*E).var_F = 16;
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(16);
        }
    };
    MaridiaSpikeyShell_1 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3AB as u16, timer = 1}
        (*E).base.timer = 0;
    };
    MaridiaSpikeyShell_2 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3BD as u16, timer = 1}
        (*E).base.timer = 0;
    };
    MaridiaSpikeyShell_3 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_C));
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_01) as i16) < 0 {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
        }
        MaridiaSpikeyShell_8(word!(game.cur_enemy_index), game);
    };
    MaridiaSpikeyShell_4 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_B, var_A));
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_02) as i16) >= 0 {
            (*E).var_E = 0;
        }
        MaridiaSpikeyShell_8(word!(game.cur_enemy_index), game);
    };
    MaridiaSpikeyShell_5 [] {
        let v1: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v1;
        if v1 == 0 {
            (*E).var_E = 4;
        }
    };
    MaridiaSpikeyShell_6 [] {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).var_F = (*E).var_F.wrapping_add(1);
        if (*E).var_F as i32 - 16i32 & 0x8000 == 0 {
            (*E).var_E = 2;
            (*E).var_00 = crate::rtl::rom_table_word!(
                game,
                0xa2a3ed,
                *(&raw mut (*E).parameter_1 as *mut u8).offset(1) as isize
            );
        }
    };
    MaridiaSpikeyShell_7 [] {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        crate::update_place! { previous fresh29 = (*E).var_F => (*E).var_F.wrapping_sub(1); }
        if fresh29 == 1 {
            (*E).var_E = ((byte!(@pointer ram_ptr!(game, u16; 0x5e5); 1)) as u16) & (1u16);
        }
    };
    [10659193] MaridiaSpikeyShell_Shot [] {
        if (*E).var_E as i32 - 1i32 & 0x8000 != 0 {
            Enemy_NormalShotAI_A2(game);
        }
    };
    @functions [pub fn (game: &mut Game)];
    [10662880] NorfairErraticFireball_Init [] { crate::game::erratic_fireball::update(true, game); };
    [10662927] NorfairErraticFireball_Main [] { crate::game::erratic_fireball::update(false, game); };
    [10665625] NorfairLavajumpingEnemy_Init [] { crate::game::lava_jumper::initialize(game); };
    [10661995] Flies_Init [] { crate::game::flies::initialize(game); };
    [10666562] NorfairRio_Init [] { crate::game::norfair_rio::initialize(false, game); };
    [10667763] LowerNorfairRio_Init [] { crate::game::norfair_rio::initialize(true, game); };
    @forward [pub unsafe fn (game: &mut Game)];
    [10662175] Flies_Main => update_randomized_pre_instruction(game);
    [10665682] NorfairLavajumpingEnemy_Main => update_randomized_pre_instruction(game);
    [10666615] NorfairRio_Main => update_randomized_pre_instruction(game);
    [10667812] LowerNorfairRio_Main => update_randomized_pre_instruction(game);
    [10649615] Enemy_GrappleReact_CancelBeam_A2 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10649610] Enemy_GrappleReact_KillEnemy_A2 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [10649665] Enemy_NormalFrozenAI_A2 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    Enemy_NormalPowerBombAI_A2 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10649635] Enemy_NormalTouchAI_A2 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10649645] Enemy_NormalShotAI_A2 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    @external {
        10674038 => [crate::game::rising_falling::update(true, game)];
        10674222 => [crate::game::rising_falling::update(false, game)];
        10675718 => crate::game::lava_seahorse_pair::initialize;
        10675790 => [actor_species_pre_instruction(crate::game::lava_seahorse_pair::pre_instruction(game), game)];
        10676168 => [{ Enemy_NormalTouchAI_A2(game); crate::game::lava_seahorse_pair::synchronize(game); }];
        10676174 => [{ Enemy_NormalShotAI_A2(game); crate::game::lava_seahorse_pair::synchronize(game); }];
        10676180 => [{ Enemy_NormalPowerBombAI_A2(game); crate::game::lava_seahorse_pair::synchronize(game); }];
        10663280 => [crate::game::lavaquake::update(true, game)];
        10663311 => [crate::game::lavaquake::update(false, game)];
        10653036 => [crate::game::maridia_turtles::initialize(false, game)];
        10653085 => [crate::game::maridia_turtles::initialize(true, game)];
        10654337 => [crate::game::maridia_turtles::touch(false, game)];
        10654367 => [crate::game::maridia_turtles::touch(true, game)];
        10654479 => crate::game::maridia_turtles::shot;
        10657608 => crate::game::spike_plant::initialize;
        10652077 => crate::game::mini_crocomire::initialize;
        10651420 => crate::game::goofball::initialize;
        10651548 => crate::game::goofball::advance;
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10659181]
    MaridiaSpikeyShell_Instr_A56D [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 0; };
    [10659185]
    MaridiaSpikeyShell_Instr_A571 [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 1; };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10664371] Rinka_Instr_B9B3 [jp] { crate::game::rinka::hide(k, false, game); };
    [10664381] Rinka_Instr_B9BD [jp] { crate::game::rinka::hide(k, true, game); };
    [10664391] Rinka_Instr_B9C7 [jp] { crate::game::rinka::launch(k, game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10664899] EnemyInstr_Rio_Instr_1 [jp] {
        crate::game::rio::instruction_finished(game);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10665614] NorfairLavajumpingEnemy_Instr_BE8E [jp] { crate::game::lava_jumper::instruction_finished(game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10668907] MaridiaLargeSnail_Instr_CB6B [jp] { QueueSfx2_Max6(0xe, game); };
    [10669235] MaridiaLargeSnail_Instr_CCB3 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
    [10669246] MaridiaLargeSnail_Instr_CCBE [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 1;
    };
    [10669257] MaridiaLargeSnail_Instr_CCC9 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 0;
    };
    @external {
        10675707 => [crate::game::lava_seahorse_pair::fire_ready(j, game)];
        10666441 | 10666452 | 10666463 | 10666474 | 10666485 | 10666496 | 10666507
        | 10666518 | 10666529 | 10666540 | 10666551 | 10667730 | 10667741 | 10667752
            => [crate::game::norfair_rio::instruction(ea, j, game)];
        10654593 => [crate::game::maridia_turtles::instruction(0x9381, j, game)];
        10654738 => [crate::game::maridia_turtles::instruction(0x9412, j, game)];
        10654791 => [crate::game::maridia_turtles::instruction(0x9447, j, game)];
        10654801 => [crate::game::maridia_turtles::instruction(0x9451, j, game)];
        10654827 => [crate::game::maridia_turtles::instruction(0x946b, j, game)];
        10654853 => [crate::game::maridia_turtles::instruction(0x9485, j, game)];
        10654881 => [crate::game::maridia_turtles::instruction(0x94a1, j, game)];
        10654919 => [crate::game::maridia_turtles::instruction(0x94c7, j, game)];
        10654929 => [{ QueueSfx2_Max6(0x3a, game); j }];
        10657578 => [{ QueueSfx2_Max6(0x34, game); j }];
        10657941 => [{ crate::game::spike_plant::resume(k, game); j }];
        10657959 => [crate::game::enemy_projectiles::spawn_scripted(0xdafe, j, game)];
        10652030 => [crate::game::enemy_projectiles::spawn_scripted(0xdbf2, j, game)];
        10652048 => [crate::game::mini_crocomire::resume(false, j, game)];
        10652061 => [crate::game::mini_crocomire::resume(true, j, game)];
        10651846 => [crate::game::goofball::landed(j, game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_MaridiaLargeSnail(k as i32, game); };
    MaridiaLargeSnail_Func_2 [] {
        (*E).var_0B = 0;
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            word!(game.extra_samus_y_displacement = ((*E).base.y_pos).wrapping_sub((*E).var_08));
        }
        if (*E).var_04 != 0 {
            let r18: u16 = ((*E).base.x_pos).wrapping_sub((*E).var_06);
            if (r18 & 0x8000) != 0 {
                if (*E).var_05 == 0 {
                    return;
                }
            } else if (*E).var_05 != 0 {
                return;
            }
            word!(game.extra_samus_x_displacement += r18);
        }
    };
    MaridiaLargeSnail_Func_3 [] {
        if (*E).var_04 != 0 {
            let mut v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_06) as i16);
            if v2 == 0 {
                if (*E).var_05 != 0 {
                    if (*E).var_D == 0 || (word!(game.joypad1_lastkeys) & 0x100) == 0 {
                        return;
                    }
                } else if (*E).var_D != 0 || (word!(game.joypad1_lastkeys) & 0x200) == 0 {
                    return;
                }
                (*E).var_0B = 1;
                (*E).base.x_pos = (*E).var_06;
                return;
            }
            if v2 < 0 {
                if (word!(game.joypad1_lastkeys) & 0x100) != 0 {
                    (*E).var_0B = 1;
                    (*E).base.x_pos = (*E).var_06;
                }
            } else if (word!(game.joypad1_lastkeys) & 0x200) != 0 {
                (*E).var_0B = 1;
                (*E).base.x_pos = (*E).var_06;
            }
        }
    };
    [10669926] MaridiaLargeSnail_Func_7 [] {
        let mut v2: u16 = ((*E).var_B).wrapping_add(384);
        if v2 as i32 - 0x4000i32 & 0x8000 == 0 {
            v2 = 0x4000;
        }
        (*E).var_B = v2;
        let v5: i32 = 8 * (v2 as i32 >> 8) >> 1;
        if ({ let (actor, amount) = (k, *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 >> 1) as isize) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
            let v6: i16 = (((*E).var_B) as i16).wrapping_sub(4096);
            if v6 < 0 {
                (*E).var_E = 0;
            }
            (*E).var_B = v6 as u16;
            (*E).var_F = (0xa2cfa9i32) as u16;
        }
    };
    [10669993] MaridiaLargeSnail_Func_8 [] {
        let v2: i16 = (((*E).var_B) as i16).wrapping_sub(384);
        (*E).var_B = v2 as u16;
        if v2 >= 0 {
            let v3: u16 = ((v2 as i32 & 0x7f00) >> 8) as u16;
            let v5: i32 = 8 * v3 as i32 >> 1;
            ({ let (actor, amount) = (k, *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 + 2 >> 1) as isize) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        } else {
            (*E).var_B = 0;
            (*E).var_F = (0xa2cf66i32) as u16;
        };
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10665692] NorfairLavajumpingEnemy_Func_1 [] { crate::game::lava_jumper::follow_parent(k, game); };
    [10665754] NorfairLavajumpingEnemy_Func_2 [] { crate::game::lava_jumper::launch(k, game); };
    [10665790] NorfairLavajumpingEnemy_Func_3 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Rising, game); };
    [10665852] NorfairLavajumpingEnemy_Func_4 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Turning, game); };
    [10665916] NorfairLavajumpingEnemy_Func_5 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Falling, game); };
    [10662222] Flies_4 [] { crate::game::flies::circle(k, false, game); };
    [10662268] Flies_5 [] { crate::game::flies::circle(k, true, game); };
    [10662314] Flies_6 [] { crate::game::flies::dive(k, false, game); };
    [10662354] Flies_7 [] { crate::game::flies::dive(k, true, game); };
    [10666625] NorfairRio_Func_1 [] { crate::game::norfair_rio::follow_parent(k, false, game); };
    [10666727] NorfairRio_Func_2 [] { crate::game::norfair_rio::wait_to_dive(k, false, game); };
    [10666815] NorfairRio_Func_3 [] { crate::game::norfair_rio::finish_animation(k, false, false, game); };
    [10667014] NorfairRio_Func_6 [] { crate::game::norfair_rio::finish_animation(k, false, true, game); };
    [10667822] LowerNorfairRio_Func_1 [] { crate::game::norfair_rio::follow_parent(k, true, game); };
    [10667889] LowerNorfairRio_Func_2 [] { crate::game::norfair_rio::wait_to_dive(k, true, game); };
    [10667963] LowerNorfairRio_Func_3 [] { crate::game::norfair_rio::finish_animation(k, true, false, game); };
    [10668168] LowerNorfairRio_Func_6 [] { crate::game::norfair_rio::finish_animation(k, true, true, game); };
    [10666849] NorfairRio_Func_4 [] { crate::game::norfair_rio::dive(k, false, true, game); };
    [10666929] NorfairRio_Func_5 [] { crate::game::norfair_rio::dive(k, false, false, game); };
    [10667990] LowerNorfairRio_Func_4 [] { crate::game::norfair_rio::dive(k, true, true, game); };
    [10668077] LowerNorfairRio_Func_5 [] { crate::game::norfair_rio::dive(k, true, false, game); };
    @external {
        10675796 | 10675885 | 10675953 | 10676020 | 10676041
            => [crate::game::lava_seahorse_pair::phase(k, ea as u16, game)];
        10657722 => [crate::game::spike_plant::patrol(false, game)];
        10657772 => [crate::game::spike_plant::patrol(true, game)];
        10652227 => [crate::game::mini_crocomire::patrol(false, game)];
        10652252 => [crate::game::mini_crocomire::patrol(true, game)];
        10659852 => [crate::game::gunship::phase(k, 0xa80c, game)];
        10660048 => [crate::game::gunship::phase(k, 0xa8d0, game)];
        10660162 => [crate::game::gunship::phase(k, 0xa942, game)];
        10660176 => [crate::game::gunship::phase(k, 0xa950, game)];
        10660231 => [crate::game::gunship::phase(k, 0xa987, game)];
        10660285 => [crate::game::gunship::phase(k, 0xa9bd, game)];
        10660431 => [crate::game::gunship::phase(k, 0xaa4f, game)];
        10660445 => [crate::game::gunship::phase(k, 0xaa5d, game)];
        10660500 => [crate::game::gunship::phase(k, 0xaa94, game)];
        10660514 => [crate::game::gunship::phase(k, 0xaaa2, game)];
        10660639 => [crate::game::gunship::phase(k, 0xab1f, game)];
        10660704 => [crate::game::gunship::phase(k, 0xab60, game)];
        10660718 => [crate::game::gunship::phase(k, 0xab6e, game)];
        10660773 => [crate::game::gunship::phase(k, 0xaba5, game)];
        10660807 => [crate::game::gunship::phase(k, 0xabc7, game)];
        10660891 => [crate::game::gunship::phase(k, 0xac1b, game)];
        10661079 => [crate::game::gunship::phase(k, 0xacd7, game)];
        10661134 => [crate::game::gunship::phase(k, 0xad0e, game)];
        10661165 => [crate::game::gunship::phase(k, 0xad2d, game)];
    }
}
