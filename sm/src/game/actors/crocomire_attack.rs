//! Crocomire attack reactions over the existing owned actor state.
#![forbid(unsafe_code)]
use super::access::{AccessMode, ActorKey as Key};
use crate::game::callback_tables::callback_table;
use crate::game::constants::{bank_86::addr_stru_868F8F, bank_a4::*};
use core::ffi::c_long;
use sm_game::enemy_projectiles::host::Host;
use sm_game::enemy_script::{EnemyScript, bank_base};

type Reaction<A> = fn(u16, EnemyScript, &mut A) -> EnemyScript;
const ATTACK_BOUNDARY: u16 = 0x300;
fn hit_only<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, 12, false>(k, jp, h)
}
fn ready<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    phase_jump(4, addr_kCrocomire_Ilist_BBCE as u16, h)
}
fn finish_attack<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    phase_jump(6, addr_kCrocomire_Ilist_BD2A as u16, h)
}

fn phase_jump<A: Host>(next: u16, destination: u16, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[2] = next;
    });
    EnemyScript::jump(destination)
}
fn react_passive<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, 20, true>(k, jp, h)
}
fn react_active<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, 12, true>(k, jp, h)
}

fn take_attack_hit(flags: &mut u16) -> bool {
    let pending = *flags & 0x800 != 0;
    *flags &= !0x800;
    pending
}

fn script_reached(script: &EnemyScript, bank: u8, boundary: u16) -> bool {
    ((script.offset_from(bank_base(bank)) as c_long - boundary as i32 as c_long) as i16) >= 0
}
fn react_to_attack_hit<A: Host, const NEXT: u16, const CHECK_CURSOR: bool>(
    _k: u16,
    jp: EnemyScript,
    h: &mut A,
) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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

fn await_attack_hit<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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

fn spit<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let mut shot = None;
    let next = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
        let selected = h.session().actors.controls.selected;
        crate::game::enemy_projectiles::lifecycle::spawn_from_enemy(
            h,
            index,
            selected,
            addr_stru_868F8F as u16,
        );
        h.queue_sound(3, 6, 0x1c);
    }
    next
}

fn retreat_counter<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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

fn await_boundary<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(ATTACK_BOUNDARY as i16) >= 0 {
            enemy.variables[2] = 6;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
        }
        jp
    })
}

fn expire_recoil<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[3] = enemy.variables[3].wrapping_sub(1);
        if (enemy.variables[3] as i16).wrapping_sub(2) < 0 {
            enemy.variables[3] = 0;
            enemy.variables[2] = 6;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16);
        }
        jp
    })
}

fn receive_strong_hit<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let hit = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
        h.queue_sound(2, 6, 0x54);
        jp
    } else {
        let key = Key::from_legacy(h.session().actors.controls.selected);
        h.edit_actor(key, AccessMode::Native, |enemy| enemy.variables[2] = 10);
        EnemyScript::jump(addr_kCrocomire_Ilist_BD8E as u16)
    }
}

fn backstep<A: Host>(phase: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let phase = phase >> 1;
    if phase == 15 {
        let hit = h
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .variables[1]
            & 0x100
            != 0;
        if !hit {
            let key = Key::from_legacy(h.session().actors.controls.selected);
            h.edit_actor(key, AccessMode::Native, |enemy| enemy.variables[2] = 20);
            h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
                enemy.variables[2] = 32
            });
            return EnemyScript::jump(addr_kCrocomire_Ilist_BAEA as u16);
        }
    }
    let next = restart_script(h.session().actors.controls.selected, jp, h);
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
fn choose_lunge<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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

fn lunge_counter<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    let finished = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
        h.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.variables[3] = 0
        });
        h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[1] |= 0x400
        });
        EnemyScript::jump(addr_kCrocomire_Ilist_BCD8 as u16)
    }
}

fn finish_lunge<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if enemy.variables[1] & 0x2000 == 0 {
            enemy.variables[1] &= 0xfcff;
        }
        enemy.variables[2] = 40;
    });
    EnemyScript::jump(addr_kCrocomire_Ilist_BBCE as u16)
}

fn lunge<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let mut sound = false;
    let next = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
        h.queue_sound(2, 6, 0x3b);
    }
    next
}

fn restart_script<A: Host>(k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(k), AccessMode::Native, |enemy| {
        enemy.instruction_delay = 1;
    });
    EnemyScript::jump(addr_kCrocomire_Ilist_BADE as u16)
}
fn detect_player<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let samus_x = h.session().player.x;
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if ((enemy.x.whole.wrapping_sub(samus_x) as i16).unsigned_abs() as i16).wrapping_sub(224)
            < 0
        {
            enemy.variables[1] |= 0x8000;
            enemy.variables[2] = 18;
            return EnemyScript::jump(addr_kCrocomire_Ilist_BC56 as u16);
        }
        jp
    })
}
pub(super) fn dispatch<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let key = Key::from_legacy(h.session().actors.controls.selected);
    let phase = h.resolve_actor(key, AccessMode::Native).variables[2];
    let reactions: [Reaction<A>; 21] = callback_table! { nonnull Reaction<A>;
        restart_script, ready, detect_player, await_attack_hit, spit,
        hit_only, retreat_counter, await_boundary, finish_attack, react_passive,
        react_active, react_active, expire_recoil, receive_strong_hit,
        backstep, backstep, backstep, choose_lunge,
        lunge_counter, finish_lunge, lunge,
    };
    reactions[(phase >> 1) as usize](phase, jp, h)
}
pub(super) fn choose_spit<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    if h.inputs().random & 0xfff < 1024 {
        h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[2] = 8;
            enemy.pre_instruction = 0;
        });
        return EnemyScript::jump(addr_kCrocomire_Ilist_BB36 as u16);
    }
    jp
}
