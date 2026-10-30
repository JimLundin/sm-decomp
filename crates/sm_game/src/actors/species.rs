//! Enemy species dispatch tables keyed by the routine addresses the original
//! enemy definitions name: graphics draw hooks, bank $B3 pre-instructions and
//! pipe bug state routines.
//!
//! Each table is one `match` from address to routine. Moving another routine
//! here means adding its arm; the compat dispatcher still owns the rest.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, LeaseActors};
use crate::actors::{botwoon, draygon_flight, draygon_host, escape_critters, magdollite};
use crate::ram::word;

/// The failure every table shares.
pub trait SpeciesHost {
    /// The original failure for an address no routine answers.
    fn unknown_routine(&mut self);
}

/// What the bank $B3 routines need beyond their actors' own hosts.
pub trait B3Host:
    SpeciesHost + ActorHost + ActorRuntime + LeaseActors + botwoon::Host + escape_critters::Host
{
}

impl<T> B3Host for T where
    T: SpeciesHost
        + ActorHost
        + ActorRuntime
        + LeaseActors
        + botwoon::Host
        + escape_critters::Host
        + ?Sized
{
}

/// Graphics draw hooks whose routines still run in the compatibility crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDrawHook {
    ReflecPalette,
    CeresRidleyBabyMetroidAndWings,
    WreckedShipRobotPalette,
    BlueFaceBlockPalette,
    MotherBrainBrainAndNeck,
    MotherBrainBrain,
    DeadTorizo,
}

/// What the draw hooks need beyond their actors' own hosts.
pub trait DrawHookHost: SpeciesHost + magdollite::Host + draygon_host::Host {
    fn native_draw_hook(&mut self, hook: NativeDrawHook);
}

/// An enemy species' graphics draw hook, keyed by its address.
pub fn draw_hook<H: DrawHookHost + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        10518604 => (),
        10738444 => game.native_draw_hook(NativeDrawHook::ReflecPalette),
        10851138 => draygon_flight::set_bg2_scroll(game),
        10920690 => game.native_draw_hook(NativeDrawHook::CeresRidleyBabyMetroidAndWings),
        11042892 => (),
        11055282 => magdollite::cycle_palette(game),
        11062375 => game.native_draw_hook(NativeDrawHook::WreckedShipRobotPalette),
        11069550 => game.native_draw_hook(NativeDrawHook::BlueFaceBlockPalette),
        11110278 => (),
        11110345 => game.native_draw_hook(NativeDrawHook::MotherBrainBrainAndNeck),
        11110365 => game.native_draw_hook(NativeDrawHook::MotherBrainBrain),
        11129754 => game.native_draw_hook(NativeDrawHook::DeadTorizo),
        _ => game.unknown_routine(),
    }
}

/// A pipe bug's state routine, keyed by its bank $B3 address.
pub fn pipe_bug_state<H: B3Host + ?Sized>(ea: u32, _k: u16, game: &mut H) {
    match ea {
        11766733 => crate::actors::norfair_pipe_bug::await_group(game),
        11766783 => crate::actors::norfair_pipe_bug::await_player(game),
        11766950 => crate::actors::norfair_pipe_bug::rise(game),
        11767039 => crate::actors::norfair_pipe_bug::settle(game),
        11767052 => crate::actors::norfair_pipe_bug::move_to_limit(-16, game),
        11767118 => crate::actors::norfair_pipe_bug::move_to_limit(-32, game),
        11767184 => crate::actors::norfair_pipe_bug::move_to_limit(16, game),
        11767250 => crate::actors::norfair_pipe_bug::move_to_limit(32, game),
        11767316 => crate::actors::norfair_pipe_bug::fly(2, game),
        11767349 => crate::actors::norfair_pipe_bug::fly(0, game),
        11767386 => crate::actors::norfair_pipe_bug::start_flight(game),
        11767733 => crate::actors::yellow_pipe_bug::detect_player(game),
        11767797 => crate::actors::yellow_pipe_bug::launch_delay(word!(game.cur_enemy_index), game),
        11767848 => crate::actors::yellow_pipe_bug::fly(true, game),
        11767997 => crate::actors::yellow_pipe_bug::fly(false, game),
        11768154 => crate::actors::yellow_pipe_bug::turn(true, game),
        11768280 => crate::actors::yellow_pipe_bug::turn(false, game),
        _ => {
            game.unknown_routine();
        }
    }
}
/// A bank $B3 enemy pre-instruction, keyed by its address.
pub fn b3_pre_instruction<H: B3Host + ?Sized>(ea: u32, k: u16, game: &mut H) {
    match ea {
        11763835 => (),
        11765888 => crate::actors::brinstar_pipe_bug::await_view(k, game),
        11765904 => crate::actors::brinstar_pipe_bug::await_player(k, game),
        11765987 => crate::actors::brinstar_pipe_bug::rise(k, game),
        11766044 => crate::actors::brinstar_pipe_bug::fly(k, game),
        11766142 => crate::actors::brinstar_pipe_bug::cooldown(k, game),
        11767039 => crate::actors::norfair_pipe_bug::settle(game),
        11789916 => crate::actors::escape_critters::run_for_escape(k, game),
        11789936 => crate::actors::escape_critters::wait_to_express_gratitude(k, game),
        11789952 => crate::actors::escape_critters::run_around_aimlessly(k, game),
        11771328 => crate::actors::botwoon::head_moving_around(game),
        11771517 => {
            let address = crate::actors::botwoon::head_set_spit_angle(game);
            b3_pre_instruction(address, word!(game.cur_enemy_index), game);
        }
        11771616 => crate::actors::botwoon::head_spit(true, game),
        11771700 => crate::actors::botwoon::head_spit(false, game),
        11771770 => crate::actors::botwoon::head_cooldown(k, game),
        _ => {
            game.unknown_routine();
        }
    }
}
