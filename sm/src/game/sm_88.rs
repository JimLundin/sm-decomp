use crate::game::liquid_effects;
use crate::game::player_transitions::xray::BeamPhase;
pub use crate::game::prelude::*;
use crate::game::scrolling_sky;
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime::{self, spawn_hdma, spawn_hdma_last};
use crate::game::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, TourianStatuePhase, Weather};

pub use crate::game::constants::bank_8f::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;

use crate::game::explosion_window::{self, Phase};
use crate::game::gameplay_flag_runtime::CheckBossBitForCurArea;
use crate::game::sm_a9::MotherBrain_CalculateRainbowBeamHdma;
use crate::game::xray_background::{self, finish_restore, restore_left, restore_right};

pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnDoorTransitionFunction_FadeOutScreen = 0x82e2db, fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
    fnPauseHook_Empty = 0x8883e1,
    fnCinematicFunctionOpening = 0x8b9b68,
}
crate::game::constants::typed_constants! { u32;
    fnHdmaobjPreInstr_MorphBallEyeBeamWidening = 0x88e9e6, fnHdmaobjPreInstr_MorphBallEyeBeamFull = 0x88ea3c,
    fnHdmaobjPreInstr_MorphBallEyeBeamFading = 0x88eacb,
}
crate::game::callback_dispatch! {
    pub unsafe fn CallHdmaobjInstrFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [8972135] InitializeRainbowBeam [] {
        *ram_ptr!(game; 0x74, 0) = 32;
        *ram_ptr!(game; 0x74, 1) = 71;
        *ram_ptr!(game; 0x74, 2) = 0x8f;
        word!(game.fx_layer_blending_config_c = 36);
        *ram_ptr!(game; 0x9c00, 0) = 100;
        *(ram_ptr!(game, u16; 0x9c00, 1)) = -25344i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 3)) = 0;
        word! {game; hdma_table_2[0] = 0; hdma_table_2[1] = -32736i32 as u16;
            hdma_table_2[2] = 0; }
        MotherBrain_CalculateRainbowBeamHdma(game);
    };
    @external {
        8948500 => explosion_window::begin_pre_flash; 8948530 => explosion_window::begin_prescaled_pre_flash;
        8948537 => explosion_window::begin_explosion; 8948551 => explosion_window::begin_prescaled_explosion;
        8948558 => [explosion_window::finish_power_bomb(k, game)]; 8954596 => explosion_window::begin_crystal_flash;
        8954633 => explosion_window::begin_explosion; 8954647 => [explosion_window::finish_crystal_flash(k, game)];
        8970129 => [()]; 8970130 => [()]; 8972070 => [()];
        9554681 => [xray_background::freeze_and_backup(k, game)];
        9554716 => xray_background::capture_bg1_right;
        9554775 => xray_background::capture_bg1_left; 9554830 => xray_background::build_overlay;
        9556179 => xray_background::capture_bg2_right; 9556339 => xray_background::upload_left; 9556384 => xray_background::upload_right;
        9556668 => xray_background::set_backdrop; 9557650 => [Runtime::new(game).initialize_suit_pickup_hdma()];
    }
}
pub unsafe fn SpawnBG3ScrollHdmaObject(game: &mut Game) {
    unsafe {
        static unk_88D869: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0 as VoidP);
        spawn_hdma_last(0x88, &unk_88D869, game);
    }
}
pub unsafe fn FxTypeFunc_28_CeresRidley(game: &mut Game) {
    unsafe {
        static unk_88D8E2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x5, 0xd906 as VoidP);
        static unk_88D8EA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d as VoidP);
        spawn_hdma(0x88, &unk_88D8E2, game);
        spawn_hdma(0x88, &unk_88D8EA, game);
    }
}
pub unsafe fn FxTypeFunc_CeresElevator(game: &mut Game) {
    unsafe {
        static unk_88D92C: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x5, 0xd939 as VoidP);
        spawn_hdma(0x88, &unk_88D92C, game);
    }
}
pub unsafe fn SpawnBombTorizoHaze(game: &mut Game) {
    unsafe {
        static unk_88DD36: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x2, 0x32, 0xdd4a as VoidP);
        static unk_88DD3E: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0xdd62 as VoidP);
        spawn_hdma(0x88, &unk_88DD36, game);
        spawn_hdma(0x88, &unk_88DD3E, game);
    }
}
pub fn HdmaobjPreInstr_BombTorizoHazeColorMathBgColor(k: u16, game: &mut Game) {
    {
        word!(game.fx_layer_blending_config_c = 44);
    }
}
pub unsafe fn FxTypeFunc_2C_Haze(game: &mut Game) {
    unsafe {
        static unk_88DDD4: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x32, 0xded3 as VoidP);
        static unk_88DDDD: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x32, 0xdeeb as VoidP);
        if CheckBossBitForCurArea(1, game) & 1 != 0 {
            spawn_hdma(0x88, &unk_88DDDD, game);
        } else {
            spawn_hdma(0x88, &unk_88DDD4, game);
        };
    }
}
static unk_88DF38: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_88DF34 => spawn_hdma(0x88, &unk_88DF38, game);
}
pub unsafe fn HdmaobjPreInstr_E449(k: u16, game: &mut Game) {
    unsafe {
        if (word!(game.phantom_related_layer_flag) & 0x4000) != 0 {
            word!(game.fx_layer_blending_config_c = 26);
        } else if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) != 0 {
            if byte!((*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).parameter_1; 0) == 0xff {
                word!(game.fx_layer_blending_config_c = 4);
                let v0: i32 = word!(game.hdma_object_index) as i32 >> 1;
                word!(game.hdma_object_instruction_list_pointers[v0] += 2);
                word!(game.hdma_object_instruction_timers[v0] = 1);
            }
        } else {
            word!(game.fx_layer_blending_config_c = 4);
        };
    }
}
static unk_88E4A0: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0xf, 0xe4a8 as VoidP);
pub unsafe fn sub_88E487(v0: u16, r22: u16, game: &mut Game) {
    unsafe {
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(2)).parameter_1 = v0;
        set_fields! {[(*(ram_ptr!(game, EnemyData; 0xf78)).offset(3))], 0; ai_var_D, ai_var_E}
        (*(ram_ptr!(game, EnemyData; 0xf78)).offset(3)).ai_preinstr = r22;
        spawn_hdma(0x88, &unk_88E4A0, game);
    }
}
static unk_88E722: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0, 0x2c, 0xe727 as VoidP);
static unk_88E74C: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x41, 0x26, 0xe751 as VoidP);
crate::forward_calls! { pub unsafe fn (game: &mut Game) -> u16;
    MotherBrainRisingHdmaObject =>
        spawn_hdma(0x88, &unk_88E722, game);
    SpawnMotherBrainRainbowBeamHdma =>
        spawn_hdma(0x88, &unk_88E74C, game);
}
pub unsafe fn HdmaobjPreInstr_E7BC(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.game_state) == 19 {
            *(ram_ptr!(game, u16; 0x18b4)).offset((word!(game.hdma_object_index) >> 1) as isize) =
                0;
        } else {
            word!(game.fx_layer_blending_config_c = 36);
            if word!(game.game_state) != 27 {
                MotherBrain_CalculateRainbowBeamHdma(game);
                sub_88E7ED(game);
            }
        };
    }
}
pub unsafe fn sub_88E7ED(game: &mut Game) {
    unsafe {
        let mut v0: u16 = crate::rtl::rom_table_word!(
            game,
            0x88e833,
            (word!(game.hdma_object_A[0]) >> 1) as isize
        );
        if (v0 & 0x8000) == 0 {
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
            word!(game.hdma_object_A[0] += 1);
        } else {
            word!(game.hdma_object_A[0] = 0);
            v0 = crate::rtl::rom_table_word!(game, 0x88e833, 0);
        }
        *ram_ptr!(game; 0x74, 0) = v0 as u8 & 0x1f | 0x20;
        *ram_ptr!(game; 0x74, 1) = (v0 as i32 >> 5 & 0x1f | 0x40) as u8;
        *ram_ptr!(game; 0x74, 2) = ((v0 >> 2) as i32 >> 8 & 0x1f | 0x80) as u8;
    }
}
pub unsafe fn SpawnMorphBallEyeBeamHdma(game: &mut Game) {
    unsafe {
        static unk_88E8E4: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x41, 0x26, 0xe8ec as VoidP);
        spawn_hdma(0x88, &unk_88E8E4, game);
    }
}
pub unsafe fn SpawnTitleScreenGradientObjs(game: &mut Game) {
    unsafe {
        static unk_88EB6B: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xeb85 as VoidP);
        static unk_88EB63: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x32, 0xeb73 as VoidP);
        spawn_hdma(0x88, &unk_88EB63, game);
        spawn_hdma(0x88, &unk_88EB6B, game);
    }
}
pub unsafe fn SpawnIntroCutsceneCrossfadeHdma(game: &mut Game) {
    unsafe {
        static unk_88EBFB: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0, 0x31, 0xec03 as VoidP);
        spawn_hdma(0x88, &unk_88EBFB, game);
    }
}
pub unsafe fn CinematicFunction_Intro_Func133(game: &mut Game) {
    unsafe {
        word!(game.g_word_7E0D9C = 1);
        *(ram_ptr!(game, u16; 0xd9e)) = 0x4000;
        word!(game.loop_counter_transfer_enemies_to_vram = 8);
        Runtime::new(game).initialize_wave_pair(0xd82, 0x9800);
        static unk_88EC82: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a as VoidP);
        spawn_hdma(0x88, &unk_88EC82, game);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallHdmaobjPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8491739 => [()]; 8946872 => [()]; 10785852 => [()]; 9556607 => xray_background::setup_blending;
        8947439 => [Runtime::new(game).advance_xray_background()]; 8947506 => [Runtime::new(game).advance_xray_beam(BeamPhase::Initial)];
        8947540 => [Runtime::new(game).advance_xray_beam(BeamPhase::Widening)];
        8947627 => [Runtime::new(game).advance_xray_beam(BeamPhase::Full)];
        8948020 => restore_left;
        8948154 => restore_right;
        8948232 => finish_restore;
        8948623 => explosion_window::set_power_bomb_blending;
        8948632 => explosion_window::power_bomb_afterglow;
        8949225 => [explosion_window::update_scaled(k, Phase::PowerBomb, game)];
        8949426 => [explosion_window::update_prescaled(k, Phase::PowerBomb, game)];
        8949983 => [explosion_window::update_scaled(k, Phase::PreFlash, game)];
        8950184 => [explosion_window::update_prescaled(k, Phase::PreFlash, game)];
        8954681 => explosion_window::set_crystal_flash_blending;
        8954717 => explosion_window::crystal_flash_afterglow;
        8955218 => [explosion_window::update_scaled(k, Phase::CrystalFlash, game)];
        8955459 => scrolling_sky::update_vertical_scroll; 8955507 => scrolling_sky::update_horizontal_scroll;
        8957362 => scrolling_sky::update_parallax; 8957370 => scrolling_sky::update_parallax;
        8957378 => scrolling_sky::advance_parallax;
        8958140 => [visual_runtime::advance_fireflea_lighting(game)];
        8958896 => liquid_effects::lava_surface;
        8959189 => liquid_effects::lava_background; 8963214 => liquid_effects::water_surface;
        8963465 => liquid_effects::water_background;
        8963556 => liquid_effects::water_wave;
        8963638 => liquid_effects::water_flat;
        8968609 => [visual_runtime::scroll_weather(k, Weather::Rain, game)];
        8968775 => [visual_runtime::scroll_weather(k, Weather::Spores, game)];
        8969014 => [visual_runtime::scroll_weather(k, Weather::Fog, game)];
        8969175 => [visual_runtime::advance_tourian_statue(k, TourianStatuePhase::Waiting, game)]; 8969251 => [visual_runtime::advance_tourian_statue(k, TourianStatuePhase::Shaking, game)];
        8969321 => [visual_runtime::advance_tourian_statue(k, TourianStatuePhase::Sinking, game)]; 8969402 => [visual_runtime::advance_tourian_statue(k, TourianStatuePhase::Finished, game)];
        8969539 => HdmaobjPreInstr_BombTorizoHazeColorMathBgColor;
        8969744 => [Runtime::new(game).begin_haze(k, 0x80)];
        8969749 => [Runtime::new(game).begin_haze(k, 0x20)];
        8969773 => [Runtime::new(game).advance_haze(k, HazePhase::FadingIn)];
        8969844 => [Runtime::new(game).advance_haze(k, HazePhase::FadedIn)];
        8969878 => [Runtime::new(game).advance_haze(k, HazePhase::FadingOut)];
        8970132 => [Runtime::new(game).select_draygon_layer(k)]; 8970278 => [Runtime::new(game).advance_suit_pickup(false)];
        8970332 => [Runtime::new(game).advance_suit_pickup(true)]; 8971337 => HdmaobjPreInstr_E449;
        8971623 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Phantoon)]; 8972220 => HdmaobjPreInstr_E7BC;
        fnHdmaobjPreInstr_MorphBallEyeBeamWidening => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Widening);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFull => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Full);
                }];
        fnHdmaobjPreInstr_MorphBallEyeBeamFading => [{
                    Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Fading);
                }];
        8973232 => [visual_runtime::advance_title_gradient(true, game)];
        8973266 => [visual_runtime::advance_title_gradient(false, game)];
        8973341 => [visual_runtime::advance_intro_crossfade(game)]; 8972070 => [()];
        8973494 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Intro)];
    }
}
