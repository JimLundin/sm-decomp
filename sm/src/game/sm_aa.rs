use crate::game::actors;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::ram_map::slot;
use sm_game::enemy_script::EnemyScript;

use crate::game::room_mechanisms::spawn_collision;

pub use crate::types::Consts_AA;

crate::game::constants::typed_constants! { Consts_AA;
    addr_kShaktool_Ilist_E461 = 58465, addr_kShaktool_Ilist_E3A7 = 58279,
    addr_kTorizo_Ilist_D2BF = 53951, addr_kTorizo_Ilist_D2AD = 53933,
    addr_kTorizo_Ilist_D203 = 53763, addr_kTorizo_Ilist_D1F1 = 53745,
    addr_kTorizo_Ilist_CEFF = 52991, addr_kTorizo_Ilist_CEA5 = 52901,
    addr_kTorizo_Ilist_CE43 = 52803, addr_kTorizo_Ilist_CDE1 = 52705,
    addr_locret_AAC95E = 51550, addr_kTorizo_Ilist_C188 = 49544,
    addr_off_AAC0F2 = 49394, addr_kTorizo_Ilist_BDD8 = 48600,
    addr_kTorizo_Ilist_BD0E = 48398, addr_off_AABC78 = 48248,
    addr_kTorizo_Ilist_B962 = 47458, addr_kTorizo_Ilist_B1C8 = 45512,
    addr_kTorizo_Ilist_B155 = 45397, addr_kTorizo_Ilist_B0E5 = 45285,
    addr_kTorizo_ExtSprmap_87D0 = 34768, addr_kSpritemap_Nothing_AA = 32845,
}

pub type Enemy_Shaktool = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Torizo = crate::game::enemy_layouts::EnemyLayout40;

crate::game::constants::typed_constants! { i32;
    fnnullsub_171_AA = 0xaa807b,
    fnnullsub_274 = 0xaadcaa, fnnullsub_276 = 0xaae7a6,
}
pub fn Torizo_Instr_7(_k: u16, jp: EnemyScript) -> EnemyScript {
    jp
}
/// Runs Torizo `k`'s function at `ea`; the verifier's entry.
pub fn CallTorizoFunc(ea: u32, k: u16, game: &mut Game) {
    actors::torizo::run_torizo_function(game, ea as u16, k);
}
pub fn Torizo_Func_5(k: u16, game: &mut Game) {
    actors::torizo::walking(k, game);
}
pub fn Torizo_Func_7(k: u16, game: &mut Game) {
    actors::torizo::jumping_or_falling(k, game);
}
pub fn Torizo_D5F1(k: u16, game: &mut Game) {
    actors::torizo::golden_walking(k, game);
}
pub fn Torizo_Func_8(game: &mut Game) {
    actors::torizo::standing_shot(game);
}
pub use self::Torizo_Instr_7 as Shaktool_Instr_6;

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [11191934] Torizo_Hurt [] { actors::torizo::hurt(game); };
    [11192695] GoldTorizo_Touch [] { actors::torizo::touch(game); };
    [11192700] Torizo_Shot [] { actors::torizo::shot(game); };
    [11195322] GoldTorizo_Hurt [] { actors::torizo::golden_hurt(game); };
    [11196360] TourianEntranceStatue_Init [] { sm_game::actors::statues::initialize_tourian_statue(game); };
    [11197603] Shaktool_Hurt [] { actors::shaktool::main_ai(game); };
    [11198019] Shaktool_Init [] { actors::shaktool::initialize(game); };
    [11198255] Shaktool_Touch [] { actors::shaktool::touch(game); };
    [11198260] Shaktool_Shot [] { actors::shaktool::shot(game); };
    [11200278] N00bTubeCracks_Init [] { sm_game::actors::statues::initialize_tube_cracks(game); };
    [11200293] ChozoStatue_Init [] { sm_game::actors::statues::initialize_chozo_statue(game); };
    [11200423] ChozoStatue_Main [] { sm_game::actors::statues::main_ai(game); };
    [11191972] Torizo_Main [] { actors::torizo::main_ai(game); };
    [11192447] Torizo_Init [] { actors::torizo::initialize(game); };
    [11195241] GoldTorizo_Main [] { actors::torizo::golden_main_ai(game); };
    [11196007] GoldTorizo_Shot [] { actors::torizo::golden_shot(game); };
    @forward [pub fn (game: &mut Game)];
    [11173903] Enemy_GrappleReact_CancelBeam_AA => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11173953] Enemy_NormalFrozenAI_AA => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11174004] Enemy_ClearAiPreInstr_AA [jp] {
        slot!(game, EnemyData[k as i32].ai_preinstr = (0xaa807bi32) as u16);
    };
    [11173995] Enemy_SetAiPreInstr_AA [jp.offset(1)] {
        slot!(game, EnemyData[k as i32].ai_preinstr = jp.word(0));
    };
    [11186332] Torizo_Instr_3 [] { actors::torizo::set_function(k, jp, game) };
    [11186461] Torizo_Instr_31 [jp] { actors::torizo::blow_up_gut(k, game); };
    [11186622] Torizo_Instr_33 [jp] { actors::torizo::blow_up_face(k, game); };
    [11186724] Torizo_Instr_36 [jp] { actors::torizo::set_visible(k, game); };
    [11186734] Torizo_Instr_37 [jp] { actors::torizo::set_invisible(k, game); };
    [11186765] Torizo_Instr_38 [jp] { actors::torizo::defeated(k, game); };
    [11190985] Torizo_Instr_2 [jp] { actors::torizo::lock_animation(k, game); };
    [11190993] Torizo_Instr_8 [jp] { actors::torizo::unlock_animation(k, game); };
    [11191001] Torizo_Instr_25 [] { actors::torizo::branch_if_faceless(k, jp, game) };
    [11191021] Torizo_Instr_22 [] { actors::torizo::set_link(k, jp, game) };
    [11191043] Torizo_Instr_30 [] { actors::torizo::low_health_explosions(k, jp, game) };
    [11191087] Torizo_Instr_34 [] { actors::torizo::death_explosion(k, jp, game) };
    [11191131] Torizo_Instr_12 [jp] { actors::torizo::drool_if_low_health(k, game); };
    [11191149] Torizo_Instr_10 [jp] { actors::torizo::start_turning(k, game); };
    [11191159] Torizo_Instr_11 [jp] { actors::torizo::stepped(k, game, 0); };
    [11191178] Torizo_Instr_29 [jp] { actors::torizo::stepped(k, game, 0x8000); };
    [11191200] Torizo_Instr_1 [jp] { actors::torizo::stepped(k, game, 0x2000); };
    [11191222] Torizo_Instr_28 [jp] { actors::torizo::stepped(k, game, 0xa000); };
    [11191244] Torizo_Instr_4 [] { actors::torizo::stand_up(k, jp, game) };
    [11191326] Torizo_Instr_40 [] { actors::torizo::sit_down(k, jp, game) };
    [11191408] Torizo_Instr_16 [] {
        actors::torizo::walk(k, jp, actors::torizo::list::TURNING_LEFT, actors::torizo::list::TURNING_RIGHT, 72, game)
    };
    [11191525] Torizo_Instr_27 [] {
        actors::torizo::walk(k, jp, actors::torizo::list::FACELESS_TURNING_LEFT, actors::torizo::list::FACELESS_TURNING_RIGHT, 72, game)
    };
    [11191642] Torizo_Instr_23 [] { actors::torizo::branch_if_rising(k, jp, game) };
    [11191655] Torizo_Instr_14 [] { actors::torizo::call_if_samus_close_in_front(k, jp, game) };
    [11191691] Torizo_Instr_15 [] { actors::torizo::jump_back_if_samus_close(k, jp, game) };
    [11191716] Torizo_Instr_26 [] { actors::torizo::bomb_attack(k, jp, game) };
    [11191755] Torizo_Instr_18 [jp] { actors::torizo::bomb_chozo_orbs(k, game); };
    [11191779] Torizo_Instr_20 [] { actors::torizo::bomb_sonic_boom(k, jp, game) };
    [11191794] Torizo_Instr_44 [] { actors::torizo::golden_sonic_boom(k, jp, game) };
    [11193038] Torizo_Instr_39 [] { actors::torizo::branch_if_airborne(k, jp, game) };
    [11193058] Torizo_Instr_42 [jp] { actors::torizo::start_golden_fight(k, game); };
    [11193815] Torizo_Instr_48 [jp] { actors::torizo::clear_caught_super_missile(k, game); };
    [11194747] Torizo_Instr_59 [jp] { actors::torizo::disable_eye_beam_explosions(k, game); };
    [11194759] Torizo_Instr_62 [jp] { actors::torizo::enable_eye_beam_explosions(k, game); };
    [11194855] Torizo_Instr_63 [jp] { actors::torizo::clear_stunned(k, game); };
    [11195370] Torizo_Instr_49 [] { actors::torizo::branch_if_samus_rolling_behind(k, jp, game) };
    [11195461] Torizo_Instr_53 [] { actors::torizo::call_sometimes_if_samus_in_front(k, jp, game) };
    [11195508] Torizo_Instr_55 [] { actors::torizo::call_sometimes_at_low_health(k, jp, game) };
    [11195547] Torizo_Instr_52 [] { actors::torizo::call_if_stunned_at_high_health(k, jp, game) };
    [11195578] Torizo_Instr_50 [] { actors::torizo::jump_forwards_if_samus_far(k, jp, game) };
    [11195635] Torizo_Instr_43 [jp] { actors::torizo::golden_chozo_orbs(k, game); };
    [11195645] Torizo_Instr_51 [] { actors::torizo::golden_jump_back(k, jp, game) };
    [11195686] Torizo_Instr_45 [] { actors::torizo::golden_attack(k, jp, game) };
    [11195725] Torizo_Instr_54 [] {
        actors::torizo::walk(k, jp, actors::torizo::list::GOLDEN_TURNING_LEFT, actors::torizo::list::GOLDEN_TURNING_RIGHT, 16, game)
    };
    @functions [pub fn (k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191031] Torizo_Instr_19 [] { actors::torizo::return_to_link(k, game) };
    [11191037] Torizo_Instr_32 [] { actors::torizo::return_from_gut_explosion(k, game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11186744] Torizo_Instr_35 [jp] { actors::torizo::fade_to_black(game); };
    [11186801] Torizo_Instr_6 [jp] { actors::torizo::advance_colour_change(game); };
    [11188557] Torizo_Instr_5 [jp] { actors::torizo::fade_to_active(game); };
    [11188561] Torizo_Instr_9 [jp] { actors::torizo::start_fight(game); };
    [11191114] Torizo_Instr_24 [jp] { actors::torizo::landing_dust(game); };
    [11191809] Torizo_Instr_21 [] { actors::torizo::explosive_swipe(jp, game) };
    [11191824] Torizo_Instr_17 [jp] { actors::torizo::shot_sound(game); };
    [11191832] Torizo_Instr_13 [jp] { actors::torizo::footstep_sound(game); };
    [11193054] Torizo_Instr_41 [jp] { actors::torizo::golden_palettes(game); };
    [11194601] Torizo_Instr_57 [jp] { actors::torizo::spawn_egg(game); };
    [11194611] Torizo_Instr_58 [] { actors::torizo::branch_if_egg_active(jp, game) };
    [11195279] Torizo_Instr_56 [jp] { actors::torizo::egg_released_sound(game); };
    [11195287] Torizo_Instr_60 [jp] { actors::torizo::laser_sound(game); };
    [11195295] Torizo_Instr_46 [jp] { actors::torizo::sonic_boom_sound(game); };
    [11195360] Torizo_Instr_47 [jp] { actors::torizo::golden_super_missile(game); };
    [11195446] Torizo_Instr_61 [] { actors::torizo::eye_beam(jp, game) };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11196721] Shaktool_Instr_2 [] { actors::shaktool::lower_away_from_projectile(k, jp, game) };
    [11196735] Shaktool_Instr_3 [] { actors::shaktool::raise_towards_projectile(k, jp, game) };
    [11196746] Shaktool_Instr_4 [] { actors::shaktool::lower(k, jp, game) };
    [11196755] Shaktool_Instr_5 [] { actors::shaktool::raise(k, jp, game) };
    [11196858] Shaktool_Instr_1 [] { actors::shaktool::reset_functions(k, jp, game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199529] Shaktool_Instr_9 [jp] { sm_game::actors::statues::start_lowering_acid(game); };
    [11199542] Shaktool_Instr_11 [jp] { sm_game::actors::statues::set_lowered_acid(game); };
    [11199549] Shaktool_Instr_10 [jp] { sm_game::actors::statues::unlock_samus(game); };
    [11199871] Shaktool_Instr_8 [jp] { sm_game::actors::statues::grab_sound(game); };
    [11199879] Shaktool_Instr_13 [jp] { sm_game::actors::statues::footstep_sound(game); };
    [11200240] Shaktool_Instr_14 [jp] { sm_game::actors::statues::release_samus(game); };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199887] Shaktool_Instr_12 [] { sm_game::actors::statues::spike_clearing_footstep(game, k, jp) };
    [11199960] Shaktool_Instr_7 [] { sm_game::actors::statues::movement(game, k, jp) };
}

/// Spawns the collision PLM with `header` at the selected block.
pub(crate) fn spawn_collision_plm(header: u16, game: &mut Game) {
    unsafe { spawn_collision(header, game) };
}
/// Runs the bank $AA function at `address` for the current enemy.
pub(crate) fn run_aa_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11192360 | 11195885] Torizo_Func_6 [] { actors::torizo::attacking(k, game); };
    [11199557] sub_AAE445 [] { sm_game::actors::statues::lower_norfair(game, k); };
    [11200430] Shaktool_PreInstr_0 [] { sm_game::actors::statues::wrecked_ship(game, k); };
    [11197612] Shaktool_DCAC [] { actors::shaktool::arm_piece(k, game); };
    [11197655] Shaktool_DCD7 [] { actors::shaktool::head(k, game); };
    [11197733] Shaktool_DD25 [] { actors::shaktool::final_piece(k, game); };
}
