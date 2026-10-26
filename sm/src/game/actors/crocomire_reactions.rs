//! Crocomire attack-script reactions retain native actor words and captured script images.
//! Graphics and the death driver remain in the bank; RAM remains authoritative.
#![forbid(unsafe_code)]
use super::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::game::callback_tables::callback_table;
use crate::game::constants::{
    bank_86::{addr_kEproj_DustCloudExplosion, addr_stru_868F8F},
    bank_a4::*,
    enemy_properties::kEnemyProps_Tangible,
};
use crate::game::enemy_layouts::EnemyLayout72 as Crocomire;
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::enemy_script::{EnemyScript, bank_base};
use crate::game::music::{QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::session_runtime::Runtime;
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::abs16;
use crate::types::{Point16U, SpawnHardcodedPlmArgs, VoidP};
use core::ffi::c_long;

type Reaction = fn(u16, EnemyScript, &mut Game) -> EnemyScript;
const ATTACK_BOUNDARY: u16 = 0x300;
const HIT_ONLY: Reaction = react_to_attack_hit::<12, false>;
const READY: Reaction = |_, _jp, game| phase_jump(4, addr_kCrocomire_Ilist_BBCE as u16, game);
const FINISH_ATTACK: Reaction =
    |_, _jp, game| phase_jump(6, addr_kCrocomire_Ilist_BD2A as u16, game);

fn phase_jump(next: u16, destination: u16, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[2] = next;
    });
    EnemyScript::jump(destination)
}
const REACT_PASSIVE: Reaction = react_to_attack_hit::<20, true>;
const REACT_ACTIVE: Reaction = react_to_attack_hit::<12, true>;

fn take_attack_hit(flags: &mut u16) -> bool {
    let pending = *flags & 0x800 != 0;
    *flags &= !0x800;
    pending
}

fn script_reached(script: &EnemyScript, bank: u8, boundary: u16) -> bool {
    ((script.offset_from(bank_base(bank)) as c_long - boundary as i32 as c_long) as i16) >= 0
}
fn react_to_attack_hit<const NEXT: u16, const CHECK_CURSOR: bool>(
    _k: u16,
    jp: EnemyScript,
    game: &mut Game,
) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_attack_hit(&mut enemy.variables[1]) {
            enemy.variables[2] = NEXT;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BC30 as u16);
        } else if CHECK_CURSOR && script_reached(&jp, enemy.bank, addr_kCrocomire_Ilist_BDA2 as u16)
        {
            return EnemyScript::jump(addr_kCrocomire_Ilist_BD2A as u16);
        }
        jp
    })
}

static REACTIONS: [Reaction; 21] = callback_table! { nonnull Reaction;
    restart_script, READY, detect_player, await_attack_hit, spit,
    HIT_ONLY, retreat_counter, await_boundary, FINISH_ATTACK, REACT_PASSIVE,
    REACT_ACTIVE, REACT_ACTIVE, expire_recoil, receive_strong_hit,
    backstep, backstep, backstep, choose_lunge,
    lunge_counter, finish_lunge, lunge,
};
fn await_attack_hit(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_attack_hit(&mut enemy.variables[1]) && enemy.variables[3] != 0 {
            enemy.variables[2] = 12;
            EnemyScript::jump(addr_kCrocomire_Ilist_BC30 as u16)
        } else if (enemy.x.whole as i16).wrapping_sub(ATTACK_BOUNDARY as i16) < 0 {
            enemy.variables[2] = 10;
            EnemyScript::jump(addr_kCrocomire_Ilist_BE7E as u16)
        } else if script_reached(&jp, enemy.bank, addr_kCrocomire_Ilist_BC34 as u16) {
            EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16)
        } else {
            jp
        }
    })
}

fn spit(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut shot = None;
    let next = Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_attack_hit(&mut enemy.variables[1]) {
            enemy.variables[2] = 12;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BC30 as u16);
        }
        let index = enemy.pre_instruction;
        if (index as i16).wrapping_sub(18) < 0 {
            enemy.pre_instruction = index.wrapping_add(2);
            shot = Some(index);
            jp
        } else {
            enemy.variables[2] = 6;
            EnemyScript::jump(addr_kCrocomire_Ilist_BBCA as u16)
        }
    });
    if let Some(index) = shot {
        SpawnEprojWithGfx(index, game.cur_enemy_index(), addr_stru_868F8F as u16, game);
        QueueSfx3_Max6(0x1c, game);
    }
    next
}

fn retreat_counter(_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[3] = enemy.variables[3].saturating_sub(1);
        if enemy.variables[3] != 0 {
            enemy.variables[2] = 12;
            EnemyScript::jump(addr_kCrocomire_Ilist_BC34 as u16)
        } else {
            enemy.variables[2] = 6;
            EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16)
        }
    })
}

fn await_boundary(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(ATTACK_BOUNDARY as i16) >= 0 {
            enemy.variables[2] = 6;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
        }
        jp
    })
}

fn expire_recoil(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[3] = enemy.variables[3].wrapping_sub(1);
        if (enemy.variables[3] as i16).wrapping_sub(2) < 0 {
            enemy.variables[3] = 0;
            enemy.variables[2] = 6;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
        }
        jp
    })
}

fn receive_strong_hit(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let hit = Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        let flags = enemy.variables[1];
        if flags & 0x800 == 0 {
            return false;
        }
        enemy.variables[1] = flags & 0x1f00 | 0xa000;
        enemy.variables[3] = 1;
        enemy.variables[4] = 10;
        enemy.variables[2] = 12;
        true
    });
    if hit {
        QueueSfx2_Max6(0x54, game);
        jp
    } else {
        let key = Key::from_legacy(game.cur_enemy_index());
        Runtime::new(game).edit_actor(key, AccessMode::Native, |enemy| enemy.variables[2] = 10);
        EnemyScript::jump(addr_kCrocomire_Ilist_BD8E as u16)
    }
}

fn backstep(phase: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let phase = phase >> 1;
    if phase == 15 {
        let hit = Runtime::new(game)
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .variables[1]
            & 0x100
            != 0;
        if !hit {
            let key = Key::from_legacy(game.cur_enemy_index());
            let mut runtime = Runtime::new(game);
            runtime.edit_actor(key, AccessMode::Native, |enemy| enemy.variables[2] = 20);
            runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
                enemy.variables[2] = 32
            });
            return EnemyScript::jump(addr_kCrocomire_Ilist_BAEA as u16);
        }
    }
    let next = restart_script(game.cur_enemy_index(), jp, game);
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if phase == 16 {
            if enemy.variables[3] == 0 {
                enemy.variables[1] |= 0x2000;
                enemy.variables[2] = 36;
                return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
            }
        } else {
            if phase == 14 {
                enemy.variables[1] |= 0x200;
            }
            enemy.variables[3] = if phase == 14 { 32 } else { 16 };
            enemy.variables[2] = if phase == 14 { 30 } else { 34 };
        }
        next
    })
}
fn choose_lunge(_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(672) < 0 {
            enemy.variables[2] = 36;
            enemy.variables[3] = 3;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
        }
        if enemy.variables[1] & 0x4000 == 0 {
            enemy.variables[2] = 38;
            enemy.variables[1] &= !0x400;
        } else {
            enemy.variables[3] = 5;
            enemy.pre_instruction = enemy.variables[2];
            enemy.variables[2] = 42;
        }
        EnemyScript::jump(addr_kCrocomire_Ilist_BCD8 as u16)
    })
}

fn lunge_counter(_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut runtime = Runtime::new(game);
    let finished = runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[3] = enemy.variables[3].saturating_sub(1);
        if enemy.variables[3] == 0 {
            enemy.variables[2] = 40;
            enemy.variables[1] &= !0x400;
            true
        } else {
            enemy.variables[2] = 36;
            false
        }
    });
    if finished {
        EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16)
    } else {
        runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.variables[3] = 0
        });
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[1] |= 0x400
        });
        EnemyScript::jump(addr_kCrocomire_Ilist_BCD8 as u16)
    }
}

fn finish_lunge(_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if enemy.variables[1] & 0x2000 == 0 {
            enemy.variables[1] &= 0xfcff;
        }
        enemy.variables[2] = 40;
    });
    EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16)
}

fn lunge(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let mut sound = false;
    let next = Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if enemy.variables[3] == 0 {
            enemy.variables[1] &= !0x4000;
            enemy.instruction_delay = 1;
            enemy.variables[2] = enemy.pre_instruction;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BCD8 as u16);
        }
        if enemy.variables[1] & 0x4000 != 0 {
            enemy.variables[3] = enemy.variables[3].wrapping_sub(1);
            sound = true;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BCD8 as u16);
        }
        enemy.variables[2] = 12;
        jp
    });
    if sound {
        QueueSfx2_Max6(0x3b, game);
    }
    next
}

fn restart_script(k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript {
    Runtime::new(game).edit_actor(Key::from_legacy(k), AccessMode::Native, |enemy| {
        enemy.instruction_delay = 1;
    });
    EnemyScript::jump(addr_kCrocomire_Ilist_BADE as u16)
}
fn detect_player(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let samus_x = game.samus_x_pos();
    Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (abs16(enemy.x.whole.wrapping_sub(samus_x)) as i16).wrapping_sub(224) < 0 {
            enemy.variables[1] |= 0x8000;
            enemy.variables[2] = 18;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BC56 as u16);
        }
        jp
    })
}
pub(crate) fn dispatch(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let key = Key::from_legacy(game.cur_enemy_index());
    let phase = Runtime::new(game)
        .resolve_actor(key, AccessMode::Native)
        .variables[2];
    REACTIONS[(phase >> 1) as usize](phase, jp, game)
}
pub(crate) fn choose_spit(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    if game.random_number() & 0xfff < 1024 {
        Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[2] = 8;
            enemy.pre_instruction = 0;
        });
        return EnemyScript::jump(addr_kCrocomire_Ilist_BB36 as u16);
    }
    jp
}

fn cap_byte(value: u8, limit: u8) -> u8 {
    if value.wrapping_sub(limit) as i8 >= 0 {
        limit
    } else {
        value
    }
}
pub(crate) fn fill_scroll(game: &mut Game) {
    let scroll = word!(game.reg_BG2VOFS);
    word_transfers!(game, descending, 256; le(crocomire_bg2_scroll_hdma_data, 255) <- value(scroll));
}
pub(crate) fn melt_down(game: &mut Game) {
    fill_scroll(game);
    slot!(game, Crocomire[0].var_B, |old| old & !0x800);
    update_background(word!(game.cur_enemy_index), game);
    let key = word!(game.cur_enemy_index) as i32;
    if slot!(game, Crocomire[key].base.y_pos) as i32 - 280 & 0x8000 != 0 {
        let mut cursor = (8 * (word!(game.reg_BG2VOFS) as i32 + 288 & 0xfff8)) as u16;
        for _ in 0..32 {
            word!(game.tilemap_stuff[cursor >> 1] = 824);
            cursor = cursor.wrapping_add(2);
        }
        word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
        slot!(game, Crocomire[0].base.extra_properties, |old| old
            & !0x8000);
        let low = slot!(game, Crocomire[key].var_D) as u8;
        slot!(byte game, Crocomire[key].var_D[0] = low.wrapping_add(0x80));
        let next = (low.overflowing_add(0x80).1 as u8)
            .wrapping_add(slot!(byte game, Crocomire[key].var_D[1]))
            .wrapping_add(3);
        let high = cap_byte(next, 48);
        slot!(byte game, Crocomire[key].var_D[1] = high);
        let carry = slot!(byte game, Crocomire[key].var_E[0])
            .overflowing_add(high)
            .1;
        slot!(byte game, Crocomire[key].var_E[0] = match slot!(byte game, Crocomire[key].var_E[0]) { old => old.wrapping_add(high) });
        let high = cap_byte(
            (carry as u8).wrapping_add(slot!(byte game, Crocomire[key].var_E[1])),
            3,
        );
        slot!(byte game, Crocomire[key].var_E[1] = high);
        let velocity = slot!(game, Crocomire[key].var_E) as u8;
        let carry = slot!(byte game, Crocomire[key].var_F[1])
            .overflowing_add(velocity)
            .1;
        slot!(byte game, Crocomire[key].var_F[1] = match slot!(byte game, Crocomire[key].var_F[1]) { old => old.wrapping_add(velocity) });
        let (step, carry) =
            (carry as u8).overflowing_add(slot!(byte game, Crocomire[key].var_E[1]));
        let carry = carry
            | slot!(byte game, Crocomire[key].base.y_pos[0])
                .overflowing_add(step)
                .1;
        slot!(byte game, Crocomire[key].base.y_pos[0] = match slot!(byte game, Crocomire[key].base.y_pos[0]) { old => old.wrapping_add(step) });
        slot!(byte game, Crocomire[key].base.y_pos[1] = match slot!(byte game, Crocomire[key].base.y_pos[1]) { old => old.wrapping_add(carry as u8) });
    } else {
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[0].var_D = 48);
    }
}
pub(crate) fn melt_up(game: &mut Game) {
    fill_scroll(game);
    let key = word!(game.cur_enemy_index) as i32;
    if slot!(game, Crocomire[key].base.y_pos) as i32 - 218 & 0x8000 != 0 {
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
    } else {
        update_background(word!(game.cur_enemy_index), game);
        let mut acceleration = (slot!(game, Crocomire[0].var_D) as i16).wrapping_add(256);
        if slot!(game, Crocomire[0].var_D) as i32 - 7680 & 0x8000 == 0 {
            acceleration = 7936;
        }
        slot!(game, Crocomire[0].var_D = acceleration as u16);
        let low = slot!(game, Crocomire[0].var_E) as u8;
        let borrow = low < slot!(byte game, Crocomire[0].var_D[1]);
        slot!(byte game, Crocomire[0].var_E[0] = low.wrapping_sub(slot!(byte game, Crocomire[0].var_D[1])));
        let mut high = (slot!(byte game, Crocomire[0].var_E[1]) as i8).wrapping_sub(borrow as i8);
        if high < 0 {
            slot!(byte game, Crocomire[0].var_E[0] = 0xff);
            high = 0;
        }
        slot!(byte game, Crocomire[0].var_E[1] = high as u8);
        let fraction = slot!(byte game, Crocomire[0].var_F[1]);
        let borrow = fraction < slot!(byte game, Crocomire[0].var_E[0]);
        slot!(byte game, Crocomire[0].var_F[1] = fraction.wrapping_sub(slot!(byte game, Crocomire[0].var_E[0])));
        let y = slot!(game, Crocomire[0].base.y_pos) as u8;
        let step = (borrow as u8).wrapping_add(slot!(byte game, Crocomire[0].var_E[1]));
        slot!(byte game, Crocomire[0].base.y_pos[0] = y.wrapping_sub(step));
        slot!(byte game, Crocomire[0].base.y_pos[1] = match slot!(byte game, Crocomire[0].base.y_pos[1]) { old => old.wrapping_sub((y < step) as u8) });
    }
}
pub(crate) fn update_background(k: u16, game: &mut Game) {
    word!(game.reg_BG2VOFS = 67u16.wrapping_sub(slot!(game, Crocomire[k as i32].base.y_pos)));
    let mut index = 32u16;
    let spritemap = loop {
        let candidate = slot!(game, Crocomire[0].base.spritemap_pointer);
        if candidate == crate::rtl::rom_table_word!(game, 0xa48b79, (index >> 1) as isize) {
            break candidate;
        }
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            crate::game::visual_runtime::align_crocomire_background(game);
            return;
        }
    };
    word!(
        game.reg_BG2VOFS += {
            let address = 0xa40000 | u32::from(spritemap);
            if spritemap & 0x8000 == 0 {
                crate::game::session_runtime::native_rom_diagnostic(address, game);
            }
            game.capture_native_rom()
                .table_word(crate::rom::lorom_offset(address) as usize, 14)
        }
    );
    crate::game::visual_runtime::align_crocomire_background(game);
}
pub(crate) fn update_palette(game: &mut Game) {
    if word!(game.door_transition_flag_enemies) == 0 {
        if slot!(game, Crocomire[0].base.flash_timer) != 0
            && (word!(game.random_enemy_counter) & 2) != 0
        {
            word_transfers!(game, descending, 8; native(0xc0ee) <- value(0x7fff));
        } else {
            word_transfers!(game, descending, 8; native(0xc0ee) <- rom(0xa4b89d, 7));
        }
    }
}
fn wall_counter(runtime: &Runtime<'_>, key: u16) -> u16 {
    runtime
        .resolve_actor(Key::from_legacy(key), AccessMode::Native)
        .variables[3]
}
fn set_wall_counter(runtime: &mut Runtime<'_>, key: u16, value: u16) {
    runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
        enemy.variables[3] = value
    });
}
pub(crate) fn wall_step(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let mut cursor = wall_counter(&runtime, 0);
    let image = runtime.loaded_image().expect("ROM must be loaded");
    let target = image.table_word(
        crate::rom::lorom_offset(0xa498ca) as usize,
        (cursor >> 1) as isize,
    );
    if target == 0x8080 {
        set_wall_counter(&mut runtime, 0x40, 0x8080);
        set_wall_counter(&mut runtime, 0, 128);
        drop(runtime);
        word_transfers!(game, descending, 16; le(palette_buffer, 191) <- rom(0xa4b91d, 15));
        Runtime::new(game).edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
        return;
    }
    let current = wall_counter(&runtime, 0x40);
    if current != target {
        let step = wall_counter(&runtime, 0xc0);
        let next = if (current as i16).wrapping_sub(target as i16) >= 0 {
            current.wrapping_sub(step)
        } else {
            current.wrapping_add(step)
        };
        set_wall_counter(&mut runtime, 0x40, next);
        return;
    }
    if target & 0x8000 != 0 {
        let countdown = wall_counter(&runtime, 0x80);
        if countdown != 0 {
            set_wall_counter(&mut runtime, 0x80, countdown.wrapping_sub(1));
            set_wall_counter(&mut runtime, 0, cursor.wrapping_sub(2));
            drop(runtime);
            QueueSfx2_Max6(0x2b, game);
            return;
        }
        cursor = cursor.wrapping_add(2);
        let countdown = image.table_word(
            crate::rom::lorom_offset(0xa498ca) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0x80, countdown);
        cursor = cursor.wrapping_add(2);
        let step = image.table_word(
            crate::rom::lorom_offset(0xa498ca) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0xc0, step);
    }
    set_wall_counter(&mut runtime, 0, cursor.wrapping_add(2));
}
pub(crate) fn rumble(game: &mut Game) {
    let remaining = slot!(game, Crocomire[0].var_20);
    if remaining != 0 {
        slot!(game, Crocomire[0].var_20 = remaining - 1);
        if remaining == 1 {
            slot!(game, Crocomire[0].var_20 = 32);
            QueueSfx3_Max6(0x22, game);
        }
    }
}
fn contact_edge(game: &mut Game) -> i32 {
    slot!(game, Crocomire[0].base.x_pos) as i32
        - slot!(game, Crocomire[0].base.x_width) as i32
        - word!(game.samus_x_radius) as i32
}
fn contact(game: &mut Game) {
    super::combat::contact_reaction(&mut Runtime::new(game), super::api::DeathHandling::Finalize);
}
pub(crate) fn touch(game: &mut Game) {
    contact(game);
    slot!(game, Crocomire[0].var_B, |flags| flags | 0x4000);
    word!(game.extra_samus_x_displacement = -4i32 as u16);
}
pub(crate) fn push_player(game: &mut Game) {
    if slot!(game, Crocomire[0].var_A) == 0
        && ((contact_edge(game) - word!(game.samus_x_pos) as i32) as i16) < 0
    {
        contact(game);
        word!(game.samus_x_pos = contact_edge(game) as u16);
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        word!(game.extra_samus_x_displacement = -4i32 as u16);
        word!(game.extra_samus_y_displacement = -1i32 as u16);
    }
}
fn next_hit_count(flags: u16) -> u16 {
    ((flags & 0xf) + 1).min(15)
}
pub(crate) fn count_hit(game: &mut Game) {
    let count = next_hit_count(slot!(game, Crocomire[0].var_B));
    slot!(game, Crocomire[0].var_B, |flags| flags | count);
}
pub(crate) fn impact(game: &mut Game) {
    let index = word!(game.collision_detection_index) as i32;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.projectile_x_pos[index]),
        y: word!(game.projectile_y_pos[index]),
    };
    let appearance = if word!(game.projectile_type[index]) & 0x200 == 0 {
        6
    } else {
        29
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, appearance, game);
}
pub(crate) fn hit(game: &mut Game) {
    slot!(game, Crocomire[0].base.invincibility_timer = 0);
    let mut advance = ((slot!(game, Crocomire[0].base.x_pos) as i32
        - slot!(game, Crocomire[0].base.x_width) as i32
        - 256
        - word!(game.layer1_x_pos) as i32) as i16)
        >= 0;
    if !advance {
        let projectile = word!(game.projectile_type[word!(game.collision_detection_index)]);
        if projectile & 0xf10 == 0 {
            slot!(game, Crocomire[0].base.instruction_timer = 8);
            impact(game);
            return;
        }
        let recoil = [2u16, 1, 3]
            .get(((projectile & 0xf00) >> 8) as usize)
            .copied()
            .unwrap_or(0);
        advance = recoil != 0;
        if advance {
            slot!(game, Crocomire[0].var_D, |old| old.wrapping_add(recoil));
        }
    }
    if advance {
        let count = next_hit_count(slot!(game, Crocomire[0].var_B));
        if slot!(game, Crocomire[0].var_B) & 0x800 == 0 {
            let mut delay = 8;
            if slot!(game, Crocomire[0].var_C) == 8 {
                delay = 8;
            }
            slot!(game, Crocomire[0].base.instruction_timer, |old| old
                .wrapping_add(delay));
        }
        slot!(
            game,
            Crocomire[0].var_B = count | slot!(game, Crocomire[0].var_B) & 0xb7f0 | 0x800
        );
        slot!(game, Crocomire[0].var_E = 10);
    }
    slot!(game, Crocomire[0].base.flash_timer, |old| old
        .wrapping_add(14));
    slot!(game, Crocomire[0].base.ai_handler_bits, |old| old | 2);
}

pub(crate) fn select_melt_animation<const SKELETON: bool>(key: Key, game: &mut Game) {
    Runtime::new(game).edit_actor(key, AccessMode::Native, |enemy| {
        let instruction = match enemy.y.whole as i16 {
            y if y.wrapping_sub(280) >= 0 => addr_kCrocomire_Ilist_BF64 as u16,
            y if y.wrapping_sub(264) >= 0 => addr_kCrocomire_Ilist_BF6C as u16,
            y if y.wrapping_sub(248) >= 0 => addr_kCrocomire_Ilist_BF72 as u16,
            _ => addr_kCrocomire_Ilist_BF78 as u16,
        };
        // Each skeleton list starts 0x1a bytes after its corresponding flesh list.
        enemy.instruction = instruction + if SKELETON { 0x1a } else { 0 };
        enemy.instruction_delay = 1;
    });
}

pub(crate) fn melt_skeleton_up(game: &mut Game) {
    select_melt_animation::<true>(Key::from_legacy(game.cur_enemy_index()), game);
    emit_lava_plume(game);
    melt_up(game);
}

pub(crate) fn melt_flesh_down(game: &mut Game) {
    emit_lava_plume(game);
    select_melt_animation::<false>(Key::from_legacy(0), game);
    melt_down(game);
}

pub(crate) fn emit_lava_plume(game: &mut Game) {
    if word!(game.g_word_7E9018 -= 1, reread) == 0 {
        word!(game.g_word_7E9018 = 6);
        let mut v0: u16 = (word!(game.random_number)) & (0x3fu16);
        if (word!(game.random_number) & 2) == 0 {
            v0 = !(v0 as i32) as u16;
        }
        let x: u16 = (slot!(game, Crocomire[0i32].base.x_pos)).wrapping_add(v0);
        let y: u16 = (word!(game.lava_acid_y_pos) as i32 + 16
            - (((word!(game.random_number)) & (0x1f00u16)) as i32 >> 8))
            as u16;
        create_sprite(x, y, 21, 0, game);
    }
}

pub(crate) fn melt_rumble_down(game: &mut Game) {
    rumble(game);
    emit_lava_plume(game);
    melt_down(game);
}

pub(crate) fn melt_flesh_up(game: &mut Game) {
    emit_lava_plume(game);
    select_melt_animation::<false>(Key::from_legacy(word!(game.cur_enemy_index)), game);
    melt_up(game);
}

pub(crate) fn melt_rumble_up(game: &mut Game) {
    rumble(game);
    emit_lava_plume(game);
    melt_up(game);
}

pub(crate) fn queue_tilemap_upload(size: u16, bg_mask: u8, game: &mut Game) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    // The native bank word overlaps the first destination byte, including on BE.
    word_transfers!(game, ascending, 1;
        native(entry) <- value(size),
        native(entry + 2) <- value(0x2000),
        native(entry + 4) <- value(126),
    );
    word_transfers!(game, ascending, 1;
        native(entry + 5) <- value(u16::from(game.g_ram[0x59] & bg_mask) << 8),
    );
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

fn load_melt_tilemap(table: u32, game: &mut Game) {
    let mut i: i32 = 0;
    loop {
        let index: i32 = i >> 1;
        if crate::rtl::rom_table_word!(game, table, index as isize) == 0xffff {
            break;
        }
        word!(
            game.tilemap_stuff[(index + 32)] =
                crate::rtl::rom_table_word!(game, table, index as isize)
        );
        i += 2;
    }
    queue_tilemap_upload((i + 1024) as u16, 0xfc, game);
}

pub(crate) fn prepare_flesh_tilemap(game: &mut Game) {
    game.g_ram[0x68c] = 48;
    word!(game.croco_target_0688 = 48);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_BF64 as u16, instruction_timer = 1);
    word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
    let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
    slot!(game, Crocomire[E1].base; current_instruction = addr_kCrocomire_Ilist_BF98 as u16, instruction_timer = 1);
    slot!(game, Crocomire[E1].base.properties, |old| old & 0xd2ff
        | 0x2c00);
    slot!(
        game,
        Crocomire[E1].base.x_pos = slot!(game, Crocomire[E].base.x_pos)
    );
    slot!(
        game,
        Crocomire[E1].base.y_pos = (slot!(game, Crocomire[E].base.y_pos)).wrapping_add(16)
    );
    word_transfers!(game, (4), 256;
        le(tilemap_stuff, 0) <- value(824),
        le(tilemap_stuff, 1) <- value(824),
    );
    load_melt_tilemap(0xa49c79, game);
}

pub(crate) fn prepare_skeleton_tilemap(game: &mut Game) {
    fill_scroll(game);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].base.instruction_timer = 1);
    game.g_ram[0x68c] = 48;
    word!(game.croco_target_0688 = 48);
    slot!(
        game,
        Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_BF7E as u16
    );
    word_transfers!(game, ascending, 1024; le(tilemap_stuff, 0) <- value(824));
    load_melt_tilemap(0xa49e7b, game);
}

macro_rules! native_graphics_word {
    ($game:ident, $address:expr, $offset:expr) => {{
        let address = $address;
        if address & 0x8000 == 0 {
            crate::game::session_runtime::native_rom_diagnostic(address, $game);
        }
        $game.capture_native_rom().table_word(
            crate::rtl::lorom_offset(address) as usize + $offset as usize,
            0,
        )
    }};
}

pub(crate) fn queue_melt_graphics(offset: u16, live_size: bool, game: &mut Game) {
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    for (source, destination) in [(0, 0), (6, 2), (4, 4), (2, 5)] {
        let value = native_graphics_word!(game, 0xa49bc5 + source, {
            if source == 0 && live_size {
                word!(game.g_word_7E068A)
            } else {
                offset
            }
        });
        word_transfers!(game, ascending, 1; native(entry + destination) <- value(value));
    }
}

pub(crate) fn advance_melt_upload(game: &mut Game) {
    let offset = word!(game.g_word_7E068A);
    let tail = word!(game.vram_write_queue_tail);
    if native_graphics_word!(game, 0xa49bc5, word!(game.g_word_7E068A)) == 0xffff {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        word!(game.g_word_7E068A = 0);
    } else {
        queue_melt_graphics(offset, true, game);
        word!(game.g_word_7E068A = offset.wrapping_add(8));
        word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    };
}

pub(crate) fn prepare_melt_graphics(game: &mut Game) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| (old).wrapping_add(2));
    word! { game;
        g_word_7E0692 = 256;
        croco_cur_vline_idx = 0;
        g_word_7E0698 = native_graphics_word!(game, 0xa49bc5, word!(game.croco_word_7E069A));
        g_word_7E0694 = word!(game.g_word_7E0698);
        g_word_7E0696 = native_graphics_word!(game, 0xa49bc7, word!(game.croco_word_7E069A));
        g_word_7E068E = native_graphics_word!(game, 0xa49bc9, word!(game.croco_word_7E069A));
    }
    let bank: u8 = native_graphics_word!(game, 0xa49bcb, word!(game.croco_word_7E069A)) as u8;
    let mut i: i32 = word!(game.croco_word_7E069A) as i32 + 8;
    loop {
        let mut v2: u16 = native_graphics_word!(game, 0xa49bc5, i);
        if v2 == 0xffff {
            break;
        }
        let mut v3: u16 = native_graphics_word!(game, 0xa49bc7, i);
        for _ in 0..=word!(game.g_word_7E068E) {
            word_transfers!(game, ascending, 1; native(v3) <- value(native_graphics_word!(game, (u32::from(bank) << 16) | u32::from(v2), 0)));
            v3 = (v3).wrapping_add(2);
            v2 = (v2).wrapping_add(2);
        }
        i = i as u16 as i32 + 4;
    }
    word! {game; croco_word_7E069A = (i + 2) as u16; g_word_7E068A = (i + 2) as u16; }
    word_transfers!(game, descending, 65; native(0x71c) <- value(0));
}

pub(crate) fn power_bomb(game: &mut Game) {
    let E: i32 = 0;
    if slot!(game, Crocomire[E].var_A) == 0 {
        slot!(game, Crocomire[E].var_D = 3);
        if slot!(game, Crocomire[E].var_C) != 24 {
            slot!(game, Crocomire[E].var_B, |old| old & 0x3ff0 | 0x8000);
            slot!(game, Crocomire[E].var_E = 10);
            slot!(game, Crocomire[E].base.flash_timer, |old| old
                .wrapping_add(4));
            slot!(game, Crocomire[E].base.ai_handler_bits, |old| old | 2u16);
            slot!(game, Crocomire[E].var_C = 24);
            let mut spritemap_pointer: u16 = slot!(game, Crocomire[E].base.spritemap_pointer);
            let mut n: i32 =
                native_graphics_word!(game, 0xa40000 | u32::from(spritemap_pointer), 0) as i32;
            let instruction = loop {
                let tag = native_graphics_word!(game, 0xa40000 | u32::from(spritemap_pointer), 6);
                match u32::from(tag) {
                    addr_kCrocomire_BigSprmap_D600 => break addr_kCrocomire_Ilist_BDAE as u16,
                    addr_kCrocomire_BigSprmap_D51C => break addr_kCrocomire_Ilist_BDB2 as u16,
                    _ => {}
                }
                spritemap_pointer = (spritemap_pointer).wrapping_add(8);
                n -= 1;
                if n == 0 {
                    break addr_kCrocomire_Ilist_BDB6 as u16;
                }
            };
            slot!(game, Crocomire[E].base; current_instruction = instruction, instruction_timer = 1);
        }
    }
}

pub(crate) fn approach_wall(game: &mut Game) {
    update_wall(game);
    word_transfers!(game, ascending, 1; native(0xcd24) <- value(257));
    if word!(game.samus_x_pos) as i32 - 1312i32 & 0x8000 == 0 {
        word_transfers!(game, ascending, 1; native(0xcd24) <- value(256));
    }
    update_background(word!(game.cur_enemy_index), game);
}

fn update_wall(game: &mut Game) {
    let E0: i32 = 0;
    if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1536i32 & 0x8000 != 0 {
        word! { game, 0; kraid_unk9000, g_word_7E9002, g_word_7E9006, g_word_7E900A }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1552i32 & 0x8000 != 0 {
        if word!(game.kraid_unk9000) == 0 {
            word!(game.kraid_unk9000 = 1);
            game.eproj_spawn_pt = Point16U { x: 1536, y: 176 };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
        }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1552i32 & 0x8000 != 0 {
        word! { game, 0; g_word_7E9002, g_word_7E9006, g_word_7E900A }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1568i32 & 0x8000 != 0 {
        if word!(game.g_word_7E9002) == 0 {
            word!(game.g_word_7E9002 = 1);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x61, 0xb, 0xb74b as VoidP), game);
            game.eproj_spawn_pt = Point16U { x: 1568, y: 176 };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
        }
    } else {
        let x_pos: u16 = slot!(game, Crocomire[E0].base.x_pos);
        if x_pos as i32 - 1568i32 & 0x8000 != 0 {
            word! { game, 0; g_word_7E9006, g_word_7E900A }
        } else if x_pos as i32 - 1584i32 & 0x8000 != 0 {
            if word!(game.g_word_7E9006) == 0 {
                word!(game.g_word_7E9006 = 1);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x62, 0xb, 0xb74b as VoidP), game);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x63, 0xb, 0xb74b as VoidP), game);
                game.eproj_spawn_pt = Point16U { x: 1584, y: 176 };
                SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
            }
        } else if ((x_pos) as i16).wrapping_sub((1600u16) as i16) >= 0 {
            demolish_wall(game);
            slot!(game, Crocomire[E0].var_20 = 1);
            word!(game.g_word_7E9018 = 1);
            let _v3: u16 = word!(game.cur_enemy_index);
            let EK: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, Crocomire[EK].var_A, |old| (old).wrapping_add(2));
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].var_00 = 2);
            }
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].var_01 = 0);
            }
            QueueSfx2_Max6(0x3b, game);
            word_transfers!(game, ascending, 1; native(0x9016) <- value(0));
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x4e, 0x3, 0xb757 as VoidP), game);
            slot!(game, Crocomire[EK].base; current_instruction = addr_kCrocomire_Ilist_BFB0 as u16, instruction_timer = 1);
            slot!(game, Crocomire[EK].base.properties, |old| (old)
                | ((kEnemyProps_Tangible) as u16));
            let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
            slot!(game, Crocomire[E1].base; instruction_timer = 0x7fff, current_instruction = addr_kCrocomire_Ilist_BF62 as u16);
            slot!(
                game,
                Crocomire[0x40i32].base.properties =
                    slot!(game, Crocomire[0x40i32].base.properties) | (0x100u16)
            );

            slot!(game, Crocomire[EK]; var_E = 0, var_F = 0);
            slot!(game, Crocomire[EK].var_D = 2048);
            slot!(game, Crocomire[0i32].base.y_height = 16);
        }
    };
}

fn demolish_wall(game: &mut Game) {
    for x in 0x61..=0x6a {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(x, 0xb, 0xb74f as VoidP), game);
    }
    for (offset, y) in [176, 192, 176, 192, 192, 192, 192].into_iter().enumerate() {
        game.eproj_spawn_pt = Point16U {
            x: 1536 + 16 * offset as u16,
            y,
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
    }
}
