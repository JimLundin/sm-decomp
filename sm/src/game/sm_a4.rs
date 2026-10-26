use crate::game::actors;
use crate::game::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::game::actors::api as actor_api;
use crate::game::actors::crocomire_reactions::{
    advance_melt_upload, approach_wall, emit_lava_plume, melt_flesh_down, melt_flesh_up,
    melt_rumble_down, melt_rumble_up, melt_skeleton_up, prepare_flesh_tilemap,
    prepare_melt_graphics, prepare_skeleton_tilemap, queue_melt_graphics, queue_tilemap_upload,
    select_melt_animation, wall_step,
};
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
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

use crate::game::enemy_projectiles::{ClearEprojs, SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::music::QueueSfx2_Max6;
use crate::game::player_update::minimap;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::visual_runtime::{align_crocomire_background, spawn_hdma};

pub type Enemy_Crocomire = crate::game::enemy_layouts::EnemyLayout72;

crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
type Crocomire = Enemy_Crocomire;
pub fn nullsub_305(game: &mut Game) {}
static off_A48C14: [Func_V; 45] = callback_table! { nonnull Func_V;
    approach_wall, Crocomire_Func_52, Crocomire_Func_36, melt_rumble_up, melt_rumble_down,
    Crocomire_Func_36, melt_rumble_up, melt_rumble_down, prepare_flesh_tilemap, prepare_melt_graphics,
    advance_melt_upload, melt_flesh_up, Crocomire_Func_64, Crocomire_Func_65, Crocomire_Func_66,
    melt_flesh_down, Crocomire_8D47, melt_flesh_up, melt_flesh_down, Crocomire_8D47,
    melt_flesh_up, melt_flesh_down, prepare_skeleton_tilemap, prepare_melt_graphics, advance_melt_upload,
    Crocomire_Func_63, melt_skeleton_up, Crocomire_9506, Crocomire_Func_65, Crocomire_Func_66,
    Crocomire_Func_49, Crocomire_Func_68, wall_step, Crocomire_Func_71,
    Crocomire_Func_72, Crocomire_Func_73, Crocomire_Func_88, Crocomire_9B65, Crocomire_Func_90,
    advance_phase, Crocomire_9B86, nullsub_305, advance_phase, Crocomire_Func_29,
    Crocomire_Func_69,
};
pub fn Crocomire_Func_29(game: &mut Game) {
    {
        word! { game, 0; reg_BG2HOFS, reg_BG2VOFS }
    }
}
pub unsafe fn Crocomire_Func_36(game: &mut Game) {
    unsafe {
        actors::crocomire_reactions::rumble(game);
        align_crocomire_background(game);
        Crocomire_8D47(game);
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
pub unsafe fn Crocomire_Func_49(game: &mut Game) {
    unsafe {
        select_melt_animation::<true>(Key::from_legacy(0), game);
        emit_lava_plume(game);
        actors::crocomire_reactions::melt_down(game);
        let E: i32 = 0;
        if slot!(game, Crocomire[E].var_A) == 62 {
            QueueMusic_Delayed8(6, game);
            slot!(game, Crocomire[E].var_A = 88);
            slot!(
                game,
                Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_E1D2 as u16
            );
            *(ram_ptr!(game, u16; 0xcd20, 4)) = 257;
            word!(game.debug_disable_minimap = 0);
            slot!(
                game,
                Crocomire[0x40i32].base.properties =
                    slot!(game, Crocomire[0x40i32].base.properties) | (0x200u16)
            );

            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x4e, 0x3, 0xb753 as VoidP), game);
            word! { game, 0; camera_distance_index, croco_target_0688 }
        }
    }
}

pub unsafe fn Crocomire_Func_52(game: &mut Game) {
    unsafe {
        let v0: u16 = *(ram_ptr!(game, u16; 0x9015, 1));
        if *(ram_ptr!(game, u16; 0x9015, 1)) as i32 - 22i32 & 0x8000 != 0 {
            let v1: u16 = *(ram_ptr!(game, u16; 0x9015, 1));
            crate::update_place! { unaligned_word fresh17 = &raw mut (*(ram_ptr!(game, u16; 0x9015, 1))) => (u16::from_le(fresh17.read_unaligned())).wrapping_add(2); }
            SpawnEprojWithGfx(
                crate::rtl::rom_table_word!(game, 0xa49156, (v0 >> 1) as isize),
                v1,
                addr_stru_868F9D as u16,
                game,
            );
        }
        melt_rumble_down(game);
    }
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
pub unsafe fn Crocomire_Func_65(game: &mut Game) {
    unsafe {
        emit_lava_plume(game);
        let E0: i32 = 0;
        let E1: i32 = 0x40;
        let E2: i32 = 0x80;
        let mut crocom_var_D: u16 = slot!(game, Crocomire[E2].var_D);
        slot!(game, Crocomire[E1].var_D, |old| old.wrapping_sub(1));
        if (slot!(game, Crocomire[E1].var_D) & 2) != 0 {
            crocom_var_D = (crocom_var_D).wrapping_add(4);
        }
        slot!(game, Crocomire[E0].base.x_pos = crocom_var_D);
        if Crocomire_Func_67(game) != 0 {
            actors::crocomire_reactions::update_background(word!(game.cur_enemy_index), game);
            let mut v4: u16 = (word!(game.g_word_7E0694)).wrapping_sub(3);
            let skip_tilemap_update: bool = (word!(game.g_word_7E0694) as i32 - 19i32 & 0x8000
                != 0)
                && ((((word!(game.g_word_7E0692)) as i16).wrapping_sub(20480) >= 0)
                    || ({
                        v4 = 16;
                        false
                    }));
            if skip_tilemap_update {
            } else {
                word!(game.g_word_7E0694 = v4);
                let mut v5: u16 = (word!(game.g_word_7E0692)).wrapping_add(384);
                if word!(game.g_word_7E0692) as i32 - 20096i32 & 0x8000 == 0 {
                    v5 = 20480;
                }
                word!(game.g_word_7E0692 = v5);
                let mut r18: u16 = 0;
                let mut v6: u16 =
                    (2 * (slot!(game, Crocomire[E1].base.y_pos) as i32 - 72i32)) as u16;
                let mut v7: u16 = word!(game.g_word_7E0694);
                word!(game.message_box_animation_y1 = word!(game.g_word_7E0694));
                do_until! { ((v7) as i16).wrapping_sub((word!(game.g_word_7E0698)) as i16) >= 0 ;
                    word!(
                        game.crocomire_bg2_scroll_hdma_data[(v6 >> 1)] =
                            (word!(game.reg_BG2VOFS) as i32 + v7 as i32
                                - word!(game.message_box_animation_y1) as i32)
                                as u16
                    );
                    let v8: bool = u16::overflowing_add(word!(game.g_word_7E0692), r18).1;
                    r18 = (r18).wrapping_add(word!(game.g_word_7E0692));
                    if !v8 {
                        v7 = v7.wrapping_add(1);
                    }
                    word!(game.message_box_animation_y1 += 1);
                    v6 = (v6).wrapping_add(2);
                }
                if ((v6) as i16).wrapping_sub(512) < 0 {
                    let v9: u16 = word!(game.reg_BG2VOFS);
                    do_until! { ((v6) as i16).wrapping_sub(512) >= 0 ;
                        word!(game.crocomire_bg2_scroll_hdma_data[(v6 >> 1)] = v9);
                        v6 = (v6).wrapping_add(2);
                    }
                }
                return;
            }
        }
        let EK: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[EK].var_A, |old| (old).wrapping_add(2));
        let mut i: i32 = word!(game.croco_word_7E069A) as i32;
        while *((rom_ptr!(game, u16; 0xa49bc5) as *mut u8).offset(i as isize) as *mut u16) != 0xffff
        {
            i += 8;
        }
        word!(game.croco_word_7E069A = (i + 2) as u16);
        word!(game.hdma_object_channels_bitmask[(slot!(game, Crocomire[E0].var_1F) >> 1)] = 0);
    }
}
static kCrocoEraseLineMasks: [u8; 8] = [0x7f, 0xbf, 0xdf, 0xef, 0xf7, 0xfb, 0xfd, 0xfe];
pub unsafe fn Crocomire_Func_67(game: &mut Game) -> u16 {
    unsafe {
        let mut n: i32 = *ram_ptr!(game; 0x68c) as i32;
        loop {
            if word!(game.croco_cur_vline_idx) > 48 {
                return 1;
            }
            let rr: i32 = byte!(@pointer rom_ptr!(game; 0xa49697); word!(game.croco_cur_vline_idx) as isize)
                as i32;
            if ((*ram_ptr!(game; 0x69c, rr as isize)) as i8)
                .wrapping_sub((word!(game.croco_target_0688)) as i8)
                < 0
            {
                break;
            }
            if word!(game.croco_cur_vline_idx += 1, reread) >= 128 {
                word!(game.croco_cur_vline_idx = 0);
                return 0;
            }
        }
        crate::native_assertion! {
            '_c2rust_label: word!(game.croco_cur_vline_idx) <= 48;
            b"croco_cur_vline_idx <= 48\0", b"src/sm_a4.c\0", 1215,
            b"uint16 Crocomire_Func_67(void)\0"
        };
        let vline_idx: i32 = byte!(@pointer rom_ptr!(game; 0xa49697); word!(game.croco_cur_vline_idx) as isize)
            as i32;
        let mask: u8 = kCrocoEraseLineMasks[(vline_idx & 7) as usize];
        do_until! { n == 0 ;
            let q: i32 = *ram_ptr!(game; 0x69c, vline_idx as isize) as i32;
            let j: i32 = 2 * (q & 7) + ((q & !(7i32)) << 6) + 4 * (vline_idx & !(7i32));
            // Melting tiles span the $4000 graphics buffer, beyond the menu backup prefix.
            for plane_offset in [0, 1, 16, 17] {
                game.g_ram[0x4000 + j as usize + plane_offset] &= mask;
            }
            if *ram_ptr!(game; 0x69c, vline_idx as isize) == 48 {
                break;
            }
            crate::update_place! { reference fresh5 = *ram_ptr!(game; 0x69c, vline_idx as isize) => (*fresh5).wrapping_add(1); }
            n -= 1;
        }
        let mut v7: u16 = 0;
        let mut v8: u16 = 0;
        loop {
            v7 = (word!(game.g_word_7E068A)).wrapping_add(word!(game.croco_word_7E069A));
            v8 = word!(game.vram_write_queue_tail);
            if *((rom_ptr!(game, u16; 0xa49bc5) as *mut u8).offset(v7 as i32 as isize) as *mut u16)
                != 0xffff
            {
                break;
            }
            word!(game.g_word_7E068A = 0);
        }
        queue_melt_graphics(v7, false, game);
        word!(game.vram_write_queue_tail = (v8).wrapping_add(7));
        word!(game.g_word_7E068A += 8);
        1
    }
}
pub unsafe fn Crocomire_Func_68(game: &mut Game) {
    if (game.samus_x_pos() as i16).wrapping_sub(640) >= 0 {
        return;
    }
    QueueMusic_Delayed8(5, game);
    Runtime::new(game).write_slice_native(0xcd23, 256);
    {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb757 as VoidP), game);
    }
    word!(game.camera_distance_index = 6);
    let mut runtime = Runtime::new(game);
    // Fixed, disjoint records; no callback or fallible operation between these writes.
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.properties = enemy.properties & 0x7bff | kEnemyProps_Tangible as u16;
        enemy.variables[3] = 4;
        enemy.variables[1] = 0;
        enemy.radius.y = 56;
        enemy.variables[0] = enemy.variables[0].wrapping_add(2);
    });
    runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
        enemy.properties |= 0x500;
        enemy.variables[3] = 0;
    });
    for (key, countdown) in [(0x80, 10), (0xc0, 1)] {
        runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
            enemy.variables[3] = countdown
        });
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
fn accelerated_speed(fraction: u16, whole: u16, acceleration: u32, limit: u16) -> u32 {
    let speed = ((u32::from(whole) << 16) | u32::from(fraction)).wrapping_add(acceleration);
    if ((speed >> 16) as i16).wrapping_sub(limit as i16) >= 0 {
        speed & 0xffff | (u32::from(limit) << 16)
    } else {
        speed
    }
}

fn advance_phase(game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[0] = enemy.variables[0].wrapping_add(2);
    });
}

pub fn Crocomire_Func_69(game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.x.whole = enemy.x.whole.wrapping_sub(2);
        if (enemy.x.whole as i16).wrapping_sub(480) < 0 {
            enemy.x.whole = 480;
            enemy.y.whole = 54;
            enemy.variables[0] = 62;
        } else {
            enemy.y.whole = 220;
        }
    });
}

pub fn Crocomire_Func_72(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(224) < 0 {
            let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 0x8000, 2);
            enemy.variables[4] = speed as u16;
            enemy.pre_instruction = (speed >> 16) as u16;
            enemy.x.advance(speed as i32);
        }
    });
    let finished = runtime.edit_actor(Key::from_legacy(0x80), AccessMode::Native, |enemy| {
        let previous = enemy.variables[3];
        enemy.variables[3] = previous.saturating_sub(1);
        previous == 1
    });
    if finished {
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[4] = 0;
            enemy.instruction = addr_kCrocomire_Ilist_E14A as u16;
            enemy.instruction_delay = 1;
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
    }
}

pub fn Crocomire_Func_73(game: &mut Game) {
    let impact = Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 2048, 5);
        enemy.variables[4] = speed as u16;
        enemy.pre_instruction = (speed >> 16) as u16;
        enemy.y.advance(0xe000);
        enemy.x.advance(speed as i32);
        (enemy.x.whole as i16).wrapping_sub(576) >= 0
    });
    if impact {
        QueueSfx2_Max6(0x25, game);
        let mut runtime = Runtime::new(game);
        let palette = runtime
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .palette;
        runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.palette = palette
        });
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.instruction = addr_kCrocomire_Ilist_E158 as u16;
            enemy.instruction_delay = 1;
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
    }
}

pub unsafe fn Crocomire_Func_88(game: &mut Game) {
    let finished =
        Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            if (enemy.instruction as i16).wrapping_add(0x1e3a) < 0 {
                let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 4096, 6);
                enemy.variables[4] = speed as u16;
                enemy.pre_instruction = (speed >> 16) as u16;
                false
            } else {
                enemy.instruction = addr_kCrocomire_Ilist_E1CC as u16;
                enemy.instruction_delay = 1;
                enemy.x.whole = enemy.x.whole.wrapping_add(64);
                enemy.y.whole = enemy.y.whole.wrapping_add(21);
                enemy.radius.y = 28;
                enemy.radius.x = 40;
                true
            }
        });
    if finished {
        {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb753 as VoidP), game);
        }
        scatter(DropSource::Crocomire, 0, game);
        advance_phase(game);
    }
}

pub fn Crocomire_Func_90(game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.instruction as i16).wrapping_add(0x1e3a) >= 0 {
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        }
    });
}

pub unsafe fn Crocomire_9B65(game: &mut Game) {
    {
        let mut runtime = Runtime::new(game);
        runtime.write_slice_native(0xcd20, 257);
        runtime.write_slice_native(0xcd22, 257);
    }
    {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1e, 0x3, 0xb753 as VoidP), game);
    }
    advance_phase(game);
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
        word!(game.boss_id = 6);
        word_transfers!(game, ascending, 2048; le(tilemap_stuff, 0) <- value(824));
        if (gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game) & 2) != 0 {
            *(ram_ptr!(game, u16; 0xcd20)) = 257;
            *(ram_ptr!(game, u16; 0xcd20, 2)) = 257;
            word!(game.croco_target_0688 = 0);
            let E: i32 = 0;
            slot!(game, Crocomire[E].base.properties, |old| old & 0x7bff | 0x400);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x20, 0x3, 0xb753 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1e, 0x3, 0xb753 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x61, 0xb, 0xb747 as VoidP), game);
            slot!(game, Crocomire[E].var_A = 84);
            slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E1CC as u16, instruction_timer = 1);
            slot!(game, Crocomire[E].base.x_pos = 576);
            slot!(game, Crocomire[E].base; y_pos = 144, y_height = 28, x_width = 40);
            queue_tilemap_upload(2048, 0xfc, game);
        } else {
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
    @prefix { let E: i32 = 0; };
    Crocomire_8D47 [] {
        let crocom_var_D: u16 = slot!(game, Crocomire[E].var_D);
        if crocom_var_D != 0 {
            slot!(game, Crocomire[E].var_D = (crocom_var_D).wrapping_sub(1));
        } else {
            slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
            slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
            slot!(game, Crocomire[E].var_E = 768);
        };
    };
    Crocomire_Func_71 [] {
        let crocom_var_D: u16 = slot!(game, Crocomire[E].var_D);
        if crocom_var_D != 0 {
            slot!(game, Crocomire[E].var_D = (crocom_var_D).wrapping_sub(1));
            let mut v3: i16 = word!(game.croco_target_0688) as i16;
            let v4: i32 = word!(game.croco_target_0688) as i32 >> 1;
            if crate::rtl::rom_table_word!(game, 0xa499cb, v4 as isize) != 0xffff {
                let v5: u16 = word!(game.vram_write_queue_tail);
                let mut v6: *mut VramWriteEntry =
                    gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
                (*v6).size = 512;
                (*v6).src.addr = crate::rtl::rom_table_word!(game, 0xa499d9, v4 as isize);
                *(&raw mut (*v6).src.bank as *mut u16) = 173;
                (*v6).vram_dst = (crate::rtl::rom_table_word!(game, 0xa499cb, v4 as isize) as i32
                    + ((*ram_ptr!(game; 0x52) as i32 & 7) << 13)) as u16;
                word!(game.vram_write_queue_tail = (v5).wrapping_add(7));
                word!(game.croco_target_0688 = ((v3) as u16).wrapping_add(2));
            }
        } else {
            slot!(game, Crocomire[E].base.x_pos = 480);
            slot!(game, Crocomire[E].base.y_pos = 54);
            slot!(game, Crocomire[0x80i32].var_D = 80);
            slot!(game, Crocomire[E]; var_E = 0, var_F = 0);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x20, 0x3, 0xb753 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1e, 0x3, 0xb757 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x70, 0xb, 0xb747 as VoidP), game);
            QueueSfx2_Max6(0x29, game);
            slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E158 as u16, instruction_timer = 1);
            slot!(game, Crocomire[E].base.palette_index = 0);
            word_transfers!(game, descending, 16; le(palette_buffer, 159) <- rom(0xa4b8fd, 15));
            ClearEprojs(game);
            let mut v8: u16 = 8;
            for _ in 0..8 {
                v8 = SpawnEprojWithGfx(
                    v8,
                    word!(game.cur_enemy_index),
                    addr_kEproj_CrocomireSpikeWallPieces as u16,
                    game,
                );
            }
            QueueSfx2_Max6(0x30, game);
            advance_phase(game);
        };
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
