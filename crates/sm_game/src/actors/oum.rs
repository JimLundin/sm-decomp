//! Oum, Maridia's rolling snail, which bounces into place, rolls and attacks, and carries
//! or holds back Samus as she rides or pushes it (bank $A2:CA2B-$A2:D80B). Was `large_snail`.
use crate::actors::access::ActorKey;
use crate::actors::api::{
    self, Axis, DeathHandling, MovementResult::Blocked, ShotHandling, SlopePolicy,
};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::actors::species::{PreInstructionHost, SpeciesHost, pre_instruction};
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use crate::ram::words;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable::Linear;

part_view! {
    Oum: EnemyLayout40 {
        words: [function, y_speed_index, attack_cooldown, direction, bounces, bounce_function, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [new_list_index, list_index, animation_finished, attack_rotates, touching, samus_left, previous_x, var_07,
            previous_y, var_09, var_0a, stop, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14,
            var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
    }
}

const IDLE: u16 = 0xcde6;
const ROLLING: u16 = 0xce2b;
const ATTACKING: u16 = 0xcf40;
const FALLING: u16 = 0xcf66;
const RISING: u16 = 0xcfa9;
/// $A2:CB77: the instruction lists by index: bit 0 facing left, bit 1 rolling forwards,
/// bit 2 attacking.
const LISTS: u32 = 0xa2_cb77;
const ATTACK_COOLDOWN: u16 = 0x80;
const BOUNCES: u16 = 3;

fn left(runtime: &impl ActorLease) -> bool {
    api::player_delta(runtime, combat::current(runtime), Axis::X) & 0x8000 != 0
}
fn near(runtime: &mut impl ActorLease, axis: Axis, distance: u16) -> bool {
    let key = combat::current(runtime);
    api::player_within(runtime, key, axis, distance)
}
/// Set oum instruction list ($A2:CFDF), if its index changed.
fn set_instruction_list(runtime: &mut impl ActorLease, oum: i32) {
    let index = session_slot!(runtime, Oum[oum].new_list_index);
    if index != session_slot!(runtime, Oum[oum].list_index) {
        session_slot!(runtime, Oum[oum].list_index = index);
        let list = table(runtime, LISTS, index as isize);
        session_slot!(runtime, Oum[oum].base; current_instruction = list, instruction_timer = 1, timer = 0);
    }
}
/// Initialisation AI - oum ($A2:CCD4).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let oum = current_slot(runtime);
    session_slot!(runtime, Oum[oum]; y_speed_index = 0, direction = 0, new_list_index = 0, list_index = 0,
        animation_finished = 0, attack_rotates = 0, var_0a = 0, bounces = BOUNCES, attack_cooldown = ATTACK_COOLDOWN);
    session_slot!(runtime, Oum[oum].base.current_instruction = 0xca4b);
    session_slot!(runtime, Oum[oum]; function = IDLE, bounce_function = FALLING);
}
/// Check if touching Samus and update previous position ($A2:CD23); the function.
fn prepare(runtime: &mut impl ActorLease, oum: i32) -> u16 {
    session_slot!(runtime, Oum[oum]; touching = 0, samus_left = 0);
    if near(runtime, Axis::Y, 0x20) && near(runtime, Axis::X, 0x18) {
        session_slot!(runtime, Oum[oum].touching = 1);
        if left(runtime) {
            session_slot!(runtime, Oum[oum].samus_left = 1);
        }
    }
    let x = session_slot!(runtime, Oum[oum].base.x_pos);
    let y = session_slot!(runtime, Oum[oum].base.y_pos);
    session_slot!(runtime, Oum[oum]; previous_x = x, previous_y = y);
    session_slot!(runtime, Oum[oum].function)
}
/// Oum function - idle ($A2:CDE6) or rolling ($A2:CE2B), up to its bounce function; the
/// bounce function still to run.
fn begin_action(runtime: &mut impl ActorLease, oum: i32, rolling: bool) -> Option<u16> {
    if session_slot!(runtime, Oum[oum].bounces) != 0 {
        return Some(session_slot!(runtime, Oum[oum].bounce_function));
    }
    let key = combat::current(runtime);
    if rolling {
        if api::move_vertical(runtime, key, 1 << 16) != Blocked {
            session_slot!(runtime, Oum[oum]; y_speed_index = 0, bounce_function = FALLING, bounces = BOUNCES);
        }
        return None;
    }
    session_slot!(runtime, Oum[oum].new_list_index = 0);
    if left(runtime) {
        session_slot!(runtime, Oum[oum]; direction = 1, new_list_index = 1);
    }
    set_instruction_list(runtime, oum);
    if near(runtime, Axis::X, 0x18) {
        session_slot!(runtime, Oum[oum].new_list_index, |index| index | 2);
        set_instruction_list(runtime, oum);
        session_slot!(runtime, Oum[oum].function = ROLLING);
    }
    None
}
fn pushed(runtime: &impl ActorLease, oum: i32) -> bool {
    session_slot!(runtime, Oum[oum].touching) != 0
        && session_slot!(runtime, Oum[oum].direction) == session_slot!(runtime, Oum[oum].samus_left)
}
/// Oum rolling, unless stopped: attack Samus near, or roll on, turning at a wall.
fn roll(runtime: &mut impl ActorLease, oum: i32) {
    if session_slot!(runtime, Oum[oum].stop) != 0 {
        return;
    }
    let cooldown = session_slot!(runtime, Oum[oum].attack_cooldown).wrapping_sub(1);
    session_slot!(runtime, Oum[oum].attack_cooldown = cooldown);
    let expired = cooldown & 0x8000 != 0;
    if expired {
        session_slot!(runtime, Oum[oum].attack_cooldown = 0);
    }
    if expired
        && near(runtime, Axis::X, 0x20)
        && session_slot!(runtime, Oum[oum].attack_rotates) != 0
        && session_slot!(runtime, Oum[oum].bounces) == 0
    {
        session_slot!(runtime, Oum[oum]; new_list_index = 0, direction = 0);
        if left(runtime) {
            session_slot!(runtime, Oum[oum]; new_list_index = 1, direction = 1);
        }
        session_slot!(runtime, Oum[oum].attack_cooldown = ATTACK_COOLDOWN);
        session_slot!(runtime, Oum[oum].new_list_index, |index| index & 1 | 4);
        set_instruction_list(runtime, oum);
        session_slot!(runtime, Oum[oum].function = ATTACKING);
        return;
    }
    let left = session_slot!(runtime, Oum[oum].direction) != 0;
    let index = if left { 66 } else { 64 };
    let fraction = runtime.lease_enemy_speed(Linear, index + 1);
    let mut whole = runtime.lease_enemy_speed(Linear, index);
    // Rolling into Samus as she pushes slows it by 10h px a frame.
    if pushed(runtime, oum) {
        whole = whole.wrapping_add(if left { 0xfff0 } else { 0x10 });
    }
    let key = combat::current(runtime);
    let speed = ((u32::from(whole) << 16) | u32::from(fraction)) as i32;
    let blocked = api::move_horizontal(runtime, key, speed, SlopePolicy::Solid) == Blocked;
    if pushed(runtime, oum) {
        let back = if session_slot!(runtime, Oum[oum].direction) != 0 {
            0x10
        } else {
            0xfff0
        };
        session_slot!(runtime, Oum[oum].base.x_pos, |x| x.wrapping_add(back));
    }
    if blocked {
        session_slot!(runtime, Oum[oum].direction, |direction| direction ^ 1);
        session_slot!(runtime, Oum[oum].new_list_index, |index| index ^ 4);
        set_instruction_list(runtime, oum);
    }
}
/// Oum function - attacking ($A2:CF40): roll again once the attack's animation finishes.
fn attack(runtime: &mut impl ActorLease, oum: i32) {
    if session_slot!(runtime, Oum[oum].animation_finished) != 0 {
        session_slot!(runtime, Oum[oum].animation_finished = 0);
        session_slot!(runtime, Oum[oum].new_list_index, |i| i.wrapping_sub(2));
        set_instruction_list(runtime, oum);
        session_slot!(runtime, Oum[oum].function = ROLLING);
    }
}
/// Handle pushing Samus ($A2:CD35) and stop if player is pressing towards oum ($A2:CD77):
/// carry her first, then resist.
fn finish(runtime: &mut impl ActorLease, oum: i32) {
    session_slot!(runtime, Oum[oum].stop = 0);
    if api::touches_player_from_above(runtime, combat::current(runtime)) {
        let y = session_slot!(runtime, Oum[oum].base.y_pos);
        let moved = y.wrapping_sub(session_slot!(runtime, Oum[oum].previous_y));
        runtime.write_slice_le(words::EXTRA_SAMUS_Y_DISPLACEMENT, moved);
    }
    if session_slot!(runtime, Oum[oum].touching) == 0 {
        return;
    }
    let x = session_slot!(runtime, Oum[oum].base.x_pos);
    let moved = x.wrapping_sub(session_slot!(runtime, Oum[oum].previous_x)) as i16;
    let side = session_slot!(runtime, Oum[oum].samus_left) != 0;
    if (moved < 0) == side {
        let carried = runtime
            .word(words::EXTRA_SAMUS_X_DISPLACEMENT)
            .wrapping_add(moved as u16);
        runtime.write_slice_le(words::EXTRA_SAMUS_X_DISPLACEMENT, carried);
    }
    if moved == 0 && (session_slot!(runtime, Oum[oum].direction) != 0) != side {
        return;
    }
    let towards = if (moved == 0 && side) || moved < 0 {
        0x100
    } else {
        0x200
    }; // Right, left.
    if runtime.word(words::JOYPAD1_LASTKEYS) & towards != 0 {
        let x = session_slot!(runtime, Oum[oum].previous_x);
        session_slot!(runtime, Oum[oum]; stop = 1);
        session_slot!(runtime, Oum[oum].base.x_pos = x);
    }
}
/// Main AI - oum ($A2:CD13), each step one lease; the bounce function runs through the
/// pre-instruction table between them.
pub fn main_ai<H: LeaseActors + PreInstructionHost + SpeciesHost>(game: &mut H) {
    let (oum, function) = {
        let mut lease = game.lease();
        let oum = current_slot(&lease);
        (oum, prepare(&mut lease, oum))
    };
    match function {
        IDLE | ROLLING => {
            let bounce = begin_action(&mut game.lease(), oum, function == ROLLING);
            if let Some(bounce) = bounce {
                pre_instruction(0xa2_0000 | u32::from(bounce), game);
            }
            if function == ROLLING {
                roll(&mut game.lease(), oum);
            }
        }
        ATTACKING => {
            let mut lease = game.lease();
            let oum = current_slot(&lease);
            attack(&mut lease, oum);
        }
        _ => game.unknown_routine(),
    }
    let mut lease = game.lease();
    let oum = current_slot(&lease);
    finish(&mut lease, oum);
}
/// Oum bounce function - falling ($A2:CF66) or rising ($A2:CFA9), for enemy `k`.
pub fn bounce<H: LeaseActors + ?Sized>(rising: bool, k: u16, game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let oum = i32::from(k);
    let step = if rising { 0xfe80 } else { 0x180 };
    let mut index = session_slot!(runtime, Oum[oum].y_speed_index).wrapping_add(step);
    if !rising && index.wrapping_sub(0x4000) & 0x8000 == 0 {
        index = 0x4000;
    }
    session_slot!(runtime, Oum[oum].y_speed_index = index);
    if rising && (index as i16) < 0 {
        session_slot!(runtime, Oum[oum]; y_speed_index = 0, bounce_function = FALLING);
        return;
    }
    let offset = 8 * usize::from(index >> 8) + if rising { 4 } else { 0 };
    let speed = runtime.lease_enemy_speed32(offset) as i32;
    let blocked = api::move_vertical(runtime, ActorKey::from_legacy(k), speed) == Blocked;
    if !rising && blocked {
        let bounces = session_slot!(runtime, Oum[oum].bounces).wrapping_sub(1);
        let next = (session_slot!(runtime, Oum[oum].y_speed_index) as i16).wrapping_sub(0x1000);
        let bounces = if next < 0 { 0 } else { bounces };
        session_slot!(runtime, Oum[oum]; bounces = bounces, y_speed_index = next as u16, bounce_function = RISING);
    }
}
/// Enemy touch - oum - hurts Samus ($A2:D388) or doesn't ($A2:D38C): push her aside 4 px.
pub fn touch<H: LeaseActors + ?Sized>(hurts: bool, game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    if hurts {
        combat::contact_reaction(runtime, DeathHandling::Finalize);
    }
    if !api::touches_player_from_above(runtime, combat::current(runtime)) {
        let x = session_slot!(runtime, Oum[current_slot(runtime)].base.x_pos);
        let left = (runtime.word(words::SAMUS_X_POS).wrapping_sub(x) as i16) < 0;
        let push = if left { 0xfffc } else { 4 };
        let displacement = runtime
            .word(words::EXTRA_SAMUS_X_DISPLACEMENT)
            .wrapping_add(push);
        runtime.write_slice_le(words::EXTRA_SAMUS_X_DISPLACEMENT, displacement);
    }
}
/// Enemy shot - oum ($A2:D3B4).
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    combat::shot_reaction(&mut lease, ShotHandling::Finalize);
    lease.queue_sound(2, 6, 0x57);
}
/// Instructions - set animation finished flag ($A2:CCB3), and attack allowing rotation
/// flag = 1 ($A2:CCBE) or 0 ($A2:CCC9).
pub fn instruction<H: LeaseActors + ?Sized>(ea: u32, j: EnemyScript, game: &mut H) -> EnemyScript {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let oum = current_slot(runtime);
    let rotates = u16::from(ea == 0xa2_ccbe);
    match ea {
        0xa2_ccb3 => session_slot!(runtime, Oum[oum].animation_finished = 1),
        _ => session_slot!(runtime, Oum[oum].attack_rotates = rotates),
    }
    j
}
