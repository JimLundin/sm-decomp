//! Enemy species dispatch tables keyed by the routine addresses the original
//! enemy definitions name: AI, pre-instructions, instructions, contact and
//! shot reactions, graphics draw hooks, bank $B3 pre-instructions and pipe bug
//! state routines.
//!
//! Each table is one `match` from address to routine. Arms whose routines
//! still run in the compatibility crate go to its host: a named native enum
//! for the small tables, the same address for the large ones. Moving a
//! routine here means moving its arm.
use crate::actors::access::{AccessMode::LittleEndian, AccessMode::Native};
use crate::actors::api::DeathHandling;
use crate::actors::gunship::Host as _;
use crate::actors::host::{ActorLease, ActorRuntime, Death, LeaseScope, Shot};
use crate::actors::{
    alcoon, bang, beetom, boyon, cacatac, choot, covern, crawlers, dragon, draygon_death,
    draygon_eye, draygon_flight, draygon_grab, elevator, evir, fire_pillar, flies, fune, geruta,
    gunship, kago, metroid, mini_crocomire, mochtroid, multiviola, oum, phantoon, polyp, puyo,
    reflec, rinka, rio, rippers, roach, shutters, sidehopper, skree, space_pirates, spike_platform,
    spore_spawn, squeept, tatori, torizo, tripper, waver, work_robot, wrecked_ship_spark, yard,
    zoa,
};
use crate::combat_rules::LatchMode;
use crate::enemy_script::EnemyScript;
use crate::session_access::SessionAccess as _;

/// The scheduler's lease, which the AI, pre-instruction and instruction tables run on: the
/// actor lease, with the common reactions, the pirates' projectile spawns, and what the
/// compatibility side still answers.
pub trait TableLease:
    ActorLease
    + crate::actors::boss_lease::BossLease
    + torizo::Lease
    + spore_spawn::Lease
    + crate::actors::draygon_host::Lease
    + space_pirates::PirateHost
    + ActorRuntime
{
    /// An AI routine whose arm is still in the compatibility table, or the original failure.
    fn lease_native_ai(&mut self, ea: u32);
    /// A pre-instruction whose arm is still in the compatibility table, for the current
    /// enemy `k` read before dispatch.
    fn lease_native_pre_instruction(&mut self, ea: u32, k: u16);
    /// An instruction whose arm is still in the compatibility table.
    fn lease_native_instruction(&mut self, ea: u32, k: u16, j: EnemyScript) -> EnemyScript;
    /// Print the original's diagnostic when Reflec cannot tell which projectile hit it.
    fn lease_warn_reflection(&mut self);
    /// Run Phantoon's `callback` as an owned call.
    fn lease_phantoon(&mut self, callback: phantoon::Callback);
}

/// The contact reactions that return at once: $A4:B950, $A5:804C, $A7:804C
/// and $A9:B5C5.
pub const EMPTY_TOUCHES: [u32; 4] = [10795344, 10846284, 10977356, 11122117];

/// The shot reactions that return at once.
pub const EMPTY_SHOTS: [u32; 6] = [10649676, 10846284, 10911820, 10977356, 10982581, 11192769];

/// The draw hooks that return at once: $A0:804C, the room's default, and
/// $A8:804C and $A9:8686.
pub const EMPTY_DRAW_HOOKS: [u32; 3] = [10518604, 11042892, 11110278];

/// A bank $B3 enemy pre-instruction on the scheduler's lease, keyed by its address.
pub fn b3_pre_instruction<L: TableLease>(ea: u32, k: u16, runtime: &mut L) {
    match ea {
        11763835 => (),
        11767039 | 11771328 | 11771517 | 11771616 | 11771700 | 11771770 => {
            runtime.lease_native_pre_instruction(ea, k);
        }
        _ => crate::diagnostics::Fatal::unreachable(runtime),
    }
}

/// Run the bank $A5 functions Draygon's dispatch shares but does not own: Spore
/// Spawn's ($A5:EB1A-$A5:EBEE), while retaining the scheduler's lease.
pub fn draygon_function_on_lease<L>(ea: u32, k: u16, game: &mut L)
where
    L: crate::actors::host::ActorLease + spore_spawn::Lease + crate::actors::boss_lease::BossLease,
{
    use crate::actors::host::ActorLease as Owned;
    match ea {
        10873626 => (),
        10873627 => Owned::owned_lease_call(game, spore_spawn::descend),
        10873682 => Owned::owned_lease_call(game, |game| spore_spawn::orbit(k, game)),
        10873755 => Owned::owned_lease_call(game, spore_spawn::begin_return),
        10873838 => Owned::owned_lease_call(game, spore_spawn::finish_return),
        _ => crate::diagnostics::Fatal::unreachable(game),
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
        10659396 => game.with_lease(gunship::initialize_top),
        10659538 => game.with_lease(gunship::initialize_bottom),
        10659673 => game.with_lease(|runtime| {
            let function = gunship::main(runtime);
            // The original pre-instruction samples the selector after main AI.
            let index = runtime.session().actors.controls.selected;
            if !gunship_pre_instruction(function, index, runtime) {
                runtime.run_other_pre_instruction(function);
            }
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
        // Draygon ($A5).
        10847879 => game.with_lease(|runtime| {
            draygon_flight::initialize(
                |runtime| {
                    crate::actors::boss_lease::BossLease::owned_step(
                        runtime,
                        crate::room_visuals::initialize_draygon_layer,
                    )
                },
                runtime,
            )
        }),
        10847996 | 10863750 => game.with_lease(|runtime| {
            draygon_flight::main_ai(ea == 10847996, draygon_function_on_lease, runtime)
        }),
        10851661 => game.with_lease(draygon_death::hurt_ai),
        10864025 => {
            game.with_lease(|runtime| draygon_flight::initialize_tail_or_arms(false, runtime))
        }
        10864045 => {
            game.with_lease(|runtime| draygon_flight::initialize_tail_or_arms(true, runtime))
        }
        10851818 => game.with_lease(draygon_death::touch),
        10851824 => game.with_lease(draygon_death::shot),
        10851847 => game.with_lease(draygon_death::power_bomb),
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
        11191934 => game.with_lease(torizo::hurt),
        11192695 => game.with_lease(torizo::touch),
        11192700 => game.with_lease(torizo::shot),
        11195322 => game.with_lease(torizo::golden_hurt),
        11191972 => game.with_lease(torizo::main_ai),
        11192447 => game.with_lease(torizo::initialize),
        11195241 => game.with_lease(torizo::golden_main_ai),
        11196007 => game.with_lease(torizo::golden_shot),
        // Bank $B2: the space pirates.
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
        11186332 => game.with_lease(|runtime| torizo::set_function(k, j, runtime)),
        11186461 => {
            game.with_lease(|runtime| torizo::blow_up_gut(k, runtime));
            j
        }
        11186622 => {
            game.with_lease(|runtime| torizo::blow_up_face(k, runtime));
            j
        }
        11186724 => {
            game.with_lease(|runtime| torizo::set_visible(k, runtime));
            j
        }
        11186734 => {
            game.with_lease(|runtime| torizo::set_invisible(k, runtime));
            j
        }
        11186765 => {
            game.with_lease(|runtime| torizo::defeated(k, runtime));
            j
        }
        11190985 => {
            game.with_lease(|runtime| torizo::lock_animation(k, runtime));
            j
        }
        11190993 => {
            game.with_lease(|runtime| torizo::unlock_animation(k, runtime));
            j
        }
        11191001 => game.with_lease(|runtime| torizo::branch_if_faceless(k, j, runtime)),
        11191021 => game.with_lease(|runtime| torizo::set_link(k, j, runtime)),
        11191043 => game.with_lease(|runtime| torizo::low_health_explosions(k, j, runtime)),
        11191087 => game.with_lease(|runtime| torizo::death_explosion(k, j, runtime)),
        11191131 => {
            game.with_lease(|runtime| torizo::drool_if_low_health(k, runtime));
            j
        }
        11191149 => {
            game.with_lease(|runtime| torizo::start_turning(k, runtime));
            j
        }
        11191159 => {
            game.with_lease(|runtime| torizo::stepped(k, runtime, 0));
            j
        }
        11191178 => {
            game.with_lease(|runtime| torizo::stepped(k, runtime, 0x8000));
            j
        }
        11191200 => {
            game.with_lease(|runtime| torizo::stepped(k, runtime, 0x2000));
            j
        }
        11191222 => {
            game.with_lease(|runtime| torizo::stepped(k, runtime, 0xa000));
            j
        }
        11191244 => game.with_lease(|runtime| torizo::stand_up(k, j, runtime)),
        11191326 => game.with_lease(|runtime| torizo::sit_down(k, j, runtime)),
        11191408 => game.with_lease(|runtime| {
            torizo::walk(
                k,
                j,
                torizo::list::TURNING_LEFT,
                torizo::list::TURNING_RIGHT,
                72,
                runtime,
            )
        }),
        11191525 => game.with_lease(|runtime| {
            torizo::walk(
                k,
                j,
                torizo::list::FACELESS_TURNING_LEFT,
                torizo::list::FACELESS_TURNING_RIGHT,
                72,
                runtime,
            )
        }),
        11191642 => game.with_lease(|runtime| torizo::branch_if_rising(k, j, runtime)),
        11191655 => game.with_lease(|runtime| torizo::call_if_samus_close_in_front(k, j, runtime)),
        11191691 => game.with_lease(|runtime| torizo::jump_back_if_samus_close(k, j, runtime)),
        11191716 => game.with_lease(|runtime| torizo::bomb_attack(k, j, runtime)),
        11191755 => {
            game.with_lease(|runtime| torizo::bomb_chozo_orbs(k, runtime));
            j
        }
        11191779 => game.with_lease(|runtime| torizo::bomb_sonic_boom(k, j, runtime)),
        11191794 => game.with_lease(|runtime| torizo::golden_sonic_boom(k, j, runtime)),
        11193038 => game.with_lease(|runtime| torizo::branch_if_airborne(k, j, runtime)),
        11193058 => {
            game.with_lease(|runtime| torizo::start_golden_fight(k, runtime));
            j
        }
        11193815 => {
            game.with_lease(|runtime| torizo::clear_caught_super_missile(k, runtime));
            j
        }
        11194747 => {
            game.with_lease(|runtime| torizo::disable_eye_beam_explosions(k, runtime));
            j
        }
        11194759 => {
            game.with_lease(|runtime| torizo::enable_eye_beam_explosions(k, runtime));
            j
        }
        11194855 => {
            game.with_lease(|runtime| torizo::clear_stunned(k, runtime));
            j
        }
        11195370 => {
            game.with_lease(|runtime| torizo::branch_if_samus_rolling_behind(k, j, runtime))
        }
        11195461 => {
            game.with_lease(|runtime| torizo::call_sometimes_if_samus_in_front(k, j, runtime))
        }
        11195508 => game.with_lease(|runtime| torizo::call_sometimes_at_low_health(k, j, runtime)),
        11195547 => {
            game.with_lease(|runtime| torizo::call_if_stunned_at_high_health(k, j, runtime))
        }
        11195578 => game.with_lease(|runtime| torizo::jump_forwards_if_samus_far(k, j, runtime)),
        11195635 => {
            game.with_lease(|runtime| torizo::golden_chozo_orbs(k, runtime));
            j
        }
        11195645 => game.with_lease(|runtime| torizo::golden_jump_back(k, j, runtime)),
        11195686 => game.with_lease(|runtime| torizo::golden_attack(k, j, runtime)),
        11195725 => game.with_lease(|runtime| {
            torizo::walk(
                k,
                j,
                torizo::list::GOLDEN_TURNING_LEFT,
                torizo::list::GOLDEN_TURNING_RIGHT,
                16,
                runtime,
            )
        }),
        11191031 => game.with_lease(|runtime| torizo::return_to_link(k, runtime)),
        11191037 => game.with_lease(|runtime| torizo::return_from_gut_explosion(k, runtime)),
        11186744 => {
            game.with_lease(torizo::fade_to_black);
            j
        }
        11186801 => {
            game.with_lease(torizo::advance_colour_change);
            j
        }
        11188557 => {
            game.with_lease(torizo::fade_to_active);
            j
        }
        11188561 => {
            game.with_lease(torizo::start_fight);
            j
        }
        11191114 => {
            game.with_lease(torizo::landing_dust);
            j
        }
        11191809 => game.with_lease(|runtime| torizo::explosive_swipe(j, runtime)),
        11191824 => {
            game.with_lease(torizo::shot_sound);
            j
        }
        11191832 => {
            game.with_lease(torizo::footstep_sound);
            j
        }
        11193054 => {
            game.with_lease(torizo::golden_palettes);
            j
        }
        11194601 => {
            game.with_lease(torizo::spawn_egg);
            j
        }
        11194611 => game.with_lease(|runtime| torizo::branch_if_egg_active(j, runtime)),
        11195279 => {
            game.with_lease(torizo::egg_released_sound);
            j
        }
        11195287 => {
            game.with_lease(torizo::laser_sound);
            j
        }
        11195295 => {
            game.with_lease(torizo::sonic_boom_sound);
            j
        }
        11195360 => {
            game.with_lease(torizo::golden_super_missile);
            j
        }
        11195446 => game.with_lease(|runtime| torizo::eye_beam(j, runtime)),
        // Draygon ($A5).
        10852159 => game.with_lease(|runtime| draygon_death::spawn_dying_sprite(21, j, runtime)),
        10852178 => game.with_lease(|runtime| draygon_death::spawn_dying_sprite(3, j, runtime)),
        10852197 => game.with_lease(|runtime| draygon_death::spawn_dying_sprite(29, j, runtime)),
        10852216 => game.with_lease(|runtime| draygon_death::spawn_dying_sprite(24, j, runtime)),
        10854268 | 10854318 => {
            game.with_lease(|runtime| draygon_death::spawn_goop(ea == 10854318, j, runtime))
        }
        10851549 => game.with_lease(|runtime| draygon_flight::set_instruction_lists(j, runtime)),
        10852150 => game.with_lease(|runtime| draygon_eye::set_function(k, j, runtime)),
        10852563 => game.with_lease(|runtime| draygon_flight::paralyse_tail_and_arms(j, runtime)),
        10852591 => game.with_lease(|runtime| draygon_flight::set_intangible(j, runtime)),
        10853274 => game.with_lease(|runtime| draygon_grab::tail_whip_hit(j, runtime)),
        10853898 => game.with_lease(|runtime| draygon_flight::displace_graphics(j, runtime)),
        10854231 => game.with_lease(|runtime| draygon_eye::set_function(0, j, runtime)),
        10863739 => game.with_lease(|runtime| draygon_eye::set_function(0x40, j, runtime)),
        _ => return Err(j),
    })
}

/// The contact reactions that run on one actor lease, keyed by address. False when `ea` is
/// not one of them.
pub fn lease_touch<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    match ea {
        10670984 => game.with_lease(|runtime| oum::touch(true, runtime)),
        10670988 => game.with_lease(|runtime| oum::touch(false, runtime)),
        11192695 => game.with_lease(torizo::touch),
        10851818 => game.with_lease(draygon_death::touch),
        10518563 | 10780707 => game.with_lease(|runtime| {
            crate::actors::combat::contact_reaction(runtime, DeathHandling::Finalize)
        }),
        // Ridley's contact leaves the death pending.
        10936153 => game.with_lease(|runtime| {
            crate::actors::combat::contact_reaction(runtime, DeathHandling::LeavePending)
        }),
        11700076 => game.with_lease(space_pirates::touch_on),
        _ => return false,
    }
    true
}

/// The shot reactions that run on one actor lease, keyed by address. False when `ea` is not
/// one of them.
pub fn lease_shot<H: LeaseScope + ?Sized>(ea: u32, game: &mut H) -> bool {
    match ea {
        10671028 => game.with_lease(oum::shot),
        11192700 => game.with_lease(torizo::shot),
        11192770 => game.with_lease(torizo::standing_shot),
        10851824 => game.with_lease(draygon_death::shot),
        10518573 | 10780717 | 10846253 => game.with_lease(|runtime| {
            crate::actors::combat::shot_reaction(
                runtime,
                crate::actors::api::ShotHandling::Finalize,
            )
        }),
        10846278 => game.with_lease(crate::actors::combat::create_dud),
        11700089 => game.with_lease(space_pirates::shot_on),
        11700168 => game.with_lease(space_pirates::check_shot_on),
        11700286 => game.with_lease(space_pirates::reflect_on),
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
        10851138 => game.with_lease(draygon_flight::set_bg2_scroll),
        _ => return false,
    }
    true
}

/// An enemy species' AI routine on the scheduler's lease, keyed by its address, for the
/// routines [`lease_ai`] does not run. A routine that continues through the pre-instruction
/// table runs it as its own owned call, where the runtime had opened a second lease.
pub fn table_ai<L: TableLease>(ea: u32, runtime: &mut L) {
    use crate::actors::combat::{die, latch, resume_ai, select_frozen_ai};
    match ea {
        // The common enemy AI ($A0:8000-$A0:8041) the enemy banks share: grapple AI - no
        // interaction, Samus latches on (with or without invincibility), kill enemy and
        // cancel grapple beam, and the normal enemy frozen AI.
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => resume_ai(runtime),
        10649605 | 10780677 => latch(runtime, LatchMode::Passive),
        11042836 => latch(runtime, LatchMode::Active),
        10649610 | 10715146 | 11042826 | 11763722 => {
            die(runtime, crate::actors::api::DeathKind::Grapple)
        }
        10649615 | 10715151 | 10846223 | 10911759 | 10977295 | 11042831 | 11042846 | 11173903
        | 11763727 => select_frozen_ai(runtime),
        10649665 | 10715201 | 10780737 | 10846273 | 10911809 | 10977345 | 11042881 | 11173953
        | 11763777 => crate::actors::scheduling::update_frozen_enemy(runtime),
        // Initialisation AI - enemy $F153 (unused spinning turtle eye) ($B3:86FB).
        11765499 => {
            let key = crate::actors::combat::current(&*runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.properties |=
                    crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16;
                actor.instruction = 0x86a7;
            });
        }
        // Main AI - mini-Crocomire ($A2:89F0) and cacatac ($A2:9FB3): their function.
        10652144 | 10657715 => {
            let mode = if ea == 10652144 { LittleEndian } else { Native };
            let key = crate::actors::combat::current(&*runtime);
            let function = runtime.resolve_actor(key, mode).pre_instruction;
            runtime.owned_lease_call(|runtime| {
                table_pre_instruction(0xa2_0000 | u32::from(function), runtime)
            });
        }
        10738716 => {
            if let Some(target) = reflec::reflect_shot(runtime) {
                runtime.lease_warn_reflection();
                reflec::absorb_shot(target, runtime);
            }
        }
        // Enemy shot - Wrecked Ship spark ($A8:E70E) and blue Brinstar face block ($A8:E91D).
        11069198 | 11069725 => wrecked_ship_spark::pass_shot(runtime),
        11763747 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        11763757 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871
        | 11108407 | 11173943 | 11698231 | 11763767 => {
            ActorRuntime::power_bomb_reaction(runtime, Death::Finalize)
        }
        10715212 => (),
        10725800 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        10723079 => (),
        10715171 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        10715181 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        10738715 => (),
        10780748 => (),
        10795344 => (),
        10811067 => (),
        10846284 => (),
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
        11791130 => (),
        10649635 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        10649645 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        10780707 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        10780717 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        // Bank $A6: the boulder, the fire arc and Mini-Kraid, with the bank's
        // common reactions.
        10911779 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        10911789 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        // Bank $A7: Kraid, Phantoon, the etecoons and the dachora, with the
        // bank's common reactions.
        10997235 => runtime.lease_phantoon(phantoon::Callback::Initialize),
        10997333 => runtime.lease_phantoon(phantoon::Callback::InitializePart),
        10997414 => runtime.lease_phantoon(phantoon::Callback::Main),
        11001151 => runtime.lease_phantoon(phantoon::Callback::Hurt),
        11001237 => runtime.lease_phantoon(phantoon::Callback::Touch),
        11001243 => runtime.lease_phantoon(phantoon::Callback::Shot),
        10977325 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        10977340 => ActorRuntime::power_bomb_reaction(runtime, Death::LeavePending),
        // Bank $A8: the common grapple, contact, shot and frozen reactions.
        11042851 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        11042861 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        // Bank $AA: the Torizos, Shaktool and the statues.
        0xb20000..=0xb2ffff => space_pirates::dispatch_ai(ea, runtime),
        // Bank $A5.
        10846243 => ActorRuntime::contact_reaction(runtime, Death::Finalize),
        10846253 => ActorRuntime::shot_reaction(runtime, Shot::Finalize),
        // Bank $A3: the crawlers, divers, platforms, bugs and fish.
        10720554 => elevator::main_ai(runtime),
        10739851 | 10741442 => {
            let function = crawlers::main_ai(runtime);
            runtime.owned_lease_call(|runtime| table_pre_instruction(function, runtime));
        }
        10730277 => bang::main_ai(runtime),
        10656381 => {
            if !puyo::main_ai(runtime) {
                crate::diagnostics::Fatal::unreachable(runtime);
            }
        }
        10653138 | 10653998 => {
            if !tatori::main_ai(ea == 10653998, runtime) {
                crate::diagnostics::Fatal::unreachable(runtime);
            }
        }
        11062135 => work_robot::initialize(runtime),
        11062220 => work_robot::initialize_deactivated(runtime),
        10914607 => spike_platform::initialize(runtime),
        10914733 => spike_platform::run(runtime),
        11063693 => work_robot::shot_deactivated(runtime),
        11063698 => work_robot::shot(runtime),
        10735204 => {
            let function = yard::prepare(runtime);
            runtime.owned_lease_call(|runtime| table_pre_instruction(function, runtime));
        }
        10663876 => {
            if !rinka::main_ai(runtime) {
                crate::diagnostics::Fatal::unreachable(runtime);
            }
        }
        10669331 => oum::main_ai(runtime),
        10658809 => cacatac::owtch::initialize(runtime),
        10658942 => cacatac::owtch::main_ai(runtime),
        10659193 => cacatac::owtch::shot(runtime),
        10742351 => metroid::initialize(runtime),
        10742502 => metroid::frozen(runtime),
        10742579 => metroid::hurt(runtime),
        10742680 => metroid::main_ai(runtime),
        10743275 => metroid::touch(runtime),
        10743559 => metroid::shot(runtime),
        10743874 => metroid::power_bomb(runtime),
        10726351 => sidehopper::main_ai(runtime),
        // Bank $A8.
        11045147 => evir::main_ai(runtime),
        11057165 => beetom::main_ai(runtime),
        11058734 => beetom::touch(runtime),
        11058860 => beetom::shot(runtime),
        11053955 => kago::shot(runtime),
        10662880 => multiviola::initialize(runtime),
        10662927 => multiviola::main_ai(runtime),
        10663311 => polyp::main_ai(runtime),
        10675113 => rippers::shot(runtime),
        10664931 => rio::main_ai(runtime),
        10666562 => geruta::initialize(false, runtime),
        10667763 => geruta::initialize(true, runtime),
        10665625 => squeept::initialize(runtime),
        10662175 | 10665682 | 10666615 | 10667812 => {
            crate::enemy_projectiles::host::Effects::next_random(runtime);
            let function = geruta::function(runtime);
            runtime.owned_lease_call(|runtime| table_pre_instruction(function, runtime));
        }
        10661995 => flies::initialize(runtime),
        10652077 => mini_crocomire::initialize(runtime),
        10716909 => waver::initialize(runtime),
        10717004 => waver::main_ai(runtime),
        10651420 => boyon::initialize(runtime),
        10651548 => boyon::main_ai(runtime),
        10675718 => dragon::initialize(runtime),
        10675790 => {
            let function = geruta::function(runtime);
            runtime.owned_lease_call(|runtime| table_pre_instruction(function, runtime));
        }
        10676168 | 10676174 | 10676180 => dragon::react(ea, runtime),
        10915836 => fire_pillar::initialize(runtime),
        10915875 => fire_pillar::main_ai(runtime),
        _ => runtime.lease_native_ai(ea),
    }
}

/// Gunship pre-instructions on the active actor lease; `index` is captured by
/// the caller before dispatch. The bottom's RTL remains a no-op.
pub fn gunship_pre_instruction(ea: u32, index: u16, runtime: &mut impl gunship::Lease) -> bool {
    let [low, high, _, _] = ea.to_le_bytes();
    match ea {
        10659852 | 10660048 | 10660162 | 10660176 | 10660231 | 10660285 | 10660431 | 10660445
        | 10660500 | 10660514 | 10660639 | 10660704 | 10660718 | 10660773 | 10660807 | 10660891
        | 10661079 | 10661134 | 10661165 => {
            gunship::run(runtime, index, u16::from_le_bytes([low, high]))
        }
        10659799 => (),
        _ => return false,
    }
    true
}

/// The Torizo pre-instructions selected before entering a physical callback.
pub fn is_torizo_pre_instruction(ea: u32) -> bool {
    matches!(ea, 11192146 | 11192364 | 11195889 | 11192360 | 11195885)
}

/// The empty bank $AA functions the pre-instruction table answers: $AA:807B,
/// the Torizo's and Shaktool's nothing movements ($AA:C95E, $AA:DCAA) and
/// $AA:DCAB, $AA:E7A6 and $AA:E7DA. With the Torizo pre-instructions they are
/// the table's only bank $AA addresses; any other fails as unreachable.
pub const EMPTY_AA_FUNCTIONS: [u32; 6] =
    [11174011, 11192670, 11197610, 11197611, 11200422, 11200474];

/// Run a selected Torizo pre-instruction using the caller's captured enemy.
pub fn torizo_pre_instruction<L: torizo::Lease>(ea: u32, k: u16, game: &mut L) {
    match ea {
        11192146 => torizo::walking(k, game),
        11192364 => torizo::jumping_or_falling(k, game),
        11195889 => torizo::golden_walking(k, game),
        11192360 | 11195885 => torizo::attacking(k, game),
        _ => unreachable!(),
    }
}

/// An enemy species' pre-instruction on the scheduler's lease, keyed by its address. The
/// current enemy index is read once, before dispatch, as the original did.
pub fn table_pre_instruction<L: TableLease>(ea: u32, runtime: &mut L) {
    let k = runtime.session().actors.controls.selected;
    if is_torizo_pre_instruction(ea) {
        torizo_pre_instruction(ea, k, runtime);
        return;
    }
    if matches!(
        ea,
        10659852
            | 10660048
            | 10660162
            | 10660176
            | 10660231
            | 10660285
            | 10660431
            | 10660445
            | 10660500
            | 10660514
            | 10660639
            | 10660704
            | 10660718
            | 10660773
            | 10660807
            | 10660891
            | 10661079
            | 10661134
            | 10661165
    ) {
        gunship_pre_instruction(ea, k, runtime);
        return;
    }
    match ea {
        10730661 => bang::movement_delay(runtime),
        10730693 => bang::target_samus(runtime),
        10730780 => bang::fly(false, runtime),
        10730796 => bang::fly(true, runtime),
        10739857 => crawlers::crawl(k, crate::actors::api::Axis::X, true, runtime),
        10740072 => crawlers::crawl(k, crate::actors::api::Axis::Y, true, runtime),
        10741448 => crawlers::crawl(k, crate::actors::api::Axis::X, false, runtime),
        10741746 => crawlers::crawl(k, crate::actors::api::Axis::Y, false, runtime),
        10741637 => crawlers::falling(k, runtime),
        10518651 => (),
        10669926 => oum::bounce(false, k, runtime),
        10669993 => oum::bounce(true, k, runtime),
        10657722 => cacatac::moving(false, runtime),
        10735526 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cce2, runtime),
        10735543 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccea, runtime),
        10735549 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3ccf2, runtime),
        10735566 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccfa, runtime),
        10735572 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd02, runtime),
        10735589 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd0a, runtime),
        10735595 => yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd12, runtime),
        10735612 => yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd1a, runtime),
        10735456 => yard::hiding(k, runtime),
        10736051 => yard::airborne(k, runtime),
        10657772 => cacatac::moving(true, runtime),
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
        ea if EMPTY_AA_FUNCTIONS.contains(&ea) => (),
        0xb30000..=0xb3ffff => b3_pre_instruction(ea, k, runtime),
        // Bank $A7: Phantoon, the etecoons and the dachora.
        10999114 | 10999271 | 10999309 | 10999416 | 10999481 | 10999522 | 10999597 | 10999615
        | 10999655 | 10999765 | 10999850 | 10999900 | 10999953 | 10999980 | 11000086 | 11000110
        | 11000136 | 11000401 | 11000454 | 11000535 | 11000637 | 10999388 | 10998953 | 10999022
        | 10999048 | 10999190 | 10999508 | 10999688 | 10999799 | 10999924 | 11000203 => runtime
            .lease_phantoon(phantoon::Callback::PreInstruction {
                function: ea as u16,
                actor: k,
            }),
        // Bank $AA: the Torizos, Shaktool and the statues.
        10666625 => geruta::flames(k, false, runtime),
        10666727 => geruta::idle(k, false, runtime),
        10666815 => geruta::finish_animation(k, false, false, runtime),
        10667014 => geruta::finish_animation(k, false, true, runtime),
        10667822 => geruta::flames(k, true, runtime),
        10667889 => geruta::idle(k, true, runtime),
        10667963 => geruta::finish_animation(k, true, false, runtime),
        10668168 => geruta::finish_animation(k, true, true, runtime),
        10666849 => geruta::swoop(k, false, true, runtime),
        10666929 => geruta::swoop(k, false, false, runtime),
        10667990 => geruta::swoop(k, true, true, runtime),
        10668077 => geruta::swoop(k, true, false, runtime),
        10665692 => squeept::flame(k, runtime),
        10665754 => squeept::jump(k, runtime),
        10665790 => squeept::fly(k, squeept::Flight::Rising, runtime),
        10665852 => squeept::fly(k, squeept::Flight::Flipping, runtime),
        10665916 => squeept::fly(k, squeept::Flight::Falling, runtime),
        10662222 => flies::circle(k, false, runtime),
        10662268 => flies::circle(k, true, runtime),
        10662314 => flies::dive(k, false, runtime),
        10662354 => flies::dive(k, true, runtime),
        10652227 => mini_crocomire::patrol(false, runtime),
        10652252 => mini_crocomire::patrol(true, runtime),
        10675796 | 10675885 | 10675953 | 10676020 | 10676041 => dragon::step(k, ea as u16, runtime),
        _ => runtime.lease_native_pre_instruction(ea, k),
    }
}

/// An enemy species' instruction on the scheduler's lease, keyed by its address, for the
/// instructions [`lease_instruction`] does not run; `k` is the enemy and `j` the script
/// position.
pub fn table_instruction<L: TableLease>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    runtime: &mut L,
) -> EnemyScript {
    match ea {
        // Instruction - enemy $0FB2 = [[Y]] ($AA:806B, $B3:806B) and = RTS ($AA:8074, $B3:8074).
        11173995 | 11174004 | 11763819 | 11763828 => {
            let given = ea & 0xffff == 0x806b;
            let value = if given { j.word(0) } else { 0x807b };
            let key = crate::actors::access::ActorKey::from_legacy(k);
            runtime.edit_actor(key, Native, |actor| actor.pre_instruction = value);
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
            crate::effects::SoundEffects::queue_sound(runtime, 1, sound, 6);
            j
        }
        // Instruction - queue sound [[Y]], sound library 2 or 3, max 6 ($A5:9F60, $A5:9F6E).
        10854240 | 10854254 => {
            crate::enemy_projectiles::host::Effects::queue_sound(
                runtime,
                if ea == 10854240 { 2 } else { 3 },
                6,
                j.word(0),
            );
            j.offset(1)
        }
        // Instruction - spawn mini-Crocomire ($A2:897E) or cacatac spike ($A2:A0A7) projectile.
        10652030 | 10657959 => {
            let definition = if ea == 10652030 { 0xdbf2 } else { 0xdafe };
            let enemy = runtime.session().actors.controls.selected;
            ActorRuntime::spawn_projectile(runtime, j.word(0), enemy, definition);
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
            runtime.session_mut().room.entry.irq_policy = 12;
            runtime.words_stored();
            j
        }
        // Death quake sound timer = [[Y]] ($A5:E872) and queue sound [[Y]] ($A5:E895).
        10872946 => {
            // $7E:9000, the bosses' RAM's first word.
            runtime.session_mut().actors.extended.blocks[3][0] = j.word(0);
            runtime.words_stored();
            j.offset(1)
        }
        10872981 => {
            crate::enemy_projectiles::host::Effects::queue_sound(runtime, 2, 6, j.word(0));
            j.offset(1)
        }
        10725992 => sidehopper::queue_sound(j, runtime),
        10657941 => cacatac::resume_moving(k, j, runtime),
        10734646 | 10734655 | 10734664 | 10734687 | 10734712 => {
            yard::instruction(ea, k, j, runtime)
        }
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216
        | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601
        | 11063613 | 11063659 => work_robot::instruction(ea, k, j, runtime),
        10654593 | 10654738 | 10654791 | 10654801 | 10654827 | 10654853 | 10654881 | 10654919 => {
            tatori::instruction(ea, j, runtime)
        }
        10669235 | 10669246 | 10669257 => oum::instruction(ea, j, runtime),
        10659181 | 10659185 => cacatac::owtch::set_function_index(ea, k, j, runtime),
        10742437 => metroid::draining_sound(j, runtime),
        // Bang's queue acquired suit sound effect: 56h, library 2.
        10730104 => {
            crate::effects::SoundEffects::queue_sound(runtime, 1, 0x56, 6);
            j
        }
        10738632 => reflec::set_reflection_axis(k, j, runtime),
        10741344 | 10739650 => crawlers::set_function(k, j, runtime),
        11048485 | 11048497 | 11048522 | 11048547 | 11048572 | 11048597 | 11048628 => {
            fune::instruction(ea, j, runtime)
        }
        11067164 | 11067187 | 11067193 | 11067199 | 11067235 | 11067249 => {
            alcoon::instruction(ea, j, runtime)
        }
        // Bank $A7: Kraid's steps and the calls into Phantoon's functions; Kraid's
        // identity instruction returns the script unchanged.
        10977418 => {
            runtime.lease_phantoon(phantoon::Callback::Instruction {
                address: u32::from(j.word(0)) | 0xa70000,
                actor: k,
            });
            j.offset(1)
        }
        // Bank $AA: the Torizos, Shaktool and the statues; the two identity
        // instructions return the script unchanged.
        11190984 | 11196831 => j,
        0xb20000..=0xb2ffff => space_pirates::dispatch_instruction(ea, k, j, runtime),
        10915247 | 10915347..=10915793 => fire_pillar::instruction(ea, j, runtime),
        10664899 => {
            rio::set_animation_finished(runtime);
            j
        }
        10666441 | 10666452 | 10666463 | 10666474 | 10666485 | 10666496 | 10666507 | 10666518
        | 10666529 | 10666540 | 10666551 | 10667730 | 10667741 | 10667752 => {
            geruta::instruction(ea, j, runtime)
        }
        10665614 => {
            squeept::set_animation_finished(runtime);
            j
        }
        10652048 => mini_crocomire::set_moving(false, j, runtime),
        10652061 => mini_crocomire::set_moving(true, j, runtime),
        10716899 => waver::set_spin_finished(j, runtime),
        10651846 => boyon::start_bounce(j, runtime),
        10675707 => dragon::set_animation_finished(j, runtime),
        _ => runtime.lease_native_instruction(ea, k, j),
    }
}
