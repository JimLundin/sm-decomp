//! Enemy species dispatch tables keyed by the routine addresses the original
//! enemy definitions name: AI, pre-instructions, instructions, contact and
//! shot reactions, graphics draw hooks, bank $B3 pre-instructions and pipe bug
//! state routines.
//!
//! Each table is one `match` from address to routine. Arms whose routines
//! still run in the compatibility crate go to its host: a named native enum
//! for the small tables, the same address for the large ones. Moving a
//! routine here means moving its arm.
use crate::actors::host::{ActorHost, ActorRuntime, Death, LeaseActors, Shot};
use crate::actors::{
    alcoon, atomic, bang, beetom, blue_brinstar_face_block, botwoon, boulder, bull, cacatac,
    ceres_doors, ceres_ridley, choot, covern, crawlers, crocomire_reactions, dachora_run,
    draygon_death, draygon_eye, draygon_flight, draygon_grab, draygon_host, draygon_sweep,
    elevator, escape_critters, etecoon, evir, fire_arc, fireflea, fune, gamet, geega, geruta, kago,
    kihunter, kraid, magdollite, metroid, mini_kraid, mochtroid, morph_ball_eye, multiviola, oum,
    phantoon, phantoon_actions, polyp, powamp, puyo, reflec, ridley, rinka, rio, rippers, roach,
    shaktool, sidehopper, skree, skultera, space_pirates, squeept, statues, tatori, torizo,
    tripper, work_robot, wrecked_ship_spark, yapping_maw, yard, zeb, zebetites, zoa,
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
        10670984 => oum::touch(true, game),
        10670988 => oum::touch(false, game),
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
        10671028 => oum::shot(game),
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
    CeresRidleyBabyMetroidAndWings,
    BlueFaceBlockPalette,
    MotherBrainBrainAndNeck,
    MotherBrainBrain,
    DeadTorizo,
}

/// What the draw hooks need beyond their actors' own hosts.
pub trait DrawHookHost: SpeciesHost + LeaseActors + magdollite::Host + draygon_host::Host {
    fn native_draw_hook(&mut self, hook: NativeDrawHook);
}

/// An enemy species' graphics draw hook, keyed by its address.
pub fn draw_hook<H: DrawHookHost>(ea: u32, game: &mut H) {
    match ea {
        10518604 => (),
        10738444 => reflec::cycle_palette(game),
        10851138 => draygon_flight::set_bg2_scroll(game),
        10920690 => game.native_draw_hook(NativeDrawHook::CeresRidleyBabyMetroidAndWings),
        11042892 => (),
        11055282 => magdollite::cycle_palette(game),
        11062375 => work_robot::palette(game),
        11069550 => game.native_draw_hook(NativeDrawHook::BlueFaceBlockPalette),
        11110278 => (),
        11110345 => game.native_draw_hook(NativeDrawHook::MotherBrainBrainAndNeck),
        11110365 => game.native_draw_hook(NativeDrawHook::MotherBrainBrain),
        11129754 => game.native_draw_hook(NativeDrawHook::DeadTorizo),
        _ => game.unknown_routine(),
    }
}

/// A pipe bug's state routine, keyed by its bank $B3 address.
pub fn pipe_bug_state<H: B3Host>(ea: u32, game: &mut H) {
    match ea {
        11766733 => crate::actors::gamet::await_group(game),
        11766783 => crate::actors::gamet::await_player(game),
        11766950 => crate::actors::gamet::rise(game),
        11767039 => crate::actors::gamet::settle(game),
        11767052 => crate::actors::gamet::move_to_limit(-16, game),
        11767118 => crate::actors::gamet::move_to_limit(-32, game),
        11767184 => crate::actors::gamet::move_to_limit(16, game),
        11767250 => crate::actors::gamet::move_to_limit(32, game),
        11767316 => crate::actors::gamet::fly(2, game),
        11767349 => crate::actors::gamet::fly(0, game),
        11767386 => crate::actors::gamet::start_flight(game),
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
    + ceres_ridley::Host
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
    + boulder::Host
    + ceres_doors::Host
    + fire_arc::Host
    + mini_kraid::Host
    + zebetites::Host
    + PreInstructionHost
    + fireflea::Host
    + skultera::Host
    + atomic::Host
    + crate::actors::host::EnemyHost
    + crate::effects::SoundEffects
{
    /// An AI routine whose arm is still in the compatibility table.
    fn native_ai(&mut self, ea: u32);
    /// Kill the current enemy with the grapple, through one lease.
    fn grapple_kill(&mut self);
    /// Write `operand` to the multiplicand register, for Yapping Maw.
    fn write_multiplicand(&mut self, operand: u8);
    /// Install Draygon's BG layers as its initialisation does.
    fn install_draygon_layers(&mut self);
    /// A bank $A5 function Draygon's dispatch shares with Spore Spawn, whose
    /// routines still run in the compatibility crate.
    fn native_draygon_function(&mut self, ea: u32, k: u16);
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
        10846283 => (),
        10849331 => draygon_flight::goop_move_until_off_screen(true, game),
        10849714 => draygon_flight::goop_move_until_off_screen(false, game),
        10850078 => draygon_grab::move_to_target(k, game),
        10850262 => draygon_grab::rising_spiral(k, game),
        10850516 => draygon_grab::tail_whip(k, game),
        10850565 => draygon_grab::final_tail_whips(k, game),
        10850596 => draygon_grab::move_samus_with_draygon(k, game),
        10863757 => draygon_eye::look_at_samus(k, false, game),
        10863891 => draygon_eye::look_at_samus(k, true, game),
        _ => game.native_draygon_function(ea, k),
    }
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
        11765819 => zeb::initialize(game),
        11766625 => gamet::initialize(game),
        11767628 => geega::initialize(game),
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
        // Bank $A6: the boulder, the fire arc, Mini-Kraid, Ceres's steam and
        // doors and the zebetites, with the bank's common reactions.
        10911759 => crate::actors::combat::select_frozen_ai(&mut LeaseActors::lease(game)),
        10911779 => game.contact_reaction(Death::Finalize),
        10911789 => game.shot_reaction(Shot::Finalize),
        10911809 => crate::actors::scheduling::update_frozen_enemy(&mut LeaseActors::lease(game)),
        10913525 => boulder::initialize(game),
        10913683 => boulder::main_ai(game),
        10914693 | 10914713 => fire_arc::initialize_spike_platform_bottom(game),
        10917060 => fire_arc::initialize(game),
        10917390 => fire_arc::main_ai(game),
        10918488 => mini_kraid::initialize(game),
        10918594 => mini_kraid::main_ai(game),
        10918946 => mini_kraid::touch(game),
        10918969 => mini_kraid::shot(game),
        10940337 => ceres_doors::initialize_steam(game),
        10940429 => ceres_doors::steam_main(game),
        10942149 => ceres_doors::initialize_door(game),
        10942309 => ceres_doors::door_main(game),
        10943346 => zebetites::initialize(game),
        10943539 => zebetites::main_ai(game),
        10943911 => zebetites::touch(game),
        10943916 => zebetites::shot(game),
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
        10847879 => draygon_flight::initialize(|game: &mut H| game.install_draygon_layers(), game),
        10847996 => draygon_flight::main_ai(true, draygon_function, game),
        10863750 => draygon_flight::main_ai(false, draygon_function, game),
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
        // Bank $A3: the crawlers, divers, platforms, bugs and fish.
        10717536 => skree::initialize(skree::Kind::Metaree, game),
        10717561 => skree::main_ai(skree::Kind::Metaree, game),
        10717967 => skree::shot(skree::Kind::Metaree, game),
        10718509 => fireflea::initialize(game),
        10718702 => fireflea::main_ai(game),
        10718827 => fireflea::touch(game),
        10718851 => fireflea::power_bomb(game),
        10718857 => fireflea::shot(game),
        10719413 => skultera::initialize(game),
        10719531 => skultera::main_ai(game),
        10720486 => elevator::initialize(game),
        10720554 => elevator::main_ai(game),
        10722463 => tripper::initialize(true, game),
        10722490 => tripper::initialize(false, game),
        10722582 => tripper::main_ai(game),
        10723080 => tripper::shot(game),
        10725245 => mochtroid::initialize(game),
        10725264 => mochtroid::main_ai(game),
        10725715 => mochtroid::touch(game),
        10720995 => crawlers::initialize(crawlers::Kind::Sciser, game),
        10721595 => crawlers::initialize(crawlers::Kind::Zero, game),
        10729071 => crawlers::initialize(crawlers::Kind::Viola, game),
        10739779 => crawlers::initialize(crawlers::Kind::OrangeGeemer, game),
        10740436 => crawlers::initialize(crawlers::Kind::Zeela, game),
        10741148 => crawlers::initialize(crawlers::Kind::Sova, game),
        10741353 => crawlers::initialize(crawlers::Kind::Geemer, game),
        10739851 | 10741442 => {
            let function = crawlers::main_ai(game);
            pre_instruction(function, game);
        }
        10730163 => bang::initialize(game),
        10731261 => bang::shot(game),
        10738643 => reflec::initialize(game),
        10723661 => roach::initialize(game),
        10724048 => roach::main_ai(game),
        10656319 => puyo::initialize(game),
        10656381 => {
            if !puyo::main_ai(game) {
                game.unknown_routine();
            }
        }
        10653036 | 10653085 => tatori::initialize(ea == 10653085, game),
        10653138 | 10653998 => {
            if !tatori::main_ai(ea == 10653998, game) {
                game.unknown_routine();
            }
        }
        10654337 | 10654367 => tatori::touch(ea == 10654367, game),
        10654479 => tatori::shot(game),
        10663426 => rinka::initialize(game),
        11062135 => work_robot::initialize(game),
        11062220 => work_robot::initialize_deactivated(game),
        11062326 => work_robot::main_ai(game),
        11063668 => work_robot::touch(game),
        11063693 => work_robot::shot_deactivated(game),
        11063698 => work_robot::shot(game),
        11065545 => bull::initialize(game),
        11065611 => bull::main_ai(game),
        11066132 => bull::shot(game),
        10735074 => yard::initialize(game),
        10735204 => {
            let function = yard::prepare(game);
            pre_instruction(function, game);
        }
        10736560 => yard::touch(game),
        10736745 => yard::shot(game),
        11059657 => powamp::initialize(game),
        11059740 => {
            let function = powamp::function(game);
            pre_instruction(function, game);
        }
        11060670 => powamp::touch(game),
        11060719 => powamp::shot(game),
        11060799 => powamp::power_bomb(game),
        11071880 => kihunter::initialize(game),
        11072020 => kihunter::initialize_wings(game),
        11072092 | 11072098 => kihunter::main_ai(game),
        11073281 => kihunter::shot(game),
        11068983 => wrecked_ship_spark::initialize(game),
        11069070 => wrecked_ship_spark::main_ai(game),
        10663876 => {
            if !rinka::main_ai(game) {
                game.unknown_routine();
            }
        }
        10664233 => rinka::frozen(game),
        10664263 | 10664269 => rinka::react(ea == 10664269, game),
        10664275 => rinka::power_bomb(game),
        10669268 => oum::initialize(game),
        10669331 => oum::main_ai(game),
        10670988 => oum::touch(false, game),
        10671028 => oum::shot(game),
        10657608 => cacatac::initialize(game),
        10658809 => cacatac::owtch::initialize(game),
        10658942 => cacatac::owtch::main_ai(game),
        10659193 => cacatac::owtch::shot(game),
        10674038 => choot::initialize(game),
        10674222 => choot::main_ai(game),
        10742351 => metroid::initialize(game),
        10742502 => metroid::frozen(game),
        10742579 => metroid::hurt(game),
        10742680 => metroid::main_ai(game),
        10743275 => metroid::touch(game),
        10743559 => metroid::shot(game),
        10743874 => metroid::power_bomb(game),
        10726153 => sidehopper::initialize(game),
        10726351 => sidehopper::main_ai(game),
        10728522 => zoa::initialize(game),
        10728572 => zoa::main_ai(game),
        10733230 => skree::initialize(skree::Kind::Skree, game),
        10733255 => skree::main_ai(skree::Kind::Skree, game),
        10733557 => skree::shot(skree::Kind::Skree, game),
        // Bank $A8.
        11044832 => evir::initialize(game),
        11045040 => evir::initialize_projectile(game),
        11045147 => evir::main_ai(game),
        11045278 => evir::projectile_main_ai(game),
        11045638 | 11045644 | 11045650 => evir::react(ea, game),
        11047000 => morph_ball_eye::initialize(game),
        11057014 => beetom::initialize(game),
        11057165 => beetom::main_ai(game),
        11058734 => beetom::touch(game),
        11058860 => beetom::shot(game),
        11047138 => {
            if let Some(function) = morph_ball_eye::main_ai(game) {
                pre_instruction(function, game);
            }
        }
        11048675 => fune::initialize(game),
        11048752 => fune::main_ai(game),
        11049710 => covern::initialize(game),
        11049788 => covern::main_ai(game),
        11053894 => kago::initialize(game),
        11053941 => kago::main_ai(game),
        11053955 => kago::shot(game),
        11066573 => alcoon::initialize(game),
        11066731 => alcoon::main_ai(game),
        11068296 => atomic::initialize(game),
        11068355 => atomic::main_ai(game),
        11069486 => blue_brinstar_face_block::initialize(game),
        11069614 => blue_brinstar_face_block::main_ai(game),
        10920181 => ceres_ridley::init(game),
        10920584 => ceres_ridley::main(game),
        10920659 => ceres_ridley::hurt(game),
        10924583 => ridley::main(game),
        10924682 => ridley::time_frozen(game),
        10924695 => ridley::hurt(game),
        10929814 => ridley::explosion_init(game),
        10930388 => ridley::explosion_main(game),
        10936202 => ridley::shot(game),
        10936242 => ridley::power_bomb(game),
        11051537 => {
            if !yapping_maw::main_ai(game, |operand, game| game.write_multiplicand(operand)) {
                game.unknown_routine();
            }
            yapping_maw::update_segments(game);
        }
        11055370 => pre_instruction(magdollite::main_ai(game), game),
        11765882 => pre_instruction(zeb::state_address(game), game),
        11766686 => {
            pipe_bug_state(gamet::state_address(game), game);
            gamet::reset_offscreen(game);
        }
        11767726 => pipe_bug_state(geega::state_address(game), game),
        11789909 => pre_instruction(escape_critters::main_ai(game), game),
        10662880 => multiviola::initialize(game),
        10662927 => multiviola::main_ai(game),
        10663280 => polyp::initialize(game),
        10663311 => polyp::main_ai(game),
        10674643 => rippers::initialize(rippers::Kind::Gripper, game),
        10674721 => rippers::main_ai(rippers::Kind::Gripper, game),
        10674968 => rippers::initialize(rippers::Kind::RipperII, game),
        10675027 => rippers::main_ai(rippers::Kind::RipperII, game),
        10675113 => rippers::shot(game),
        10675359 => rippers::initialize(rippers::Kind::Ripper, game),
        10675418 => rippers::main_ai(rippers::Kind::Ripper, game),
        10664909 => rio::initialize(game),
        10664931 => rio::main_ai(game),
        10666562 => geruta::initialize(false, game),
        10667763 => geruta::initialize(true, game),
        10665625 => squeept::initialize(game),
        10662175 | 10665682 | 10666615 | 10667812 => {
            game.next_random();
            pre_instruction(geruta::function(game), game);
        }
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
    + crate::actors::host::EnemyHost
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
        _ => game.native_pre_instruction(ea, k),
    }
}

/// What the instruction table needs beyond its actors' own hosts.
pub trait InstructionHost:
    SpeciesHost
    + ridley::Host
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
    + ceres_doors::Host
    + mini_kraid::Host
    + skultera::Host
    + crate::actors::host::EnemyHost
    + crate::effects::SoundEffects
    + botwoon::Host
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
        10717526 | 10733220 => skree::set_attack_ready(j, game),
        10722411 | 10722433 => tripper::set_x_movement(false, j, game),
        10722422 | 10722444 => tripper::set_x_movement(true, j, game),
        10725992 => sidehopper::queue_sound(j, game),
        10657941 => cacatac::resume_moving(k, j, game),
        11068957 | 11068970 => wrecked_ship_spark::set_intangible(game, ea == 11068957, j),
        10734646 | 10734655 | 10734664 | 10734687 | 10734712 => yard::instruction(ea, k, j, game),
        11072806 | 11072996 | 11073151 | 11073234 | 11073240 => {
            kihunter::instruction(game, ea, k, j)
        }
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216
        | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601
        | 11063613 | 11063659 => work_robot::instruction(ea, k, j, game),
        10654593 | 10654738 | 10654791 | 10654801 | 10654827 | 10654853 | 10654881 | 10654919 => {
            tatori::instruction(ea, j, game)
        }
        10664371 | 10664381 | 10664391 => rinka::instruction(ea, k, j, game),
        10669235 | 10669246 | 10669257 => oum::instruction(ea, j, game),
        10659181 | 10659185 => cacatac::owtch::set_function_index(ea, k, j, game),
        10742437 => metroid::draining_sound(j, game),
        10742449 => metroid::random_cry(j, game),
        11044751 | 11044763 | 11044790 | 11044811 => evir::instruction(ea, j, game),
        10726142 => sidehopper::ready_to_hop(j, game),
        10728489 => zoa::set_x_speed_index(4, j, game),
        10728500 => zoa::set_x_speed_index(8, j, game),
        10728511 => zoa::set_x_speed_index(12, j, game),
        // Bang's queue acquired suit sound effect: 56h, library 2.
        10730104 => {
            game.queue_sound(1, 0x56, 6);
            j
        }
        10730152 => bang::set_finished_growing(j, game),
        10738632 => reflec::set_reflection_axis(k, j, game),
        10741344 | 10739650 => crawlers::set_function(k, j, game),
        10719382 | 10719392 | 10719402 => skultera::instruction(ea, j, game),
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
        // Bank $A6: Mini-Kraid and the steam.
        10918694 => {
            mini_kraid::step(game);
            j
        }
        10918772 => mini_kraid::choose_animation(game),
        10918834 => {
            mini_kraid::roar(game);
            j
        }
        10918852 => {
            mini_kraid::spit_left(game);
            j
        }
        10918914 => {
            mini_kraid::spit_right(game);
            j
        }
        10940711 => ceres_doors::steam_activate(k, j, game),
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
        0xa60000..=0xa6ffff => match ridley::instruction(ea, j, game) {
            Ok(next) => next,
            Err(j) => game.native_instruction(ea, k, j),
        },
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
        _ => game.native_instruction(ea, k, j),
    }
}
