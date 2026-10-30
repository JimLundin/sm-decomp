//! Crocomire's fight AI: the instruction that runs its fight function, and the
//! functions, by index (bank $A4:86A6-$A4:8A10).
use crate::actors::access::{AccessMode, ActorKey as Key};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::constants::bank_86::ADDR_STRU_868F8F;
use crate::enemy_projectiles::host::Host;
use crate::enemy_script::{EnemyScript, bank_base};
use core::ffi::c_long;

/// Crocomire's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Crocomire {
    pub base: EnemyBase,
    /// The death sequence index, also the melting tiles' table subindex.
    pub death_sequence_index: u16,
    pub fight_flags: u16,
    pub fight_function: u16,
    /// The step and hop counter; in the death sequence its high byte is
    /// the Y acceleration.
    pub step_counter: u16,
    /// The death sequence's Y velocity.
    pub y_velocity: u16,
    /// The projectile counter; in the death sequence its high byte is
    /// the Y subposition.
    pub projectile_counter: u16,
    pad_0fb4: [u8; 26700],
    pub unread_7800: u16,
    pub unread_7802: u16,
    pad_7804: [u16; 29],
    pub melting_hdma_object: u16,
    pad_7840: [u8; 1984],
    pub acid_damage_sound_timer: u16,
    pad_8002: [u16; 31],
}

same_words!(Crocomire: EnemyLayout72 {
    death_sequence_index = var_a, fight_flags = var_b, fight_function = var_c,
    step_counter = var_d, y_velocity = var_e, projectile_counter = var_f, unread_7800 = var_00,
    unread_7802 = var_01, melting_hdma_object = var_1f, acid_damage_sound_timer = var_20,
});

/// The actor words Crocomire's fight uses, by `variables` lane.
pub mod lane {
    pub const DEATH_SEQUENCE_INDEX: usize = 0;
    pub const FIGHT_FLAGS: usize = 1;
    pub const FIGHT_FUNCTION: usize = 2;
    pub const STEP_COUNTER: usize = 3;
    pub const Y_VELOCITY: usize = 4;
}

/// `fight_flags` bits: the hit count in bits 0-3, damaged, Samus hit by the
/// claw and awake; the bank log leaves the bits only the unused sequence uses
/// unnamed.
pub mod flag {
    pub const HIT_COUNT: u16 = 0xf;
    pub const UNUSED_100: u16 = 0x100;
    pub const UNUSED_200: u16 = 0x200;
    pub const UNUSED_400: u16 = 0x400;
    pub const DAMAGED: u16 = 0x800;
    pub const UNUSED_2000: u16 = 0x2000;
    pub const SAMUS_HIT_BY_CLAW: u16 = 0x4000;
    pub const AWAKE: u16 = 0x8000;
}

/// Fight function indices, stored in `fight_function` ($A4:86B3 table).
pub mod fight {
    pub const STEP_FORWARD: u16 = 2;
    pub const ASLEEP: u16 = 4;
    pub const STEPPING_FORWARD: u16 = 6;
    pub const PROJECTILE_ATTACK: u16 = 8;
    pub const NEAR_SPIKE_WALL_CHARGE: u16 = 0xa;
    pub const STEPPING_BACK: u16 = 0xc;
    pub const BACK_OFF_FROM_SPIKE_WALL: u16 = 0xe;
    pub const WAIT_FOR_FIRST_DAMAGE: u16 = 0x12;
    pub const WAIT_FOR_SECOND_DAMAGE: u16 = 0x14;
    pub const UNUSED_SET_INITIAL_INSTRUCTION_LIST: u16 = 0x1c;
    pub const UNUSED_CHOOSE_FORWARD_ATTACK: u16 = 0x1e;
    pub const UNUSED_DO_NOTHING_AND_STEP_FORWARD: u16 = 0x20;
    pub const UNUSED_MOVE_UNTIL_CLAW_HITS_SAMUS: u16 = 0x22;
    pub const UNUSED_MOVE_CLAWS_AND_STEP_FORWARD: u16 = 0x24;
    pub const UNUSED_STEP_FORWARD: u16 = 0x26;
    pub const UNUSED_MOVING_CLAWS: u16 = 0x28;
    pub const UNUSED_LUNGE: u16 = 0x2a;
}

/// Bank $A4 Crocomire instruction lists.
pub mod instruction_list {
    use crate::constants::bank_a4::*;
    pub const INITIAL: u16 = ADDR_K_CROCOMIRE_ILIST_BADE as u16;
    pub const CHARGE_FORWARD_ONE_STEP: u16 = ADDR_K_CROCOMIRE_ILIST_BAEA as u16;
    pub const PROJECTILE_ATTACK: u16 = ADDR_K_CROCOMIRE_ILIST_BB36 as u16;
    pub const STEP_FORWARD_AFTER_DELAY: u16 = ADDR_K_CROCOMIRE_ILIST_BBCA as u16;
    pub const STEP_FORWARD: u16 = ADDR_K_CROCOMIRE_ILIST_BBCE as u16;
    pub const STEP_BACK: u16 = ADDR_K_CROCOMIRE_ILIST_BC30 as u16;
    pub const STEPPING_BACK: u16 = ADDR_K_CROCOMIRE_ILIST_BC34 as u16;
    pub const WAIT_FOR_DAMAGE: u16 = ADDR_K_CROCOMIRE_ILIST_BC56 as u16;
    pub const WAIT_FOR_DAMAGE_MOVING_CLAWS: u16 = ADDR_K_CROCOMIRE_ILIST_BCD8 as u16;
    pub const WAIT_FOR_DAMAGE_ROAR: u16 = ADDR_K_CROCOMIRE_ILIST_BD2A as u16;
    pub const WAIT_FOR_DAMAGE_ROAR_CLOSE_MOUTH: u16 = ADDR_K_CROCOMIRE_ILIST_BD8E as u16;
    pub const WAIT_FOR_DAMAGE_ROAR_END: u16 = ADDR_K_CROCOMIRE_ILIST_BDA2 as u16;
    pub const NEAR_SPIKE_WALL_CHARGE: u16 = ADDR_K_CROCOMIRE_ILIST_BE7E as u16;
    pub const BACK_OFF_FROM_SPIKE_WALL: u16 = ADDR_K_CROCOMIRE_ILIST_BF3C as u16;
}

use flag::*;
use instruction_list as list;
use lane::*;

type FightFunction<A> = fn(u16, EnemyScript, &mut A) -> EnemyScript;
/// Crocomire charges the spike wall from X 300h, and the unused lunge from 2A0h.
const SPIKE_WALL_X: u16 = 0x300;
const LUNGE_X: i16 = 0x2a0;
/// Asleep, Crocomire wakes when Samus is within E0h px.
const WAKE_RANGE: i16 = 0xe0;
/// The projectile attack fires 9 projectiles ($86:8F8F), with sound 1Ch.
const LAST_PROJECTILE: i16 = 18;
const PROJECTILE_SOUND: u16 = 0x1c;
/// The power bombed charge roars (sound 54h) and backs off for 0Ah frames; the
/// unused lunge snaps (sound 3Bh).
const ROAR_SOUND: u16 = 0x54;
const SNAP_SOUND: u16 = 0x3b;
const CHARGE_FRAMES: u16 = 10;
/// Keep the hit count and damage bits and set awake and an unused bit.
const CHARGE_KEEP: u16 = 0x1f00;
const CHARGE_SET: u16 = 0xa000;
/// The projectile attack starts at random draws below 400h of 1000h.
const PROJECTILE_CHANCE: u16 = 0x400;
const PROJECTILE_DRAW: u16 = 0xfff;

/// Fight AI - index Ah - near spike wall charge ($A4:87B2).
fn near_spike_wall_charge<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, { fight::STEPPING_BACK }, false>(k, jp, h)
}
/// Fight AI - index 2 - step forward ($A4:86E8).
fn step_forward<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    set_fight_function(fight::ASLEEP, list::STEP_FORWARD, h)
}
/// Fight AI - index 10h - roar and step forwards ($A4:87FB, unused).
fn roar_and_step_forwards<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    set_fight_function(fight::STEPPING_FORWARD, list::WAIT_FOR_DAMAGE_ROAR, h)
}

fn set_fight_function<A: Host>(next: u16, destination: u16, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[FIGHT_FUNCTION] = next;
    });
    EnemyScript::jump(destination)
}
/// Fight AI - index 12h - wait for first damage ($A4:8812).
fn wait_for_first_damage<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, { fight::WAIT_FOR_SECOND_DAMAGE }, true>(k, jp, h)
}
/// Fight AI - index 14h/16h - wait for second damage ($A4:8836, $A4:885A).
fn wait_for_second_damage<A: Host>(k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    react_to_attack_hit::<A, { fight::STEPPING_BACK }, true>(k, jp, h)
}

/// Whether Crocomire has been damaged since last asked, clearing the flag.
fn take_damage(flags: &mut u16) -> bool {
    let pending = *flags & DAMAGED != 0;
    *flags &= !DAMAGED;
    pending
}

fn script_reached(script: &EnemyScript, bank: u8, boundary: u16) -> bool {
    ((script.offset_from(bank_base(bank)) as c_long - boundary as i32 as c_long) as i16) >= 0
}
/// When damaged, step back into fight function `NEXT`; with `CHECK_CURSOR`,
/// roar again once the roar has finished.
fn react_to_attack_hit<A: Host, const NEXT: u16, const CHECK_CURSOR: bool>(
    _k: u16,
    jp: EnemyScript,
    h: &mut A,
) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_damage(&mut enemy.variables[FIGHT_FLAGS]) {
            enemy.variables[FIGHT_FUNCTION] = NEXT;
            return EnemyScript::jump(list::STEP_BACK);
        } else if CHECK_CURSOR && script_reached(&jp, enemy.bank, list::WAIT_FOR_DAMAGE_ROAR_END) {
            return EnemyScript::jump(list::WAIT_FOR_DAMAGE_ROAR);
        }
        jp
    })
}

/// Fight AI - index 6 - stepping forward ($A4:8717).
fn stepping_forward<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_damage(&mut enemy.variables[FIGHT_FLAGS]) && enemy.variables[STEP_COUNTER] != 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_BACK;
            EnemyScript::jump(list::STEP_BACK)
        } else if (enemy.x.whole as i16).wrapping_sub(SPIKE_WALL_X as i16) < 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::NEAR_SPIKE_WALL_CHARGE;
            EnemyScript::jump(list::NEAR_SPIKE_WALL_CHARGE)
        } else if script_reached(&jp, enemy.bank, list::STEPPING_BACK) {
            EnemyScript::jump(list::STEP_FORWARD)
        } else {
            jp
        }
    })
}

/// Fight AI - index 8 - projectile attack ($A4:876C).
fn projectile_attack<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let mut shot = None;
    let next = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if take_damage(&mut enemy.variables[FIGHT_FLAGS]) {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_BACK;
            return EnemyScript::jump(list::STEP_BACK);
        }
        let index = enemy.pre_instruction;
        if (index as i16).wrapping_sub(LAST_PROJECTILE) < 0 {
            enemy.pre_instruction = index.wrapping_add(2);
            shot = Some(index);
            jp
        } else {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_FORWARD;
            EnemyScript::jump(list::STEP_FORWARD_AFTER_DELAY)
        }
    });
    if let Some(index) = shot {
        let selected = h.session().actors.controls.selected;
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(
            h,
            index,
            selected,
            ADDR_STRU_868F8F as u16,
        );
        h.queue_sound(3, 6, PROJECTILE_SOUND);
    }
    next
}

/// Fight AI - index Ch - stepping back ($A4:87CA).
fn stepping_back<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[STEP_COUNTER] = enemy.variables[STEP_COUNTER].saturating_sub(1);
        if enemy.variables[STEP_COUNTER] != 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_BACK;
            EnemyScript::jump(list::STEPPING_BACK)
        } else {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_FORWARD;
            EnemyScript::jump(list::STEP_FORWARD)
        }
    })
}

/// Fight AI - index Eh - back off from spike wall ($A4:87E9).
fn back_off_from_spike_wall<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(SPIKE_WALL_X as i16) >= 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_FORWARD;
            return EnemyScript::jump(list::STEP_FORWARD);
        }
        jp
    })
}

/// Fight AI - index 18h - power bombed charge ($A4:887E).
fn power_bombed_charge<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[STEP_COUNTER] = enemy.variables[STEP_COUNTER].wrapping_sub(1);
        if (enemy.variables[STEP_COUNTER] as i16).wrapping_sub(2) < 0 {
            enemy.variables[STEP_COUNTER] = 0;
            enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_FORWARD;
            return EnemyScript::jump(list::STEP_FORWARD);
        }
        jp
    })
}

/// Fight AI - index 1Ah - do near spike wall charge unless damaged ($A4:889A,
/// unused).
fn charge_unless_damaged<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let hit = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        let flags = enemy.variables[FIGHT_FLAGS];
        if flags & DAMAGED == 0 {
            return false;
        }
        enemy.variables[FIGHT_FLAGS] = flags & CHARGE_KEEP | CHARGE_SET;
        enemy.variables[STEP_COUNTER] = 1;
        enemy.variables[Y_VELOCITY] = CHARGE_FRAMES;
        enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_BACK;
        true
    });
    if hit {
        h.queue_sound(2, 6, ROAR_SOUND);
        jp
    } else {
        let key = Key::from_legacy(h.session().actors.controls.selected);
        h.edit_actor(key, AccessMode::Native, |enemy| {
            enemy.variables[FIGHT_FUNCTION] = fight::NEAR_SPIKE_WALL_CHARGE
        });
        EnemyScript::jump(list::WAIT_FOR_DAMAGE_ROAR_CLOSE_MOUTH)
    }
}

/// Fight AI - index 1Ch/1Eh/20h - the unused sequence: set the initial
/// instruction list, choose a forward attack, or step forward.
fn unused_sequence<A: Host>(function: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let step = function >> 1;
    if step == fight::UNUSED_CHOOSE_FORWARD_ATTACK >> 1 {
        let hit = h
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .variables[FIGHT_FLAGS]
            & UNUSED_100
            != 0;
        if !hit {
            let key = Key::from_legacy(h.session().actors.controls.selected);
            h.edit_actor(key, AccessMode::Native, |enemy| {
                enemy.variables[FIGHT_FUNCTION] = fight::WAIT_FOR_SECOND_DAMAGE
            });
            h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
                enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_DO_NOTHING_AND_STEP_FORWARD
            });
            return EnemyScript::jump(list::CHARGE_FORWARD_ONE_STEP);
        }
    }
    let next = lock_up(h.session().actors.controls.selected, jp, h);
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if step == fight::UNUSED_DO_NOTHING_AND_STEP_FORWARD >> 1 {
            if enemy.variables[STEP_COUNTER] == 0 {
                enemy.variables[FIGHT_FLAGS] |= UNUSED_2000;
                enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_MOVE_CLAWS_AND_STEP_FORWARD;
                return EnemyScript::jump(list::STEP_FORWARD);
            }
        } else {
            let first = step == fight::UNUSED_SET_INITIAL_INSTRUCTION_LIST >> 1;
            if first {
                enemy.variables[FIGHT_FLAGS] |= UNUSED_200;
            }
            enemy.variables[STEP_COUNTER] = if first { 32 } else { 16 };
            enemy.variables[FIGHT_FUNCTION] = if first {
                fight::UNUSED_CHOOSE_FORWARD_ATTACK
            } else {
                fight::UNUSED_MOVE_UNTIL_CLAW_HITS_SAMUS
            };
        }
        next
    })
}
/// Fight AI - index 22h - unused sequence - move forward until hit Samus with
/// claw ($A4:895E).
fn move_until_claw_hits_samus<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(LUNGE_X) < 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_MOVE_CLAWS_AND_STEP_FORWARD;
            enemy.variables[STEP_COUNTER] = 3;
            return EnemyScript::jump(list::STEP_FORWARD);
        }
        if enemy.variables[FIGHT_FLAGS] & SAMUS_HIT_BY_CLAW == 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_STEP_FORWARD;
            enemy.variables[FIGHT_FLAGS] &= !UNUSED_400;
        } else {
            enemy.variables[STEP_COUNTER] = 5;
            enemy.pre_instruction = enemy.variables[FIGHT_FUNCTION];
            enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_LUNGE;
        }
        EnemyScript::jump(list::WAIT_FOR_DAMAGE_MOVING_CLAWS)
    })
}

/// Fight AI - index 24h - unused sequence - move claws and step forward ($A4:89A8).
fn move_claws_and_step_forward<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    let finished = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[STEP_COUNTER] = enemy.variables[STEP_COUNTER].saturating_sub(1);
        if enemy.variables[STEP_COUNTER] == 0 {
            enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_MOVING_CLAWS;
            enemy.variables[FIGHT_FLAGS] &= !UNUSED_400;
            true
        } else {
            enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_MOVE_CLAWS_AND_STEP_FORWARD;
            false
        }
    });
    if finished {
        EnemyScript::jump(list::STEP_FORWARD)
    } else {
        h.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.variables[STEP_COUNTER] = 0
        });
        h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[FIGHT_FLAGS] |= UNUSED_400
        });
        EnemyScript::jump(list::WAIT_FOR_DAMAGE_MOVING_CLAWS)
    }
}

/// Fight AI - index 26h - unused sequence - step forward ($A4:89DE).
fn unused_step_forward<A: Host>(_k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if enemy.variables[FIGHT_FLAGS] & UNUSED_2000 == 0 {
            enemy.variables[FIGHT_FLAGS] &= !(UNUSED_100 | UNUSED_200);
        }
        enemy.variables[FIGHT_FUNCTION] = fight::UNUSED_MOVING_CLAWS;
    });
    EnemyScript::jump(list::STEP_FORWARD)
}

/// Fight AI - index 28h - unused sequence - moving claws ($A4:89F9).
fn moving_claws<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let mut sound = false;
    let next = h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if enemy.variables[STEP_COUNTER] == 0 {
            enemy.variables[FIGHT_FLAGS] &= !SAMUS_HIT_BY_CLAW;
            enemy.instruction_delay = 1;
            enemy.variables[FIGHT_FUNCTION] = enemy.pre_instruction;
            return EnemyScript::jump(list::WAIT_FOR_DAMAGE_MOVING_CLAWS);
        }
        if enemy.variables[FIGHT_FLAGS] & SAMUS_HIT_BY_CLAW != 0 {
            enemy.variables[STEP_COUNTER] = enemy.variables[STEP_COUNTER].wrapping_sub(1);
            sound = true;
            return EnemyScript::jump(list::WAIT_FOR_DAMAGE_MOVING_CLAWS);
        }
        enemy.variables[FIGHT_FUNCTION] = fight::STEPPING_BACK;
        jp
    });
    if sound {
        h.queue_sound(2, 6, SNAP_SOUND);
    }
    next
}

/// Fight AI - index 0 - lock up / set initial Crocomire instruction list ($A4:86DE).
fn lock_up<A: Host>(k: u16, _jp: EnemyScript, h: &mut A) -> EnemyScript {
    h.edit_actor(Key::from_legacy(k), AccessMode::Native, |enemy| {
        enemy.instruction_delay = 1;
    });
    EnemyScript::jump(list::INITIAL)
}
/// Fight AI - index 4 - asleep ($A4:86F2): wake once Samus comes near.
fn asleep<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let samus_x = h.session().player.x;
    h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if ((enemy.x.whole.wrapping_sub(samus_x) as i16).unsigned_abs() as i16)
            .wrapping_sub(WAKE_RANGE)
            < 0
        {
            enemy.variables[FIGHT_FLAGS] |= AWAKE;
            enemy.variables[FIGHT_FUNCTION] = fight::WAIT_FOR_FIRST_DAMAGE;
            return EnemyScript::jump(list::WAIT_FOR_DAMAGE);
        }
        jp
    })
}
/// Instruction - fight AI ($A4:86A6): run the fight function. Was `dispatch`.
pub fn fight_ai<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    let key = Key::from_legacy(h.session().actors.controls.selected);
    let function = h.resolve_actor(key, AccessMode::Native).variables[FIGHT_FUNCTION];
    let functions: [FightFunction<A>; 21] = [
        lock_up as FightFunction<A>,
        step_forward as FightFunction<A>,
        asleep as FightFunction<A>,
        stepping_forward as FightFunction<A>,
        projectile_attack as FightFunction<A>,
        near_spike_wall_charge as FightFunction<A>,
        stepping_back as FightFunction<A>,
        back_off_from_spike_wall as FightFunction<A>,
        roar_and_step_forwards as FightFunction<A>,
        wait_for_first_damage as FightFunction<A>,
        wait_for_second_damage as FightFunction<A>,
        wait_for_second_damage as FightFunction<A>,
        power_bombed_charge as FightFunction<A>,
        charge_unless_damaged as FightFunction<A>,
        unused_sequence as FightFunction<A>,
        unused_sequence as FightFunction<A>,
        unused_sequence as FightFunction<A>,
        move_until_claw_hits_samus as FightFunction<A>,
        move_claws_and_step_forward as FightFunction<A>,
        unused_step_forward as FightFunction<A>,
        moving_claws as FightFunction<A>,
    ];
    functions[(function >> 1) as usize](function, jp, h)
}
/// Instruction - maybe start projectile attack ($A4:8752). Was `choose_spit`.
pub fn maybe_start_projectile_attack<A: Host>(_k: u16, jp: EnemyScript, h: &mut A) -> EnemyScript {
    if h.inputs().random & PROJECTILE_DRAW < PROJECTILE_CHANCE {
        h.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[FIGHT_FUNCTION] = fight::PROJECTILE_ATTACK;
            enemy.pre_instruction = 0;
        });
        return EnemyScript::jump(list::PROJECTILE_ATTACK);
    }
    jp
}
