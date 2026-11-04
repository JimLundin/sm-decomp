//! Zeb emergence, horizontal flight, reset and animation selection.
//! Native actor words remain authoritative across the existing query boundaries.
use crate::actors::access::{ActorAccess, ActorKey as Key};
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, abs16};
use crate::actors::queries;
use crate::actors::slot_view::{SlotState, slot_view};

/// A lease on a game with the zebs' effects.
pub trait Lease: BossLease + ActorAccess + SlotState {}
impl<T: BossLease + ActorAccess + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A zeb's slot: its direction (bit 15 left), spawn position, cooldown,
    /// animation phase and state routine; the Y it rises to and the phase
    /// whose instruction list is set. Parameter 1 picks the alternate lists.
    Zeb, ZebEdit {
        record: [
            direction = var_a, spawn_x = var_b, spawn_y = var_c, cooldown = var_d,
            phase = var_e, state = var_f, parameter_1 = parameter_1,
        ],
        extended: [rise_target_y = var_00, shown_phase = var_01],
    }
}

/// Main AI - zeb ($B3:887A): run its state routine.
pub fn main_ai<H: Lease>(game: &mut H) {
    let k = game.current_enemy();
    match state_address(game) {
        11765888 => await_view(k, game),
        11765904 => await_player(k, game),
        11765987 => rise(k, game),
        11766044 => fly(k, game),
        11766142 => cooldown(k, game),
        _ => game.unreachable(),
    }
}

pub fn initialize<H: Lease>(game: &mut H) {
    let k = game.current_enemy();
    let mut zeb = ZebEdit::of(game, k);
    let (x, y) = (zeb.actor().x.whole, zeb.actor().y.whole);
    *zeb.spawn_x() = x;
    *zeb.spawn_y() = y;
    *zeb.rise_target_y() = y.wrapping_sub(16);
    *zeb.state() = 0x8880;
    *zeb.cooldown() = 48;
    *zeb.phase() = 0;
    *zeb.shown_phase() = 0;
    zeb.actor().instruction = 0x87ab;
    if *zeb.parameter_1() != 0 {
        zeb.actor().instruction = 0x8a1d;
    }
}
pub fn state_address<H: Lease>(game: &H) -> u32 {
    0xb30000 | u32::from(Zeb::of(game, game.current_enemy()).state())
}
fn update_animation<H: Lease>(game: &mut H) {
    let k = game.current_enemy();
    let zeb = Zeb::of(game, k);
    let phase = zeb.phase();
    if phase != zeb.shown_phase() {
        *ZebEdit::of(game, k).shown_phase() = phase;
        let alternate = Zeb::of(game, k).parameter_1() != 0;
        let list = crate::enemy_lists::pipe_bug_instruction_list(game, alternate, phase as isize);
        let mut zeb = ZebEdit::of(game, k);
        let actor = zeb.actor();
        actor.instruction = list;
        actor.instruction_delay = 1;
        actor.timer = 0;
    }
}
pub fn await_view<H: Lease>(_: u16, game: &mut H) {
    if !game.current_outside_screen() {
        let current = game.current_enemy();
        *ZebEdit::of(game, current).state() = 0x8890;
    }
}
pub fn await_player<H: Lease>(k: u16, game: &mut H) {
    let dy = game.player_delta(k, Axis::Y) as i16;
    if dy < 0 && (i32::from(dy) + 96) & 0x8000 == 0 {
        let dx = game.player_delta(k, Axis::X);
        {
            let mut zeb = ZebEdit::of(game, k);
            let direction = zeb.direction();
            *direction = (*direction & 0x7fff) | (dx & 0x8000);
        }
        if abs16(
            Zeb::of(game, k)
                .actor()
                .x
                .whole
                .wrapping_sub(game.player_x()),
        ) < 0x40
        {
            {
                let mut zeb = ZebEdit::of(game, k);
                *zeb.state() = 0x88e3;
                zeb.actor().properties &= !0x100;
                zeb.actor().timer = 0;
                *zeb.phase() = if *zeb.direction() & 0x8000 != 0 { 0 } else { 2 };
            }
            update_animation(game);
        }
    }
}
pub fn rise<H: Lease>(k: u16, game: &mut H) {
    {
        let mut zeb = ZebEdit::of(game, k);
        let y = &mut zeb.actor().y;
        let previous = y.fraction;
        y.fraction = previous.wrapping_sub(1);
        y.whole = (i32::from(y.whole) + i32::from(previous != 0) - 2) as u16;
    }
    let zeb = Zeb::of(game, k);
    let y = zeb.actor().y.whole;
    if (zeb.rise_target_y() as i16).wrapping_sub(y as i16) >= 0 && y < game.player_y() {
        *ZebEdit::of(game, k).phase() |= 1;
        update_animation(game);
        *ZebEdit::of(game, k).state() = 0x891c;
    }
}
pub fn fly<H: Lease>(k: u16, game: &mut H) {
    {
        let mut zeb = ZebEdit::of(game, k);
        let step = if *zeb.direction() & 0x8000 == 0 {
            2
        } else {
            0xfffe
        };
        // The original also stores the X subposition to itself here.
        let x = &mut zeb.actor().x;
        x.whole = x.whole.wrapping_add(step);
    }
    if queries::leaving_screen(&*game, Key::from_legacy(k)) as u8 & 1 != 0 {
        {
            let mut zeb = ZebEdit::of(game, k);
            let (spawn_x, spawn_y) = (*zeb.spawn_x(), *zeb.spawn_y());
            let actor = zeb.actor();
            actor.x.whole = spawn_x;
            actor.x.fraction = 0;
            actor.y.whole = spawn_y;
            actor.y.fraction = spawn_y;
            *zeb.phase() = 0;
        }
        update_animation(game);
        let mut zeb = ZebEdit::of(game, k);
        zeb.actor().properties |= 0x100;
        *zeb.cooldown() = 48;
        *zeb.state() = 0x897e;
    }
}
pub fn cooldown<H: Lease>(k: u16, game: &mut H) {
    let mut zeb = ZebEdit::of(game, k);
    let previous = *zeb.cooldown();
    *zeb.cooldown() = previous.wrapping_sub(1);
    if previous == 1 {
        *zeb.state() = 0x8890;
    }
}
