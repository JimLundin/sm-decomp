//! Powamp, Maridia's balloon plant: its body inflates, rises with a wiggle and sinks, and
//! bursts into spikes when killed; its balloon, the previous slot, is drawn above it
//! (bank $A8:C143-$A8:C6B2).
#![forbid(unsafe_code)]
use crate::actors::api::DeathKind;
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Death, Shot};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_lists::floater_spike_instruction_list;

slot_view! {
    /// A Powamp body's slot. Its Y velocity is a whole word over a subword; `dying` is set
    /// once it takes fatal damage.
    Powamp, PowampEdit {
        record: [
            y_velocity = var_a, y_subvelocity = var_b, wiggle_index = var_c, wiggle_timer = var_d,
            function_timer = var_e, function = var_f, dying = parameter_2,
        ],
        extended: [],
    }
}
slot_view! {
    /// A Powamp balloon's slot: its spawn position and the height a grappled body rises to.
    Balloon, BalloonEdit {
        record: [
            spawn_x = var_a, spawn_y = var_b, grappled_rise = var_d, function = var_f,
            balloon = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [],
    }
}

const DEFLATED_RESTING: u16 = 0xc283;
const INFLATING: u16 = 0xc2a6;
const RISING: u16 = 0xc2cf;
const FINISH_WIGGLE: u16 = 0xc36b;
const GRAPPLED_RISING: u16 = 0xc3e1;
const GRAPPLED_FINISH_WIGGLE: u16 = 0xc469;
const GRAPPLED_RESTING: u16 = 0xc4dc;
const DEFLATING: u16 = 0xc500;
const SINKING: u16 = 0xc51d;
const BALLOON: u16 = 0xc568; // An RTL: the balloon has no function.
const FATAL_DAMAGE: u16 = 0xc569;
const DEATH_SEQUENCE: u16 = 0xc59f;
/// The body's wiggle X offsets, by wiggle index.
const WIGGLE: [u16; 12] = [0, 1, 2, 3, 2, 1, 0, 0xffff, 0xfffe, 0xfffd, 0xfffe, 0xffff];
/// The body is the slot after its balloon.
const BALLOON_SLOT: u16 = 64;

/// Spawn powamp spike enemy projectiles ($A8:C223), in all eight directions.
fn spawn_spikes(game: &mut impl Lease, k: u16) {
    for direction in (0..=7).rev() {
        game.spawn_projectile(direction, k, 0xd298);
    }
}
/// Set powamp balloon Y position ($A8:C234): over the body, by the balloon's inflation.
fn place_balloon(game: &mut impl Lease, body: u16) {
    let balloon = body - BALLOON_SLOT;
    let list = Balloon::of(game, balloon).actor().instruction;
    let y = balloon_offset(list).wrapping_add(Powamp::of(game, body).actor().y.whole);
    BalloonEdit::of(game, balloon).actor().y.whole = y;
}
/// Instruction lists - balloon - inflate ($A8:C183) and deflate ($A8:C191).
pub const INFLATE: u16 = 0xc183;
pub const DEFLATE: u16 = 0xc191;
/// Balloon Y offsets by animation frame into the inflate ($A8:C277) and deflate ($A8:C27D)
/// instruction lists: -12, -16, -20 and back.
const INFLATE_OFFSETS: [u16; 3] = [0xfff4, 0xfff0, 0xffec];
const DEFLATE_OFFSETS: [u16; 3] = [0xffec, 0xfff0, 0xfff4];
/// The balloon's Y offset from the body for its instruction list pointer `list`
/// ($A8:C234): by its frame into the inflate list, or into the deflate list from $C191
/// on, and the first offset past the third frame.
pub fn balloon_offset(list: u16) -> u16 {
    let (start, offsets) = match list.wrapping_sub(DEFLATE) & 0x8000 != 0 {
        true => (INFLATE, INFLATE_OFFSETS),
        false => (DEFLATE, DEFLATE_OFFSETS),
    };
    let frame = list.wrapping_sub(4).wrapping_sub(start) >> 1;
    let frame = if frame.wrapping_sub(6) & 0x8000 == 0 {
        0
    } else {
        frame
    };
    offsets[usize::from(frame >> 1)]
}
/// Wiggle the body each 5 frames; true only when a requested stop at the centre is reached.
fn wiggle(game: &mut impl Lease, body: u16, stop_at_centre: bool) -> bool {
    let expired = Powamp::of(game, body).wiggle_timer() == 1;
    {
        let mut part = PowampEdit::of(game, body);
        let old = *part.wiggle_timer();
        *part.wiggle_timer() = old.wrapping_sub(1);
    }
    if !expired && Powamp::of(game, body).wiggle_timer() & 0x8000 == 0 {
        return false;
    }
    *PowampEdit::of(game, body).wiggle_timer() = 5;
    let balloon = body - BALLOON_SLOT;
    let spawn_x = Balloon::of(game, balloon).spawn_x();
    let x = WIGGLE[Powamp::of(game, body).wiggle_index() as usize].wrapping_add(spawn_x);
    BalloonEdit::of(game, balloon).actor().x.whole = x;
    PowampEdit::of(game, body).actor().x.whole = x;
    if stop_at_centre && centred(game, body) {
        return true;
    }
    let index = Powamp::of(game, body).wiggle_index();
    let wrap = match stop_at_centre {
        true => (index.wrapping_add(1) as i32 - 12) & 0x8000 == 0,
        false => (index as i32 - 11) & 0x8000 == 0,
    };
    *PowampEdit::of(game, body).wiggle_index() = if wrap { 0 } else { index.wrapping_add(1) };
    false
}
fn centred(game: &impl Lease, body: u16) -> bool {
    matches!(Powamp::of(game, body).wiggle_index(), 0 | 6)
}
/// Move the body by its Y velocity; whether blocked.
fn move_vertically(game: &mut impl Lease, body: u16) -> bool {
    let amount = {
        // The zero acceleration stores the velocity back, refreshing the observations.
        let mut powamp = PowampEdit::of(game, body);
        let (velocity, subvelocity) = (*powamp.y_velocity(), *powamp.y_subvelocity());
        *powamp.y_subvelocity() = subvelocity;
        *powamp.y_velocity() = velocity;
        (u32::from(velocity) << 16) | u32::from(subvelocity)
    };
    game.move_vertical(body, amount as i32)
}
fn begin_deflating(game: &mut impl Lease, body: u16) {
    {
        let mut part = PowampEdit::of(game, body);
        *part.function() = DEFLATING;
        *part.function_timer() = 10;
    }
    let mut part = BalloonEdit::of(game, body - BALLOON_SLOT);
    part.actor().instruction_delay = 1;
    part.actor().instruction = DEFLATE;
}
/// A lease on a game with the powamps' effects.
pub trait Lease: BossLease + SlotState {}
impl<T: BossLease + SlotState + ?Sized> Lease for T {}

/// Main AI - powamp ($A8:C21C): run its function.
pub fn main_ai(game: &mut impl Lease) {
    let k = game.current_enemy();
    let function = Powamp::of(game, k).function();
    run(game, function, k);
}
/// Powamp function - body, by `function`; then set the balloon's Y position.
pub fn run(game: &mut impl Lease, mut function: u16, k: u16) {
    let body = k;
    let balloon = body - BALLOON_SLOT;
    let grappled = |game: &mut _| Powamp::of(game, body).actor().ai_handlers & 1 != 0;
    if function == RISING && grappled(game) {
        *PowampEdit::of(game, body).function() = GRAPPLED_RISING;
        function = GRAPPLED_RISING;
    }
    match function {
        DEFLATED_RESTING | INFLATING | DEFLATING | DEATH_SEQUENCE => {
            let expired = Powamp::of(game, body).function_timer() == 1;
            {
                let mut part = PowampEdit::of(game, body);
                let old = *part.function_timer();
                *part.function_timer() = old.wrapping_sub(1);
            }
            if !expired && Powamp::of(game, body).function_timer() & 0x8000 == 0 {
                return place_balloon(game, body);
            }
            match function {
                DEFLATED_RESTING => {
                    {
                        let mut part = BalloonEdit::of(game, balloon);
                        part.actor().instruction_delay = 1;
                        part.actor().instruction = INFLATE;
                    }
                    let mut part = PowampEdit::of(game, body);
                    *part.function() = INFLATING;
                    *part.function_timer() = 10;
                }
                INFLATING => {
                    {
                        let mut part = PowampEdit::of(game, body);
                        *part.function() = RISING;
                        *part.y_velocity() = 0xffff;
                        *part.y_subvelocity() = 0x8000;
                    }
                    // Instruction list - body - fast ($A8:C163).
                    let mut part = PowampEdit::of(game, body);
                    part.actor().instruction_delay = 1;
                    part.actor().instruction = 0xc163;
                }
                DEFLATING => {
                    let mut part = PowampEdit::of(game, body);
                    *part.function() = SINKING;
                    *part.y_velocity() = 1;
                    *part.y_subvelocity() = 0;
                }
                _ => {
                    BalloonEdit::of(game, balloon).actor().health = 0;
                    BalloonEdit::of(game, balloon).actor().properties |= 0x200;
                    spawn_spikes(game, k);
                    return game.kill_current(DeathKind::Normal { animation: 0 });
                }
            }
        }
        RISING | GRAPPLED_RISING => {
            let grappled_rise = function == GRAPPLED_RISING;
            if grappled_rise && !grappled(game) {
                *PowampEdit::of(game, body).function() = FINISH_WIGGLE;
                return;
            }
            wiggle(game, body, false);
            let reached = move_vertically(game, body) || {
                let home = Balloon::of(game, balloon).spawn_y();
                let rise = if grappled_rise {
                    Balloon::of(game, balloon).grappled_rise()
                } else {
                    0x40
                };
                (home as i32 - rise as i32 - Powamp::of(game, body).actor().y.whole as i32) as i16
                    >= 0
            };
            if reached {
                match (centred(game, body), grappled_rise) {
                    (true, true) => *PowampEdit::of(game, body).function() = GRAPPLED_RESTING,
                    (true, false) => begin_deflating(game, body),
                    (false, true) => {
                        *PowampEdit::of(game, body).function() = GRAPPLED_FINISH_WIGGLE
                    }
                    (false, false) => *PowampEdit::of(game, body).function() = FINISH_WIGGLE,
                }
            }
        }
        FINISH_WIGGLE | GRAPPLED_FINISH_WIGGLE => {
            let grappled_wiggle = function == GRAPPLED_FINISH_WIGGLE;
            if grappled_wiggle && !grappled(game) {
                *PowampEdit::of(game, body).function() = FINISH_WIGGLE;
                return;
            }
            match (wiggle(game, body, true), grappled_wiggle) {
                (true, true) => *PowampEdit::of(game, body).function() = GRAPPLED_RESTING,
                (true, false) => begin_deflating(game, body),
                (false, _) => _ = move_vertically(game, body),
            }
        }
        GRAPPLED_RESTING => {
            if !grappled(game) {
                begin_deflating(game, body);
            }
        }
        SINKING => {
            move_vertically(game, body);
            let home = Balloon::of(game, balloon).spawn_y();
            if (Powamp::of(game, body).actor().y.whole as i16).wrapping_sub(home as i16) >= 0 {
                PowampEdit::of(game, body).actor().y.whole = home;
                {
                    let mut part = PowampEdit::of(game, body);
                    *part.function() = DEFLATED_RESTING;
                    *part.function_timer() = 60;
                }
                // Instruction list - body - slow ($A8:C173).
                let mut part = PowampEdit::of(game, body);
                part.actor().instruction_delay = 1;
                part.actor().instruction = 0xc173;
            }
        }
        FATAL_DAMAGE => {
            let list = Balloon::of(game, balloon).actor().instruction;
            if (list as i32 + 0x3e6f) & 0x8000 == 0 {
                let index = (list as i32 - 4 + 15983) as u16 >> 1;
                if index != 0 {
                    let list = floater_spike_instruction_list(game, (index >> 1) as isize);
                    let mut part = BalloonEdit::of(game, balloon);
                    part.actor().instruction = list;
                    part.actor().instruction_delay = 1;
                }
            }
            let mut part = PowampEdit::of(game, body);
            *part.function() = DEATH_SEQUENCE;
            *part.function_timer() = 32;
        }
        BALLOON => return,
        _ => game.unreachable(),
    }
    place_balloon(game, body);
}
/// Initialisation AI - powamp ($A8:C1C9): the balloon or the body.
pub fn initialize(game: &mut impl Lease) {
    let powamp = game.current_enemy();
    PowampEdit::of(game, powamp).actor().properties |= 0x2000;
    {
        let mut part = PowampEdit::of(game, powamp);
        part.actor().spritemap = 0x804d;
        part.actor().instruction_delay = 1;
        part.actor().timer = 0;
    }
    if Balloon::of(game, powamp).balloon() != 0 {
        let (x, y) = (
            Balloon::of(game, powamp).actor().x.whole,
            Balloon::of(game, powamp).actor().y.whole,
        );
        {
            let mut part = BalloonEdit::of(game, powamp);
            *part.spawn_x() = x;
            *part.spawn_y() = y;
            *part.function() = BALLOON;
        }
        BalloonEdit::of(game, powamp).actor().instruction = 0xc199;
        let height = Balloon::of(game, powamp).parameter_2();
        *BalloonEdit::of(game, powamp).grappled_rise() = height;
    } else {
        {
            let mut part = PowampEdit::of(game, powamp);
            *part.function_timer() = 60;
            *part.function() = DEFLATED_RESTING;
        }
        PowampEdit::of(game, powamp).actor().instruction = 0xc173;
    }
}
/// Enemy touch - powamp ($A8:C5BE): killed, it bursts into spikes.
pub fn touch(game: &mut impl Lease) {
    let body = game.current_enemy();
    if Powamp::of(game, body).dying() != 0 {
        return;
    }
    let palette = Powamp::of(game, body).actor().palette;
    game.contact_reaction(Death::Finalize);
    if Powamp::of(game, body).actor().health == 0 {
        let balloon = game.current_enemy() - BALLOON_SLOT;
        BalloonEdit::of(game, balloon).actor().properties |= 0x200;
        PowampEdit::of(game, body).actor().palette = palette;
        spawn_spikes(game, game.current_enemy());
        PowampEdit::of(game, body).actor().palette = 2560;
    }
}
/// Enemy shot - powamp ($A8:C5EF): the balloon shares its frozen and flash state.
pub fn shot(game: &mut impl Lease) {
    let body = game.current_enemy();
    if Powamp::of(game, body).dying() != 0 {
        return;
    }
    game.shot_reaction(Shot::ReactionOnly);
    let balloon = game.current_enemy() - BALLOON_SLOT;
    if Powamp::of(game, body).actor().ai_handlers & 4 != 0 {
        let frozen = Powamp::of(game, body).actor().frozen;
        BalloonEdit::of(game, balloon).actor().frozen = frozen;
        BalloonEdit::of(game, balloon).actor().ai_handlers |= 4;
    }
    if Powamp::of(game, body).actor().ai_handlers & 2 != 0 {
        let flash = Powamp::of(game, body).actor().flash;
        BalloonEdit::of(game, balloon).actor().flash = flash;
        BalloonEdit::of(game, balloon).actor().ai_handlers |= 2;
    }
    if Powamp::of(game, body).actor().health == 0 {
        let mut part = PowampEdit::of(game, body);
        *part.function() = FATAL_DAMAGE;
        *part.dying() = 1;
    }
}
/// Power bomb reaction - powamp ($A8:C63F): the balloon shares its reaction state, or is
/// deleted with it.
pub fn power_bomb(game: &mut impl Lease) {
    game.power_bomb_reaction(Death::Finalize);
    let body = game.current_enemy();
    let balloon = body - BALLOON_SLOT;
    if Powamp::of(game, body).actor().health == 0 {
        BalloonEdit::of(game, balloon).actor().properties |= 0x200;
        return;
    }
    let (shake, invincibility) = (
        Powamp::of(game, body).actor().shake,
        Powamp::of(game, body).actor().invincibility,
    );
    let (flash, frozen) = (
        Powamp::of(game, body).actor().flash,
        Powamp::of(game, body).actor().frozen,
    );
    let bits = Powamp::of(game, body).actor().ai_handlers;
    let mut part = BalloonEdit::of(game, balloon);
    part.actor().shake = shake;
    part.actor().invincibility = invincibility;
    part.actor().flash = flash;
    part.actor().frozen = frozen;
    part.actor().ai_handlers = bits;
}
