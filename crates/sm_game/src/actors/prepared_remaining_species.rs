//! Remaining species callbacks bind to the actual owned body modules.
//! This source packet is partial: HANDOFF names the families/providers absent.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::content::ActorRoutine;
use crate::actors::remaining_owned as owned;
use crate::enemy_script::EnemyScript;

use owned::shutters::{Event, Kind};
pub trait Host:
    owned::Core
    + crate::actors::remaining_core::PreInstruction
    + owned::alcoon::Host
    + owned::atomic::Host
    + owned::bang::Host
    + owned::beetom::Host
    + owned::blue_brinstar_face_block::Host
    + owned::boulder::Host
    + owned::boyon::Host
    + owned::bull::Host
    + owned::cacatac::Host
    + owned::covern::Host
    + owned::crawlers::Host
    + owned::dragon::Host
    + owned::elevator::Host
    + owned::escape_critters::Host
    + owned::evir::Host
    + owned::fire_arc::Host
    + owned::fire_pillar::Host
    + owned::fireflea::Host
    + owned::flies::Host
    + owned::fune::Host
    + owned::gamet::Host
    + owned::geega::Host
    + owned::geruta::Host
    + owned::gunship::Host
    + owned::kago::Host
    + owned::kihunter::Host
    + owned::magdollite::Host
    + owned::mini_crocomire::Host
    + owned::morph_ball_eye::Host
    + owned::multiviola::Host
    + owned::oum::Host
    + owned::polyp::Host
    + owned::powamp::Host
    + owned::puyo::Host
    + owned::reflec::Host
    + owned::rinka::Host
    + owned::rio::Host
    + owned::shaktool::Host
    + owned::shutters::Host
    + owned::skree::Host
    + owned::skultera::Host
    + owned::spike_platform::Host
    + owned::spore_spawn::Lease
    + owned::squeept::Host
    + owned::tatori::Host
    + owned::torizo::Lease
    + owned::waver::Host
    + owned::work_robot::Host
    + owned::wrecked_ship_spark::Host
    + owned::yard::Host
    + owned::zeb::Host
{
}
impl<
    T: owned::Core
        + crate::actors::remaining_core::PreInstruction
        + owned::alcoon::Host
        + owned::atomic::Host
        + owned::bang::Host
        + owned::beetom::Host
        + owned::blue_brinstar_face_block::Host
        + owned::boulder::Host
        + owned::boyon::Host
        + owned::bull::Host
        + owned::cacatac::Host
        + owned::covern::Host
        + owned::crawlers::Host
        + owned::dragon::Host
        + owned::elevator::Host
        + owned::escape_critters::Host
        + owned::evir::Host
        + owned::fire_arc::Host
        + owned::fire_pillar::Host
        + owned::fireflea::Host
        + owned::flies::Host
        + owned::fune::Host
        + owned::gamet::Host
        + owned::geega::Host
        + owned::geruta::Host
        + owned::gunship::Host
        + owned::kago::Host
        + owned::kihunter::Host
        + owned::magdollite::Host
        + owned::mini_crocomire::Host
        + owned::morph_ball_eye::Host
        + owned::multiviola::Host
        + owned::oum::Host
        + owned::polyp::Host
        + owned::powamp::Host
        + owned::puyo::Host
        + owned::reflec::Host
        + owned::rinka::Host
        + owned::rio::Host
        + owned::shaktool::Host
        + owned::shutters::Host
        + owned::skree::Host
        + owned::skultera::Host
        + owned::spike_platform::Host
        + owned::spore_spawn::Lease
        + owned::squeept::Host
        + owned::tatori::Host
        + owned::torizo::Lease
        + owned::waver::Host
        + owned::work_robot::Host
        + owned::wrecked_ship_spark::Host
        + owned::yard::Host
        + owned::zeb::Host,
> Host for T
{
}
pub fn try_ai<H: Host>(routine: ActorRoutine, game: &mut H) -> bool {
    let ea = routine.0;
    match ea {
        10676698 => owned::shutters::initialize(Kind::GrowingShutter, game),
        10676918 => owned::shutters::main_ai(Kind::GrowingShutter, game),
        10677765 => owned::shutters::initialize(Kind::Platform, game),
        10677778 => owned::shutters::initialize(Kind::VerticalShutter, game),
        10677969 => owned::shutters::main_ai(Kind::Platform, game),
        10678429 => owned::shutters::reaction(Kind::Platform, Event::Touch, game),
        10678434 => owned::shutters::reaction(Kind::Platform, Event::Shot, game),
        10678454 => owned::shutters::reaction(Kind::Platform, Event::PowerBomb, game),
        10678545 => owned::shutters::initialize(Kind::HorizontalShutter, game),
        10678750 => owned::shutters::main_ai(Kind::HorizontalShutter, game),
        10678442 => owned::shutters::reaction(Kind::VerticalShutter, Event::Shot, game),
        10679256 => owned::shutters::reaction(Kind::HorizontalShutter, Event::Touch, game),
        10679310 => owned::shutters::reaction(Kind::HorizontalShutter, Event::Shot, game),
        10679322 => owned::shutters::reaction(Kind::HorizontalShutter, Event::PowerBomb, game),
        10659396 => owned::gunship::initialize_top(game),
        10659538 => owned::gunship::initialize_bottom(game),
        10659673 => {
            let function = owned::gunship::main(game);
            // The original pre-instruction samples the selector after main AI.
            let index = game.session().actors.controls.selected;
            if !try_gunship_pre_instruction(function, index, game) {
                game.run_other_pre_instruction(function);
            }
        }
        11066573 => owned::alcoon::initialize(game),
        11066731 => owned::alcoon::main_ai(game),
        10730163 => owned::bang::initialize(game),
        10731261 => owned::bang::shot(game),
        11057014 => owned::beetom::initialize(game),
        10657608 => owned::cacatac::initialize(game),
        11049710 => owned::covern::initialize(game),
        11049788 => owned::covern::main_ai(game),
        10720995 => owned::crawlers::initialize(owned::crawlers::Kind::Sciser, game),
        10721595 => owned::crawlers::initialize(owned::crawlers::Kind::Zero, game),
        10729071 => owned::crawlers::initialize(owned::crawlers::Kind::Viola, game),
        10739779 => owned::crawlers::initialize(owned::crawlers::Kind::OrangeGeemer, game),
        10740436 => owned::crawlers::initialize(owned::crawlers::Kind::Zeela, game),
        10741148 => owned::crawlers::initialize(owned::crawlers::Kind::Sova, game),
        10741353 => owned::crawlers::initialize(owned::crawlers::Kind::Geemer, game),
        11044832 => owned::evir::initialize(game),
        11045040 => owned::evir::initialize_projectile(game),
        11045278 => owned::evir::projectile_main_ai(game),
        11045638 | 11045644 | 11045650 => owned::evir::react(ea, game),
        11053894 => owned::kago::initialize(game),
        10669268 => owned::oum::initialize(game),
        10656319 => owned::puyo::initialize(game),
        10738643 => owned::reflec::initialize(game),
        10663426 => owned::rinka::initialize(game),
        10664233 => owned::rinka::frozen(game),
        10664275 => owned::rinka::power_bomb(game),
        10664263 | 10664269 => owned::rinka::react(ea == 10664269, game),
        10717536 => owned::skree::initialize(owned::skree::Kind::Metaree, game),
        10733230 => owned::skree::initialize(owned::skree::Kind::Skree, game),
        10717561 => owned::skree::main_ai(owned::skree::Kind::Metaree, game),
        10733255 => owned::skree::main_ai(owned::skree::Kind::Skree, game),
        10717967 => owned::skree::shot(owned::skree::Kind::Metaree, game),
        10733557 => owned::skree::shot(owned::skree::Kind::Skree, game),
        10654337 | 10654367 => owned::tatori::touch(ea == 10654367, game),
        11062326 => owned::work_robot::main_ai(game),
        11063668 => owned::work_robot::touch(game),
        10735074 => owned::yard::initialize(game),
        10736560 => owned::yard::touch(game),
        10736745 => owned::yard::shot(game),
        10720486 => owned::elevator::initialize(game),
        11048675 => owned::fune::initialize(game),
        11048752 => owned::fune::main_ai(game),
        11053941 => owned::kago::main_ai(game),
        10663280 => owned::polyp::initialize(game),
        10664909 => owned::rio::initialize(game),
        10653036 | 10653085 => owned::tatori::initialize(ea == 10653085, game),
        10654479 => owned::tatori::shot(game),
        10670988 => owned::oum::touch(false, game),
        10671028 => owned::oum::shot(game),
        11191934 => owned::torizo::hurt(game),
        11192695 => owned::torizo::touch(game),
        11192700 => owned::torizo::shot(game),
        11195322 => owned::torizo::golden_hurt(game),
        11191972 => owned::torizo::main_ai(game),
        11192447 => owned::torizo::initialize(game),
        11195241 => owned::torizo::golden_main_ai(game),
        11196007 => owned::torizo::golden_shot(game),
        10738716 => {
            if let Some(target) = owned::reflec::reflect_shot(game) {
                game.warn_reflection();
                owned::reflec::absorb_shot(target, game);
            }
        }
        11069198 | 11069725 => owned::wrecked_ship_spark::pass_shot(game),
        10720554 => owned::elevator::main_ai(game),
        10739851 | 10741442 => {
            let function = owned::crawlers::main_ai(game);
            {
                game.remaining_run_pre_instruction(function);
            };
        }
        10730277 => owned::bang::main_ai(game),
        10656381 => {
            if !owned::puyo::main_ai(game) {
                game.remaining_unknown();
            }
        }
        10653138 | 10653998 => {
            if !owned::tatori::main_ai(ea == 10653998, game) {
                game.remaining_unknown();
            }
        }
        11062135 => owned::work_robot::initialize(game),
        11062220 => owned::work_robot::initialize_deactivated(game),
        10914607 => owned::spike_platform::initialize(game),
        10914733 => owned::spike_platform::run(game),
        11063693 => owned::work_robot::shot_deactivated(game),
        11063698 => owned::work_robot::shot(game),
        10735204 => {
            let function = owned::yard::prepare(game);
            {
                game.remaining_run_pre_instruction(function);
            };
        }
        10663876 => {
            if !owned::rinka::main_ai(game) {
                game.remaining_unknown();
            }
        }
        10669331 => owned::oum::main_ai(game),
        10658809 => owned::cacatac::owtch::initialize(game),
        10658942 => owned::cacatac::owtch::main_ai(game),
        10659193 => owned::cacatac::owtch::shot(game),
        11045147 => owned::evir::main_ai(game),
        11057165 => owned::beetom::main_ai(game),
        11058734 => owned::beetom::touch(game),
        11058860 => owned::beetom::shot(game),
        11053955 => owned::kago::shot(game),
        10662880 => owned::multiviola::initialize(game),
        10662927 => owned::multiviola::main_ai(game),
        10663311 => owned::polyp::main_ai(game),
        10664931 => owned::rio::main_ai(game),
        10666562 => owned::geruta::initialize(false, game),
        10667763 => owned::geruta::initialize(true, game),
        10665625 => owned::squeept::initialize(game),
        10662175 | 10665682 | 10666615 | 10667812 => {
            game.roll_random();
            {
                let function = owned::geruta::function(game);
                game.remaining_run_pre_instruction(function);
            };
        }
        10661995 => owned::flies::initialize(game),
        10652077 => owned::mini_crocomire::initialize(game),
        10716909 => owned::waver::initialize(game),
        10717004 => owned::waver::main_ai(game),
        10651420 => owned::boyon::initialize(game),
        10651548 => owned::boyon::main_ai(game),
        10675718 => owned::dragon::initialize(game),
        10675790 => {
            let function = owned::geruta::function(game);
            game.remaining_run_pre_instruction(function);
        }
        10676168 | 10676174 | 10676180 => owned::dragon::react(ea, game),
        10915836 => owned::fire_pillar::initialize(game),
        10915875 => owned::fire_pillar::main_ai(game),
        10670984 => owned::oum::touch(true, game),
        11192770 => owned::torizo::standing_shot(game),
        11766625 => owned::gamet::initialize(game),
        11766686 => owned::gamet::main_ai(game),
        11068296 => owned::atomic::initialize(game),
        11068355 => owned::atomic::main_ai(game),
        10718509 => owned::fireflea::initialize(game),
        10718702 => owned::fireflea::main_ai(game),
        10718827 => owned::fireflea::touch(game),
        10718851 => owned::fireflea::power_bomb(game),
        10718857 => owned::fireflea::shot(game),
        10719413 => owned::skultera::initialize(game),
        10719531 => owned::skultera::main_ai(game),
        11197603 => owned::shaktool::main_ai(game),
        11198019 => owned::shaktool::initialize(game),
        11198255 => owned::shaktool::touch(game),
        11198260 => owned::shaktool::shot(game),
        11065545 => owned::bull::initialize(game),
        11065611 => owned::bull::main_ai(game),
        11066132 => owned::bull::shot(game),
        11071880 => owned::kihunter::initialize(game),
        11072020 => owned::kihunter::initialize_wings(game),
        11072092 | 11072098 => owned::kihunter::main_ai(game),
        11073281 => owned::kihunter::shot(game),
        11765819 => owned::zeb::initialize(game),
        11765882 => owned::zeb::main_ai(game),
        11767628 => owned::geega::initialize(game),
        11767726 => owned::geega::main_ai(game),
        11059657 => owned::powamp::initialize(game),
        11059740 => owned::powamp::main_ai(game),
        11060670 => owned::powamp::touch(game),
        11060719 => owned::powamp::shot(game),
        11060799 => owned::powamp::power_bomb(game),
        11068983 => owned::wrecked_ship_spark::initialize(game),
        11069070 => owned::wrecked_ship_spark::main_ai(game),
        11069486 => owned::blue_brinstar_face_block::initialize(game),
        11069614 => owned::blue_brinstar_face_block::main_ai(game),
        11047000 => owned::morph_ball_eye::initialize(game),
        11047138 => owned::morph_ball_eye::main_ai(game),
        11790027 | 11791077 => owned::escape_critters::initialize(ea == 11791077, game),
        11789909 => owned::escape_critters::main_ai(game),
        11054987 => owned::magdollite::initialize(game),
        11055370 => owned::magdollite::main_ai(game),
        10873386 => owned::spore_spawn::initialize(game),
        10873619 => owned::spore_spawn::main_ai(game),
        10874202 => owned::spore_spawn::shot(game),
        10874348 => owned::spore_spawn::touch(game),
        11056128 | 11056134 | 11056140 => owned::magdollite::damage(ea, game),
        10913525 => owned::boulder::initialize(game),
        10913683 => owned::boulder::main_ai(game),
        10914693 | 10914713 => owned::fire_arc::initialize_spike_platform_bottom(game),
        10917060 => owned::fire_arc::initialize(game),
        10917390 => owned::fire_arc::main_ai(game),
        _ => return false,
    }
    true
}
pub fn try_instruction<H: Host>(
    routine: ActorRoutine,
    actor: ActorKey,
    script: EnemyScript,
    game: &mut H,
) -> Option<EnemyScript> {
    let ea = routine.0;
    let k = actor.byte_index();
    let j = script;
    Some(match ea {
        10730152 => owned::bang::set_finished_growing(j, game),
        11044751 | 11044763 | 11044790 | 11044811 => owned::evir::instruction(ea, j, game),
        10664371 | 10664381 | 10664391 => owned::rinka::instruction(ea, k, j, game),
        10717526 | 10733220 => owned::skree::set_attack_ready(j, game),
        11186332 => owned::torizo::set_function(k, j, game),
        11186461 => {
            owned::torizo::blow_up_gut(k, game);
            j
        }
        11186622 => {
            owned::torizo::blow_up_face(k, game);
            j
        }
        11186724 => {
            owned::torizo::set_visible(k, game);
            j
        }
        11186734 => {
            owned::torizo::set_invisible(k, game);
            j
        }
        11186765 => {
            owned::torizo::defeated(k, game);
            j
        }
        11190985 => {
            owned::torizo::lock_animation(k, game);
            j
        }
        11190993 => {
            owned::torizo::unlock_animation(k, game);
            j
        }
        11191001 => owned::torizo::branch_if_faceless(k, j, game),
        11191021 => owned::torizo::set_link(k, j, game),
        11191043 => owned::torizo::low_health_explosions(k, j, game),
        11191087 => owned::torizo::death_explosion(k, j, game),
        11191131 => {
            owned::torizo::drool_if_low_health(k, game);
            j
        }
        11191149 => {
            owned::torizo::start_turning(k, game);
            j
        }
        11191159 => {
            owned::torizo::stepped(k, game, 0);
            j
        }
        11191178 => {
            owned::torizo::stepped(k, game, 0x8000);
            j
        }
        11191200 => {
            owned::torizo::stepped(k, game, 0x2000);
            j
        }
        11191222 => {
            owned::torizo::stepped(k, game, 0xa000);
            j
        }
        11191244 => owned::torizo::stand_up(k, j, game),
        11191326 => owned::torizo::sit_down(k, j, game),
        11191408 => owned::torizo::walk(
            k,
            j,
            owned::torizo::list::TURNING_LEFT,
            owned::torizo::list::TURNING_RIGHT,
            72,
            game,
        ),
        11191525 => owned::torizo::walk(
            k,
            j,
            owned::torizo::list::FACELESS_TURNING_LEFT,
            owned::torizo::list::FACELESS_TURNING_RIGHT,
            72,
            game,
        ),
        11191642 => owned::torizo::branch_if_rising(k, j, game),
        11191655 => owned::torizo::call_if_samus_close_in_front(k, j, game),
        11191691 => owned::torizo::jump_back_if_samus_close(k, j, game),
        11191716 => owned::torizo::bomb_attack(k, j, game),
        11191755 => {
            owned::torizo::bomb_chozo_orbs(k, game);
            j
        }
        11191779 => owned::torizo::bomb_sonic_boom(k, j, game),
        11191794 => owned::torizo::golden_sonic_boom(k, j, game),
        11193038 => owned::torizo::branch_if_airborne(k, j, game),
        11193058 => {
            owned::torizo::start_golden_fight(k, game);
            j
        }
        11193815 => {
            owned::torizo::clear_caught_super_missile(k, game);
            j
        }
        11194747 => {
            owned::torizo::disable_eye_beam_explosions(k, game);
            j
        }
        11194759 => {
            owned::torizo::enable_eye_beam_explosions(k, game);
            j
        }
        11194855 => {
            owned::torizo::clear_stunned(k, game);
            j
        }
        11195370 => owned::torizo::branch_if_samus_rolling_behind(k, j, game),
        11195461 => owned::torizo::call_sometimes_if_samus_in_front(k, j, game),
        11195508 => owned::torizo::call_sometimes_at_low_health(k, j, game),
        11195547 => owned::torizo::call_if_stunned_at_high_health(k, j, game),
        11195578 => owned::torizo::jump_forwards_if_samus_far(k, j, game),
        11195635 => {
            owned::torizo::golden_chozo_orbs(k, game);
            j
        }
        11195645 => owned::torizo::golden_jump_back(k, j, game),
        11195686 => owned::torizo::golden_attack(k, j, game),
        11195725 => owned::torizo::walk(
            k,
            j,
            owned::torizo::list::GOLDEN_TURNING_LEFT,
            owned::torizo::list::GOLDEN_TURNING_RIGHT,
            16,
            game,
        ),
        11191031 => owned::torizo::return_to_link(k, game),
        11191037 => owned::torizo::return_from_gut_explosion(k, game),
        11186744 => {
            owned::torizo::fade_to_black(game);
            j
        }
        11186801 => {
            owned::torizo::advance_colour_change(game);
            j
        }
        11188557 => {
            owned::torizo::fade_to_active(game);
            j
        }
        11188561 => {
            owned::torizo::start_fight(game);
            j
        }
        11191114 => {
            owned::torizo::landing_dust(game);
            j
        }
        11191809 => owned::torizo::explosive_swipe(j, game),
        11191824 => {
            owned::torizo::shot_sound(game);
            j
        }
        11191832 => {
            owned::torizo::footstep_sound(game);
            j
        }
        11193054 => {
            owned::torizo::golden_palettes(game);
            j
        }
        11194601 => {
            owned::torizo::spawn_egg(game);
            j
        }
        11194611 => owned::torizo::branch_if_egg_active(j, game),
        11195279 => {
            owned::torizo::egg_released_sound(game);
            j
        }
        11195287 => {
            owned::torizo::laser_sound(game);
            j
        }
        11195295 => {
            owned::torizo::sonic_boom_sound(game);
            j
        }
        11195360 => {
            owned::torizo::golden_super_missile(game);
            j
        }
        11195446 => owned::torizo::eye_beam(j, game),
        10657941 => owned::cacatac::resume_moving(k, j, game),
        10734646 | 10734655 | 10734664 | 10734687 | 10734712 => {
            owned::yard::instruction(ea, k, j, game)
        }
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216
        | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601
        | 11063613 | 11063659 => owned::work_robot::instruction(ea, k, j, game),
        10654593 | 10654738 | 10654791 | 10654801 | 10654827 | 10654853 | 10654881 | 10654919 => {
            owned::tatori::instruction(ea, j, game)
        }
        10669235 | 10669246 | 10669257 => owned::oum::instruction(ea, j, game),
        10659181 | 10659185 => owned::cacatac::owtch::set_function_index(ea, k, j, game),
        10738632 => owned::reflec::set_reflection_axis(k, j, game),
        10741344 | 10739650 => owned::crawlers::set_function(k, j, game),
        11048485 | 11048497 | 11048522 | 11048547 | 11048572 | 11048597 | 11048628 => {
            owned::fune::instruction(ea, j, game)
        }
        11067164 | 11067187 | 11067193 | 11067199 | 11067235 | 11067249 => {
            owned::alcoon::instruction(ea, j, game)
        }
        10915247 | 10915347..=10915793 => owned::fire_pillar::instruction(ea, j, game),
        10664899 => {
            owned::rio::set_animation_finished(game);
            j
        }
        10666441 | 10666452 | 10666463 | 10666474 | 10666485 | 10666496 | 10666507 | 10666518
        | 10666529 | 10666540 | 10666551 | 10667730 | 10667741 | 10667752 => {
            owned::geruta::instruction(ea, j, game)
        }
        10665614 => {
            owned::squeept::set_animation_finished(game);
            j
        }
        10652048 => owned::mini_crocomire::set_moving(false, j, game),
        10652061 => owned::mini_crocomire::set_moving(true, j, game),
        10716899 => owned::waver::set_spin_finished(j, game),
        10651846 => owned::boyon::start_bounce(j, game),
        10675707 => owned::dragon::set_animation_finished(j, game),
        10872671 | 10872689 | 10872877 | 10872956 | 10873018 | 10873009 | 10873034 | 10873116
        | 10873198 | 10873265 => owned::spore_spawn::run_instruction(ea, k, j, game),
        11054610 | 11054630 | 11054640 | 11054650 | 11054661 | 11054672 | 11054682 | 11054692
        | 11054728 | 11054742 | 11054778 | 11054794 | 11054820 | 11054846 | 11054872 | 11054916 => {
            owned::magdollite::instruction(ea, j, game)
        }
        11789637 | 11791016 | 11791032 | 11789840 | 11791049 | 11791063 => {
            owned::escape_critters::instruction(ea, k, j, game)
        }
        11068957 | 11068970 => owned::wrecked_ship_spark::set_intangible(game, ea == 11068957, j),
        11072806 | 11072996 | 11073151 | 11073234 | 11073240 => {
            owned::kihunter::instruction(game, ea, k, j)
        }
        11196721 | 11196735 | 11196746 | 11196755 | 11196858 => {
            owned::shaktool::instruction(ea, k, j, game)
        }
        10719382 | 10719392 | 10719402 => owned::skultera::instruction(ea, j, game),
        _ => return None,
    })
}
pub fn try_pre_instruction<H: Host>(routine: u32, k: u16, game: &mut H) -> bool {
    let ea = routine;
    match ea {
        10730661 => owned::bang::movement_delay(game),
        10730693 => owned::bang::target_samus(game),
        10730780 => owned::bang::fly(false, game),
        10730796 => owned::bang::fly(true, game),
        10739857 => owned::crawlers::crawl(k, crate::actors::api::Axis::X, true, game),
        10740072 => owned::crawlers::crawl(k, crate::actors::api::Axis::Y, true, game),
        10741448 => owned::crawlers::crawl(k, crate::actors::api::Axis::X, false, game),
        10741746 => owned::crawlers::crawl(k, crate::actors::api::Axis::Y, false, game),
        10741637 => owned::crawlers::falling(k, game),
        10669926 => owned::oum::bounce(false, k, game),
        10669993 => owned::oum::bounce(true, k, game),
        10657722 => owned::cacatac::moving(false, game),
        10735526 => owned::yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cce2, game),
        10735543 => owned::yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccea, game),
        10735549 => owned::yard::crawl(k, crate::actors::api::Axis::Y, 0xa3ccf2, game),
        10735566 => owned::yard::crawl(k, crate::actors::api::Axis::X, 0xa3ccfa, game),
        10735572 => owned::yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd02, game),
        10735589 => owned::yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd0a, game),
        10735595 => owned::yard::crawl(k, crate::actors::api::Axis::Y, 0xa3cd12, game),
        10735612 => owned::yard::crawl(k, crate::actors::api::Axis::X, 0xa3cd1a, game),
        10735456 => owned::yard::hiding(k, game),
        10736051 => owned::yard::airborne(k, game),
        10657772 => owned::cacatac::moving(true, game),
        10666625 => owned::geruta::flames(k, false, game),
        10666727 => owned::geruta::idle(k, false, game),
        10666815 => owned::geruta::finish_animation(k, false, false, game),
        10667014 => owned::geruta::finish_animation(k, false, true, game),
        10667822 => owned::geruta::flames(k, true, game),
        10667889 => owned::geruta::idle(k, true, game),
        10667963 => owned::geruta::finish_animation(k, true, false, game),
        10668168 => owned::geruta::finish_animation(k, true, true, game),
        10666849 => owned::geruta::swoop(k, false, true, game),
        10666929 => owned::geruta::swoop(k, false, false, game),
        10667990 => owned::geruta::swoop(k, true, true, game),
        10668077 => owned::geruta::swoop(k, true, false, game),
        10665692 => owned::squeept::flame(k, game),
        10665754 => owned::squeept::jump(k, game),
        10665790 => owned::squeept::fly(k, owned::squeept::Flight::Rising, game),
        10665852 => owned::squeept::fly(k, owned::squeept::Flight::Flipping, game),
        10665916 => owned::squeept::fly(k, owned::squeept::Flight::Falling, game),
        10662222 => owned::flies::circle(k, false, game),
        10662268 => owned::flies::circle(k, true, game),
        10662314 => owned::flies::dive(k, false, game),
        10662354 => owned::flies::dive(k, true, game),
        10652227 => owned::mini_crocomire::patrol(false, game),
        10652252 => owned::mini_crocomire::patrol(true, game),
        10675796 | 10675885 | 10675953 | 10676020 | 10676041 => {
            owned::dragon::step(k, ea as u16, game)
        }
        11192146 => owned::torizo::walking(k, game),
        11192364 => owned::torizo::jumping_or_falling(k, game),
        11195889 => owned::torizo::golden_walking(k, game),
        11192360 | 11195885 => owned::torizo::attacking(k, game),
        _ => return false,
    }
    true
}
pub fn try_gunship_pre_instruction<H: Host>(ea: u32, index: u16, runtime: &mut H) -> bool {
    let [low, high, _, _] = ea.to_le_bytes();
    match ea {
        10659852 | 10660048 | 10660162 | 10660176 | 10660231 | 10660285 | 10660431 | 10660445
        | 10660500 | 10660514 | 10660639 | 10660704 | 10660718 | 10660773 | 10660807 | 10660891
        | 10661079 | 10661134 | 10661165 => {
            owned::gunship::run(runtime, index, u16::from_le_bytes([low, high]))
        }
        10659799 => (),
        _ => return false,
    }
    true
}
pub fn try_hitbox_touch<H: Host>(routine: ActorRoutine, game: &mut H) -> bool {
    match routine.0 {
        10670984 | 10670988 | 11192695 | 10874348 | 10718827 | 11060670 | 11045650 => {
            try_ai(routine, game)
        }
        _ => false,
    }
}
pub fn try_hitbox_shot<H: Host>(routine: ActorRoutine, _: u16, game: &mut H) -> bool {
    match routine.0 {
        10671028 | 11192770 | 11192700 | 11196007 | 10874202 | 10718857 | 11060719 | 11066132
        | 10717967 | 10733557 | 11045638 => try_ai(routine, game),
        _ => false,
    }
}
pub fn try_graphics_hook<H: Host>(routine: ActorRoutine, game: &mut H) -> bool {
    match routine.0 {
        11069550 => owned::blue_brinstar_face_block::cycle_palette(game),
        10738444 => owned::reflec::cycle_palette(game),
        11055282 => owned::magdollite::cycle_palette(game),
        11062375 => owned::work_robot::palette(game),
        _ => return false,
    }
    true
}
pub fn try_b3_pre_instruction<H: Host>(routine: ActorRoutine, _: u16, game: &mut H) -> bool {
    match routine.0 {
        11763835 => true,
        11767039 => {
            owned::gamet::settle(game);
            true
        }
        _ => false,
    }
}
pub fn try_pipe_bug<H: Host>(routine: ActorRoutine, actor: u16, game: &mut H) -> bool {
    match routine.0 {
        11765888 => owned::zeb::await_view(actor, game),
        11765904 => owned::zeb::await_player(actor, game),
        11765987 => owned::zeb::rise(actor, game),
        11766044 => owned::zeb::fly(actor, game),
        11766142 => owned::zeb::cooldown(actor, game),
        _ => return false,
    }
    true
}
