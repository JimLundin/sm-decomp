//! Enemy species dispatch tables keyed by the routine addresses the original
//! enemy definitions name: AI, pre-instructions, instructions, contact and
//! shot reactions, graphics draw hooks, bank $B3 pre-instructions and pipe bug
//! state routines.
//!
//! Each table is one `match` from address to routine. Arms whose routines
//! still run in the compatibility crate go to its host: a named native enum
//! for the small tables, the same address for the large ones. Moving a
//! routine here means moving its arm.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Death, LeaseActors, Shot};
use crate::actors::{
    botwoon, brinstar_pipe_bug, crocomire_reactions, draygon_death, draygon_flight, draygon_host,
    escape_critters, magdollite, norfair_pipe_bug, space_pirates, yapping_maw, yellow_pipe_bug,
};
use crate::enemy_script::EnemyScript;
use crate::ram::word;

/// The failure every table shares.
pub trait SpeciesHost {
    /// The original failure for an address no routine answers.
    fn unknown_routine(&mut self);
}

/// What the bank $B3 routines need beyond their actors' own hosts.
pub trait B3Host:
    SpeciesHost
    + ActorHost
    + ActorRuntime
    + LeaseActors
    + botwoon::Host
    + escape_critters::Host
    + crate::records::Records
{
}

impl<T> B3Host for T where
    T: SpeciesHost
        + ActorHost
        + ActorRuntime
        + LeaseActors
        + botwoon::Host
        + escape_critters::Host
        + crate::records::Records
        + ?Sized
{
}

/// Contact reactions whose routines still run in the compatibility crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTouch {
    LargeSnail { damage: bool },
    SporeSpawn,
    CeresSteam,
    KraidArmAndFoot,
    KraidArm,
    Phantoon,
    MotherBrainBrain,
    GoldenTorizo,
}

/// What the contact reactions need beyond their actors' own hosts.
pub trait TouchHost:
    SpeciesHost + ActorRuntime + crocomire_reactions::Host + draygon_host::Host + space_pirates::Pirates
{
    fn native_touch(&mut self, touch: NativeTouch);
}

/// An enemy species' contact reaction, keyed by its address.
pub fn touch<H: TouchHost + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        10518563 => game.contact_reaction(Death::Finalize),
        10670984 => game.native_touch(NativeTouch::LargeSnail { damage: true }),
        10670988 => game.native_touch(NativeTouch::LargeSnail { damage: false }),
        10780707 => game.contact_reaction(Death::Finalize),
        10795325 => crocomire_reactions::touch(game),
        10795344 => (),
        10846284 => (),
        10851818 => draygon_death::touch(game),
        10874348 => game.native_touch(NativeTouch::SporeSpawn),
        // Ridley's contact leaves the death pending.
        10936153 => game.contact_reaction(Death::LeavePending),
        10940479 => game.native_touch(NativeTouch::CeresSteam),
        10977356 => (),
        10982539 => game.native_touch(NativeTouch::KraidArmAndFoot),
        10982544 => game.native_touch(NativeTouch::KraidArm),
        11001237 => game.native_touch(NativeTouch::Phantoon),
        11122117 => (),
        11122118 => game.native_touch(NativeTouch::MotherBrainBrain),
        11192695 => game.native_touch(NativeTouch::GoldenTorizo),
        11700076 => space_pirates::touch(game),
        _ => game.unknown_routine(),
    }
}

/// Shot reactions whose routines still run in the compatibility crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeShot {
    Dud,
    LargeSnail,
    SporeSpawn,
    Ridley,
    /// Kraid's arm, with the projectile that hit it.
    KraidArm {
        projectile: u16,
    },
    Phantoon,
    MotherBrainBody,
    MotherBrainBrain,
    Torizo,
    /// The Torizo shot reaction that only runs outside Crateria.
    TorizoOutsideCrateria,
}

/// What the shot reactions need beyond their actors' own hosts.
pub trait ShotHost:
    SpeciesHost + ActorRuntime + crocomire_reactions::Host + draygon_host::Host + space_pirates::Pirates
{
    fn native_shot(&mut self, shot: NativeShot);
}

/// An enemy species' shot reaction, keyed by its address; `j` is the projectile.
pub fn shot<H: ShotHost + ?Sized>(ea: u32, j: u16, game: &mut H) {
    match ea {
        10518573 => game.shot_reaction(Shot::Finalize),
        10846278 => game.native_shot(NativeShot::Dud),
        10649676 => (),
        10671028 => game.native_shot(NativeShot::LargeSnail),
        10780717 => game.shot_reaction(Shot::Finalize),
        10795345 => crocomire_reactions::count_hit(game),
        10795368 => crocomire_reactions::impact(game),
        10795525 => crocomire_reactions::hit(game),
        10795700 => crocomire_reactions::impact(game),
        10846253 => game.shot_reaction(Shot::Finalize),
        10846284 => (),
        10851824 => draygon_death::shot(game),
        10874202 => game.native_shot(NativeShot::SporeSpawn),
        10911820 => (),
        10936202 => game.native_shot(NativeShot::Ridley),
        10977356 => (),
        10982581 => (),
        10982582 => game.native_shot(NativeShot::KraidArm { projectile: j }),
        11001243 => game.native_shot(NativeShot::Phantoon),
        11121923 => game.native_shot(NativeShot::MotherBrainBody),
        11121927 => game.native_shot(NativeShot::MotherBrainBrain),
        11192700 => game.native_shot(NativeShot::Torizo),
        11192769 => (),
        11192770 => game.native_shot(NativeShot::TorizoOutsideCrateria),
        11700089 => space_pirates::shot(game),
        11700168 => space_pirates::check_shot(game),
        11700286 => space_pirates::reflect(game),
        _ => game.unknown_routine(),
    }
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

/// What the AI table needs beyond its actors' own hosts.
pub trait AiHost:
    B3Host + DrawHookHost + yapping_maw::Host + crocomire_reactions::Host + space_pirates::Pirates
{
    /// An AI routine whose arm is still in the compatibility table.
    fn native_ai(&mut self, ea: u32);
}

/// An enemy species' AI routine, keyed by its address. Arms not here are in
/// the compatibility table until their routines move.
pub fn ai<H: AiHost + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        11763747 => game.contact_reaction(Death::Finalize),
        11763757 => game.shot_reaction(Shot::Finalize),
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871
        | 11108407 | 11173943 | 11698231 | 11763767 => game.power_bomb_reaction(Death::Finalize),
        10715212 => (),
        10725800 => game.shot_reaction(Shot::Finalize),
        10723079 => (),
        10715171 => game.contact_reaction(Death::Finalize),
        10715181 => game.shot_reaction(Shot::Finalize),
        10738715 => (),
        10780748 => (),
        10795344 => (),
        10811067 => (),
        10846284 => (),
        10851824 => draygon_death::shot(game),
        10864042 => (),
        10864068 => (),
        10874354 => (),
        10911820 => (),
        10942752 => (),
        10977356 => (),
        10982543 => (),
        10982581 => (),
        11001242 => (),
        11001873 => (),
        11042892 => (),
        11051336 => yapping_maw::initialize(game),
        11052953 => yapping_maw::touch(game),
        11052989 => yapping_maw::shot(game),
        11053109 => yapping_maw::frozen(game),
        11054987 => magdollite::initialize(game),
        11056128 | 11056134 | 11056140 => magdollite::damage(ea, game),
        11062374 => (),
        11108428 => (),
        11122117 => (),
        11173964 => (),
        11196359 => (),
        11200475 => (),
        11200476 => (),
        11698252 => (),
        11763788 => (),
        11765518 => (),
        11765819 => brinstar_pipe_bug::initialize(game),
        11766625 => norfair_pipe_bug::initialize(game),
        11767628 => yellow_pipe_bug::initialize(game),
        11769219 => botwoon::initialize(game),
        11790027 => escape_critters::initialize(false, game),
        11791077 => escape_critters::initialize(true, game),
        11791130 => (),
        10649635 => game.contact_reaction(Death::Finalize),
        10649645 => game.shot_reaction(Shot::Finalize),
        10782343 => crocomire_reactions::hurt(game),
        10811002 => crocomire_reactions::initialize_tongue(game),
        10780707 => game.contact_reaction(Death::Finalize),
        10780717 => game.shot_reaction(Shot::Finalize),
        10783322 => crocomire_reactions::initialize(game),
        10795410 => crocomire_reactions::power_bomb(game),
        // Bank $B2: the space pirates.
        0xb20000..=0xb2ffff => space_pirates::dispatch_ai(ea, game),
        _ => game.native_ai(ea),
    }
}

/// What the pre-instruction table needs beyond its actors' own hosts.
pub trait PreInstructionHost: B3Host + magdollite::Host {
    /// A pre-instruction whose arm is still in the compatibility table, for
    /// the current enemy `k` read before dispatch.
    fn native_pre_instruction(&mut self, ea: u32, k: u16);
}

/// An enemy species' pre-instruction, keyed by its address. The current
/// enemy index is read once, before dispatch, as the original did.
pub fn pre_instruction<H: PreInstructionHost + ?Sized>(ea: u32, game: &mut H) {
    let k: u16 = word!(game.cur_enemy_index);
    match ea {
        10518651 => (),
        10649723 => (),
        10652277 => (),
        10657819 => (),
        10659799 => (),
        10676097 => (),
        10730689 => (),
        10735455 => (),
        10739850 => (),
        10741441 => (),
        10998952 => (),
        11047388 => (),
        11055386 | 11055477 | 11055507 | 11055544 | 11055581 | 11055885 | 11055903 | 11055958
        | 11056039 => magdollite::run_state(ea as u16, game),
        11055761 => (),
        11055765 => magdollite::shrink_arm(game),
        11192670 => (),
        11197610 => (),
        11200422 => (),
        11174011 => (),
        11200474 => (),
        11197611 => (),
        0xb30000..=0xb3ffff => crate::actors::species::b3_pre_instruction(ea, k, game),
        _ => game.native_pre_instruction(ea, k),
    }
}

/// What the instruction table needs beyond its actors' own hosts.
pub trait InstructionHost:
    crocomire_reactions::Host
    + yapping_maw::Host
    + magdollite::Host
    + escape_critters::Host
    + space_pirates::Pirates
{
    /// An instruction whose arm is still in the compatibility table.
    fn native_instruction(&mut self, ea: u32, k: u16, j: EnemyScript) -> EnemyScript;
}

/// An enemy species' instruction, keyed by its address; `k` is the enemy and
/// `j` the script position.
pub fn instruction<H: InstructionHost + ?Sized>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    match ea {
        0xa40000..=0xa4ffff => crocomire_reactions::instruction(ea, k, j, game),
        11051207 | 11051225 | 11051243 | 11051261 | 11051279 | 11051297 | 11051315 => {
            yapping_maw::instruction(ea, j, game)
        }
        11054610 | 11054630 | 11054640 | 11054650 | 11054661 | 11054672 | 11054682 | 11054692
        | 11054728 | 11054742 | 11054778 | 11054794 | 11054820 | 11054846 | 11054872 | 11054916 => {
            magdollite::instruction(ea, j, game)
        }
        11789637 | 11791016 => escape_critters::go_to_if(false, j, game),
        11791032 => escape_critters::go_to_if(true, j, game),
        11789840 => escape_critters::move_x(k, j, game),
        11791049 => escape_critters::shift_dachora(0xfffa, j, game),
        11791063 => escape_critters::shift_dachora(6, j, game),
        0xb20000..=0xb2ffff => space_pirates::dispatch_instruction(ea, k, j, game),
        _ => game.native_instruction(ea, k, j),
    }
}
