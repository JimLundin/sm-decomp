use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::ram_map::slot;

pub use crate::game::constants::bank_86::*;

use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::music::QueueSfx2_Max6;

pub use crate::types::Consts_A8;

crate::game::constants::typed_constants! { Consts_A8;
    addr_kNorfairLavaMan_Ilist_AE0C = 44556, addr_kNorfairLavaMan_Ilist_ADDC = 44508,
    addr_kNorfairLavaMan_Ilist_ADAC = 44460, addr_kNorfairLavaMan_Ilist_AD7E = 44414,
    addr_kNorfairLavaMan_Ilist_AD50 = 44368, addr_kNorfairLavaMan_Ilist_AD3C = 44348,
    addr_kNorfairLavaMan_Ilist_AD0C = 44300, addr_kNorfairLavaMan_Ilist_ACDE = 44254,
    addr_kNorfairLavaMan_Ilist_ACB0 = 44208, addr_kNorfairLavaMan_Ilist_AC9C = 44188,
}
crate::type_aliases! {
    pub Enemy_NorfairLavaMan = crate::game::enemy_layouts::EnemyLayout40;
}

const RomPtr_A8: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa8>;
crate::game::constants::typed_constants! { i32;
    fnnullsub_170_A8 = 0xa8804c,
    fnNorfairLavaMan_Func_6 = 0xa8b0b2,
    fnNorfairLavaMan_Func_7 = 0xa8b11a, fnNorfairLavaMan_Func_8 = 0xa8b175,
    fnsub_A8B193 = 0xa8b193, fnNorfairLavaMan_Func_9 = 0xa8b1b8,
    fnNorfairLavaMan_Func_10 = 0xa8b1dd, fnNorfairLavaMan_Func_11 = 0xa8b204,
    fnsub_A8B291 = 0xa8b291, fnNorfairLavaMan_Func_13 = 0xa8b295,
    fnNorfairLavaMan_Func_15 = 0xa8b30d, fnNorfairLavaMan_Func_16 = 0xa8b31f,
    fnNorfairLavaMan_Func_17 = 0xa8b356, fnNorfairLavaMan_Func_18 = 0xa8b3a7,
}
crate::ram_map::ram_pointer_functions! {
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
}
type NorfairLavaMan = Enemy_NorfairLavaMan;
static g_off_A8AF4F: [Func_V; 3] = callback_table! { nonnull Func_V;
    NorfairLavaMan_Func_1, NorfairLavaMan_Func_2, NorfairLavaMan_Func_3,
};
pub unsafe fn NorfairLavaMan_Func_5(k: u16, game: &mut Game) {
    unsafe {
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = (0xa8b0b2i32) as u16;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 168;
        word!(
            game.variables_for_enemy_graphics_drawn_hook[0] =
                ((((slot!(game, NorfairLavaMan[k as i32].base.palette_index)) & (0xe00u16)) as i32
                    >> 4)
                    + 256) as u16
        );
        word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
        word!(game.variables_for_enemy_graphics_drawn_hook[1] = 0);
    }
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    {
        if word!(game.door_transition_flag_enemies) == 0 && {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] -= 1, reread) == 0
        } {
            word!(game.variables_for_enemy_graphics_drawn_hook[2] = 8);
            word!(game.variables_for_enemy_graphics_drawn_hook[1] += 1);
            let v0: i32 = (32 * (word!(game.variables_for_enemy_graphics_drawn_hook[1]) as i32 & 3))
                as u16 as i32
                >> 1;
            let r18: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 9) as isize);
            let r20: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 10) as isize);
            let r22: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 11) as isize);
            let r24: u16 = crate::rtl::rom_table_word!(game, 0xa8ac1c, (v0 + 12) as isize);
            let v1: i32 = word!(game.variables_for_enemy_graphics_drawn_hook[0]) as i32 >> 1;
            word! {game; palette_buffer[(v1 + 9)] = r18; palette_buffer[(v1 + 10)] = r20;
            palette_buffer[(v1 + 11)] = r22; palette_buffer[(v1 + 12)] = r24; }
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_12(k: u16, game: &mut Game) {
    unsafe {
        let mut v3: i32 = 0;
        let E: i32 = k as i32;
        if -(slot!(game, NorfairLavaMan[E].var_E) as i32) - 108i32 & 0x8000 == 0 {
            printf(
                c"Y undefined %d!\n".as_ptr(),
                -(slot!(game, NorfairLavaMan[E].var_E) as i16 as i32),
            );
        }
        let nlmn_var_B: u16 = slot!(game, NorfairLavaMan[E].var_B);
        let advance_lava_phase: bool =
            if -(slot!(game, NorfairLavaMan[E].var_E) as i32) - 108i32 & 0x8000 == 0 || {
                v3 = nlmn_var_B as i32 >> 1;
                ((slot!(game, NorfairLavaMan[E].base.y_pos) as i32
                    - crate::rtl::rom_table_word!(game, 0xa8af79, v3 as isize) as i32
                    - word!(game.samus_y_pos) as i32) as i16)
                    < 0
            } {
                slot!(game, NorfairLavaMan[E].var_F = (0xa8b291i32) as u16);
                slot!(game, NorfairLavaMan[E].var_0C = 1);
                if ((-(slot!(game, NorfairLavaMan[E].var_E) as i32)
                    - crate::rtl::rom_table_word!(game, 0xa8af55, (nlmn_var_B >> 1) as isize)
                        as i32) as i16)
                    < 0
                {
                    return;
                }
                true
            } else {
                ((-(slot!(game, NorfairLavaMan[E].var_E) as i32)
                    - crate::rtl::rom_table_word!(game, 0xa8af55, v3 as isize) as i32)
                    as i16)
                    >= 0
            };
        if advance_lava_phase {
            slot!(game, NorfairLavaMan[E].var_B, |old| old.wrapping_add(1));
            slot!(game, NorfairLavaMan[E].var_B, |old| old.wrapping_add(1));
            slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old)
                .wrapping_add(8));
            slot!(
                game,
                NorfairLavaMan[E].var_D = crate::rtl::rom_table_word!(
                    game,
                    0xa8af67,
                    (slot!(game, NorfairLavaMan[E].var_B) >> 1) as isize
                )
            );
            NorfairLavaMan_Func_20(game);
        };
    }
}
pub fn sub_A8B291(k: u16) {}
pub unsafe fn NorfairLavaMan_Func_14(k: u16, game: &mut Game) {
    unsafe {
        let E: i32 = k as i32;
        let mut nlmn_var_E: i16 = slot!(game, NorfairLavaMan[E].var_E) as i16;
        if nlmn_var_E >= 0 {
            slot!(game, NorfairLavaMan[E].var_0D = 1);
            (*gEnemySpawnData(k as i32, game).offset(31)).field_4 = 0;
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b1ddi32) as u16);
        } else if ((-(nlmn_var_E as i32)
            - crate::rtl::rom_table_word!(
                game,
                0xa8af55,
                (((slot!(game, NorfairLavaMan[E].var_B)).wrapping_sub(2)) >> 1) as isize
            ) as i32) as i16)
            < 0
        {
            slot!(game, NorfairLavaMan[E].var_B, |old| (old).wrapping_sub(2));
            slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old)
                .wrapping_sub(8));
            slot!(
                game,
                NorfairLavaMan[E].var_D = crate::rtl::rom_table_word!(
                    game,
                    0xa8af67,
                    (slot!(game, NorfairLavaMan[E].var_B) >> 1) as isize
                )
            );
            NorfairLavaMan_Func_20(game);
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_19(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (word!(game.enemy_drawing_queue[(v1 + 61)]) as i32
        - slot!(game, NorfairLavaMan[k as i32].base.y_pos) as i32
        + 2) as u16;
    word!(game.enemy_drawing_queue[(v1 + 64)] = v2);
    if v2 as i32 - 8i32 & 0x8000 != 0 {
        word!(game.enemy_drawing_queue[(v1 + 64)] = 8);
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11054987] NorfairLavaMan_Init [] {
        let v0: u16 = (2u16).wrapping_mul(
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].parameter_1),
        );
        g_off_A8AF4F[(v0 >> 1) as usize](game);
        NorfairLavaMan_Func_4(game);
        NorfairLavaMan_Func_5(word!(game.cur_enemy_index), game);
    };
    [11056128] NorfairLavaMan_Powerbomb [] {
        Enemy_NormalPowerBombAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056134] NorfairLavaMan_Touch [] {
        Enemy_NormalTouchAI_A8(game);
        NorfairLavaMan_Common(game);
    };
    [11056140] NorfairLavaMan_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        NorfairLavaMan_Common(game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    NorfairLavaMan_Func_1 [] {
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_00 = 0, var_02 = 0);
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
            slot!(game, NorfairLavaMan[E].var_00 = 1);
        }
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
        if slot!(game, NorfairLavaMan[E].var_00) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
        }
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b11ai32) as u16);
    };
    NorfairLavaMan_Func_2 [] {
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_0C = 0);
        slot!(game, NorfairLavaMan[E].var_0D = 1);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ADDC as u16);
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old).wrapping_add(32));
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b1ddi32) as u16);
        slot!(game, NorfairLavaMan[E].base.properties, |old| (old) | ((kEnemyProps_Invisible) as u16));
    };
    NorfairLavaMan_Func_3 [] {
        slot!(game, NorfairLavaMan[E].var_03 = slot!(game, NorfairLavaMan[E].base.y_pos));
        slot!(game, NorfairLavaMan[E].var_11 = slot!(game, NorfairLavaMan[E].base.x_pos));
        slot!(game, NorfairLavaMan[E]; var_C = 0, var_04 = 0);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16);
        NorfairLavaMan_Func_20(game);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old).wrapping_add(32));
        slot!(game, NorfairLavaMan[E].var_F = (0xa8b30di32) as u16);
        slot!(game, NorfairLavaMan[E].base.properties, |old| (old) | ((kEnemyProps_Invisible) as u16));
    };
    NorfairLavaMan_Func_4 [] {
        let v1: i32 = 8 * slot!(byte game, NorfairLavaMan[E].parameter_2[1]) as i32 >> 1;
        slot!(game, NorfairLavaMan[E].var_08 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize));
        slot!(game, NorfairLavaMan[E].var_07 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize));
        slot!(game, NorfairLavaMan[E].var_0A = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize));
        slot!(game, NorfairLavaMan[E].var_09 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize));
    };
    [11055370] NorfairLavaMan_Main [] {
        slot!(game, NorfairLavaMan[E].var_04, |old| old.wrapping_sub(1));
        actor_species_pre_instruction(((slot!(game, NorfairLavaMan[E].var_F)) as u32) | (0xa80000u32), game);
    };
    NorfairLavaMan_Func_13 [] {
        let mut v2 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, slot!(game, NorfairLavaMan[E].base.y_subpos)).1;
        slot!(game, NorfairLavaMan[E].base.y_subpos = ((*v2.offset(31)).field_E).wrapping_add(slot!(game, NorfairLavaMan[E].base.y_subpos)));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16);
        slot!(game, NorfairLavaMan[E].var_E, |old| (old as i32 + ((*v2.offset(31)).xpos2 as i32 + v4 as i32)) as u16);
        NorfairLavaMan_Func_14(word!(game.cur_enemy_index), game);
    };
    NorfairLavaMan_Func_20 [] {
        let nlmn_var_D: u16 = slot!(game, NorfairLavaMan[E].var_D);
        if nlmn_var_D != slot!(game, NorfairLavaMan[E].var_C) {
            slot!(game, NorfairLavaMan[E].base.current_instruction = nlmn_var_D);
            slot!(game, NorfairLavaMan[E].var_C = nlmn_var_D);
            slot!(game, NorfairLavaMan[E].base.instruction_timer = 1);
            slot!(game, NorfairLavaMan[E].base.timer = 0);
        }
    };
    NorfairLavaMan_Common [] {
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        if slot!(game, NorfairLavaMan[E].base.health) == 0 {
            slot!(game, NorfairLavaMan[E1].base.properties, |old| (old) | (0x200u16));
            slot!(game, NorfairLavaMan[E2].base.properties, |old| (old) | (0x200u16));
        }
        let frozen_timer: u16 = slot!(game, NorfairLavaMan[E].base.frozen_timer);
        if frozen_timer != 0 {
            slot!(game, NorfairLavaMan[E1].base.frozen_timer = frozen_timer);
            slot!(game, NorfairLavaMan[E1].base.ai_handler_bits, |old| (old) | (4u16));
            slot!(game, NorfairLavaMan[E2].base.frozen_timer = frozen_timer);
            slot!(game, NorfairLavaMan[E2].base.ai_handler_bits, |old| (old) | (4u16));
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11042826] Enemy_GrappleReact_KillEnemy_A8 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [11042831] Enemy_GrappleReact_CancelBeam_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042846] Enemy_GrappleReact_HurtSamus_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042851] Enemy_NormalTouchAI_A8 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [11042861] Enemy_NormalShotAI_A8 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    Enemy_NormalPowerBombAI_A8 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [11042881] Enemy_NormalFrozenAI_A8 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11054610] NorfairLavaMan_Instr_1 [jp.offset(1)] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) == 0 {
            QueueSfx2_Max6(jp.word(0), game);
        }
    };
    [11054630] NorfairLavaMan_Instr_8 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
    };
    [11054640] NorfairLavaMan_Instr_14 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
    };
    [11054650] NorfairLavaMan_Instr_2 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_01 = 1);
    };
    [11054661] NorfairLavaMan_Instr_7 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_01 = 0);
    };
    [11054672] NorfairLavaMan_Instr_10 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_sub(1));
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos, |old| old.wrapping_sub(1));
    };
    [11054682] NorfairLavaMan_Instr_12 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| old.wrapping_add(1));
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos, |old| old.wrapping_add(1));
    };
    [11054692] NorfairLavaMan_Instr_9 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v4: u16 = (slot!(game, NorfairLavaMan[E].var_03)).wrapping_add(24);
        slot!(game, NorfairLavaMan[E].base.y_pos = v4);
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos = v4);
        slot!(game, NorfairLavaMan[E1].base.properties, |old| old & !(0x100));
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        slot!(game, NorfairLavaMan[E2].base.properties, |old| old & !(0x100));
    };
    [11054728] NorfairLavaMan_Instr_11 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let nlmn_var_03: u16 = slot!(game, NorfairLavaMan[E].var_03);
        slot!(game, NorfairLavaMan[E].base.y_pos = nlmn_var_03);
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32 + 64].base.y_pos = nlmn_var_03)

;
    };
    [11054742] NorfairLavaMan_Instr_13 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let v4: u16 = (slot!(game, NorfairLavaMan[E].var_03)).wrapping_add(4);
        slot!(game, NorfairLavaMan[E].base.y_pos = v4);
        let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
        slot!(game, NorfairLavaMan[E1].base.y_pos = v4);
        slot!(game, NorfairLavaMan[E1].base.properties, |old| (old) | (0x100u16));
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        slot!(game, NorfairLavaMan[E2].base.properties, |old| (old) | (0x100u16));
    };
    [11054778] NorfairLavaMan_Instr_5 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        SpawnEprojWithGfx(
            slot!(game, NorfairLavaMan[E].var_B),
            word!(game.cur_enemy_index),
            addr_kEproj_LavaThrownByLavaman as u16,
            game,
        );
    };
    [11054794] NorfairLavaMan_Instr_15 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_add(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054820] NorfairLavaMan_Instr_4 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_sub(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054846] NorfairLavaMan_Instr_16 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_add(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(8));
    };
    [11054872] NorfairLavaMan_Instr_6 [jp] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, NorfairLavaMan[E].base.x_pos = (slot!(game, NorfairLavaMan[E].var_12)).wrapping_sub(8));
        slot!(game, NorfairLavaMan[E].base.y_pos = (slot!(game, NorfairLavaMan[E].var_13)).wrapping_sub(4));
    };
    [11054916] NorfairLavaMan_Instr_3 [jp] {
        slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_04 = 256);
    };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11055885] NorfairLavaMan_Func_15 [] {
        if *(ram_ptr!(game, u16; 0xe84))
            .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 87) as isize) as i32
            == 0xa8b175 & 0xffff
        {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_F = (0xa8b356i32) as u16)
;
        }
    };
    [11055958] NorfairLavaMan_Func_17 [] {
        if (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 != 0 {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ACB0 as u16);
            slot!(game, NorfairLavaMan[E].var_B = 0);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD50 as u16);
                slot!(game, NorfairLavaMan[E].var_B = 1);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b31fi32) as u16);
            slot!(game, NorfairLavaMan[E].var_12 = slot!(game, NorfairLavaMan[E].base.x_pos));
            slot!(game, NorfairLavaMan[E].var_13 = slot!(game, NorfairLavaMan[E].base.y_pos));
        } else {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].base.y_pos = (*(ram_ptr!(game, u16; 0xe84))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1i32) + 93i32) as isize))
            .wrapping_sub(crate::rtl::rom_table_word!(
                game,
                0xa8af79,
                (*(ram_ptr!(game, u16; 0xf68))
                    .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 1) as isize)
                    >> 1) as isize,
            )))

;
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    [11056039] NorfairLavaMan_Func_18 [] {
        let v1: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 87)]) as i32 == 0xa8b11a & 0xffff {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].var_F = (0xa8b30di32) as u16)
;
        } else {
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32].base.y_pos = (word!(game.enemy_drawing_queue[(v1 + 93)])).wrapping_sub(
                crate::rtl::rom_table_word!(
                    game,
                    0xa8af79,
                    (word!(game.enemy_drawing_queue_sizes[(v1 + 1)]) >> 1) as isize,
                ),
            ))

;
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    [11055386] NorfairLavaMan_Func_7 [] {
        slot!(game, NorfairLavaMan[E].var_00 = 0);
        slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
        if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
            slot!(game, NorfairLavaMan[E].var_00 = 1);
        }
        NorfairLavaMan_Func_20(game);
        let E2: i32 = word!(game.cur_enemy_index) as i32 + 128;
        if (slot!(game, NorfairLavaMan[E2].var_04) & 0x8000) != 0 {
            slot!(game, NorfairLavaMan[E2].var_04 = 0);
            if ({ let (actor, distance) = (word!(game.cur_enemy_index), slot!(byte game, NorfairLavaMan[E].parameter_2[0]) as u16); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ACDE as u16);
                if slot!(game, NorfairLavaMan[E].var_00) != 0 {
                    slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD7E as u16);
                }
                NorfairLavaMan_Func_20(game);
                slot!(game, NorfairLavaMan[E].var_F = (0xa8b175i32) as u16);
            }
        }
    };
    [11055477] NorfairLavaMan_Func_8 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_02 = 1);
            slot!(game, NorfairLavaMan[word!(game.cur_enemy_index) as i32 + 64i32].var_0D = 0);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b193i32) as u16);
        }
    };
    [11055507] sub_A8B193 [] {
        if slot!(game, NorfairLavaMan[E].var_02) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD0C as u16);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_ADAC as u16);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b1b8i32) as u16);
        }
    };
    [11055544] NorfairLavaMan_Func_9 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AC9C as u16);
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AD3C as u16);
            }
            NorfairLavaMan_Func_20(game);
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b11ai32) as u16);
        }
    };
    [11055581] NorfairLavaMan_Func_10 [] {
        if slot!(game, NorfairLavaMan[E].var_0D) == 0 {
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b204i32) as u16);
            slot!(game, NorfairLavaMan[E]; var_E = 0, var_0D = 0);
            slot!(game, NorfairLavaMan[E].var_B = 2);
            slot!(game, NorfairLavaMan[E].base.y_pos = slot!(game, NorfairLavaMan[E].var_03));
        }
    };
    [11055620] NorfairLavaMan_Func_11 [] {
        let v3 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        let v: u32 = (((*v3.offset(31)).ypos2) as u32).wrapping_add((slot!(game, NorfairLavaMan[E].base.y_subpos)) as u32);
        slot!(game, NorfairLavaMan[E].base.y_subpos = v as u16);
        slot!(game, NorfairLavaMan[E].base.y_pos, |old| (old as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16);
        slot!(game, NorfairLavaMan[E].var_E, |old| (old as u32)
            .wrapping_add(((*v3.offset(31)).field_14 as u32).wrapping_add(v >> 16))
            as u16);
        NorfairLavaMan_Func_12(word!(game.cur_enemy_index), game);
    };
    [11055903] NorfairLavaMan_Func_16 [] {
        if slot!(game, NorfairLavaMan[E].var_01) == 0 {
            slot!(game, NorfairLavaMan[E].var_D = addr_kNorfairLavaMan_Ilist_AE0C as u16);
            NorfairLavaMan_Func_20(game);
            *(ram_ptr!(game, u16; 0xf68))
                .offset(((word!(game.cur_enemy_index) as i32 >> 1) + 5) as isize) =
                (0xa8b295i32) as u16;
            slot!(game, NorfairLavaMan[E].var_F = (0xa8b3a7i32) as u16);
            (*gEnemySpawnData(word!(game.cur_enemy_index) as i32, game).offset(31)).field_18 = 0;
            slot!(game, NorfairLavaMan[E].base.x_pos = slot!(game, NorfairLavaMan[E].var_12));
            slot!(game, NorfairLavaMan[E].base.y_pos = slot!(game, NorfairLavaMan[E].var_13));
        }
        NorfairLavaMan_Func_19(word!(game.cur_enemy_index), game);
    };
}
