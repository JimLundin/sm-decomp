//! Geega detection, straight flight, curved turns and home reset.
//! Native actor fields retain authority and original query/ROM failure ordering.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, BossEffects};
use crate::actors::slot_view::{SlotState, slot_view};
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with the geegas' effects.
pub trait Lease: BossLease<Game: BossEffects> + SlotState {}
impl<T: BossLease<Game: BossEffects> + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A geega's slot: its state routine; the turn's arc step and whether the
    /// arc is slowing; its right and left flight speeds (whole over sub words);
    /// the X its turn started at, whether it has turned, its home position and
    /// its launch delay.
    Geega, GeegaEdit {
        record: [
            state = var_a, var_c = var_c, var_d = var_d, arc_step = var_e,
            arc_slowing = var_f, parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            right_subspeed = var_00, right_speed = var_01, left_subspeed = var_02,
            left_speed = var_03, turn_x = var_04, turned = var_06, home_x = var_07,
            home_y = var_08, launch_delay = var_20,
        ],
    }
}

/// Main AI - geega ($B3:8FAE): run its state routine.
pub fn main_ai<H: Lease + ?Sized>(game: &mut H) {
    match state_address(game) {
        11767733 => detect_player(game),
        11767797 => launch_delay(game.current_enemy(), game),
        11767848 => fly(true, game),
        11767997 => fly(false, game),
        11768154 => turn(true, game),
        11768280 => turn(false, game),
        _ => game.unreachable(),
    }
}

/// Set the geega's instruction list to `list`, to run next frame.
fn set_list<H: Lease + ?Sized>(e: u16, list: u16, game: &mut H) {
    let mut edit = GeegaEdit::of(game, e);
    let actor = edit.actor();
    actor.instruction_delay = 1;
    actor.timer = 0;
    actor.instruction = list;
}

pub fn initialize<H: Lease + ?Sized>(game: &mut H) {
    let e = game.current_enemy();
    let index = {
        let mut edit = GeegaEdit::of(game, e);
        let copied = edit.actor().x.whole;
        *edit.home_x() = copied;
        let copied = edit.actor().y.whole;
        *edit.home_y() = copied;
        let actor = edit.actor();
        actor.x.fraction = 0;
        actor.y.fraction = 0;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.instruction = 0x8efc;
        if *edit.parameter_1() == 0 {
            edit.actor().instruction = 0x8f24;
        }
        ((8 * i32::from(*edit.parameter_2())) >> 1) as isize
    };
    let right_speed = game.enemy_speed(SpeedTable::Linear, index);
    let right_subspeed = game.enemy_speed(SpeedTable::Linear, index + 1);
    let left_speed = game.enemy_speed(SpeedTable::Linear, index + 2);
    let left_subspeed = game.enemy_speed(SpeedTable::Linear, index + 3);
    let mut edit = GeegaEdit::of(game, e);
    *edit.right_speed() = right_speed;
    *edit.right_subspeed() = right_subspeed;
    *edit.left_speed() = left_speed;
    *edit.left_subspeed() = left_subspeed;
    *edit.state() = 0x8fb5;
    *edit.turned() = 0;
}
pub fn state_address<H: Lease + ?Sized>(game: &H) -> u32 {
    0xb30000 | u32::from(Geega::of(game, game.current_enemy()).state())
}
fn reset_home<H: Lease + ?Sized>(e: u16, game: &mut H) {
    let mut edit = GeegaEdit::of(game, e);
    let x = *edit.home_x();
    edit.actor().x.whole = x;
    let y = *edit.home_y();
    let actor = edit.actor();
    actor.y.whole = y;
    actor.x.fraction = 0;
    actor.y.fraction = 0;
    *edit.state() = 0x8fb5;
    *edit.turned() = 0;
}
fn horizontal<H: Lease + ?Sized>(left: bool, k: u16, game: &mut H) {
    let geega = Geega::of(game, k);
    let (high, low) = if left {
        (geega.left_speed(), geega.left_subspeed())
    } else {
        (geega.right_speed(), geega.right_subspeed())
    };
    let speed = (u32::from(high) << 16) | u32::from(low);
    GeegaEdit::of(game, k).actor().x.advance(speed as i32);
}
fn vertical<H: Lease + ?Sized>(counting_down: bool, k: u16, game: &mut H) {
    let step = {
        let mut edit = GeegaEdit::of(game, k);
        let step = edit.arc_step();
        if counting_down {
            *step = step.wrapping_sub(1);
            if *step & 0x8000 != 0 {
                *step = 0;
                *edit.arc_slowing() = 0;
                return;
            }
        } else {
            *step = step.wrapping_add(1);
        }
        *step
    };
    let index = ((8 * i32::from(step)) >> 1) as isize;
    let offset = if counting_down { 0 } else { 2 };
    let high = game.enemy_speed(SpeedTable::Quadratic, index + offset + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index + offset);
    let speed = (u32::from(high) << 16) | u32::from(low);
    GeegaEdit::of(game, k).actor().y.advance(speed as i32);
}
pub fn detect_player<H: Lease + ?Sized>(game: &mut H) {
    let e = game.current_enemy();
    let left = Geega::of(game, e).parameter_1() != 0;
    let actor = game.current_enemy();
    let dx = game.player_delta(actor, Axis::X) as i16;
    let in_x_range = if left {
        dx < 0 && (i32::from(dx) + 192) & 0x8000 == 0
    } else {
        dx >= 0 && (i32::from(dx) - 192) & 0x8000 != 0
    };
    if !in_x_range {
        return;
    }
    let actor = game.current_enemy();
    let in_y_range = game.player_within(actor, Axis::Y, 0x30);
    if in_y_range {
        let mut edit = GeegaEdit::of(game, e);
        edit.actor().properties &= !0x100;
        *edit.launch_delay() = 24;
        *edit.state() = 0x8ff5;
    }
}
pub fn launch_delay<H: Lease + ?Sized>(k: u16, game: &mut H) {
    let delay = Geega::of(game, k).launch_delay().wrapping_sub(1);
    *GeegaEdit::of(game, k).launch_delay() = delay;
    if delay == 0 {
        set_list(k, 0x8efc, game);
        let mut edit = GeegaEdit::of(game, k);
        *edit.state() = 0x9028;
        if *edit.parameter_1() == 0 {
            edit.actor().instruction = 0x8f24;
            *edit.state() = 0x90bd;
        }
    }
}
pub fn fly<H: Lease + ?Sized>(left: bool, game: &mut H) {
    let e = game.current_enemy();
    horizontal(left, game.current_enemy(), game);
    if game.current_outside_screen() {
        reset_home(e, game);
        GeegaEdit::of(game, e).actor().properties |= 0x100;
    } else if Geega::of(game, e).turned() == 0
        && if left {
            (i32::from(Geega::of(game, e).actor().x.whole) - i32::from(game.player_x()) - 48)
                & 0x8000
                != 0
        } else {
            (i32::from(game.player_x()) - i32::from(Geega::of(game, e).actor().x.whole) - 48)
                & 0x8000
                != 0
        }
    {
        let (state, instruction) = if left {
            (0x915a, 0x8f10)
        } else {
            (0x91d8, 0x8f38)
        };
        {
            let mut edit = GeegaEdit::of(game, e);
            *edit.state() = state;
            *edit.var_c() = 0;
            *edit.var_d() = 0;
            *edit.arc_step() = 40;
            *edit.arc_slowing() = 1;
            let x = edit.actor().x.whole;
            *edit.turn_x() = x;
        }
        set_list(e, instruction, game);
    }
}
pub fn turn<H: Lease + ?Sized>(left: bool, game: &mut H) {
    let e = game.current_enemy();
    let instruction = if left { 0x8efc } else { 0x8f24 };
    if game.current_outside_screen() {
        reset_home(e, game);
        set_list(e, instruction, game);
        GeegaEdit::of(game, e).actor().properties |= 0x100;
    } else {
        horizontal(left, game.current_enemy(), game);
        let counting_down = Geega::of(game, e).arc_slowing() != 0;
        vertical(counting_down, game.current_enemy(), game);
        if !counting_down {
            let geega = Geega::of(game, e);
            let (y, home_y) = (geega.actor().y.whole as i16, geega.home_y() as i16);
            let finished = if left {
                y.wrapping_sub(home_y) < 0
            } else {
                home_y.wrapping_sub(y) >= 0
            };
            if finished {
                {
                    let mut edit = GeegaEdit::of(game, e);
                    *edit.turned() = 1;
                    *edit.arc_slowing() = 1;
                    // The two legacy turns publish their next state on opposite sides of animation.
                    if left {
                        *edit.state() = 0x9028;
                    }
                }
                set_list(e, instruction, game);
                if !left {
                    *GeegaEdit::of(game, e).state() = 0x90bd;
                }
            }
        }
    }
}
