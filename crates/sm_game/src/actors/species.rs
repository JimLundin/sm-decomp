//! Enemy species dispatch tables keyed by the routine addresses the original
//! enemy definitions name: AI, pre-instructions, instructions, contact and
//! shot reactions, graphics draw hooks, bank $B3 pre-instructions and pipe bug
//! state routines.
//!
//! Each table is one `match` from address to routine. Arms whose routines
//! still run in the compatibility crate go to its host: a named native enum
//! for the small tables, the same address for the large ones. Moving a
//! routine here means moving its arm.
use crate::actors::access::{AccessMode::LittleEndian, AccessMode::Native, ActorAccess as _};
use crate::actors::host::{
    ActorHost, ActorLease as _, ActorRuntime, Death, LeaseActors, LeaseScope, Shot,
};
use crate::actors::spore_spawn::instruction as spore;
use crate::actors::{
    alcoon, bang, beetom, blue_brinstar_face_block, botwoon, boyon, cacatac, choot, covern,
    crawlers, crocomire_reactions, dragon, draygon_death, draygon_eye, draygon_flight,
    draygon_grab, draygon_host, draygon_sweep, elevator, escape_critters, evir, fire_pillar, flies,
    fune, geega, geruta, gunship, kago, magdollite, metroid, mini_crocomire, mochtroid,
    morph_ball_eye, multiviola, oum, phantoon, phantoon_actions, polyp, powamp, puyo, reflec,
    rinka, rio, rippers, roach, shutters, sidehopper, skree, space_pirates, spike_platform,
    spore_spawn, squeept, tatori, torizo, tripper, waver, work_robot, wrecked_ship_spark,
    yapping_maw, yard, zeb, zoa,
};
use crate::combat_rules::LatchMode;
use crate::enemy_projectiles::host::Effects as _;
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

/// What the contact reactions need beyond their actors' own hosts.
pub trait TouchHost:
    SpeciesHost
    + ActorRuntime
    + crocomire_reactions::Host
    + draygon_host::Host
    + space_pirates::Pirates
    + torizo::Ai
    + phantoon::Host
    + spore_spawn::Host
{
}

/// An enemy species' contact reaction, keyed by its address.
pub fn touch<H: TouchHost>(ea: u32, game: &mut H) {
    if lease_touch(ea, game) {
        return;
    }
    match ea {
        10518563 => game.contact_reaction(Death::Finalize),
        10780707 => game.contact_reaction(Death::Finalize),
        10795325 => crocomire_reactions::touch(game),
        10795344 => (),
        10846284 => (),
        10851818 => draygon_death::touch(game),
        10874348 => spore_spawn::touch(game),
        // Ridley's contact leaves the death pending.
        10936153 => game.contact_reaction(Death::LeavePending),
        10977356 => (),
        11001237 => phantoon::touch(game),
        11122117 => (),
        11192695 => torizo::touch(game),
        11700076 => space_pirates::touch(game),
        _ => game.unknown_routine(),
    }
}

/// Shot reactions whose routines still run in the compatibility crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeShot {
    Dud,
}

/// What the shot reactions need beyond their actors' own hosts.
pub trait ShotHost:
    SpeciesHost
    + ActorRuntime
    + crocomire_reactions::Host
    + draygon_host::Host
    + space_pirates::Pirates
    + torizo::Ai
    + phantoon::Host
    + spore_spawn::Host
{
    fn native_shot(&mut self, shot: NativeShot);
}

/// An enemy species' shot reaction, keyed by its address; `_j` is the projectile, which none of them reads.
pub fn shot<H: ShotHost>(ea: u32, _j: u16, game: &mut H) {
    if lease_shot(ea, game) {
        return;
    }
    match ea {
        10518573 => game.shot_reaction(Shot::Finalize),
        10846278 => game.native_shot(NativeShot::Dud),
        10649676 => (),
        10780717 => game.shot_reaction(Shot::Finalize),
        10795345 => crocomire_reactions::count_hit(game),
        10795368 => crocomire_reactions::impact(game),
        10795525 => crocomire_reactions::hit(game),
        10795700 => crocomire_reactions::impact(game),
        10846253 => game.shot_reaction(Shot::Finalize),
        10846284 => (),
        10851824 => draygon_death::shot(game),
        10874202 => spore_spawn::shot(game),
        10911820 => (),
        10977356 => (),
        10982581 => (),
        11001243 => phantoon::shot(game),
        11192700 => torizo::shot(game),
        11192769 => (),
        11192770 => torizo::standing_shot(game),
        11700089 => space_pirates::shot(game),
        11700168 => space_pirates::check_shot(game),
        11700286 => space_pirates::reflect(game),
        _ => game.unknown_routine(),
    }
}

/// Graphics draw hooks whose routines still run in the compatibility crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDrawHook {
    BlueFaceBlockPalette,
}

/// What the draw hooks need beyond their actors' own hosts.
pub trait DrawHookHost: SpeciesHost + LeaseActors + magdollite::Host + draygon_host::Host {
    fn native_draw_hook(&mut self, hook: NativeDrawHook);
}

/// An enemy species' graphics draw hook, keyed by its address.
pub fn draw_hook<H: DrawHookHost>(ea: u32, game: &mut H) {
    if lease_draw_hook(ea, game) {
        return;
    }
    match ea {
        10518604 => (),
        10851138 => draygon_flight::set_bg2_scroll(game),
        11042892 => (),
        11055282 => magdollite::cycle_palette(game),
        11069550 => game.native_draw_hook(NativeDrawHook::BlueFaceBlockPalette),
        11110278 => (),
        _ => game.unknown_routine(),
    }
}

/// A pipe bug's state routine, keyed by its bank $B3 address.
pub fn pipe_bug_state<H: B3Host>(ea: u32, game: &mut H) {
    match ea {
        11767733 => crate::actors::geega::detect_player(game),
        11767797 => crate::actors::geega::launch_delay(word!(game.CUR_ENEMY_INDEX), game),
        11767848 => crate::actors::geega::fly(true, game),
        11767997 => crate::actors::geega::fly(false, game),
        11768154 => crate::actors::geega::turn(true, game),
        11768280 => crate::actors::geega::turn(false, game),
        _ => {
            game.unknown_routine();
        }
    }
}
/// A bank $B3 enemy pre-instruction, keyed by its address.
pub fn b3_pre_instruction<H: B3Host>(ea: u32, k: u16, game: &mut H) {
    match ea {
        11763835 => (),
        11765888 => crate::actors::zeb::await_view(k, game),
        11765904 => crate::actors::zeb::await_player(k, game),
        11765987 => crate::actors::zeb::rise(k, game),
        11766044 => crate::actors::zeb::fly(k, game),
        11766142 => crate::actors::zeb::cooldown(k, game),
        11767039 => crate::actors::gamet::settle(game),
        11789916 => crate::actors::escape_critters::run_for_escape(k, game),
        11789936 => crate::actors::escape_critters::wait_to_express_gratitude(k, game),
        11789952 => crate::actors::escape_critters::run_around_aimlessly(k, game),
        11771328 => crate::actors::botwoon::head_moving_around(game),
        11771517 => {
            let address = crate::actors::botwoon::head_set_spit_angle(game);
            b3_pre_instruction(address, word!(game.CUR_ENEMY_INDEX), game);
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
    B3Host
    + DrawHookHost
    + yapping_maw::Host
    + crocomire_reactions::Host
    + space_pirates::Pirates
    + torizo::Ai
    + phantoon::Host
    + PreInstructionHost
    + crate::actors::host::EnemyHost
    + crate::effects::SoundEffects
    + spore_spawn::Host
{
    /// An AI routine whose arm is still in the compatibility table.
    fn native_ai(&mut self, ea: u32);
    /// Print the original's diagnostic when Reflec cannot tell which projectile hit it.
    fn warn_reflection(&mut self);
    /// Write `operand` to the multiplicand register, for Yapping Maw.
    fn write_multiplicand(&mut self, operand: u8);
    /// Install Draygon's BG layers as its initialisation does.
    fn install_draygon_layers(&mut self);
}

/// The bank $A5 functions Draygon's parts run, keyed by address.
pub fn draygon_function<H: AiHost>(ea: u32, k: u16, game: &mut H) {
    match ea {
        10848691 => {
            draygon_flight::fire_wall_turret(game);
            draygon_sweep::swoop_setup(false, game);
        }
        10848027 => draygon_flight::fight_intro(true, game),
        10848139 => draygon_flight::fight_intro(false, game),
        10850693 => draygon_death::drift_to_death_spot(game),
        10850964 => draygon_death::wait_for_evirs(game),
        10850987 => draygon_death::buried_by_evirs(game),
        10849817 => draygon_grab::chase_samus(game),
        10850064 => draygon_grab::repelled_by_grapple(game),
        10850600 => draygon_grab::flail_tail_and_fly_straight_up(game),
        10850644 => draygon_grab::fly_straight_up(game),
        10848244 => draygon_sweep::swoop_setup(true, game),
        10848433 => draygon_sweep::swoop(true, false, game),
        10848546 => draygon_sweep::swoop_apex(true, game),
        10848593 => draygon_sweep::swoop(true, true, game),
        10848768 => draygon_sweep::swoop(false, false, game),
        10848848 => draygon_sweep::swoop_apex(false, game),
        10848912 => draygon_sweep::swoop(false, true, game),
        10849034 => draygon_flight::goop_setup(true, game),
        10849422 => draygon_flight::goop_setup(false, game),
        10849106 => draygon_flight::goop_move_until_in_range(true, game),
        10849492 => draygon_flight::goop_move_until_in_range(false, game),
        10849198 => draygon_flight::goop_firing(true, game),
        10849584 => draygon_flight::goop_firing(false, game),
        10849331 => draygon_flight::goop_move_until_off_screen(true, game),
        10849714 => draygon_flight::goop_move_until_off_screen(false, game),
        10850078 => draygon_grab::move_to_target(k, game),
        10850262 => draygon_grab::rising_spiral(k, game),
        10850516 => draygon_grab::tail_whip(k, game),
        10850565 => draygon_grab::final_tail_whips(k, game),
        10850596 => draygon_grab::move_samus_with_draygon(k, game),
        10863757 => draygon_eye::look_at_samus(k, false, game),
        10863891 => draygon_eye::look_at_samus(k, true, game),
        // Spore Spawn's functions ($A5:EB1A-$A5:EBEE), which the dispatch shares.
        10846283 | 10873626 => (),
        10873627 => spore_spawn::descend(game),
        10873682 => spore_spawn::orbit(k, game),
        10873755 => spore_spawn::begin_return(game),
        10873838 => spore_spawn::finish_return(game),
        _ => game.unknown_routine(),
    }
}

/// The AI routines that run on one actor lease, keyed by address: the lease the scheduler
/// already runs inside, or a fresh one. False when `ea` is not one of them.
pub fn lease_ai<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    use shutters::{Event, Kind};
    match ea {
        10676698 => game.with_lease(|runtime| shutters::initialize(Kind::GrowingShutter, runtime)),
        10676918 => game.with_lease(|runtime| shutters::main_ai(Kind::GrowingShutter, runtime)),
        10677765 => game.with_lease(|runtime| shutters::initialize(Kind::Platform, runtime)),
        10677778 => game.with_lease(|runtime| shutters::initialize(Kind::VerticalShutter, runtime)),
        10677969 => game.with_lease(|runtime| shutters::main_ai(Kind::Platform, runtime)),
        10678429 => {
            game.with_lease(|runtime| shutters::reaction(Kind::Platform, Event::Touch, runtime))
        }
        10678434 => {
            game.with_lease(|runtime| shutters::reaction(Kind::Platform, Event::Shot, runtime))
        }
        10678454 => {
            game.with_lease(|runtime| shutters::reaction(Kind::Platform, Event::PowerBomb, runtime))
        }
        10678545 => {
            game.with_lease(|runtime| shutters::initialize(Kind::HorizontalShutter, runtime))
        }
        10678750 => game.with_lease(|runtime| shutters::main_ai(Kind::HorizontalShutter, runtime)),
        10678442 => game
            .with_lease(|runtime| shutters::reaction(Kind::VerticalShutter, Event::Shot, runtime)),
        10679256 => game.with_lease(|runtime| {
            shutters::reaction(Kind::HorizontalShutter, Event::Touch, runtime)
        }),
        10679310 => game.with_lease(|runtime| {
            shutters::reaction(Kind::HorizontalShutter, Event::Shot, runtime)
        }),
        10679322 => game.with_lease(|runtime| {
            shutters::reaction(Kind::HorizontalShutter, Event::PowerBomb, runtime)
        }),
        11066573 => game.with_lease(alcoon::initialize),
        11066731 => game.with_lease(alcoon::main_ai),
        10730163 => game.with_lease(bang::initialize),
        10731261 => game.with_lease(bang::shot),
        11057014 => game.with_lease(beetom::initialize),
        10657608 => game.with_lease(cacatac::initialize),
        10674038 => game.with_lease(choot::initialize),
        10674222 => game.with_lease(choot::main_ai),
        11049710 => game.with_lease(covern::initialize),
        11049788 => game.with_lease(covern::main_ai),
        10720995 => {
            game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Sciser, runtime))
        }
        10721595 => game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Zero, runtime)),
        10729071 => game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Viola, runtime)),
        10739779 => {
            game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::OrangeGeemer, runtime))
        }
        10740436 => game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Zeela, runtime)),
        10741148 => game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Sova, runtime)),
        10741353 => {
            game.with_lease(|runtime| crawlers::initialize(crawlers::Kind::Geemer, runtime))
        }
        11044832 => game.with_lease(evir::initialize),
        11045040 => game.with_lease(evir::initialize_projectile),
        11045278 => game.with_lease(evir::projectile_main_ai),
        11045638 | 11045644 | 11045650 => game.with_lease(|runtime| evir::react(ea, runtime)),
        11053894 => game.with_lease(kago::initialize),
        10669268 => game.with_lease(oum::initialize),
        10656319 => game.with_lease(puyo::initialize),
        10738643 => game.with_lease(reflec::initialize),
        10663426 => game.with_lease(rinka::initialize),
        10664233 => game.with_lease(rinka::frozen),
        10664275 => game.with_lease(rinka::power_bomb),
        10664263 | 10664269 => game.with_lease(|runtime| rinka::react(ea == 10664269, runtime)),
        10723661 => game.with_lease(roach::initialize),
        10724048 => game.with_lease(roach::main_ai),
        10726153 => game.with_lease(sidehopper::initialize),
        10717536 => game.with_lease(|runtime| skree::initialize(skree::Kind::Metaree, runtime)),
        10733230 => game.with_lease(|runtime| skree::initialize(skree::Kind::Skree, runtime)),
        10717561 => game.with_lease(|runtime| skree::main_ai(skree::Kind::Metaree, runtime)),
        10733255 => game.with_lease(|runtime| skree::main_ai(skree::Kind::Skree, runtime)),
        10717967 => game.with_lease(|runtime| skree::shot(skree::Kind::Metaree, runtime)),
        10733557 => game.with_lease(|runtime| skree::shot(skree::Kind::Skree, runtime)),
        10654337 | 10654367 => game.with_lease(|runtime| tatori::touch(ea == 10654367, runtime)),
        10722463 => game.with_lease(|runtime| tripper::initialize(true, runtime)),
        10722490 => game.with_lease(|runtime| tripper::initialize(false, runtime)),
        10722582 => game.with_lease(tripper::main_ai),
        10723080 => game.with_lease(tripper::shot),
        11062326 => game.with_lease(work_robot::main_ai),
        11063668 => game.with_lease(work_robot::touch),
        10735074 => game.with_lease(yard::initialize),
        10736560 => game.with_lease(yard::touch),
        10736745 => game.with_lease(yard::shot),
        10863723 => game.with_lease(draygon_eye::initialize),
        10720486 => game.with_lease(elevator::initialize),
        11048675 => game.with_lease(fune::initialize),
        11048752 => game.with_lease(fune::main_ai),
        11053941 => game.with_lease(kago::main_ai),
        10725245 => game.with_lease(mochtroid::initialize),
        10725264 => game.with_lease(mochtroid::main_ai),
        10725715 => game.with_lease(mochtroid::touch),
        10663280 => game.with_lease(polyp::initialize),
        10664909 => game.with_lease(rio::initialize),
        10674643 => game.with_lease(|runtime| rippers::initialize(rippers::Kind::Gripper, runtime)),
        10674968 => {
            game.with_lease(|runtime| rippers::initialize(rippers::Kind::RipperII, runtime))
        }
        10675359 => game.with_lease(|runtime| rippers::initialize(rippers::Kind::Ripper, runtime)),
        10674721 => game.with_lease(|runtime| rippers::main_ai(rippers::Kind::Gripper, runtime)),
        10675027 => game.with_lease(|runtime| rippers::main_ai(rippers::Kind::RipperII, runtime)),
        10675418 => game.with_lease(|runtime| rippers::main_ai(rippers::Kind::Ripper, runtime)),
        10653036 | 10653085 => {
            game.with_lease(|runtime| tatori::initialize(ea == 10653085, runtime))
        }
        10654479 => game.with_lease(tatori::shot),
        10728522 => game.with_lease(zoa::initialize),
        10728572 => game.with_lease(zoa::main_ai),
        10670988 => game.with_lease(|runtime| oum::touch(false, runtime)),
        10671028 => game.with_lease(oum::shot),
        _ => return false,
    }
    true
}

/// The instructions that run on one actor lease, keyed by address; `k` is the enemy and `j`
/// the script position, handed back unrun when `ea` is not one of them.
pub fn lease_instruction<H: LeaseScope + ?Sized>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> Result<EnemyScript, EnemyScript> {
    Ok(match ea {
        10730152 => game.with_lease(|runtime| bang::set_finished_growing(j, runtime)),
        11044751 | 11044763 | 11044790 | 11044811 => {
            game.with_lease(|runtime| evir::instruction(ea, j, runtime))
        }
        10742449 => game.with_lease(|runtime| metroid::random_cry(j, runtime)),
        10664371 | 10664381 | 10664391 => {
            game.with_lease(|runtime| rinka::instruction(ea, k, j, runtime))
        }
        10726142 => game.with_lease(|runtime| sidehopper::ready_to_hop(j, runtime)),
        10717526 | 10733220 => game.with_lease(|runtime| skree::set_attack_ready(j, runtime)),
        10722411 | 10722433 => {
            game.with_lease(|runtime| tripper::set_x_movement(false, j, runtime))
        }
        10722422 | 10722444 => game.with_lease(|runtime| tripper::set_x_movement(true, j, runtime)),
        10728489 => game.with_lease(|runtime| zoa::set_x_speed_index(4, j, runtime)),
        10728500 => game.with_lease(|runtime| zoa::set_x_speed_index(8, j, runtime)),
        10728511 => game.with_lease(|runtime| zoa::set_x_speed_index(12, j, runtime)),
        _ => return Err(j),
    })
}

/// The contact reactions that run on one actor lease, keyed by address. False when `ea` is
/// not one of them.
pub fn lease_touch<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    match ea {
        10670984 => game.with_lease(|runtime| oum::touch(true, runtime)),
        10670988 => game.with_lease(|runtime| oum::touch(false, runtime)),
        _ => return false,
    }
    true
}

/// The shot reactions that run on one actor lease, keyed by address. False when `ea` is not
/// one of them.
pub fn lease_shot<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    match ea {
        10671028 => game.with_lease(oum::shot),
        _ => return false,
    }
    true
}

/// The graphics drawn hooks that run on one actor lease, keyed by address. False when `ea` is
/// not one of them.
pub fn lease_draw_hook<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    match ea {
        10738444 => game.with_lease(reflec::cycle_palette),
        11062375 => game.with_lease(work_robot::palette),
        _ => return false,
    }
    true
}

/// An enemy species' AI routine, keyed by its address. Arms not here are in
/// the compatibility table until their routines move.
pub fn ai<H: AiHost>(ea: u32, game: &mut H) {
    if lease_ai(ea, game) {
        return;
    }
    use crate::actors::combat::{die, latch, resume_ai, select_frozen_ai};
    match ea {
        // The common enemy AI ($A0:8000-$A0:8041) the enemy banks share: grapple AI - no
        // interaction, Samus latches on (with or without invincibility), kill enemy and
        // cancel grapple beam, and the normal enemy frozen AI.
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => {
            resume_ai(&mut LeaseActors::lease(game))
        }
        10649605 | 10780677 => latch(&mut LeaseActors::lease(game), LatchMode::Passive),
        11042836 => latch(&mut LeaseActors::lease(game), LatchMode::Active),
        10649610 | 10715146 | 11042826 | 11763722 => die(
            &mut LeaseActors::lease(game),
            crate::actors::api::DeathKind::Grapple,
        ),
        10649615 | 10715151 | 10846223 | 10911759 | 10977295 | 11042831 | 11042846 | 11173903
        | 11763727 => select_frozen_ai(&mut LeaseActors::lease(game)),
        10649665 | 10715201 | 10780737 | 10846273 | 10911809 | 10977345 | 11042881 | 11173953
        | 11763777 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        // Initialisation AI - enemy $F153 (unused spinning turtle eye) ($B3:86FB).
        11765499 => {
            let mut runtime = LeaseActors::lease(game);
            let key = crate::actors::combat::current(&runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.properties |=
                    crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16;
                actor.instruction = 0x86a7;
            });
        }
        // Main AI - mini-Crocomire ($A2:89F0) and cacatac ($A2:9FB3): their function.
        10652144 | 10657715 => {
            let runtime = LeaseActors::lease(game);
            let mode = if ea == 10652144 { LittleEndian } else { Native };
            let key = crate::actors::combat::current(&runtime);
            let function = runtime.resolve_actor(key, mode).pre_instruction;
            drop(runtime);
            pre_instruction(0xa2_0000 | u32::from(function), game);
        }
        10738716 => {
            if let Some(target) = reflec::reflect_shot(game) {
                game.warn_reflection();
                reflec::absorb_shot(target, game);
            }
        }
        // Enemy shot - Wrecked Ship spark ($A8:E70E) and blue Brinstar face block ($A8:E91D).
        11069198 | 11069725 => wrecked_ship_spark::pass_shot(game),
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
        11765819 => zeb::initialize(game),
        11767628 => geega::initialize(game),
        11769219 => botwoon::initialize(game),
        11769448 => botwoon::main_ai(&mut LeaseActors::lease(game)),
        11771903 | 11771969 => botwoon::contact(ea == 11771969, &mut LeaseActors::lease(game)),
        11771926 => botwoon::shot(&mut LeaseActors::lease(game)),
        10873386 => spore_spawn::initialize(game),
        10873619 => spore_spawn::main_ai(game),
        10874202 => spore_spawn::shot(game),
        10874348 => spore_spawn::touch(game),
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
        // Bank $A6: the boulder, the fire arc and Mini-Kraid, with the bank's
        // common reactions.
        10911779 => game.contact_reaction(Death::Finalize),
        10911789 => game.shot_reaction(Shot::Finalize),
        // Bank $A7: Kraid, Phantoon, the etecoons and the dachora, with the
        // bank's common reactions.
        10997235 => phantoon::initialize(game),
        10997333 => phantoon::initialize_part(game),
        10997414 => phantoon::main_ai(game),
        11001151 => phantoon::hurt(game),
        11001237 => phantoon::touch(game),
        11001243 => phantoon::shot(game),
        10977325 => game.shot_reaction(Shot::Finalize),
        10977340 => game.power_bomb_reaction(Death::LeavePending),
        // Bank $A8: the common grapple, contact, shot and frozen reactions.
        11042851 => game.contact_reaction(Death::Finalize),
        11042861 => game.shot_reaction(Shot::Finalize),
        // Bank $AA: the Torizos, Shaktool and the statues.
        11191934 => torizo::hurt(game),
        11192695 => torizo::touch(game),
        11192700 => torizo::shot(game),
        11195322 => torizo::golden_hurt(game),
        11191972 => torizo::main_ai(game),
        11192447 => torizo::initialize(game),
        11195241 => torizo::golden_main_ai(game),
        11196007 => torizo::golden_shot(game),
        // Bank $B2: the space pirates.
        0xb20000..=0xb2ffff => space_pirates::dispatch_ai(ea, game),
        // Bank $A5.
        10847879 => draygon_flight::initialize(|game: &mut H| game.install_draygon_layers(), game),
        10847996 => draygon_flight::main_ai(true, draygon_function, game),
        10863750 => draygon_flight::main_ai(false, draygon_function, game),
        10851661 => draygon_death::hurt_ai(game),
        10864025 => draygon_flight::initialize_tail_or_arms(false, game),
        10864045 => draygon_flight::initialize_tail_or_arms(true, game),
        10846243 => game.contact_reaction(Death::Finalize),
        10846253 => game.shot_reaction(Shot::Finalize),
        10851818 => draygon_death::touch(game),
        10851847 => draygon_death::power_bomb(game),
        // Bank $A3: the crawlers, divers, platforms, bugs and fish.
        10720554 => elevator::main_ai(game),
        10739851 | 10741442 => {
            let function = crawlers::main_ai(game);
            pre_instruction(function, game);
        }
        10730277 => bang::main_ai(game),
        10656381 => {
            if !puyo::main_ai(game) {
                game.unknown_routine();
            }
        }
        10653138 | 10653998 => {
            if !tatori::main_ai(ea == 10653998, game) {
                game.unknown_routine();
            }
        }
        11062135 => work_robot::initialize(game),
        11062220 => work_robot::initialize_deactivated(game),
        10914607 => spike_platform::initialize(game),
        10914733 => spike_platform::run(game),
        10659396 => gunship::initialize_top(game),
        10659538 => gunship::initialize_bottom(game),
        10659673 => {
            let function = gunship::main(game);
            pre_instruction(function, game);
        }
        11063693 => work_robot::shot_deactivated(game),
        11063698 => work_robot::shot(game),
        10735204 => {
            let function = yard::prepare(game);
            pre_instruction(function, game);
        }
        11059657 => powamp::initialize(game),
        11059740 => {
            let function = powamp::function(game);
            pre_instruction(function, game);
        }
        11060670 => powamp::touch(game),
        11060719 => powamp::shot(game),
        11060799 => powamp::power_bomb(game),
        11068983 => wrecked_ship_spark::initialize(game),
        11069070 => wrecked_ship_spark::main_ai(game),
        10663876 => {
            if !rinka::main_ai(game) {
                game.unknown_routine();
            }
        }
        10669331 => oum::main_ai(game),
        10658809 => cacatac::owtch::initialize(game),
        10658942 => cacatac::owtch::main_ai(game),
        10659193 => cacatac::owtch::shot(game),
        10742351 => metroid::initialize(game),
        10742502 => metroid::frozen(game),
        10742579 => metroid::hurt(game),
        10742680 => metroid::main_ai(game),
        10743275 => metroid::touch(game),
        10743559 => metroid::shot(game),
        10743874 => metroid::power_bomb(game),
        10726351 => sidehopper::main_ai(game),
        // Bank $A8.
        11045147 => evir::main_ai(game),
        11047000 => morph_ball_eye::initialize(game),
        11057165 => beetom::main_ai(game),
        11058734 => beetom::touch(game),
        11058860 => beetom::shot(game),
        11047138 => {
            if let Some(function) = morph_ball_eye::main_ai(game) {
                pre_instruction(function, game);
            }
        }
        11053955 => kago::shot(game),
        11069486 => blue_brinstar_face_block::initialize(game),
        11069614 => blue_brinstar_face_block::main_ai(game),
        11051537 => {
            if !yapping_maw::main_ai(game, |operand, game| game.write_multiplicand(operand)) {
                game.unknown_routine();
            }
            yapping_maw::update_segments(game);
        }
        11055370 => pre_instruction(magdollite::main_ai(game), game),
        11765882 => pre_instruction(zeb::state_address(game), game),
        11767726 => pipe_bug_state(geega::state_address(game), game),
        11789909 => pre_instruction(escape_critters::main_ai(game), game),
        10662880 => multiviola::initialize(game),
        10662927 => multiviola::main_ai(game),
        10663311 => polyp::main_ai(game),
        10675113 => rippers::shot(game),
        10664931 => rio::main_ai(game),
        10666562 => geruta::initialize(false, game),
        10667763 => geruta::initialize(true, game),
        10665625 => squeept::initialize(game),
        10662175 | 10665682 | 10666615 | 10667812 => {
            game.next_random();
            pre_instruction(geruta::function(game), game);
        }
        10661995 => flies::initialize(game),
        10652077 => mini_crocomire::initialize(game),
        10716909 => waver::initialize(game),
        10717004 => waver::main_ai(game),
        10651420 => boyon::initialize(game),
        10651548 => boyon::main_ai(game),
        10675718 => dragon::initialize(game),
        10675790 => pre_instruction(geruta::function(game), game),
        10676168 | 10676174 | 10676180 => dragon::react(ea, game),
        10915836 => fire_pillar::initialize(game),
        10915875 => fire_pillar::main_ai(game),
        _ => game.native_ai(ea),
    }
}

/// What the pre-instruction table needs beyond its actors' own hosts.
pub trait PreInstructionHost:
    B3Host + magdollite::Host + torizo::Ai + phantoon::Host + crate::actors::host::EnemyHost
{
    /// A pre-instruction whose arm is still in the compatibility table, for
    /// the current enemy `k` read before dispatch.
    fn native_pre_instruction(&mut self, ea: u32, k: u16);
    /// Magdollite's original diagnostic for an undefined lava height.
    fn warn_lava_height(&mut self, value: i32);
    /// Start the morph ball eye's beam effect.
    fn initialize_morph_eye_beam(&mut self);
}

/// An enemy species' pre-instruction, keyed by its address. The current
/// enemy index is read once, before dispatch, as the original did.
pub fn pre_instruction<H: PreInstructionHost>(ea: u32, game: &mut H) {
    let k: u16 = word!(game.CUR_ENEMY_INDEX);
    match ea {
        10730661 => bang::movement_delay(game),
        10730693 => bang::target_samus(game),
        10730780 => bang::fly(false, game),
        10730796 => bang::fly(true, game),
        10739857 => crawlers::crawl(k, crate::actors::api::Axis::X, true, game),
        10740072 => crawlers::crawl(k, crate::actors::api::Axis::Y, true, game),
        10741448 => crawlers::crawl(k, crate::actors::api::Axis::X, false, game),
        10741746 => crawlers::crawl(k, crate::actors::api::Axis::Y, false, game),
        10741637 => crawlers::falling(k, game),
        10518651 => (),
        10669926 => oum::bounce(false, k, game),
        10669993 => oum::bounce(true, k, game),
        10657722 => cacatac::moving(false, game),
        10735526 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cce2, game),
        10735543 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccea, game),
        10735549 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3ccf2, game),
        10735566 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccfa, game),
        10735572 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd02, game),
        10735589 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd0a, game),
        10735595 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd12, game),
        10735612 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd1a, game),
        10735456 => yard::hiding(k, game),
        10736051 => yard::airborne(k, game),
        11059843 | 11059878 | 11059919 | 11060075 | 11060193 | 11060329 | 11060444 | 11060480
        | 11060509 | 11060584 | 11060585 | 11060639 => powamp::run(game, ea as u16, k),
        10659852 | 10660048 | 10660162 | 10660176 | 10660231 | 10660285 | 10660431 | 10660445
        | 10660500 | 10660514 | 10660639 | 10660704 | 10660718 | 10660773 | 10660807 | 10660891
        | 10661079 | 10661134 | 10661165 => gunship::run(game, k, ea as u16),
        10657772 => cacatac::moving(true, game),
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
        // Bank $A7: Phantoon, the etecoons and the dachora.
        10999114 => phantoon::wavy_fade_in(game, k),
        10999271 => phantoon::figure_eight(game, k),
        10999309 => phantoon::eye_follows_samus(game, k),
        10999416 => phantoon::swooping(game, k),
        10999481 => phantoon::fade_out_with_swoop(game, k),
        10999522 => phantoon::reappear(game, k),
        10999597 => phantoon::fade_in_before_figure_eight(game),
        10999615 => phantoon::become_solid_after_flame_rain(game, k),
        10999655 => phantoon::fade_in_during_flame_rain(game, k),
        10999765 => phantoon::fade_out_during_flame_rain(game, k),
        10999850 => phantoon::fade_out_before_first_flame_rain(game, k),
        10999900 => phantoon::fade_out_before_enrage(game, k),
        10999953 => phantoon::fade_in_enraged(game, k),
        10999980 => phantoon::enraged(game, k),
        11000086 => phantoon::fade_out_after_enrage(game, k),
        11000110 => phantoon::finish_swoop_after_fatal_shot(game, k),
        11000136 => phantoon::dying_fade_in_out(game, k),
        11000401 => phantoon::wavy_dying_cry(game, k),
        11000454 => phantoon::dying_fade_out(game, k),
        11000535 => phantoon::almost_dead(game, k),
        11000637 => phantoon::dead(game, k),
        10999388 => phantoon::become_solid_and_swoop(game, k),
        10998953 => phantoon::countdown(game, k, phantoon::function::SPAWN_STARTING_FLAMES),
        10999022 => phantoon::countdown(game, k, phantoon::function::WAIT_BEFORE_SPINNING_FLAMES),
        10999048 => phantoon::countdown(game, k, phantoon::function::SPAWN_FLAMES_BEFORE_FIGHT),
        10999190 => phantoon::countdown(game, k, phantoon::function::PICK_ROUND_1_PATTERN),
        10999508 => phantoon::countdown(game, k, phantoon::function::WAIT_AFTER_FADE_OUT),
        10999688 => {
            phantoon::countdown(game, k, phantoon::function::FOLLOW_SAMUS_DURING_FLAME_RAIN)
        }
        10999799 => phantoon::countdown(game, k, phantoon::function::SPAWN_FLAME_RAIN),
        10999924 => phantoon::countdown(game, k, phantoon::function::MOVE_TO_TOP_CENTRE),
        11000203 => phantoon::countdown(game, k, phantoon::function::DYING_EXPLOSIONS),
        // Bank $AA: the Torizos, Shaktool and the statues.
        11192146 => torizo::walking(k, game),
        11192364 => torizo::jumping_or_falling(k, game),
        11195889 => torizo::golden_walking(k, game),
        11192360 | 11195885 => torizo::attacking(k, game),
        11047153 | 11047214 | 11047264 | 11047374 => {
            if morph_ball_eye::run_function(ea as u16, k, game) {
                game.initialize_morph_eye_beam();
                morph_ball_eye::start_tracking(k, game);
            }
        }
        11055620 => {
            let (actor, diagnostic) = magdollite::grow_arm(game);
            if let Some(value) = diagnostic {
                game.warn_lava_height(value);
            }
            magdollite::finish_arm_growth(actor, game);
        }
        10666625 => geruta::flames(k, false, game),
        10666727 => geruta::idle(k, false, game),
        10666815 => geruta::finish_animation(k, false, false, game),
        10667014 => geruta::finish_animation(k, false, true, game),
        10667822 => geruta::flames(k, true, game),
        10667889 => geruta::idle(k, true, game),
        10667963 => geruta::finish_animation(k, true, false, game),
        10668168 => geruta::finish_animation(k, true, true, game),
        10666849 => geruta::swoop(k, false, true, game),
        10666929 => geruta::swoop(k, false, false, game),
        10667990 => geruta::swoop(k, true, true, game),
        10668077 => geruta::swoop(k, true, false, game),
        10665692 => squeept::flame(k, game),
        10665754 => squeept::jump(k, game),
        10665790 => squeept::fly(k, squeept::Flight::Rising, game),
        10665852 => squeept::fly(k, squeept::Flight::Flipping, game),
        10665916 => squeept::fly(k, squeept::Flight::Falling, game),
        10662222 => flies::circle(k, false, game),
        10662268 => flies::circle(k, true, game),
        10662314 => flies::dive(k, false, game),
        10662354 => flies::dive(k, true, game),
        10652227 => mini_crocomire::patrol(false, game),
        10652252 => mini_crocomire::patrol(true, game),
        10675796 | 10675885 | 10675953 | 10676020 | 10676041 => dragon::step(k, ea as u16, game),
        _ => game.native_pre_instruction(ea, k),
    }
}

/// What the instruction table needs beyond its actors' own hosts.
pub trait InstructionHost:
    SpeciesHost
    + crocomire_reactions::Host
    + yapping_maw::Host
    + magdollite::Host
    + escape_critters::Host
    + space_pirates::Pirates
    + draygon_host::Host
    + LeaseActors
    + torizo::Ai
    + phantoon::Host
    + crate::actors::host::EnemyHost
    + crate::effects::SoundEffects
    + botwoon::Host
    + spore_spawn::Host
{
    /// An instruction whose arm is still in the compatibility table.
    fn native_instruction(&mut self, ea: u32, k: u16, j: EnemyScript) -> EnemyScript;
}

/// The bank $A7 functions Phantoon's instruction lists call, keyed by address.
fn call_a7_function<H: InstructionHost>(ea: u32, k: u16, game: &mut H) {
    match ea {
        10997485 => phantoon::materialisation_sound(game),
        10997598 => phantoon_actions::spawn_casual_flame(k, game),
        10997823 => phantoon::open_eye(game),
        10997878 => phantoon::pick_pattern(game),
        _ => game.unknown_routine(),
    }
}

/// An enemy species' instruction, keyed by its address; `k` is the enemy and
/// `j` the script position.
pub fn instruction<H: InstructionHost>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let j = match lease_instruction(ea, k, j, game) {
        Ok(script) => return script,
        Err(j) => j,
    };
    match ea {
        // Instruction - enemy $0FB2 = [[Y]] ($AA:806B, $B3:806B) and = RTS ($AA:8074, $B3:8074).
        11173995 | 11174004 | 11763819 | 11763828 => {
            let given = ea & 0xffff == 0x806b;
            let value = if given { j.word(0) } else { 0x807b };
            let key = crate::actors::access::ActorKey::from_legacy(k);
            LeaseActors::lease(game).edit_actor(key, Native, |actor| actor.pre_instruction = value);
            if given { j.offset(1) } else { j }
        }
        // Instruction - nothing ($A2:88C5, $A8:B75E).
        10651845 | 11056990 => j,
        // Instruction - queue the splashed out of water, tatori spinning, cacatac spikes
        // and Botwoon spit sound effects ($A2:CB6B, $A2:94D1, $A2:9F2A, $B3:9572).
        10668907 | 10654929 | 10657578 | 11769202 => {
            let sound = match ea {
                10668907 => 0xe,
                10654929 => 0x3a,
                10657578 => 0x34,
                _ => 0x7c,
            };
            game.queue_sound(1, sound, 6);
            j
        }
        // Instruction - queue sound [[Y]], sound library 2 or 3, max 6 ($A5:9F60, $A5:9F6E).
        10854240 | 10854254 => {
            LeaseActors::lease(game).queue_sound(if ea == 10854240 { 2 } else { 3 }, 6, j.word(0));
            j.offset(1)
        }
        // Instruction - spawn mini-Crocomire ($A2:897E) or cacatac spike ($A2:A0A7) projectile.
        10652030 | 10657959 => {
            let definition = if ea == 10652030 { 0xdbf2 } else { 0xdafe };
            let enemy = crate::actors::host::EnemyHost::current_enemy(game);
            game.spawn_projectile(j.word(0), enemy, definition);
            j.offset(1)
        }
        // The baby Metroid's go to instructions ($A9:CFB4, $A9:CFCA, $A9:F920, $A9:F936, $A9:F990).
        11128756 => crate::actors::baby_metroid_cutscene::go_to_initial(k, j),
        11128778 => crate::actors::baby_metroid_cutscene::go_to_draining_mother_brain(k, j),
        11139360 => crate::actors::baby_metroid_encounter::go_to_normal(k, j),
        11139382 => crate::actors::baby_metroid_encounter::go_to_latched_on(k, j),
        11139472 => crate::actors::baby_metroid_encounter::go_to_remorse(k, j),
        // Instruction - room loading interrupt command = Draygon's room - begin HUD drawing.
        10852501 | 10853514 => {
            LeaseActors::lease(game)
                .write_slice_le(crate::ram::words::ROOM_LOADING_IRQ_HANDLER, 12);
            j
        }
        spore::INCREASE_MAX_X_RADIUS => spore_spawn::increase_max_x_radius(j, game),
        spore::CLEAR_DAMAGED_FLAG => spore_spawn::clear_damaged_flag(j, game),
        spore::SET_MAX_X_RADIUS_AND_ANGLE_DELTA => {
            spore_spawn::set_max_x_radius_and_angle_delta(j, game)
        }
        spore::HARDEN => spore_spawn::harden(j, game),
        spore::SET_FUNCTION => spore_spawn::set_function(j, game),
        // Death quake sound timer = [[Y]] ($A5:E872) and queue sound [[Y]] ($A5:E895).
        10872946 => {
            let at = crate::ram::words::BOSS_DEATH_QUAKE_SOUND_TIMER;
            LeaseActors::lease(game).write_slice_le(at, j.word(0));
            j.offset(1)
        }
        10872981 => {
            LeaseActors::lease(game).queue_sound(2, 6, j.word(0));
            j.offset(1)
        }
        10873009 => {
            game.scatter_spore_spawn_drop(k);
            j
        }
        10873034 | 10873116 => spore_spawn::fade(ea == 10873116, j, game),
        10873198 => spore_spawn::emit_spore(j, game),
        10873265 => spore_spawn::emit_hurt(j, game),
        10852159 => draygon_death::spawn_dying_sprite(21, j, game),
        10852178 => draygon_death::spawn_dying_sprite(3, j, game),
        10852197 => draygon_death::spawn_dying_sprite(29, j, game),
        10852216 => draygon_death::spawn_dying_sprite(24, j, game),
        10854268 | 10854318 => draygon_death::spawn_goop(ea == 10854318, j, game),
        10725992 => sidehopper::queue_sound(j, game),
        10657941 => cacatac::resume_moving(k, j, game),
        11068957 | 11068970 => wrecked_ship_spark::set_intangible(game, ea == 11068957, j),
        10734646 | 10734655 | 10734664 | 10734687 | 10734712 => yard::instruction(ea, k, j, game),
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216
        | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601
        | 11063613 | 11063659 => work_robot::instruction(ea, k, j, game),
        10654593 | 10654738 | 10654791 | 10654801 | 10654827 | 10654853 | 10654881 | 10654919 => {
            tatori::instruction(ea, j, game)
        }
        10669235 | 10669246 | 10669257 => oum::instruction(ea, j, game),
        10659181 | 10659185 => cacatac::owtch::set_function_index(ea, k, j, game),
        10742437 => metroid::draining_sound(j, game),
        // Bang's queue acquired suit sound effect: 56h, library 2.
        10730104 => {
            game.queue_sound(1, 0x56, 6);
            j
        }
        10738632 => reflec::set_reflection_axis(k, j, game),
        10741344 | 10739650 => crawlers::set_function(k, j, game),
        11048485 | 11048497 | 11048522 | 11048547 | 11048572 | 11048597 | 11048628 => {
            fune::instruction(ea, j, game)
        }
        11067164 | 11067187 | 11067193 | 11067199 | 11067235 | 11067249 => {
            alcoon::instruction(ea, j, game)
        }
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
        // Bank $A7: Kraid's steps and the calls into Phantoon's functions; Kraid's
        // identity instruction returns the script unchanged.
        10977418 => {
            call_a7_function(j.word(0) as u32 | 0xa70000, k, game);
            j.offset(1)
        }
        // Bank $AA: the Torizos, Shaktool and the statues; the two identity
        // instructions return the script unchanged.
        11190984 | 11196831 => j,
        11186332 => torizo::set_function(k, j, game),
        11186461 => {
            torizo::blow_up_gut(k, game);
            j
        }
        11186622 => {
            torizo::blow_up_face(k, game);
            j
        }
        11186724 => {
            torizo::set_visible(k, game);
            j
        }
        11186734 => {
            torizo::set_invisible(k, game);
            j
        }
        11186765 => {
            torizo::defeated(k, game);
            j
        }
        11190985 => {
            torizo::lock_animation(k, game);
            j
        }
        11190993 => {
            torizo::unlock_animation(k, game);
            j
        }
        11191001 => torizo::branch_if_faceless(k, j, game),
        11191021 => torizo::set_link(k, j, game),
        11191043 => torizo::low_health_explosions(k, j, game),
        11191087 => torizo::death_explosion(k, j, game),
        11191131 => {
            torizo::drool_if_low_health(k, game);
            j
        }
        11191149 => {
            torizo::start_turning(k, game);
            j
        }
        11191159 => {
            torizo::stepped(k, game, 0);
            j
        }
        11191178 => {
            torizo::stepped(k, game, 0x8000);
            j
        }
        11191200 => {
            torizo::stepped(k, game, 0x2000);
            j
        }
        11191222 => {
            torizo::stepped(k, game, 0xa000);
            j
        }
        11191244 => torizo::stand_up(k, j, game),
        11191326 => torizo::sit_down(k, j, game),
        11191408 => torizo::walk(
            k,
            j,
            torizo::list::TURNING_LEFT,
            torizo::list::TURNING_RIGHT,
            72,
            game,
        ),
        11191525 => torizo::walk(
            k,
            j,
            torizo::list::FACELESS_TURNING_LEFT,
            torizo::list::FACELESS_TURNING_RIGHT,
            72,
            game,
        ),
        11191642 => torizo::branch_if_rising(k, j, game),
        11191655 => torizo::call_if_samus_close_in_front(k, j, game),
        11191691 => torizo::jump_back_if_samus_close(k, j, game),
        11191716 => torizo::bomb_attack(k, j, game),
        11191755 => {
            torizo::bomb_chozo_orbs(k, game);
            j
        }
        11191779 => torizo::bomb_sonic_boom(k, j, game),
        11191794 => torizo::golden_sonic_boom(k, j, game),
        11193038 => torizo::branch_if_airborne(k, j, game),
        11193058 => {
            torizo::start_golden_fight(k, game);
            j
        }
        11193815 => {
            torizo::clear_caught_super_missile(k, game);
            j
        }
        11194747 => {
            torizo::disable_eye_beam_explosions(k, game);
            j
        }
        11194759 => {
            torizo::enable_eye_beam_explosions(k, game);
            j
        }
        11194855 => {
            torizo::clear_stunned(k, game);
            j
        }
        11195370 => torizo::branch_if_samus_rolling_behind(k, j, game),
        11195461 => torizo::call_sometimes_if_samus_in_front(k, j, game),
        11195508 => torizo::call_sometimes_at_low_health(k, j, game),
        11195547 => torizo::call_if_stunned_at_high_health(k, j, game),
        11195578 => torizo::jump_forwards_if_samus_far(k, j, game),
        11195635 => {
            torizo::golden_chozo_orbs(k, game);
            j
        }
        11195645 => torizo::golden_jump_back(k, j, game),
        11195686 => torizo::golden_attack(k, j, game),
        11195725 => torizo::walk(
            k,
            j,
            torizo::list::GOLDEN_TURNING_LEFT,
            torizo::list::GOLDEN_TURNING_RIGHT,
            16,
            game,
        ),
        11191031 => torizo::return_to_link(k, game),
        11191037 => torizo::return_from_gut_explosion(k, game),
        11186744 => {
            torizo::fade_to_black(game);
            j
        }
        11186801 => {
            torizo::advance_colour_change(game);
            j
        }
        11188557 => {
            torizo::fade_to_active(game);
            j
        }
        11188561 => {
            torizo::start_fight(game);
            j
        }
        11191114 => {
            torizo::landing_dust(game);
            j
        }
        11191809 => torizo::explosive_swipe(j, game),
        11191824 => {
            torizo::shot_sound(game);
            j
        }
        11191832 => {
            torizo::footstep_sound(game);
            j
        }
        11193054 => {
            torizo::golden_palettes(game);
            j
        }
        11194601 => {
            torizo::spawn_egg(game);
            j
        }
        11194611 => torizo::branch_if_egg_active(j, game),
        11195279 => {
            torizo::egg_released_sound(game);
            j
        }
        11195287 => {
            torizo::laser_sound(game);
            j
        }
        11195295 => {
            torizo::sonic_boom_sound(game);
            j
        }
        11195360 => {
            torizo::golden_super_missile(game);
            j
        }
        11195446 => torizo::eye_beam(j, game),
        0xb20000..=0xb2ffff => space_pirates::dispatch_instruction(ea, k, j, game),
        // Bank $A5.
        10851549 => draygon_flight::set_instruction_lists(j, game),
        10852150 => draygon_eye::set_function(k, j, game),
        10852563 => draygon_flight::paralyse_tail_and_arms(j, game),
        10852591 => draygon_flight::set_intangible(j, game),
        10853274 => draygon_grab::tail_whip_hit(j, game),
        10853898 => draygon_flight::displace_graphics(j, game),
        10854231 => draygon_eye::set_function(0, j, game),
        10863739 => draygon_eye::set_function(0x40, j, game),
        10915247 | 10915347..=10915793 => fire_pillar::instruction(ea, j, game),
        11769031 | 11769095 | 11769111 | 11769175 => botwoon::set_hitbox(8, 16, j, game),
        11769047 | 11769079 | 11769127 | 11769159 => botwoon::set_hitbox(12, 12, j, game),
        11769063 | 11769143 => botwoon::set_hitbox(16, 8, j, game),
        11769191 => botwoon::set_spitting(j, game),
        10664899 => {
            rio::set_animation_finished(game);
            j
        }
        10666441 | 10666452 | 10666463 | 10666474 | 10666485 | 10666496 | 10666507 | 10666518
        | 10666529 | 10666540 | 10666551 | 10667730 | 10667741 | 10667752 => {
            geruta::instruction(ea, j, game)
        }
        10665614 => {
            squeept::set_animation_finished(game);
            j
        }
        10652048 => mini_crocomire::set_moving(false, j, game),
        10652061 => mini_crocomire::set_moving(true, j, game),
        10716899 => waver::set_spin_finished(j, game),
        10651846 => boyon::start_bounce(j, game),
        10675707 => dragon::set_animation_finished(j, game),
        _ => game.native_instruction(ea, k, j),
    }
}
