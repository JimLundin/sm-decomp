use crate::game::actors;
use crate::game::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::game::actors::api as actor_api;
use crate::game::actors::crocomire_reactions::{
    advance_melt_scroll, advance_melt_upload, advance_phase, approach_wall, await_skeleton_script,
    complete_skeleton_melt, emit_melt_debris, finish_skeleton_escape, land_skeleton,
    launch_skeleton, melt_flesh_down, melt_flesh_up, melt_rumble_down, melt_rumble_up,
    melt_skeleton_up, open_skeleton_exit, prepare_flesh_tilemap, prepare_melt_graphics,
    prepare_skeleton_tilemap, prepare_spike_wall, queue_tilemap_upload, release_skeleton_room,
    reposition_skeleton, rumble_and_wait, wait_for_melt, wall_step,
};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_host::BossWordOrder;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::ram_map::slot;
use crate::ram_map::words as ram;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a4::*;

use crate::game::enemy_projectiles::SpawnEprojWithRoomGfx;
use crate::game::music::QueueSfx2_Max6;
use crate::game::player_update::minimap;
use crate::game::visual_runtime::spawn_hdma;

pub type Enemy_Crocomire = crate::game::enemy_layouts::EnemyLayout72;

type Crocomire = Enemy_Crocomire;
pub fn nullsub_305(game: &mut Game) {}
static off_A48C14: [Func_V; 45] = callback_table! { nonnull Func_V;
    approach_wall, emit_melt_debris, rumble_and_wait, melt_rumble_up, melt_rumble_down,
    rumble_and_wait, melt_rumble_up, melt_rumble_down, prepare_flesh_tilemap, prepare_melt_graphics,
    advance_melt_upload, melt_flesh_up, Crocomire_Func_64, advance_melt_scroll, Crocomire_Func_66,
    melt_flesh_down, wait_for_melt, melt_flesh_up, melt_flesh_down, wait_for_melt,
    melt_flesh_up, melt_flesh_down, prepare_skeleton_tilemap, prepare_melt_graphics, advance_melt_upload,
    Crocomire_Func_63, melt_skeleton_up, Crocomire_9506, advance_melt_scroll, Crocomire_Func_66,
    complete_skeleton_melt, release_skeleton_room, wall_step, prepare_spike_wall,
    launch_skeleton, land_skeleton, finish_skeleton_escape, open_skeleton_exit, await_skeleton_script,
    advance_phase, Crocomire_9B86, nullsub_305, advance_phase, Crocomire_Func_29,
    reposition_skeleton,
};
pub fn Crocomire_Func_29(game: &mut Game) {
    {
        word! { game, 0; reg_BG2HOFS, reg_BG2VOFS }
    }
}
fn spawn_random_dust(game: &mut Game) {
    let random = game.random_number();
    let mut offset = random & 0x1f;
    if (random as i16).wrapping_sub(4096) >= 0 {
        offset = offset.wrapping_neg();
    }
    spawn_dust(EnemyScript::Stop, offset, game);
}
pub unsafe fn Crocomire_Func_64(game: &mut Game) {
    unsafe {
        QueueSfx2_Max6(0x77, game);
        Crocomire_950F(game);
    }
}
pub unsafe fn Crocomire_9506(game: &mut Game) {
    unsafe {
        QueueSfx2_Max6(0x2d, game);
        Crocomire_950F(game);
    }
}
pub unsafe fn Crocomire_950F(game: &mut Game) {
    unsafe {
        static unk_A49559: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x10, 0x9563 as VoidP);
        let mut v1: u16 = (2 * (slot!(game, Crocomire[0x40i32].base.y_pos) as i32 - 72i32)) as u16;
        let v2: u16 = word!(game.reg_BG2VOFS);
        do_until! { (v1 & 0x8000) != 0 ;
            word!(game.crocomire_bg2_scroll_hdma_data[(v1 >> 1)] = v2);
            v1 = (v1).wrapping_sub(2);
        }
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        let v4: i32 = 0;
        slot!(
            game,
            Crocomire[0x80i32].var_D = slot!(game, Crocomire[v4].base.x_pos)
        );
        *(ram_ptr!(game, u16; 0xcae0)) = 255;
        *(ram_ptr!(game, u16; 0xcae0, 1)) = -13584i32 as u16;
        *(ram_ptr!(game, u16; 0xcae0, 3)) = 225;
        *(ram_ptr!(game, u16; 0xcae0, 4)) = -13330i32 as u16;
        *(ram_ptr!(game, u16; 0xcae0, 6)) = 0;
        slot!(
            game,
            Crocomire[v4].var_1F = spawn_hdma(0xa4, &unk_A49559, game)
        );
    }
}
fn spawn_dust(jp: EnemyScript, offset: u16, game: &mut Game) -> EnemyScript {
    game.eproj_spawn_pt = {
        let runtime = Runtime::new(game);
        let actor = runtime.resolve_actor(Key::from_legacy(0), AccessMode::Native);
        let random = runtime.word(ram::random_number) & 7;
        Point16U {
            x: (actor.x.whole as u32 + offset as u32 + random as u32) as u16,
            y: actor.radius.y.wrapping_add(actor.y.whole).wrapping_sub(16),
        }
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
    jp
}

fn walk(k: u16, pixels: i32, game: &mut Game) -> actor_api::MovementResult {
    actor_api::move_horizontal(
        &mut Runtime::new(game),
        Key::from_legacy(k),
        pixels << 16,
        actor_api::SlopePolicy::Ignore,
    )
}
pub fn Crocomire_9B86(game: &mut Game) {
    QueueMusic_Delayed8(6, game);
    word!(game.camera_distance_index = 0);
    let area = word!(game.area_index);
    let bits = gameplay_flag_runtime::boss_word_at(area, BossWordOrder::Little, game);
    gameplay_flag_runtime::set_boss_word_le(area, bits | 2, game);
    QueueMusic_Delayed8(6, game);
    spawn_dust(EnemyScript::Stop, 0xfff0, game);
    spawn_dust(EnemyScript::Stop, 0x10, game);
    advance_phase(game);
}

pub fn CrocomireTongue_Main() {}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10782343] Crocomire_Hurt [] {
        actors::crocomire_reactions::update_background(word!(game.cur_enemy_index), game);
        actors::crocomire_reactions::update_palette(game);
    };
    [10783322] Crocomire_Init [] {
        if !actors::crocomire_reactions::prepare_initialization(game) {
            minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
            word!(game.croco_word_7E069A = 0);
            let E_0: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, Crocomire[E_0]; var_A = 0, var_E = 0);
            *(ram_ptr!(game, u16; 0xcd20)) = 0;
            word_transfers!(game, descending, 17;
                le(target_palettes, 176) <- rom(0xa4b8bd, 16),
                le(target_palettes, 224) <- rom(0xa4b8dd, 16),
            );
            slot!(game, Crocomire[E_0].var_C = 4);
            word! {game; UNUSED_word_7E179E = 16; camera_distance_index = 2;
            enemy_bg2_tilemap_size = 1024; }
            slot!(game, Crocomire[E_0].base; current_instruction = addr_kCrocomire_Ilist_BADE as u16, extra_properties = (slot!(game, Crocomire[E_0].base.extra_properties)) | (4u16), instruction_timer = 1)
;
        };
    };
    [10783748] Crocomire_Main [] {
        let v0: i32 = slot!(game, Crocomire[0i32].var_A) as i32 >> 1;
        off_A48C14[v0 as usize](game);
        actors::crocomire_reactions::push_player(game);
        actors::crocomire_reactions::update_palette(game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    Crocomire_Func_63 [] {
        slot!(game, Crocomire[E].var_A, |old| (old).wrapping_add(2));
    };
    Crocomire_Func_66 [] {
        slot!(game, Crocomire[E]; var_E = 0, var_F = 0);
        slot!(game, Crocomire[E].var_D = 2048);
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        queue_tilemap_upload(2048, 0xfe, game);
        advance_phase(game);
    };
    [10811002] CrocomireTongue_Init [] {
        if (gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game) & 2) != 0 {
            slot!(game, Crocomire[E].base.properties, |old| old & 0xdcff | 0x300);
        } else {
            slot!(game, Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_BE56 as u16);
            slot!(game, Crocomire[E].base.extra_properties, |old| (old) | (0x404u16));
            slot!(game, Crocomire[E].base.instruction_timer = 1);
            slot!(game, Crocomire[E].var_A = 23);
            slot!(game, Crocomire[E].base.palette_index = 3584);
        };
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [10780707] Enemy_NormalTouchAI_A4 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10780717] Enemy_NormalShotAI_A4 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10780737] Enemy_NormalFrozenAI_A4 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        10795410 => actors::crocomire_reactions::power_bomb;
    }
}

crate::game::callback_dispatch! {
    #[allow(unused_unsafe)]
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10783995] Crocomire_Instr_11 [jp] { QueueSfx2_Max6(0x74, game); };
    [10784007] Crocomire_Instr_7 [jp] { QueueSfx2_Max6(0x25, game); };
    [10784019] Crocomire_Instr_19 [jp] { QueueSfx2_Max6(0x75, game); };
    [10784711] Crocomire_Instr_2 [jp] {
        {
            let mut runtime = Runtime::new(game);
            runtime.write_slice_le(ram::earthquake_type, 4);
            runtime.write_slice_le(ram::earthquake_timer, 5);
        }
        QueueSfx2_Max6(0x76, game);
    };
    [10784735] Crocomire_Instr_4 [jp] {
        let hit = Runtime::new(game)
            .resolve_actor(Key::from_legacy(0), AccessMode::Native).variables[1] & 0x800 != 0;
        if !hit {
            walk(game.cur_enemy_index(), -4, game);
        }
    };
    [10784762 | 10784767] Crocomire_Instr_3 [Crocomire_Instr_4(k, jp, game)] { spawn_random_dust(game); };
    [10784797] Crocomire_Instr_16 [] {
        if walk(game.cur_enemy_index(), -4, game) == actor_api::MovementResult::Blocked {
            Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |actor| {
                actor.variables[2] = 14;
            });
            EnemyScript::jump(addr_kCrocomire_Ilist_BF3C as u16)
        } else {
            let random = game.random_number();
            let side = if (random as i16).wrapping_sub(2048) >= 0 {
                (-32i16) as u16
            } else {
                32
            };
            spawn_dust(jp, side.wrapping_add(random & 0xf), game)
        }
    };
    [10784859] Crocomire_Instr_13 [jp] {
        let can_walk = {
            let runtime = Runtime::new(game);
            let actor = runtime.resolve_actor(Key::from_legacy(0), AccessMode::Native);
            ((actor.x.whole as i32 - actor.radius.x as i32 - 260
                - runtime.word(ram::layer1_x_pos) as i32) as i16) < 0
        };
        if can_walk {
            walk(game.cur_enemy_index(), 4, game);
        }
    };
    [10784895] Crocomire_Instr_18 [jp] { walk(k, 4, game); };
    [10784911] Crocomire_Instr_12 [Crocomire_Instr_13(k, jp, game)] { spawn_random_dust(game); };
    [10784916] Crocomire_Instr_17 [Crocomire_Instr_18(k, jp, game)] { spawn_random_dust(game); };
    @forward [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10787483] Crocomire_Instr_8 => spawn_dust(jp, 0xffe0, game);
    [10787488 | 10787503] Crocomire_Instr_6 => spawn_dust(jp, 0, game);
    [10787493] Crocomire_Instr_9 => spawn_dust(jp, 0xfff0, game);
    [10787498 | 10787513] Crocomire_Instr_5 => spawn_dust(jp, 0x10, game);
    [10787508] Crocomire_Instr_21 => spawn_dust(jp, 8, game);
    [10787518] Crocomire_Instr_23 => spawn_dust(jp, 0x18, game);
    [10787523] Crocomire_Instr_24 => spawn_dust(jp, 0x20, game);
    [10787528] Crocomire_Instr_10 => spawn_dust(jp, 0x28, game);
    [10787533] Crocomire_Instr_25 => spawn_dust(jp, 0x30, game);
    [10787538] Crocomire_Instr_26 => spawn_dust(jp, 0x38, game);
    [10787543] Crocomire_Instr_27 => spawn_dust(jp, 0x40, game);
    @external {
        10782374 => [actors::crocomire_reactions::dispatch(k, j, game)];
        10782546 => [actors::crocomire_reactions::choose_spit(k, j, game)];
    }
}
