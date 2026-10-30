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
    botwoon, brinstar_pipe_bug, crocomire_reactions, dachora_run, draygon_death, draygon_eye,
    draygon_flight, draygon_grab, draygon_host, escape_critters, etecoon, kraid, magdollite,
    norfair_pipe_bug, phantoon, phantoon_actions, shaktool, space_pirates, statues, torizo,
    yapping_maw, yellow_pipe_bug,
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
    MotherBrainBrain,
}

/// What the contact reactions need beyond their actors' own hosts.
pub trait TouchHost:
    SpeciesHost
    + ActorRuntime
    + crocomire_reactions::Host
    + draygon_host::Host
    + space_pirates::Pirates
    + torizo::Ai
    + shaktool::Ai
    + statues::Host
    + kraid::Host
    + phantoon::Host
{
    fn native_touch(&mut self, touch: NativeTouch);
}

/// An enemy species' contact reaction, keyed by its address.
pub fn touch<H: TouchHost>(ea: u32, game: &mut H) {
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
        10982539 => kraid::touch_arm_or_foot(game),
        10982544 => kraid::arm_touch(game),
        11001237 => phantoon::touch(game),
        11122117 => (),
        11122118 => game.native_touch(NativeTouch::MotherBrainBrain),
        11192695 => torizo::touch(game),
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
    MotherBrainBody,
    MotherBrainBrain,
}

/// What the shot reactions need beyond their actors' own hosts.
pub trait ShotHost:
    SpeciesHost
    + ActorRuntime
    + crocomire_reactions::Host
    + draygon_host::Host
    + space_pirates::Pirates
    + torizo::Ai
    + shaktool::Ai
    + statues::Host
    + kraid::Host
    + phantoon::Host
{
    fn native_shot(&mut self, shot: NativeShot);
}

/// An enemy species' shot reaction, keyed by its address; `j` is the projectile.
pub fn shot<H: ShotHost>(ea: u32, j: u16, game: &mut H) {
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
        10982582 => kraid::arm_shot(game, j),
        11001243 => phantoon::shot(game),
        11121923 => game.native_shot(NativeShot::MotherBrainBody),
        11121927 => game.native_shot(NativeShot::MotherBrainBrain),
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
pub fn draw_hook<H: DrawHookHost>(ea: u32, game: &mut H) {
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
pub fn pipe_bug_state<H: B3Host>(ea: u32, _k: u16, game: &mut H) {
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
pub fn b3_pre_instruction<H: B3Host>(ea: u32, k: u16, game: &mut H) {
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
    B3Host
    + DrawHookHost
    + yapping_maw::Host
    + crocomire_reactions::Host
    + space_pirates::Pirates
    + torizo::Ai
    + shaktool::Ai
    + statues::Host
    + kraid::Host
    + phantoon::Host
    + etecoon::Host
    + dachora_run::Host
{
    /// An AI routine whose arm is still in the compatibility table.
    fn native_ai(&mut self, ea: u32);
    /// Kill the current enemy with the grapple, through one lease.
    fn grapple_kill(&mut self);
}

/// An enemy species' AI routine, keyed by its address. Arms not here are in
/// the compatibility table until their routines move.
pub fn ai<H: AiHost>(ea: u32, game: &mut H) {
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
        // Bank $A7: Kraid, Phantoon, the etecoons and the dachora, with the
        // bank's common reactions.
        10982544 => kraid::arm_touch(game),
        10987865 => kraid::initialize(game),
        10988355 => kraid::initialize_arm(game),
        10988392 => kraid::initialize_lint(game, 0x80),
        10988444 => kraid::initialize_lint(game, 0xc0),
        10988490 => kraid::initialize_lint(game, 0x100),
        10988536 => kraid::initialize_foot(game),
        10988577 => kraid::main_ai(game),
        10991549 => kraid::arm_main(game),
        10991617 => kraid::lint_main_ai(game, 0x80),
        10991629 => kraid::lint_main_ai(game, 0xc0),
        10991641 => kraid::lint_main_ai(game, 0x100),
        10992118 => kraid::foot_main(game),
        10992847 | 10992862 => kraid::fingernail_touch(game),
        10992946 => kraid::fingernail_main(game, 0x180),
        10992969 => kraid::fingernail_main(game, 0x1c0),
        10997235 => phantoon::initialize(game),
        10997333 => phantoon::initialize_part(game),
        10997414 => phantoon::main_ai(game),
        11001151 => phantoon::hurt(game),
        11001237 => phantoon::touch(game),
        11001243 => phantoon::shot(game),
        11004178 => etecoon::initialize(game),
        11004224 => etecoon::main_ai(game),
        11007197 => dachora_run::initialize(game),
        11007278 => dachora_run::main_ai(game),
        10977295 => crate::actors::combat::select_frozen_ai(&mut LeaseActors::lease(game)),
        10977325 => game.shot_reaction(Shot::Finalize),
        10977340 => game.power_bomb_reaction(Death::LeavePending),
        10977345 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        10982559 => kraid::touch(game),
        10992879 | 10992941 => kraid::initialize_fingernail(game),
        // Bank $A8: the common grapple, contact, shot and frozen reactions.
        11042826 => game.grapple_kill(),
        11042831 | 11042846 => {
            crate::actors::combat::select_frozen_ai(&mut LeaseActors::lease(game))
        }
        11042851 => game.contact_reaction(Death::Finalize),
        11042861 => game.shot_reaction(Shot::Finalize),
        11042881 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        // Bank $AA: the Torizos, Shaktool and the statues.
        11191934 => torizo::hurt(game),
        11192695 => torizo::touch(game),
        11192700 => torizo::shot(game),
        11195322 => torizo::golden_hurt(game),
        11196360 => statues::initialize_tourian_statue(game),
        11197603 => shaktool::main_ai(game),
        11198019 => shaktool::initialize(game),
        11198255 => shaktool::touch(game),
        11198260 => shaktool::shot(game),
        11200278 => statues::initialize_tube_cracks(game),
        11200293 => statues::initialize_chozo_statue(game),
        11200423 => statues::main_ai(game),
        11191972 => torizo::main_ai(game),
        11192447 => torizo::initialize(game),
        11195241 => torizo::golden_main_ai(game),
        11196007 => torizo::golden_shot(game),
        11173903 => crate::actors::combat::select_frozen_ai(&mut LeaseActors::lease(game)),
        11173953 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        // Bank $B2: the space pirates.
        0xb20000..=0xb2ffff => space_pirates::dispatch_ai(ea, game),
        // Bank $A5.
        10851661 => draygon_death::hurt_ai(game),
        10864025 => draygon_flight::initialize_tail_or_arms(false, game),
        10864045 => draygon_flight::initialize_tail_or_arms(true, game),
        10846223 => crate::actors::combat::select_frozen_ai(&mut LeaseActors::lease(game)),
        10846243 => game.contact_reaction(Death::Finalize),
        10846253 => game.shot_reaction(Shot::Finalize),
        10846273 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        10851818 => draygon_death::touch(game),
        10851847 => draygon_death::power_bomb(game),
        10863723 => draygon_eye::initialize(game),
        _ => game.native_ai(ea),
    }
}

/// What the pre-instruction table needs beyond its actors' own hosts.
pub trait PreInstructionHost:
    B3Host
    + magdollite::Host
    + torizo::Ai
    + shaktool::Ai
    + statues::Host
    + phantoon::Host
    + etecoon::Host
    + dachora_run::Host
{
    /// A pre-instruction whose arm is still in the compatibility table, for
    /// the current enemy `k` read before dispatch.
    fn native_pre_instruction(&mut self, ea: u32, k: u16);
}

/// An enemy species' pre-instruction, keyed by its address. The current
/// enemy index is read once, before dispatch, as the original did.
pub fn pre_instruction<H: PreInstructionHost>(ea: u32, game: &mut H) {
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
        11004335 => etecoon::initial(game, k),
        11004471 => etecoon::hopping_bottom(game, k),
        11004674 => etecoon::running_left(game, k),
        11004716 => etecoon::running_right(game, k),
        11004752 => etecoon::jumping(game, k),
        11004877 => etecoon::wall_jump(game, k),
        11005079 => etecoon::run_to_left_run_up_point(game, k),
        11005115 => etecoon::run_to_right_run_up_point(game, k),
        11005151 => etecoon::running_for_tunnel_jump(game, k),
        11005193 => etecoon::tunnel_jump(game, k),
        11005226 => etecoon::running_through_tunnel(game, k),
        11005268 => etecoon::falling_from_tunnel_ledge(game, k),
        11005301 => etecoon::hopping_top(game, k),
        11005383 => etecoon::start_hop_top(game, k),
        11005502 => etecoon::hop_until_samus_near(game, k),
        11005594 => etecoon::running_for_failed_tunnel_jump(game, k),
        11005624 => etecoon::failed_tunnel_jump(game, k),
        11007344 => dachora_run::wait_for_samus(game, k),
        11007420 => dachora_run::start_running(game, k),
        11007469 => dachora_run::running_left(game, k),
        11007582 => dachora_run::running_right(game, k),
        11007887 => dachora_run::activate_shinespark(game, k),
        11008006 => dachora_run::shinesparking(game, k),
        11008309 => dachora_run::falling(game, k),
        11008396 => dachora_run::echo(game, k),
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
        11004416 => etecoon::start_hop_bottom(game, k),
        11004597 => etecoon::look_at_samus(game, k),
        11004955 => etecoon::landed_from_jump(game, k),
        // Bank $AA: the Torizos, Shaktool and the statues.
        11192146 => torizo::walking(k, game),
        11192364 => torizo::jumping_or_falling(k, game),
        11195889 => torizo::golden_walking(k, game),
        11192360 | 11195885 => torizo::attacking(k, game),
        11199557 => statues::lower_norfair(game, k),
        11200430 => statues::wrecked_ship(game, k),
        11197612 => shaktool::arm_piece(k, game),
        11197655 => shaktool::head(k, game),
        11197733 => shaktool::final_piece(k, game),
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
    + shaktool::Ai
    + statues::Host
    + kraid::Host
    + phantoon::Host
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
        // Bank $A7: Kraid's steps and the calls into Phantoon's functions; Kraid's
        // identity instruction returns the script unchanged.
        10977418 => {
            call_a7_function(j.word(0) as u32 | 0xa70000, k, game);
            j.offset(1)
        }
        10991155 => j,
        10979983 => match kraid::arm_swipe_when_hurt(game) {
            Some(list) => EnemyScript::jump(list),
            None => j,
        },
        10991158 => {
            kraid::step_up(game);
            j
        }
        10991164 => {
            kraid::step_down_and_shake(game);
            j
        }
        10991182 => {
            kraid::stomp_sound(game);
            j
        }
        10991194 => {
            kraid::step_back(game);
            j
        }
        10991207 => {
            kraid::step_back(game);
            j
        }
        10991220 => {
            kraid::step_forward(game);
            j
        }
        10991235 => {
            kraid::lunge_right(game);
            j
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
        11196721 => shaktool::lower_away_from_projectile(k, j, game),
        11196735 => shaktool::raise_towards_projectile(k, j, game),
        11196746 => shaktool::lower(k, j, game),
        11196755 => shaktool::raise(k, j, game),
        11196858 => shaktool::reset_functions(k, j, game),
        11199529 => {
            statues::start_lowering_acid(game);
            j
        }
        11199542 => {
            statues::set_lowered_acid(game);
            j
        }
        11199549 => {
            statues::unlock_samus(game);
            j
        }
        11199871 => {
            statues::grab_sound(game);
            j
        }
        11199879 => {
            statues::footstep_sound(game);
            j
        }
        11200240 => {
            statues::release_samus(game);
            j
        }
        11199887 => statues::spike_clearing_footstep(game, k, j),
        11199960 => statues::movement(game, k, j),
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
        _ => game.native_instruction(ea, k, j),
    }
}
