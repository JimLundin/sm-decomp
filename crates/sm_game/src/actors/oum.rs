//! Oum, Maridia's rolling snail, which bounces into place, rolls and attacks, and carries or
//! holds back Samus as she rides or pushes it (bank $A2:CA2B-$A2:D80B).
use crate::actors::access::ActorKey;
use crate::actors::api::{
    self, Axis, DeathHandling, MovementResult::Blocked, ShotHandling, SlopePolicy,
};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::controller_state::Word as Input;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::table;
use sm_data::enemy_speeds::SpeedTable::Linear;

/// A lease on a game whose slots are the Session's typed storage.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// An oum's slot.
    Oum, OumEdit {
        record: [
            function = var_a, y_speed_index = var_b, attack_cooldown = var_c, direction = var_d,
            bounces = var_e, bounce_function = var_f,
        ],
        extended: [
            new_list_index = var_00, list_index = var_01, animation_finished = var_02,
            attack_rotates = var_03, touching = var_04, samus_left = var_05, previous_x = var_06,
            previous_y = var_08, var_0a = var_0a, stop = var_0b,
        ],
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

fn left(runtime: &impl Lease) -> bool {
    api::player_delta(runtime, combat::current(runtime), Axis::X) & 0x8000 != 0
}
fn near(runtime: &mut impl Lease, axis: Axis, distance: u16) -> bool {
    let key = combat::current(runtime);
    api::player_within(runtime, key, axis, distance)
}
/// Set oum instruction list ($A2:CFDF), if its index changed.
fn set_instruction_list(runtime: &mut impl Lease, oum: u16) {
    let index = Oum::of(runtime, oum).new_list_index();
    if index != Oum::of(runtime, oum).list_index() {
        *OumEdit::of(runtime, oum).list_index() = index;
        let list = table(runtime, LISTS, index as isize);
        let mut part = OumEdit::of(runtime, oum);
        part.actor().instruction = list;
        part.actor().instruction_delay = 1;
        part.actor().timer = 0;
    }
}
/// Initialisation AI - oum ($A2:CCD4).
pub fn initialize(runtime: &mut impl Lease) {
    let oum = combat::current(runtime).byte_index();
    {
        let mut part = OumEdit::of(runtime, oum);
        *part.y_speed_index() = 0;
        *part.direction() = 0;
        *part.new_list_index() = 0;
        *part.list_index() = 0;
        *part.animation_finished() = 0;
        *part.attack_rotates() = 0;
        *part.var_0a() = 0;
        *part.bounces() = BOUNCES;
        *part.attack_cooldown() = ATTACK_COOLDOWN;
    }
    OumEdit::of(runtime, oum).actor().instruction = 0xca4b;
    let mut part = OumEdit::of(runtime, oum);
    *part.function() = IDLE;
    *part.bounce_function() = FALLING;
}
/// Check if touching Samus and update previous position ($A2:CD23); the function.
fn prepare(runtime: &mut impl Lease, oum: u16) -> u16 {
    {
        let mut part = OumEdit::of(runtime, oum);
        *part.touching() = 0;
        *part.samus_left() = 0;
    }
    if near(runtime, Axis::Y, 0x20) && near(runtime, Axis::X, 0x18) {
        *OumEdit::of(runtime, oum).touching() = 1;
        if left(runtime) {
            *OumEdit::of(runtime, oum).samus_left() = 1;
        }
    }
    let x = Oum::of(runtime, oum).actor().x.whole;
    let y = Oum::of(runtime, oum).actor().y.whole;
    {
        let mut part = OumEdit::of(runtime, oum);
        *part.previous_x() = x;
        *part.previous_y() = y;
    }
    Oum::of(runtime, oum).function()
}
/// Oum function - idle ($A2:CDE6) or rolling ($A2:CE2B), up to its bounce function; the
/// bounce function still to run.
fn begin_action(runtime: &mut impl Lease, oum: u16, rolling: bool) -> Option<u16> {
    if Oum::of(runtime, oum).bounces() != 0 {
        return Some(Oum::of(runtime, oum).bounce_function());
    }
    let key = combat::current(runtime);
    if rolling {
        if api::move_vertical(runtime, key, 1 << 16) != Blocked {
            let mut part = OumEdit::of(runtime, oum);
            *part.y_speed_index() = 0;
            *part.bounce_function() = FALLING;
            *part.bounces() = BOUNCES;
        }
        return None;
    }
    *OumEdit::of(runtime, oum).new_list_index() = 0;
    if left(runtime) {
        let mut part = OumEdit::of(runtime, oum);
        *part.direction() = 1;
        *part.new_list_index() = 1;
    }
    set_instruction_list(runtime, oum);
    if near(runtime, Axis::X, 0x18) {
        *OumEdit::of(runtime, oum).new_list_index() |= 2;
        set_instruction_list(runtime, oum);
        *OumEdit::of(runtime, oum).function() = ROLLING;
    }
    None
}
fn pushed(runtime: &impl Lease, oum: u16) -> bool {
    Oum::of(runtime, oum).touching() != 0
        && Oum::of(runtime, oum).direction() == Oum::of(runtime, oum).samus_left()
}
/// Oum rolling, unless stopped: attack Samus near, or roll on, turning at a wall.
fn roll(runtime: &mut impl Lease, oum: u16) {
    crate::probe::hit("oum.roll");
    if Oum::of(runtime, oum).stop() != 0 {
        return;
    }
    let cooldown = Oum::of(runtime, oum).attack_cooldown().wrapping_sub(1);
    *OumEdit::of(runtime, oum).attack_cooldown() = cooldown;
    let expired = cooldown & 0x8000 != 0;
    if expired {
        *OumEdit::of(runtime, oum).attack_cooldown() = 0;
    }
    if expired
        && near(runtime, Axis::X, 0x20)
        && Oum::of(runtime, oum).attack_rotates() != 0
        && Oum::of(runtime, oum).bounces() == 0
    {
        {
            let mut part = OumEdit::of(runtime, oum);
            *part.new_list_index() = 0;
            *part.direction() = 0;
        }
        if left(runtime) {
            let mut part = OumEdit::of(runtime, oum);
            *part.new_list_index() = 1;
            *part.direction() = 1;
        }
        *OumEdit::of(runtime, oum).attack_cooldown() = ATTACK_COOLDOWN;
        {
            let mut part = OumEdit::of(runtime, oum);
            let index = *part.new_list_index();
            *part.new_list_index() = (index & 1) | 4;
        }
        set_instruction_list(runtime, oum);
        *OumEdit::of(runtime, oum).function() = ATTACKING;
        return;
    }
    let left = Oum::of(runtime, oum).direction() != 0;
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
        let back = if Oum::of(runtime, oum).direction() != 0 {
            0x10
        } else {
            0xfff0
        };
        let mut part = OumEdit::of(runtime, oum);
        let old = part.actor().x.whole;
        part.actor().x.whole = old.wrapping_add(back);
    }
    if blocked {
        *OumEdit::of(runtime, oum).direction() ^= 1;
        *OumEdit::of(runtime, oum).new_list_index() ^= 4;
        set_instruction_list(runtime, oum);
    }
}
/// Oum function - attacking ($A2:CF40): roll again once the attack's animation finishes.
fn attack(runtime: &mut impl Lease, oum: u16) {
    if Oum::of(runtime, oum).animation_finished() != 0 {
        *OumEdit::of(runtime, oum).animation_finished() = 0;
        {
            let mut part = OumEdit::of(runtime, oum);
            let old = *part.new_list_index();
            *part.new_list_index() = old.wrapping_sub(2);
        }
        set_instruction_list(runtime, oum);
        *OumEdit::of(runtime, oum).function() = ROLLING;
    }
}
/// Handle pushing Samus ($A2:CD35) and stop if player is pressing towards oum ($A2:CD77):
/// carry her first, then resist.
fn finish(runtime: &mut impl Lease, oum: u16) {
    *OumEdit::of(runtime, oum).stop() = 0;
    if api::touches_player_from_above(runtime, combat::current(runtime)) {
        let y = Oum::of(runtime, oum).actor().y.whole;
        let moved = y.wrapping_sub(Oum::of(runtime, oum).previous_y());
        runtime.session_mut().player.contact.external_y.whole = moved;
    }
    if Oum::of(runtime, oum).touching() == 0 {
        return;
    }
    let x = Oum::of(runtime, oum).actor().x.whole;
    let moved = x.wrapping_sub(Oum::of(runtime, oum).previous_x()) as i16;
    let side = Oum::of(runtime, oum).samus_left() != 0;
    if (moved < 0) == side {
        let displacement = &mut runtime.session_mut().player.contact.external_x.whole;
        *displacement = displacement.wrapping_add(moved as u16);
    }
    if moved == 0 && (Oum::of(runtime, oum).direction() != 0) != side {
        return;
    }
    let towards = if (moved == 0 && side) || moved < 0 {
        0x100
    } else {
        0x200
    }; // Right, left.
    if *Input::Joypad1Lastkeys.get(&runtime.session().control) & towards != 0 {
        let x = Oum::of(runtime, oum).previous_x();
        let mut part = OumEdit::of(runtime, oum);
        *part.stop() = 1;
        part.actor().x.whole = x;
    }
}
/// Main AI - oum ($A2:CD13) on the table lease; the bounce function runs through the
/// pre-instruction table as its own owned call, where the game opened a fresh lease.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
    let (oum, function) = {
        let oum = combat::current(&*runtime).byte_index();
        (oum, prepare(runtime, oum))
    };
    match function {
        IDLE | ROLLING => {
            let bounce = begin_action(runtime, oum, function == ROLLING);
            if let Some(bounce) = bounce {
                runtime.owned_lease_call(|runtime| {
                    crate::actors::species::table_pre_instruction(
                        0xa2_0000 | u32::from(bounce),
                        runtime,
                    )
                });
            }
            if function == ROLLING {
                roll(runtime, oum);
            }
        }
        ATTACKING => {
            let oum = combat::current(&*runtime).byte_index();
            attack(runtime, oum);
        }
        _ => crate::diagnostics::Fatal::unreachable(runtime),
    }
    let oum = combat::current(&*runtime).byte_index();
    finish(runtime, oum);
}
/// Oum bounce function - falling ($A2:CF66) or rising ($A2:CFA9), for enemy `k`.
pub fn bounce(rising: bool, k: u16, runtime: &mut impl crate::actors::species::TableLease) {
    let oum = k;
    let step = if rising { 0xfe80 } else { 0x180 };
    let mut index = Oum::of(runtime, oum).y_speed_index().wrapping_add(step);
    if !rising && index.wrapping_sub(0x4000) & 0x8000 == 0 {
        index = 0x4000;
    }
    *OumEdit::of(runtime, oum).y_speed_index() = index;
    if rising && (index as i16) < 0 {
        {
            let mut part = OumEdit::of(runtime, oum);
            *part.y_speed_index() = 0;
            *part.bounce_function() = FALLING;
        }
        return;
    }
    let offset = 8 * usize::from(index >> 8) + if rising { 4 } else { 0 };
    let speed = runtime.lease_enemy_speed32(offset) as i32;
    let blocked = api::move_vertical(runtime, ActorKey::from_legacy(k), speed) == Blocked;
    if !rising && blocked {
        let bounces = Oum::of(runtime, oum).bounces().wrapping_sub(1);
        let next = (Oum::of(runtime, oum).y_speed_index() as i16).wrapping_sub(0x1000);
        let bounces = if next < 0 { 0 } else { bounces };
        let mut part = OumEdit::of(runtime, oum);
        *part.bounces() = bounces;
        *part.y_speed_index() = next as u16;
        *part.bounce_function() = RISING;
    }
}
/// Enemy touch - oum - hurts Samus ($A2:D388) or doesn't ($A2:D38C): push her aside 4 px.
pub fn touch(hurts: bool, runtime: &mut impl Lease) {
    if hurts {
        combat::contact_reaction(runtime, DeathHandling::Finalize);
    }
    if !api::touches_player_from_above(runtime, combat::current(runtime)) {
        let oum = combat::current(runtime).byte_index();
        let x = Oum::of(runtime, oum).actor().x.whole;
        let left = (runtime.session().player.x.wrapping_sub(x) as i16) < 0;
        let push = if left { 0xfffc } else { 4 };
        let displacement = &mut runtime.session_mut().player.contact.external_x.whole;
        *displacement = displacement.wrapping_add(push);
    }
}
/// Enemy shot - oum ($A2:D3B4).
pub fn shot(lease: &mut impl Lease) {
    combat::shot_reaction(lease, ShotHandling::Finalize);
    lease.queue_sound(2, 6, 0x57);
}
/// Instructions - set animation finished flag ($A2:CCB3), and attack allowing rotation
/// flag = 1 ($A2:CCBE) or 0 ($A2:CCC9).
pub fn instruction(
    ea: u32,
    j: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    let oum = combat::current(runtime).byte_index();
    let rotates = u16::from(ea == 0xa2_ccbe);
    match ea {
        0xa2_ccb3 => *OumEdit::of(runtime, oum).animation_finished() = 1,
        _ => *OumEdit::of(runtime, oum).attack_rotates() = rotates,
    }
    j
}
