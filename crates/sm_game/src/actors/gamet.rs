//! Gamet's ordered five-part emergence and flight.
//! Native actor words remain authoritative across the existing query boundaries.
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::host::{Axis, BossEffects};
use crate::actors::slot_view::{SlotState, slot_view};
use sm_data::enemy_speeds::SpeedTable;

/// A boss state whose slots are the Session's typed storage.
pub trait Lease: BossState + SlotState {}
impl<T: BossState + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A gamet's slot: its state function and the one planned next, its flight speed
    /// index, its step counter, its spawn position, the Y it rose to and its flight delay.
    /// The second parameter's low byte is the group's trigger distance, its high byte the
    /// flight speed.
    Gamet, GametEdit {
        record: [
            state = var_a, speed_index = var_b, counter = var_c, spawn_x = var_d,
            spawn_y = var_e, next_state = var_f, parameter_2 = parameter_2,
        ],
        extended: [rest_y = var_01, flight_delay = var_02],
    }
}

/// The five parts' slots past the first.
const PARTS: [u16; 5] = [0, 64, 128, 192, 256];

pub fn initialize<H: Lease + ?Sized>(game: &mut H) {
    let k = game.current_enemy();
    let mut gamet = GametEdit::of(game, k);
    let x = gamet.actor().x.whole;
    *gamet.spawn_x() = x;
    let y = gamet.actor().y.whole;
    *gamet.spawn_y() = y;
    gamet.actor().properties |= 0x100;
    let speed = gamet.parameter_2().to_le_bytes()[1];
    *gamet.speed_index() = 8u16.wrapping_mul(u16::from(speed));
    *gamet.counter() = 0;
    let actor = gamet.actor();
    actor.instruction_delay = 1;
    actor.timer = 0;
    actor.instruction = 0x8ae1;
    *gamet.state() = 0x8bcd;
}
/// Main AI - gamet ($B3:8B9E): run its state, then send it back to its
/// pipe once it is off screen.
pub fn main_ai<L: BossLease<Game: BossEffects> + SlotState + ?Sized>(game: &mut L) {
    match state_address(game) {
        11766733 => await_group(game),
        11766783 => await_player(game),
        11766950 => rise(game),
        11767039 => settle(game),
        11767052 => move_to_limit(-16, game),
        11767118 => move_to_limit(-32, game),
        11767184 => move_to_limit(16, game),
        11767250 => move_to_limit(32, game),
        11767316 => fly(2, game),
        11767349 => fly(0, game),
        11767386 => start_flight(game),
        _ => game.unreachable(),
    }
    reset_offscreen(game);
}
pub fn state_address<H: Lease + ?Sized>(game: &H) -> u32 {
    let k = game.current_enemy();
    0xb30000 | u32::from(Gamet::of(game, k).state())
}
pub fn reset_offscreen<H: Lease + ?Sized>(game: &mut H) {
    if game.current_outside_screen() {
        let k = game.current_enemy();
        let mut gamet = GametEdit::of(game, k);
        gamet.actor().properties |= 0x100;
        *gamet.state() = 0x8bcd;
        let x = *gamet.spawn_x();
        gamet.actor().x.whole = x;
        let y = *gamet.spawn_y();
        gamet.actor().y.whole = y;
    }
}
fn speed<H: BossState + ?Sized>(index: isize, game: &H) -> u32 {
    let high = game.enemy_speed(SpeedTable::Linear, index);
    let low = game.enemy_speed(SpeedTable::Linear, index + 1);
    (u32::from(high) << 16) | u32::from(low)
}
fn facing_animation<H: BossState + ?Sized>(actor: u16, game: &mut H) -> u16 {
    if game.player_delta(actor, Axis::X) & 0x8000 == 0 {
        0x8b21
    } else {
        0x8ae1
    }
}
fn plan_group<H: Lease + ?Sized>(k: u16, game: &mut H) {
    // Finish each field's five stores before starting the next field.
    for offset in PARTS {
        *GametEdit::of(game, k + offset).state() = 0x8ca6;
    }
    for (offset, delay) in PARTS.into_iter().zip([104, 96, 88, 112, 120]) {
        *GametEdit::of(game, k + offset).flight_delay() = delay;
    }
    for (offset, state) in PARTS
        .into_iter()
        .zip([0x8cff, 0x8d0c, 0x8d4e, 0x8d90, 0x8dd2])
    {
        *GametEdit::of(game, k + offset).next_state() = state;
    }
}
pub fn await_group<H: Lease + ?Sized>(game: &mut H) {
    let current = game.current_enemy();
    if Gamet::of(game, current).parameter_2().to_le_bytes()[0] != 0
        && PARTS[1..].iter().all(|&offset| {
            let k = game.current_enemy() + offset;
            Gamet::of(game, k).state() == 0x8bcd
        })
    {
        let k = game.current_enemy();
        *GametEdit::of(game, k).state() = 0x8bff;
    }
}
pub fn await_player<H: Lease + ?Sized>(game: &mut H) {
    let k = game.current_enemy();
    let distance = u16::from(Gamet::of(game, k).parameter_2().to_le_bytes()[0]);
    let in_range = game.player_within(k, Axis::X, distance);
    if in_range
        && (Gamet::of(game, k).actor().y.whole as i16).wrapping_sub(game.player_y() as i16) >= 0
    {
        {
            let mut gamet = GametEdit::of(game, k);
            let counter = gamet.counter();
            *counter = counter.wrapping_add(1);
            let actor = gamet.actor();
            actor.instruction_delay = 1;
            actor.timer = 0;
        }
        let instruction = facing_animation(k, game);
        for offset in PARTS {
            GametEdit::of(game, k + offset).actor().instruction = instruction;
        }
        plan_group(k, game);
    }
}
pub fn rise<H: Lease + ?Sized>(game: &mut H) {
    let k = game.current_enemy();
    GametEdit::of(game, k).actor().properties &= !0x100;
    let amount = speed(66, game);
    GametEdit::of(game, k)
        .actor()
        .y
        .advance(amount.cast_signed());
    if (Gamet::of(game, k).actor().y.whole as i16).wrapping_sub(game.player_y() as i16) < 0 {
        {
            let mut gamet = GametEdit::of(game, k);
            let next = *gamet.next_state();
            *gamet.state() = next;
            let y = gamet.actor().y.whole;
            *gamet.rest_y() = y;
            let actor = gamet.actor();
            actor.instruction_delay = 1;
            actor.timer = 0;
        }
        let instruction = facing_animation(k, game);
        GametEdit::of(game, k).actor().instruction = instruction;
    }
}
/// Also a bank $B3 pre-instruction, which the game dispatches on a lease.
pub fn settle<H: Lease + ?Sized>(game: &mut H) {
    let k = game.current_enemy();
    let mut gamet = GametEdit::of(game, k);
    let counter = gamet.counter();
    *counter = counter.wrapping_add(1);
    *gamet.state() = 0x8e5a;
}
pub fn move_to_limit<H: Lease + ?Sized>(offset: i32, game: &mut H) {
    let k = game.current_enemy();
    {
        let mut gamet = GametEdit::of(game, k);
        let counter = gamet.counter();
        *counter = counter.wrapping_add(1);
    }
    let amount = speed(if offset < 0 { 66 } else { 64 }, game);
    GametEdit::of(game, k)
        .actor()
        .y
        .advance(amount.cast_signed());
    let gamet = Gamet::of(game, k);
    let delta = (i32::from(gamet.actor().y.whole) - offset - i32::from(gamet.rest_y())) as i16;
    if if offset < 0 { delta < 0 } else { delta >= 0 } {
        let mut gamet = GametEdit::of(game, k);
        let y = gamet.rest_y().wrapping_add(offset as u16);
        let actor = gamet.actor();
        actor.y.whole = y;
        actor.y.fraction = 0;
        *gamet.state() = 0x8e5a;
    }
}
pub fn fly<H: Lease + ?Sized>(speed_offset: isize, game: &mut H) {
    let k = game.current_enemy();
    let index = (Gamet::of(game, k).speed_index() >> 1) as isize;
    let amount = speed(index + speed_offset, game);
    GametEdit::of(game, k)
        .actor()
        .x
        .advance(amount.cast_signed());
}
pub fn start_flight<H: Lease + ?Sized>(game: &mut H) {
    let k = game.current_enemy();
    {
        let mut gamet = GametEdit::of(game, k);
        let counter = gamet.counter();
        *counter = counter.wrapping_add(1);
        if (*gamet.counter() as i16).wrapping_sub(*gamet.flight_delay() as i16) < 0 {
            return;
        }
        *gamet.counter() = 0;
        let actor = gamet.actor();
        actor.instruction_delay = 1;
        actor.timer = 0;
        *gamet.state() = 0x8e14;
        gamet.actor().instruction = 0x8b05;
    }
    if game.player_delta(k, Axis::X) & 0x8000 == 0 {
        let mut gamet = GametEdit::of(game, k);
        *gamet.state() = 0x8e35;
        gamet.actor().instruction = 0x8b45;
    }
}
