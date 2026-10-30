use crate::game::actors;
use crate::game::ceres_doors;
use crate::game::fire_geysers;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
pub use sm_game::actors::ceres_ridley::{
    handle_getaway_if_active as HandleCeresRidleyGetawayCutscene,
    spawn_getaway_walls as CeresRidley_Func_25,
};
pub use sm_game::actors::ridley::{
    HomingReversal, deal_contact_damage as Ridley_Func_98,
    draw_baby_metroid_and_door as Ridley_A2F2, homing_velocity, touch as sub_A6DF59,
};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

const VANISH: u16 = ceres_doors::TANGIBLE | ceres_doors::INVISIBLE;

/// The steam's next script at `bank:address`, as the original ROM pointer.
pub(crate) fn steam_script(bank: u8, address: u16, game: &mut Game) -> EnemyScript {
    let record = unsafe { crate::rtl::rom_record(bank, address, game) };
    EnemyScript::at(record.image, record.base)
}

/// Ridley's "Warning: X undefined" line, for `still_falling_apart`.
pub(crate) fn warn_x_undefined() {
    unsafe { printf(b"Warning: X undefined\n\0" as *const u8 as *const c_char) };
}
/// The boulder runs in `sm_game::actors::boulder`.
pub fn Boulder_Init(game: &mut Game) {
    sm_game::actors::boulder::initialize(game);
}
pub fn Boulder_Main(game: &mut Game) {
    sm_game::actors::boulder::main_ai(game);
}
crate::game::callback_dispatch! {
    pub fn CallNuclearWaffleFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10917397] NuclearWaffle_Func_1 [] { sm_game::actors::fire_arc::inactive(game) };
    [10917506] NuclearWaffle_Func_2 [] {
        crate::game::enemy_projectiles::advance_waffle_orbit(game);
    };
}

crate::game::callback_dispatch! {
    pub fn CallRidleyFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };

    @functions [pub fn (game: &mut Game)];
    [10920795] CeresRidley_Func_3 [] { sm_game::actors::ceres_ridley::initial(game) };
    [10921977] CeresRidley_A7F9 [] { sm_game::actors::ceres_ridley::fireballs_shooting(game) };
    [10922044] CeresRidley_Func_14 [] { sm_game::actors::ceres_ridley::lunge_setup(game) };
    [10922148] CeresRidley_Func_17 [] { sm_game::actors::ceres_ridley::swoop_move_to_position(game) };
    [10922196] CeresRidley_Func_18 [] { sm_game::actors::ceres_ridley::swoop_aiming_down(game) };
    [10922232] CeresRidley_Func_19 [] { sm_game::actors::ceres_ridley::swoop_aiming_left(game) };
    [10922275] CeresRidley_Func_20 [] { sm_game::actors::ceres_ridley::swoop_aiming_up(game) };
    [10922311] CeresRidley_Func_21 [] { sm_game::actors::ceres_ridley::swoop_aiming_up_faster(game) };
    [10922513] CeresRidley_Func_24 [] { sm_game::actors::ceres_ridley::set_up_getaway(game) };
    [10922576] CeresRidley_Func_26 [] { sm_game::actors::ceres_ridley::self_destruct_activated(game) };
    [10924787] Ridley_Func_3 [] { sm_game::actors::ridley::liftoff(game) };
    [10924833] Ridley_Func_3b [] { sm_game::actors::ridley::decide_action(game) };
    [10925048] Ridley_Func_6 [] { sm_game::actors::ridley::lunge_missed_move_to_position(game) };
    [10925460] Ridley_Func_13 [] { sm_game::actors::ridley::swoop_decelerate(game) };
    [10925508] Ridley_Func_14 [] { sm_game::actors::ridley::hover(game) };
    [10925541] Ridley_Func_15 [] { sm_game::actors::ridley::hover_to_wall_behind(game) };
    [10925587] Ridley_Func_16 [] { sm_game::actors::ridley::hover_to_wall_in_front(game) };
    [10925838] Ridley_Func_21 [] { sm_game::actors::ridley::pogo_descending(game) };
    [10927438] Ridley_Func_43 [] { sm_game::actors::ridley::dodge_power_bomb(game) };
    [10927514] Ridley_Func_44 [] { sm_game::actors::ridley::fake_retreat_move_to_position(game) };
    [10928206] Ridley_Func_54 [] { sm_game::actors::ceres_ridley::self_destruct_activating(game) };
    [10929464] Ridley_Func_63 [] { sm_game::actors::ridley::dying_grabbed_samus(game) };
    [10929470] Ridley_C53E [] { sm_game::actors::ridley::death_roar(game) };
    [10929489] Ridley_C551 [] { sm_game::actors::ridley::death_move_to_death_spot(game) };
    [10929544] Ridley_Func_64 [] { sm_game::actors::ridley::lower_acid_and_release_samus(game) };
    [10929576] Ridley_Func_65 [] { sm_game::actors::ridley::start_falling_apart(game) };
    CeresRidley_Init [] { sm_game::actors::ceres_ridley::init(game) };
    CeresRidley_Main [] { sm_game::actors::ceres_ridley::main(game) };
    [10920788] CeresRidley_Func_2 [] { sm_game::actors::ceres_ridley::transition_to_flying(game) };
    [10920841] CeresRidley_Func_4 [] { sm_game::actors::ceres_ridley::fade_in_eye(game) };
    [10920927] CeresRidley_Func_5 [] { sm_game::actors::ceres_ridley::fade_in_body(game) };
    [10921080] CeresRidley_Func_7 [] { sm_game::actors::ceres_ridley::roar_before_flying(game) };
    [10921647] CeresRidley_Func_9 [] { sm_game::actors::ceres_ridley::liftoff_accelerating(game) };
    [10921672] CeresRidley_Func_10 [] { sm_game::actors::ceres_ridley::liftoff_decelerating(game) };
    [10921704] CeresRidley_Func_11 [] { sm_game::actors::ceres_ridley::hovering(game) };
    [10921858] CeresRidley_Func_13 [] { sm_game::actors::ceres_ridley::fireballs_move_to_position(game) };
    [10922062] CeresRidley_Func_15 [] { sm_game::actors::ceres_ridley::lunge(game) };
    [10922125] CeresRidley_Func_16 [] { sm_game::actors::ceres_ridley::swoop_setup(game) };
    [10922353] CeresRidley_Func_22 [] { sm_game::actors::ceres_ridley::real_retreat_rising(game) };
    CeresRidley_A994 [] { sm_game::actors::ceres_ridley::real_retreat_risen(game) };
    CeresRidley_Func_28 [] { sm_game::actors::ceres_ridley::handle_getaway(game) };
    CeresRidley_Func_29 [] { sm_game::actors::ceres_ridley::set_mode7_rotation(game) };
    Ridley_Main [] { sm_game::actors::ridley::main(game) };
    Ridley_Func_2 [] { sm_game::actors::ridley::time_frozen(game) };
    Ridley_Hurt [] { sm_game::actors::ridley::hurt(game) };
    [10925036] Ridley_Func_5 [] { sm_game::actors::ridley::lunge_missed(game) };
    [10925121] Ridley_Func_8 [] { sm_game::actors::ridley::swoop(game) };
    [10925141] Ridley_B455 [] { sm_game::actors::ridley::swoop_move_to_position(game) };
    [10925203] Ridley_Func_9 [] { sm_game::actors::ridley::swoop_aiming_down(game) };
    [10925265] Ridley_Func_10 [] { sm_game::actors::ridley::swoop_aiming_horizontal(game) };
    [10925334] Ridley_Func_11 [] { sm_game::actors::ridley::swoop_aiming_up(game) };
    [10925396] Ridley_Func_12 [] { sm_game::actors::ridley::swoop_aiming_up_faster(game) };
    [10925735] Ridley_Func_19 [] { sm_game::actors::ridley::pogo_fly_to_position(game) };
    [10925789] Ridley_Func_20 [] { sm_game::actors::ridley::pogo_wait_at_position(game) };
    [10926009] Ridley_Func_22 [] { sm_game::actors::ridley::pogo_ascending(game) };
    Ridley_Func_27 [] { sm_game::actors::ridley::grab_samus_from_pogo(game) };
    Ridley_Func_28 [] { sm_game::actors::ridley::set_pogo_x_direction(game) };
    Ridley_Func_30 [] { sm_game::actors::ridley::move_samus_with_feet(game) };
    Ridley_Func_31 [] { sm_game::actors::ridley::set_grabbed_samus_offset(game) };
    Ridley_Func_32 [] { sm_game::actors::ridley::lunge_missed_samus(game) };
    [10926775] Ridley_Func_33 [] { sm_game::actors::ridley::lunge(game) };
    [10926991] Ridley_Func_33b [] { sm_game::actors::ridley::grabbed_samus(game) };
    [10927044] Ridley_Func_34 [] { sm_game::actors::ridley::grabbed_samus_rising(game) };
    [10927089] Ridley_Func_35 [] { sm_game::actors::ridley::grabbed_samus_rising_to_middle(game) };
    [10927150] Ridley_Func_36 [] { sm_game::actors::ridley::grabbed_samus_release(game) };
    Ridley_Func_39 [] { sm_game::actors::ridley::set_tangibility(game) };
    Ridley_Func_42 [] { sm_game::actors::ridley::lunge_if_power_bomb(game) };
    [10927548] Ridley_Func_45 [] { sm_game::actors::ridley::fake_retreat_rising(game) };
    [10927619] Ridley_Func_47 [] { sm_game::actors::ridley::retrieve_baby_metroid(game) };
    Ridley_Func_50 [] { sm_game::actors::ridley::baby_metroid_held(game) };
    Ridley_Func_51 [] { sm_game::actors::ridley::baby_metroid_held_by_feet(game) };
    Ridley_Func_52 [] { sm_game::actors::ridley::baby_metroid_start_falling(game) };
    Ridley_Func_53 [] { sm_game::actors::ridley::baby_metroid_falling(game) };
    Ridley_Func_57 [] { sm_game::actors::ceres_ridley::japanese_text_appears(game) };
    Ridley_Func_69 [] { sm_game::actors::ridley::spawn_small_explosion(game) };
    Ridley_Func_83 [] { sm_game::actors::ridley::handle_normal_pogo_tail(game) };
    Ridley_Func_84 [] { sm_game::actors::ridley::handle_pogo_tail_stab(game) };
    Ridley_Func_86 [] { sm_game::actors::ridley::calculate_tail_segment_positions(game) };
    Ridley_Func_91 [] { sm_game::actors::ridley::tail_whip_sound(game) };
    Ridley_Func_101 [] { sm_game::actors::ridley::ceres_health_palette(game) };
    Ridley_Func_114 [] { sm_game::actors::ridley::face_room_middle(game) };
    Ridley_Func_116 [] { sm_game::actors::ridley::wings_animation_timer(game) };
    Ridley_Func_117 [] { sm_game::actors::ridley::initialise_ribs(game) };
    Ridley_Func_118 [] { sm_game::actors::ridley::handle_ribs(game) };
    Ridley_Func_120 [] { sm_game::actors::ridley::draw_wings(game) };
    Ridley_Func_126 [] { sm_game::actors::ridley::start_death_sequence(game) };
    [10920823] CeresRidley_A377 [] { sm_game::actors::ceres_ridley::wait_to_fade_in(game) };
    [10921045] CeresRidley_Func_6 [] { sm_game::actors::ceres_ridley::wait_to_roar(game) };
    [10922400] CeresRidley_Func_23 [] { sm_game::actors::ceres_ridley::write_palettes_and_spawn_walls(game) };
    [10927602] Ridley_Func_46 [] { sm_game::actors::ridley::drop_baby_metroid(game) };
    [10929608] Ridley_Func_66 [] { sm_game::actors::ridley::falling_apart(game) };
    [10929626] Ridley_Func_67 [] { sm_game::actors::ridley::still_falling_apart(game) };
    @external {
        10922575 => [()];
        10929664 => [()];
    }
}
crate::game::address_dispatch! {
    pub fn CallRidleyFunc_var40(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10927772 => Ridley_Func_50; 10927795 => Ridley_Func_51; 10927818 => Ridley_Func_52;
        10927836 => Ridley_Func_53; 10927897 => [()];
    }
}

// Tail words alias the shared tilemap workspace; keep each load/store ordered.

crate::game::callback_dispatch! {
    pub fn CallBabyMetroidInstr(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
    call (k, game); default Unreachable(game) as u16;

    @functions [pub fn (k: u16, game: &mut Game) -> u16];
    [10928073] BabyMetroid_Instr_2 [] { sm_game::actors::ridley::baby_metroid_goto_if_still(k, game) };
    [10928097] BabyMetroid_Instr_3 [] { sm_game::actors::ridley::baby_metroid_load_palette(k, game) };
    [10928114] BabyMetroid_Instr_1 [] { sm_game::actors::ridley::baby_metroid_goto_if_moving(k, game) };
    [10928120] BabyMetroid_Goto [] { sm_game::actors::ridley::baby_metroid_goto(k, game) };
}
crate::game::callback_dispatch! {
    pub fn CallCeresSteamFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10940441] CeresSteam_Func_1 [] { sm_game::actors::ceres_doors::steam_rotating_offset(k, game) };
    @external {
        10940404 => [()];
    }
}
crate::game::callback_dispatch! {
    pub fn CallCeresDoor(ea: u32, game: &mut Game) => ea;
    setup { let k: u16 = word!(game.cur_enemy_index); }
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10942373] CeresDoor_Func_4 [] { sm_game::actors::ceres_doors::mode7_wall(game) };
    [10942544] CeresDoor_Func_7 [] { sm_game::actors::ceres_doors::elevator_animations(game) };
    @forward [pub fn (game: &mut Game)];
    [10942315] CeresDoor_Func_2 => sm_game::actors::ceres_doors::escape_rumble(0x14, game);
    [10942320] CeresDoor_Func_3 => sm_game::actors::ceres_doors::escape_rumble(0x1d, game);
    @external {
        10942397 => [sm_game::actors::ceres_doors::elevator_default(k, game)];
        10942428 => [sm_game::actors::ceres_doors::rumbling_and_explosions(k, game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10914693 | 10914713] SpikeyPlatform2ndEnemy_Init [] { sm_game::actors::fire_arc::initialize_spike_platform_bottom(game) };
    [10917060] NuclearWaffle_Init [] { sm_game::actors::fire_arc::initialize(game) };
    [10917390] NuclearWaffle_Main [] { sm_game::actors::fire_arc::main_ai(game) };
    [10918488] FakeKraid_Init [] { sm_game::actors::mini_kraid::initialize(game); };
    [10918594] FakeKraid_Main [] { sm_game::actors::mini_kraid::main_ai(game); };
    [10918946] FakeKraid_Touch [] { sm_game::actors::mini_kraid::touch(game); };
    [10918969] FakeKraid_Shot [] { sm_game::actors::mini_kraid::shot(game); };

    [10940337] CeresSteam_Init [] { sm_game::actors::ceres_doors::initialize_steam(game) };
    [10940429] CeresSteam_Main [] { sm_game::actors::ceres_doors::steam_main(game) };
    [10942149] CeresDoor_Init [] { sm_game::actors::ceres_doors::initialize_door(game) };
    [10942309] CeresDoor_Main [] { sm_game::actors::ceres_doors::door_main(game) };
    [10943346] Zebetites_Init [] { sm_game::actors::zebetites::initialize(game); };
    [10943539] Zebetites_Main [] { sm_game::actors::zebetites::main_ai(game); };
    [10943911] Zebetites_Touch [] { sm_game::actors::zebetites::touch(game); };
    [10943916] Zebetites_Shot [] { sm_game::actors::zebetites::shot(game); };
    @functions [pub fn (game: &mut Game)];
    [10920659] CeresRidley_Hurt [] { sm_game::actors::ceres_ridley::hurt(game) };
    [10929814] RidleysExplosion_Init [] { sm_game::actors::ridley::explosion_init(game) };
    [10930388] RidleysExplosion_Main [] { sm_game::actors::ridley::explosion_main(game) };
    [10936202] Ridley_Shot [] { sm_game::actors::ridley::shot(game) };
    [10936242] Ridley_Powerbomb [] { sm_game::actors::ridley::power_bomb(game) };
    @forward [pub fn (game: &mut Game)];
    [10911759] Enemy_GrappleReact_CancelBeam_A6 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10911779] Enemy_NormalTouchAI_A6 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10911789] Enemy_NormalShotAI_A6 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10911809] Enemy_NormalFrozenAI_A6 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        10915836 => [fire_geysers::initialize(game)];
        10915875 => [fire_geysers::run(game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918694] FakeKraid_Instr_2 [jp] { sm_game::actors::mini_kraid::step(game); };
    @functions [pub fn (_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918772] FakeKraid_Instr_1 [] { sm_game::actors::mini_kraid::choose_animation(game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918834] FakeKraid_Instr_3 [jp] { sm_game::actors::mini_kraid::roar(game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918852] FakeKraid_Instr_4 [jp] { sm_game::actors::mini_kraid::spit_left(game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918914] FakeKraid_Instr_5 [jp] { sm_game::actors::mini_kraid::spit_right(game); };

    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10940711] CeresSteam_Instr_2 [] { sm_game::actors::ceres_doors::steam_activate(k, jp, game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10937534] Ridley_Instr_5 [] { sm_game::actors::ridley::roar(jp, game) };
    [10937546] Ridley_Instr_6 [] { sm_game::actors::ridley::stop_roaring(jp, game) };
    [10937554] Ridley_Instr_10 [] { sm_game::actors::ridley::goto_if_samus_health_low(jp, game) };
    [10937582] Ridley_Instr_4 [] { sm_game::actors::ridley::goto_by_holding_samus(jp, game) };
    [10937592] Ridley_Instr_3 [] { sm_game::actors::ridley::goto_if_not_holding_samus(jp, game) };
    [10937601] Ridley_Instr_2 [] { sm_game::actors::ridley::set_feet_distance(jp, game) };
    [10937623] Ridley_Instr_1 [] { sm_game::actors::ridley::goto_if_not_facing_left(jp, game) };
    [10937631] Ridley_Instr_14 [] { sm_game::actors::ridley::move_by(jp, game) };
    [10938140] Ridley_Instr_9 [] { sm_game::actors::ridley::flip_left(jp, game) };
    [10938151] Ridley_Instr_7 [] { sm_game::actors::ridley::face_forward(jp, game) };
    [10938159] Ridley_Instr_8 [] { sm_game::actors::ridley::flip_right(jp, game) };
    [10938445] Ridley_Instr_11 [] { sm_game::actors::ridley::calculate_fireball_velocities(jp, game) };
    [10938729] Ridley_Instr_15 [] { sm_game::actors::ridley::start_ceres_liftoff(jp, game) };
    [10938742] Ridley_Instr_16 [] { sm_game::actors::ridley::start_norfair_liftoff(jp, game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10938628] Ridley_Instr_12 [] { sm_game::actors::ridley::fireball_with_afterburn(jp, game) };
    [10938633] Ridley_Instr_13 [] { sm_game::actors::ridley::fireball_without_afterburn(jp, game) };

    @external {
        10915247 | 10915347..=10915793 => [fire_geysers::instruction(ea, j, game)];
        10940701 => [ceres_doors::change_properties(k, VANISH, 0, j, game)];
        10940725 => [ceres_doors::change_properties(k, 0, VANISH, j, game)];
        10942014 => ceres_doors::branch_unless_samus_near;
        10942058 => [ceres_doors::branch_unless_boss_defeated(j, game)];
        10942072 => [ceres_doors::branch_unless_escape(j, game)];
        10942091 => [ceres_doors::change_properties(k, ceres_doors::TANGIBLE, 0, j, game)];
        10942101 => [ceres_doors::change_properties(k, 0, ceres_doors::TANGIBLE, j, game)];
        10942111 => [ceres_doors::mark_open(k, true, j, game)];
        10942118 => [ceres_doors::change_properties(k, ceres_doors::INVISIBLE, 0, j, game)];
        10942128 => [ceres_doors::mark_open(k, false, j, game)];
        10942131 => [ceres_doors::change_properties(k, 0, ceres_doors::INVISIBLE, j, game)];
        10942141 => [ceres_doors::play_open_sound(j, game)];
    }
}
