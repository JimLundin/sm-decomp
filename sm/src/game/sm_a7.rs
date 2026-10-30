use crate::game::actors;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::rtl::rom_record;
use core::mem::offset_of;
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_86::*;

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct SpriteDrawInstr {
        pub field_0: u16;
        pub field_2: VoidP;
    }
}

pub use crate::types::Consts_A7;

crate::game::constants::typed_constants! { Consts_A7;
    addr_kDachora_Ilist_F4C1 = 62657, addr_kDachora_Ilist_F4B9 = 62649,
    addr_kDachora_Ilist_F4B3 = 62643, addr_kDachora_Ilist_F48B = 62603,
    addr_kDachora_Ilist_F45B = 62555, addr_kDachora_Ilist_F407 = 62471,
    addr_kDachora_Ilist_F3FF = 62463, addr_kDachora_Ilist_F3F7 = 62455,
    addr_kDachora_Ilist_F3C9 = 62409, addr_kDachora_Ilist_F399 = 62361,
    addr_kDachora_Ilist_F345 = 62277, addr_kDachora_Palette = 61989,
    addr_kEtecoon_Ilist_E8D6 = 59606, addr_kEtecoon_Ilist_E8CE = 59598,
    addr_kEtecoon_Ilist_E8C8 = 59592, addr_kEtecoon_Ilist_E8AC = 59564,
    addr_stru_A7E898 = 59544, addr_kEtecoon_Ilist_E894 = 59540,
    addr_stru_A7E880 = 59520, addr_kEtecoon_Ilist_E876 = 59510,
    addr_kEtecoon_Ilist_E870 = 59504, addr_stru_A7E862 = 59490,
    addr_kEtecoon_Ilist_E854 = 59476, addr_kEtecoon_Ilist_E83C = 59452,
    addr_stru_A7E828 = 59432, addr_kEtecoon_Ilist_E81E = 59422,
    addr_kKraid_Ilist_CC9D = 52381, addr_kKraid_Ilist_CC91 = 52369,
    addr_kKraid_Ilist_CC81 = 52353, addr_kKraid_Ilist_CC53 = 52307,
    addr_kKraid_Ilist_CC4D = 52301, addr_kKraid_Ilist_CC47 = 52295,
    addr_kKraid_Ilist_CC41 = 52289, addr_locret_A7B831 = 47153,
    addr_kKraidsRoomBg = 42774, addr_kKraid_Sprmap_A5DF = 42463,
    addr_kKraidTilemaps_3 = 41160, addr_kKraidTilemaps_2 = 40392,
    addr_kKraidTilemaps_1 = 39624, addr_kKraidTilemaps_0 = 38856,
    addr_stru_A79764 = 38756, addr_stru_A7974A = 38730,
    addr_stru_A796DA = 38618, addr_kKraid_Sprmap_8C6C = 35948,
    addr_kKraid_Ilist_8B0A = 35594, addr_kKraid_Ilist_8B04 = 35588,
    addr_kKraid_Ilist_8AFE = 35582, addr_kKraid_Ilist_8AF0 = 35568,
    addr_kKraid_Ilist_8AA4 = 35492, addr_kKraid_Ilist_8A41 = 35393,
    addr_kKraid_Ilist_89F3 = 35315, addr_kKraid_Ilist_8887 = 34951,
    addr_off_A78885 = 34949, addr_kKraid_Ilist_87BD = 34749,
    addr_off_A787BB = 34747, addr_kKraid_Ilist_86F3 = 34547,
    addr_kKraid_Ilist_86ED = 34541, addr_kKraid_Ilist_86E7 = 34535,
    addr_kSpritemap_Nothing_A7 = 32845,
}

crate::type_aliases! {
    pub Enemy_Phantoon, Enemy_Etecoon, Enemy_Dachora = crate::game::enemy_layouts::EnemyLayout8;
}

crate::game::constants::typed_constants! { i32;
    fnnullsub_170_A7 = 0xa7804c,
    fnUnpauseHook_Kraid_IsDead = 0xa7c1fb, fnUnpauseHook_Kraid_IsAlive = 0xa7c24e,
    fnKraid_UnpauseHook_IsSinking = 0xa7c2a0, fnPauseHook_Kraid = 0xa7c325,
    fnnullsub_237 = 0xa7d4a8, fnPhantoon_Spawn8FireballsInCircleAtStart = 0xa7d4a9,
    fnPhantoon_WaitBetweenSpawningAndSpinningFireballs = 0xa7d4ee, fnPhantoon_SpawnFireballsBeforeFight = 0xa7d508,
    fnPhantoon_WavyFadeIn = 0xa7d54a, fnPhantoon_PickPatternForRound1 = 0xa7d596,
    fnPhantoon_MovePhantoonInFigure8ThenOpenEye = 0xa7d5e7, fnPhantoon_EyeFollowsSamusUntilTimerRunsOut = 0xa7d60d,
    fnPhantoon_BecomesSolidAndBodyVuln = 0xa7d65c, fnPhantoon_IsSwooping = 0xa7d678,
    fnPhantoon_FadeoutWithSwoop = 0xa7d6b9, fnPhantoon_WaitAfterFadeOut = 0xa7d6d4,
    fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt = 0xa7d6e2, fnPhantoon_FadeInBeforeFigure8 = 0xa7d72d,
    fnPhantoon_BecomeSolidAfterRainingFireballs = 0xa7d73f, fnPhantoon_FadeInDuringFireballRain = 0xa7d767,
    fnPhantoon_FollowSamusWithEyeDuringFireballRain = 0xa7d788, fnPhantoon_FadeOutDuringFireballRain = 0xa7d7d5,
    fnPhantoon_SpawnRainingFireballs = 0xa7d7f7, fnPhantoon_FadeOutBeforeFirstFireballRain = 0xa7d82a,
    fnPhantoon_FadeOutBeforeEnrage = 0xa7d85c, fnPhantoon_MoveEnragedPhantoonToTopCenter = 0xa7d874,
    fnPhantoon_FadeInEnragedPhantoon = 0xa7d891, fnPhantoon_Enraged = 0xa7d8ac,
    fnPhantoon_FadeoutAfterEnrage = 0xa7d916, fnPhantoon_CompleteSwoopAfterFatalShot = 0xa7d92e,
    fnPhantoon_DyingPhantoonFadeInOut = 0xa7d948, fnPhantoon_DyingPhantoonExplosions = 0xa7d98b,
    fnPhantoon_WavyDyingPhantoonAndCry = 0xa7da51, fnPhantoon_DyingFadeOut = 0xa7da86,
    fnPhantoon_AlmostDead = 0xa7dad7, fnPhantoon_Dead = 0xa7db3d,
    fnEtecoon_Func_4 = 0xa7e9af, fnEtecoon_Func_5 = 0xa7ea00,
    fnEtecoon_Func_6 = 0xa7ea37, fnEtecoon_Func_7 = 0xa7eab5,
    fnEtecoon_Func_8 = 0xa7eb02, fnEtecoon_Func_9 = 0xa7eb2c,
    fnEtecoon_Func_10 = 0xa7eb50, fnEtecoon_Func_11 = 0xa7ebcd,
    fnEtecoon_Func_12 = 0xa7ec1b, fnEtecoon_Func_16 = 0xa7ec97,
    fnEtecoon_Func_17 = 0xa7ecbb, fnEtecoon_Func_18 = 0xa7ecdf,
    fnEtecoon_Func_19 = 0xa7ed09, fnEtecoon_Func_20 = 0xa7ed2a,
    fnEtecoon_Func_21 = 0xa7ed54, fnEtecoon_Func_22 = 0xa7ed75,
    fnEtecoon_Func_23 = 0xa7edc7, fnEtecoon_Func_24 = 0xa7ee3e,
    fnEtecoon_Func_25 = 0xa7ee9a, fnEtecoon_Func_26 = 0xa7eeb8,
    fnDachora_Func_2 = 0xa7f570, fnDachora_Func_3 = 0xa7f5bc,
    fnDachora_Func_4 = 0xa7f5ed, fnDachora_Func_5 = 0xa7f65e,
    fnDachora_Func_7 = 0xa7f78f, fnDachora_Func_8 = 0xa7f806,
    fnDachora_Func_11 = 0xa7f935, fnDachora_Func_12 = 0xa7f98c,
}
crate::game::callback_dispatch! {
    pub fn CallEnemyInstrExtFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10997485] Phantoon_Func_1 [] { sm_game::actors::phantoon::materialisation_sound(game); };
    [10997823] Phantoon_StartTrackingSamusAndInitEyeTimer [] { sm_game::actors::phantoon::open_eye(game); };
    [10997878] Phantoon_PickPatternForRound2 [] { sm_game::actors::phantoon::pick_pattern(game); };
    @external {
        10997598 => [actors::phantoon_actions::spawn_casual_flame(k, game)];
    }
}
pub fn Kraid_Touch_ArmFoot(game: &mut Game) {
    sm_game::actors::kraid::touch_arm_or_foot(game);
}
pub fn Kraid_Arm_Shot(j: u16, game: &mut Game) {
    sm_game::actors::kraid::arm_shot(game, j);
}
pub fn Kraid_CheckIfDead(game: &mut Game) -> u16 {
    sm_game::actors::kraid::is_dead(game)
}
pub fn Kraid_Instr_1(_k: u16, jp: EnemyScript) -> EnemyScript {
    jp
}
pub fn Kraid_Death_SinkThroughFloor(game: &mut Game) {
    sm_game::actors::kraid::sink_through_floor(game);
}
pub fn Kraid_FadeInBg_FadeInBp6(game: &mut Game) {
    sm_game::actors::kraid::fade_in_bg6(game);
}
/// The original "X undefined" debug message Kraid's foot and fingernails print.
pub(crate) fn print_x_undefined() {
    unsafe { printf(b"X undefined\n\0" as *const u8 as *const c_char) };
}
/// Sets the shot AI of Kraid's enemy definition at `enemy_ptr`.
pub(crate) fn set_kraid_shot_ai(enemy_ptr: u16, shot_ai: u16, game: &mut Game) {
    (unsafe { rom_record(0xa0, enemy_ptr, game) }).set_word(offset_of!(EnemyDef, shot_ai), shot_ai);
    #[cfg(sm_verify_dll)]
    crate::verify_probe::hit("actors.kraid.shot_ai.disabled");
}
crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10982544] KraidsArm_Touch [] { sm_game::actors::kraid::arm_touch(game); };
    [10987865] Kraid_Init [] { sm_game::actors::kraid::initialize(game); };
    [10988355] KraidsArm_Init [] { sm_game::actors::kraid::initialize_arm(game); };
    [10988392] KraidsTopLint_Init [] { sm_game::actors::kraid::initialize_lint(game, 0x80); };
    [10988444] KraidsMiddleLint_Init [] { sm_game::actors::kraid::initialize_lint(game, 0xc0); };
    [10988490] KraidsBottomLint_Init [] { sm_game::actors::kraid::initialize_lint(game, 0x100); };
    [10988536] KraidsFoot_Init [] { sm_game::actors::kraid::initialize_foot(game); };
    [10988577] Kraid_Main [] { sm_game::actors::kraid::main_ai(game); };
    [10991549] KraidsArm_Main [] { sm_game::actors::kraid::arm_main(game); };
    [10991617] KraidsTopLint_Main [] { sm_game::actors::kraid::lint_main_ai(game, 0x80); };
    [10991629] KraidsMiddleLint_Main [] { sm_game::actors::kraid::lint_main_ai(game, 0xc0); };
    [10991641] KraidsBottomLint_Main [] { sm_game::actors::kraid::lint_main_ai(game, 0x100); };
    [10992118] KraidsFoot_Main [] { sm_game::actors::kraid::foot_main(game); };
    [10992847 | 10992862] KraidsGoodFingernail_Touch [] { sm_game::actors::kraid::fingernail_touch(game); };
    [10992946] KraidsGoodFingernail_Main [] { sm_game::actors::kraid::fingernail_main(game, 0x180); };
    [10992969] KraidsBadFingernail_Main [] { sm_game::actors::kraid::fingernail_main(game, 0x1c0); };
    [10997235] Phantoon_Init [] { sm_game::actors::phantoon::initialize(game); };
    [10997333] Phantoon2_Init [] { sm_game::actors::phantoon::initialize_part(game); };
    [10997414] Phantoon_Main [] { sm_game::actors::phantoon::main_ai(game); };
    [11001151] Phantoon_Hurt [] { sm_game::actors::phantoon::hurt(game); };
    [11001237] Phantoon_Touch [] { sm_game::actors::phantoon::touch(game); };
    [11001243] Phantoon_Shot [] { sm_game::actors::phantoon::shot(game); };
    [11004178] Etecoon_Init [] { sm_game::actors::etecoon::initialize(game); };
    [11004224] Etecoon_Main [] { sm_game::actors::etecoon::main_ai(game); };
    [11007197] Dachora_Init [] { sm_game::actors::dachora_run::initialize(game); };
    [11007278] Dachora_Main [] { sm_game::actors::dachora_run::main_ai(game); };
    @forward [pub fn (game: &mut Game)];
    [10977295] Enemy_GrappleReact_CancelBeam_A7 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10977325] Enemy_NormalShotAI_A7 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10977340]
    Enemy_NormalPowerBombAI_SkipDeathAnim_A7 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    [10977345] Enemy_NormalFrozenAI_A7 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @forward [pub fn (game: &mut Game)];
    [10982559] Kraid_Touch => sm_game::actors::kraid::touch(game);
    Kraid_Shot => sm_game::actors::kraid::shot(game);
    @forward [pub fn (game: &mut Game)];
    [10992879] KraidsGoodFingernail_Init => sm_game::actors::kraid::initialize_fingernail(game);
    [10992941] KraidsBadFingernail_Init => sm_game::actors::kraid::initialize_fingernail(game);
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10977418] EnemyInstr_Call_A7 [] {
        CallEnemyInstrExtFunc(((jp.word(0)) as u32) | (0xa70000u32), k, game);
        jp.offset(1)
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10979983] Kraid_Instr_9 [] {
        match sm_game::actors::kraid::arm_swipe_when_hurt(game) {
            Some(list) => EnemyScript::jump(list),
            None => jp,
        }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10991158] Kraid_Instr_DecYpos [jp] { sm_game::actors::kraid::step_up(game); };
    [10991164] Kraid_Instr_IncrYpos_Shake [jp] { sm_game::actors::kraid::step_down_and_shake(game); };
    [10991182] Kraid_Instr_PlaySound_0x76 [jp] { sm_game::actors::kraid::stomp_sound(game); };
    [10991194] Kraid_Instr_XposMinus3 [jp] { sm_game::actors::kraid::step_back(game); };
    [10991207] Kraid_Instr_XposMinus3b [jp] { sm_game::actors::kraid::step_back(game); };
    [10991220] Kraid_Instr_XposPlus3 [jp] { sm_game::actors::kraid::step_forward(game); };
    [10991235] Kraid_Instr_MoveHimRight [jp] { sm_game::actors::kraid::lunge_right(game); };
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10999114] Phantoon_WavyFadeIn [] { sm_game::actors::phantoon::wavy_fade_in(game, k); };
    [10999271] Phantoon_MovePhantoonInFigure8ThenOpenEye [] { sm_game::actors::phantoon::figure_eight(game, k); };
    [10999309] Phantoon_EyeFollowsSamusUntilTimerRunsOut [] { sm_game::actors::phantoon::eye_follows_samus(game, k); };
    [10999416] Phantoon_IsSwooping [] { sm_game::actors::phantoon::swooping(game, k); };
    [10999481] Phantoon_FadeoutWithSwoop [] { sm_game::actors::phantoon::fade_out_with_swoop(game, k); };
    [10999522] Phantoon_MoveLeftOrRightAndPickEyeOpenPatt [] { sm_game::actors::phantoon::reappear(game, k); };
    @functions [pub fn (_k: u16, game: &mut Game)];
    [10999597] Phantoon_FadeInBeforeFigure8 [] { sm_game::actors::phantoon::fade_in_before_figure_eight(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10999615] Phantoon_BecomeSolidAfterRainingFireballs [] { sm_game::actors::phantoon::become_solid_after_flame_rain(game, k); };
    [10999655] Phantoon_FadeInDuringFireballRain [] { sm_game::actors::phantoon::fade_in_during_flame_rain(game, k); };
    [10999765] Phantoon_FadeOutDuringFireballRain [] { sm_game::actors::phantoon::fade_out_during_flame_rain(game, k); };
    [10999850] Phantoon_FadeOutBeforeFirstFireballRain [] { sm_game::actors::phantoon::fade_out_before_first_flame_rain(game, k); };
    [10999900] Phantoon_FadeOutBeforeEnrage [] { sm_game::actors::phantoon::fade_out_before_enrage(game, k); };
    [10999953] Phantoon_FadeInEnragedPhantoon [] { sm_game::actors::phantoon::fade_in_enraged(game, k); };
    [10999980] Phantoon_Enraged [] { sm_game::actors::phantoon::enraged(game, k); };
    [11000086] Phantoon_FadeoutAfterEnrage [] { sm_game::actors::phantoon::fade_out_after_enrage(game, k); };
    [11000110] Phantoon_CompleteSwoopAfterFatalShot [] { sm_game::actors::phantoon::finish_swoop_after_fatal_shot(game, k); };
    [11000136] Phantoon_DyingPhantoonFadeInOut [] { sm_game::actors::phantoon::dying_fade_in_out(game, k); };
    [11000401] Phantoon_WavyDyingPhantoonAndCry [] { sm_game::actors::phantoon::wavy_dying_cry(game, k); };
    [11000454] Phantoon_DyingFadeOut [] { sm_game::actors::phantoon::dying_fade_out(game, k); };
    [11000535] Phantoon_AlmostDead [] { sm_game::actors::phantoon::almost_dead(game, k); };
    [11000637] Phantoon_Dead [] { sm_game::actors::phantoon::dead(game, k); };
    [11004335] Etecoon_Func_4 [] { sm_game::actors::etecoon::initial(game, k); };
    [11004471] Etecoon_Func_6 [] { sm_game::actors::etecoon::hopping_bottom(game, k); };
    [11004674] Etecoon_Func_8 [] { sm_game::actors::etecoon::running_left(game, k); };
    [11004716] Etecoon_Func_9 [] { sm_game::actors::etecoon::running_right(game, k); };
    [11004752] Etecoon_Func_10 [] { sm_game::actors::etecoon::jumping(game, k); };
    [11004877] Etecoon_Func_11 [] { sm_game::actors::etecoon::wall_jump(game, k); };
    [11005079] Etecoon_Func_16 [] { sm_game::actors::etecoon::run_to_left_run_up_point(game, k); };
    [11005115] Etecoon_Func_17 [] { sm_game::actors::etecoon::run_to_right_run_up_point(game, k); };
    [11005151] Etecoon_Func_18 [] { sm_game::actors::etecoon::running_for_tunnel_jump(game, k); };
    [11005193] Etecoon_Func_19 [] { sm_game::actors::etecoon::tunnel_jump(game, k); };
    [11005226] Etecoon_Func_20 [] { sm_game::actors::etecoon::running_through_tunnel(game, k); };
    [11005268] Etecoon_Func_21 [] { sm_game::actors::etecoon::falling_from_tunnel_ledge(game, k); };
    [11005301] Etecoon_Func_22 [] { sm_game::actors::etecoon::hopping_top(game, k); };
    [11005383] Etecoon_Func_23 [] { sm_game::actors::etecoon::start_hop_top(game, k); };
    [11005502] Etecoon_Func_24 [] { sm_game::actors::etecoon::hop_until_samus_near(game, k); };
    [11005594] Etecoon_Func_25 [] { sm_game::actors::etecoon::running_for_failed_tunnel_jump(game, k); };
    [11005624] Etecoon_Func_26 [] { sm_game::actors::etecoon::failed_tunnel_jump(game, k); };
    [11007344] Dachora_Func_2 [] { sm_game::actors::dachora_run::wait_for_samus(game, k); };
    [11007420] Dachora_Func_3 [] { sm_game::actors::dachora_run::start_running(game, k); };
    [11007469] Dachora_Func_4 [] { sm_game::actors::dachora_run::running_left(game, k); };
    [11007582] Dachora_Func_5 [] { sm_game::actors::dachora_run::running_right(game, k); };
    [11007887] Dachora_Func_7 [] { sm_game::actors::dachora_run::activate_shinespark(game, k); };
    [11008006] Dachora_Func_8 [] { sm_game::actors::dachora_run::shinesparking(game, k); };
    [11008309] Dachora_Func_11 [] { sm_game::actors::dachora_run::falling(game, k); };
    [11008396] Dachora_Func_12 [] { sm_game::actors::dachora_run::echo(game, k); };
    @functions [pub fn (v0: u16, game: &mut Game)];
    [10999388] Phantoon_BecomesSolidAndBodyVuln [] { sm_game::actors::phantoon::become_solid_and_swoop(game, v0); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10998953] Phantoon_Spawn8FireballsInCircleAtStart [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::SPAWN_STARTING_FLAMES); };
    [10999022] Phantoon_WaitBetweenSpawningAndSpinningFireballs [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::WAIT_BEFORE_SPINNING_FLAMES); };
    [10999048] Phantoon_SpawnFireballsBeforeFight [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::SPAWN_FLAMES_BEFORE_FIGHT); };
    [10999190] Phantoon_PickPatternForRound1 [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::PICK_ROUND_1_PATTERN); };
    [10999508] Phantoon_WaitAfterFadeOut [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::WAIT_AFTER_FADE_OUT); };
    [10999688] Phantoon_FollowSamusWithEyeDuringFireballRain [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::FOLLOW_SAMUS_DURING_FLAME_RAIN); };
    [10999799] Phantoon_SpawnRainingFireballs [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::SPAWN_FLAME_RAIN); };
    [10999924] Phantoon_MoveEnragedPhantoonToTopCenter [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::MOVE_TO_TOP_CENTRE); };
    [11000203] Phantoon_DyingPhantoonExplosions [] { sm_game::actors::phantoon::countdown(game, k, sm_game::actors::phantoon::function::DYING_EXPLOSIONS); };
    [11004416] Etecoon_Func_5 [] { sm_game::actors::etecoon::start_hop_bottom(game, k); };
    [11004597] Etecoon_Func_7 [] { sm_game::actors::etecoon::look_at_samus(game, k); };
    [11004955] Etecoon_Func_12 [] { sm_game::actors::etecoon::landed_from_jump(game, k); };
}
/// Runs the bank $A7 function at `address` for the current enemy.
pub(crate) fn run_a7_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
}
