//! Powamp, Maridia's balloon plant: its body inflates, rises with a wiggle and sinks, and
//! bursts into spikes when killed; its balloon, the previous slot, is drawn above it
//! (bank $A8:C143-$A8:C6B2). Was `maridia_floater`.
#![forbid(unsafe_code)]
use crate::actors::api::DeathKind;
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{BossEffects, Death, Shot};
use crate::actors::layouts::part_view;
use crate::enemy_lists::floater_spike_instruction_list;
use crate::session_slot;

part_view! {
    /// A Powamp body's slot. Its Y velocity is a whole word over a subword; `dying` is set
    /// once it takes fatal damage.
    Powamp: EnemyLayout8 {
        words: [y_velocity, y_subvelocity, wiggle_index, wiggle_timer, function_timer, function, parameter_1, dying],
    }
}
part_view! {
    /// A Powamp balloon's slot: its spawn position and the height a grappled body rises to.
    Balloon: EnemyLayout8 {
        words: [spawn_x, spawn_y, var_c, grappled_rise, var_e, function, balloon, parameter_2],
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
const BALLOON_SLOT: i32 = 64;

/// Spawn powamp spike enemy projectiles ($A8:C223), in all eight directions.
fn spawn_spikes(game: &mut impl Lease, k: u16) {
    for direction in (0..=7).rev() {
        game.spawn_projectile(direction, k, 0xd298);
    }
}
/// Set powamp balloon Y position ($A8:C234): over the body, by the balloon's inflation.
fn place_balloon(game: &mut impl Lease, body: i32) {
    let balloon = body - BALLOON_SLOT;
    let list = session_slot!(game, Balloon[balloon].base.current_instruction);
    let y = balloon_offset(list).wrapping_add(session_slot!(game, Powamp[body].base.y_pos));
    session_slot!(game, Balloon[balloon].base.y_pos = y);
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
fn wiggle(game: &mut impl Lease, body: i32, stop_at_centre: bool) -> bool {
    let expired = session_slot!(game, Powamp[body].wiggle_timer) == 1;
    session_slot!(game, Powamp[body].wiggle_timer, |old| old.wrapping_sub(1));
    if !expired && session_slot!(game, Powamp[body].wiggle_timer) & 0x8000 == 0 {
        return false;
    }
    session_slot!(game, Powamp[body].wiggle_timer = 5);
    let balloon = body - BALLOON_SLOT;
    let spawn_x = session_slot!(game, Balloon[balloon].spawn_x);
    let x = WIGGLE[session_slot!(game, Powamp[body].wiggle_index) as usize].wrapping_add(spawn_x);
    session_slot!(game, Balloon[balloon].base.x_pos = x);
    session_slot!(game, Powamp[body].base.x_pos = x);
    if stop_at_centre && centred(game, body) {
        return true;
    }
    let index = session_slot!(game, Powamp[body].wiggle_index);
    let wrap = match stop_at_centre {
        true => (index.wrapping_add(1) as i32 - 12) & 0x8000 == 0,
        false => (index as i32 - 11) & 0x8000 == 0,
    };
    session_slot!(
        game,
        Powamp[body].wiggle_index = if wrap { 0 } else { index.wrapping_add(1) }
    );
    false
}
fn centred(game: &impl Lease, body: i32) -> bool {
    matches!(session_slot!(game, Powamp[body].wiggle_index), 0 | 6)
}
/// Move the body by its Y velocity; whether blocked.
fn move_vertically(game: &mut impl Lease, body: i32) -> bool {
    // The zero acceleration still reads and publishes the split native words.
    session_slot!(game, Powamp[body](y_velocity, y_subvelocity) += 0);
    let amount = session_slot!(game, Powamp[body](y_velocity, y_subvelocity));
    game.move_vertical(body as u16, amount as i32)
}
fn begin_deflating(game: &mut impl Lease, body: i32) {
    session_slot!(game, Powamp[body]; function = DEFLATING, function_timer = 10);
    session_slot!(game, Balloon[body - BALLOON_SLOT].base; instruction_timer = 1, current_instruction = DEFLATE);
}
/// A lease on a game with the powamps' effects.
pub trait Lease: BossLease<Game: BossEffects> {}
impl<T: BossLease<Game: BossEffects> + ?Sized> Lease for T {}

/// Main AI - powamp ($A8:C21C): run its function.
pub fn main_ai(game: &mut impl Lease) {
    let k = game.current_enemy();
    let function = session_slot!(game, Powamp[i32::from(k)].function);
    run(game, function, k);
}
/// Powamp function - body, by `function`; then set the balloon's Y position.
pub fn run(game: &mut impl Lease, mut function: u16, k: u16) {
    let body = i32::from(k);
    let balloon = body - BALLOON_SLOT;
    let grappled = |game: &mut _| session_slot!(game, Powamp[body].base.ai_handler_bits) & 1 != 0;
    if function == RISING && grappled(game) {
        session_slot!(game, Powamp[body].function = GRAPPLED_RISING);
        function = GRAPPLED_RISING;
    }
    match function {
        DEFLATED_RESTING | INFLATING | DEFLATING | DEATH_SEQUENCE => {
            let expired = session_slot!(game, Powamp[body].function_timer) == 1;
            session_slot!(game, Powamp[body].function_timer, |old| old.wrapping_sub(1));
            if !expired && session_slot!(game, Powamp[body].function_timer) & 0x8000 == 0 {
                return place_balloon(game, body);
            }
            match function {
                DEFLATED_RESTING => {
                    session_slot!(game, Balloon[balloon].base; instruction_timer = 1, current_instruction = INFLATE);
                    session_slot!(game, Powamp[body]; function = INFLATING, function_timer = 10);
                }
                INFLATING => {
                    session_slot!(game, Powamp[body]; function = RISING, y_velocity = 0xffff, y_subvelocity = 0x8000);
                    // Instruction list - body - fast ($A8:C163).
                    session_slot!(game, Powamp[body].base; instruction_timer = 1, current_instruction = 0xc163);
                }
                DEFLATING => {
                    session_slot!(game, Powamp[body]; function = SINKING, y_velocity = 1, y_subvelocity = 0);
                }
                _ => {
                    session_slot!(game, Balloon[balloon].base.health = 0);
                    session_slot!(game, Balloon[balloon].base.properties, |old| old | 0x200);
                    spawn_spikes(game, k);
                    return game.kill_current(DeathKind::Normal { animation: 0 });
                }
            }
        }
        RISING | GRAPPLED_RISING => {
            let grappled_rise = function == GRAPPLED_RISING;
            if grappled_rise && !grappled(game) {
                return session_slot!(game, Powamp[body].function = FINISH_WIGGLE);
            }
            wiggle(game, body, false);
            let reached = move_vertically(game, body) || {
                let home = session_slot!(game, Balloon[balloon].spawn_y);
                let rise = if grappled_rise {
                    session_slot!(game, Balloon[balloon].grappled_rise)
                } else {
                    0x40
                };
                (home as i32 - rise as i32 - session_slot!(game, Powamp[body].base.y_pos) as i32)
                    as i16
                    >= 0
            };
            if reached {
                match (centred(game, body), grappled_rise) {
                    (true, true) => session_slot!(game, Powamp[body].function = GRAPPLED_RESTING),
                    (true, false) => begin_deflating(game, body),
                    (false, true) => {
                        session_slot!(game, Powamp[body].function = GRAPPLED_FINISH_WIGGLE)
                    }
                    (false, false) => session_slot!(game, Powamp[body].function = FINISH_WIGGLE),
                }
            }
        }
        FINISH_WIGGLE | GRAPPLED_FINISH_WIGGLE => {
            let grappled_wiggle = function == GRAPPLED_FINISH_WIGGLE;
            if grappled_wiggle && !grappled(game) {
                return session_slot!(game, Powamp[body].function = FINISH_WIGGLE);
            }
            match (wiggle(game, body, true), grappled_wiggle) {
                (true, true) => session_slot!(game, Powamp[body].function = GRAPPLED_RESTING),
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
            let home = session_slot!(game, Balloon[balloon].spawn_y);
            if (session_slot!(game, Powamp[body].base.y_pos) as i16).wrapping_sub(home as i16) >= 0
            {
                session_slot!(game, Powamp[body].base.y_pos = home);
                session_slot!(game, Powamp[body]; function = DEFLATED_RESTING, function_timer = 60);
                // Instruction list - body - slow ($A8:C173).
                session_slot!(game, Powamp[body].base; instruction_timer = 1, current_instruction = 0xc173);
            }
        }
        FATAL_DAMAGE => {
            let list = session_slot!(game, Balloon[balloon].base.current_instruction);
            if (list as i32 + 0x3e6f) & 0x8000 == 0 {
                let index = (list as i32 - 4 + 15983) as u16 >> 1;
                if index != 0 {
                    let list = floater_spike_instruction_list(game, (index >> 1) as isize);
                    session_slot!(game, Balloon[balloon].base; current_instruction = list, instruction_timer = 1);
                }
            }
            session_slot!(game, Powamp[body]; function = DEATH_SEQUENCE, function_timer = 32);
        }
        BALLOON => return,
        _ => return game.foreign(|game| game.unreachable()),
    }
    place_balloon(game, body);
}
/// Initialisation AI - powamp ($A8:C1C9): the balloon or the body.
pub fn initialize(game: &mut impl Lease) {
    let powamp = i32::from(game.current_enemy());
    session_slot!(game, Powamp[powamp].base.properties, |old| old | 0x2000);
    session_slot!(game, Powamp[powamp].base; spritemap_pointer = 0x804d, instruction_timer = 1, timer = 0);
    if session_slot!(game, Balloon[powamp].balloon) != 0 {
        let (x, y) = (
            session_slot!(game, Balloon[powamp].base.x_pos),
            session_slot!(game, Balloon[powamp].base.y_pos),
        );
        session_slot!(game, Balloon[powamp]; spawn_x = x, spawn_y = y, function = BALLOON);
        session_slot!(game, Balloon[powamp].base.current_instruction = 0xc199);
        let height = session_slot!(game, Balloon[powamp].parameter_2);
        session_slot!(game, Balloon[powamp].grappled_rise = height);
    } else {
        session_slot!(game, Powamp[powamp]; function_timer = 60, function = DEFLATED_RESTING);
        session_slot!(game, Powamp[powamp].base.current_instruction = 0xc173);
    }
}
/// Enemy touch - powamp ($A8:C5BE): killed, it bursts into spikes.
pub fn touch(game: &mut impl Lease) {
    let body = i32::from(game.current_enemy());
    if session_slot!(game, Powamp[body].dying) != 0 {
        return;
    }
    let palette = session_slot!(game, Powamp[body].base.palette_index);
    game.contact_reaction(Death::Finalize);
    if session_slot!(game, Powamp[body].base.health) == 0 {
        let balloon = i32::from(game.current_enemy()) - BALLOON_SLOT;
        session_slot!(game, Balloon[balloon].base.properties, |old| old | 0x200);
        session_slot!(game, Powamp[body].base.palette_index = palette);
        spawn_spikes(game, game.current_enemy());
        session_slot!(game, Powamp[body].base.palette_index = 2560);
    }
}
/// Enemy shot - powamp ($A8:C5EF): the balloon shares its frozen and flash state.
pub fn shot(game: &mut impl Lease) {
    let body = i32::from(game.current_enemy());
    if session_slot!(game, Powamp[body].dying) != 0 {
        return;
    }
    game.shot_reaction(Shot::ReactionOnly);
    let balloon = i32::from(game.current_enemy()) - BALLOON_SLOT;
    if session_slot!(game, Powamp[body].base.ai_handler_bits) & 4 != 0 {
        let frozen = session_slot!(game, Powamp[body].base.frozen_timer);
        session_slot!(game, Balloon[balloon].base.frozen_timer = frozen);
        session_slot!(game, Balloon[balloon].base.ai_handler_bits, |old| old | 4);
    }
    if session_slot!(game, Powamp[body].base.ai_handler_bits) & 2 != 0 {
        let flash = session_slot!(game, Powamp[body].base.flash_timer);
        session_slot!(game, Balloon[balloon].base.flash_timer = flash);
        session_slot!(game, Balloon[balloon].base.ai_handler_bits, |old| old | 2);
    }
    if session_slot!(game, Powamp[body].base.health) == 0 {
        session_slot!(game, Powamp[body]; function = FATAL_DAMAGE, dying = 1);
    }
}
/// Power bomb reaction - powamp ($A8:C63F): the balloon shares its reaction state, or is
/// deleted with it.
pub fn power_bomb(game: &mut impl Lease) {
    game.power_bomb_reaction(Death::Finalize);
    let body = i32::from(game.current_enemy());
    let balloon = body - BALLOON_SLOT;
    if session_slot!(game, Powamp[body].base.health) == 0 {
        return session_slot!(game, Balloon[balloon].base.properties, |old| old | 0x200);
    }
    let (shake, invincibility) = (
        session_slot!(game, Powamp[body].base.shake_timer),
        session_slot!(game, Powamp[body].base.invincibility_timer),
    );
    let (flash, frozen) = (
        session_slot!(game, Powamp[body].base.flash_timer),
        session_slot!(game, Powamp[body].base.frozen_timer),
    );
    let bits = session_slot!(game, Powamp[body].base.ai_handler_bits);
    session_slot!(game, Balloon[balloon].base; shake_timer = shake, invincibility_timer = invincibility,
        flash_timer = flash, frozen_timer = frozen, ai_handler_bits = bits);
}
