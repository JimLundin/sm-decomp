//! Crocomire attack-script reactions retain native actor words and captured script images.
//! Graphics and the death driver remain in the bank; RAM remains authoritative.
#![forbid(unsafe_code)]
use super::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::game::callback_tables::callback_table;
use crate::game::constants::{bank_86::addr_stru_868F8F, bank_a4::*};
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::enemy_script::{EnemyScript, bank_base};
use crate::game::music::{QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::rtl::abs16;
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
