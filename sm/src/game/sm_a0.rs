use crate::game::combat_rules::LatchMode;
pub use crate::game::prelude::*;
use crate::game::{
    actors::{api as actors, scheduling as actor_scheduling},
    session_runtime::Runtime,
};

use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};

static kRoomShakes: [u16; 144] = [
    1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0,
    3, 3, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2, 3, 0, 3, 0,
    0, 3, 0, 3, 3, 3, 3, 3, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 2, 2, 2, 2,
    3, 0, 3, 0, 0, 3, 0, 3, 3, 3, 3, 3, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 2,
    0, 0, 2, 2, 0, 0, 3, 0, 0, 0, 0, 3, 0, 0, 3, 3,
];

macro_rules! adjust_room_shake_scroll {
    ($game:ident, $adjust:ident, $shake_index:ident) => {
        word!(
            $game.reg_BG1HOFS,
            $adjust(kRoomShakes[$shake_index as usize])
        );
        word!(
            $game.reg_BG1VOFS,
            $adjust(kRoomShakes[($shake_index + 1) as usize])
        );
        word!(
            $game.reg_BG2HOFS,
            $adjust(kRoomShakes[($shake_index + 2) as usize])
        );
        word!(
            $game.reg_BG2VOFS,
            $adjust(kRoomShakes[($shake_index + 3) as usize])
        );
    };
}

pub unsafe fn HandleRoomShaking(game: &mut Game) {
    {
        if word!(game.earthquake_timer) != 0
            && word!(game.time_is_frozen_flag) == 0
            && word!(game.earthquake_type) as i32 - 36i32 & 0x8000 != 0
        {
            let v0: i32 = 8 * word!(game.earthquake_type) as i32 >> 1;
            if (word!(game.earthquake_timer) & 2) != 0 {
                adjust_room_shake_scroll!(game, wrapping_sub, v0);
            } else {
                adjust_room_shake_scroll!(game, wrapping_add, v0);
            }
            word!(game.earthquake_timer -= 1);
            if word!(game.earthquake_type) as i32 - 18i32 & 0x8000 == 0 {
                actor_scheduling::shake_active_enemies(&mut Runtime::new(game));
            }
        }
        word!(game.frame_counter_every_frame += 1);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallEnemyGfxDrawHook(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10518604 => [()]; 10738444 => sm_a3::Reflec_Func_1; 10851138 => sm_a5::Draygon_Func_36;
        10920690 => sm_a6::Ridley_A2F2; 11042892 => [()]; 11055282 => sm_a8::NorfairLavaMan_Func_6;
        11062375 => sm_a8::WreckedShipRobot_Func_1; 11069550 => sm_a8::BlueBrinstarFaceBlock_Func_1;
        11110278 => [()]; 11110345 => sm_a9::MotherBrain_DrawBrainNeck_EnemyGfxDrawHook;
        11110365 => sm_a9::MotherBrainsBrain_GfxDrawHook; 11129754 => sm_a9::DeadTorizo_MainGfxHook;
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @external {
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => [actors::resume_ai(&mut Runtime::new(game))];
        10649605 | 10780677 => [actors::latch(&mut Runtime::new(game), LatchMode::Passive)];
        11042836 => [actors::latch(&mut Runtime::new(game), LatchMode::Active)];
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871 | 11108407 | 11173943 | 11698231 | 11763767 => [actors::power_bomb_reaction(&mut Runtime::new(game), actors::DeathHandling::Finalize)];
        10649676 => [{
            #[cfg(sm_verify_dll)]
            crate::game::actors::reference_population_animation::dispatch_noop(game);
        }];
        10656319 => sm_a2::ThinHoppingBlobs_Init;
        10656381 => sm_a2::ThinHoppingBlobs_Main;
        10664233 => [{
                    sm_a2::Rinka_Frozen(word!(game.cur_enemy_index), game);
                }];
        10664275 => [{
                    sm_a2::Rinka_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10669268 => sm_a2::MaridiaLargeSnail_Init;
        10676698 => [mechanisms::initialize(Kind::TimedShutter, game)];
        10676918 => [mechanisms::update(Kind::TimedShutter, game)];
        10677765 => [mechanisms::initialize(Kind::Platform, game)];
        10677778 => [mechanisms::initialize(Kind::VerticalShutter, game)];
        10677969 => [mechanisms::update(Kind::Platform, game)];
        10678429 => [mechanisms::react(Kind::Platform, Event::Touch, game)];
        10678434 => [mechanisms::react(Kind::Platform, Event::Shot, game)];
        10678442 => [mechanisms::react(Kind::VerticalShutter, Event::Shot, game)];
        10678454 => [mechanisms::react(Kind::Platform, Event::PowerBomb, game)];
        10678545 => [mechanisms::initialize(Kind::HorizontalShutter, game)];
        10678750 => [mechanisms::update(Kind::HorizontalShutter, game)];
        10679256 => [mechanisms::react(Kind::HorizontalShutter, Event::Touch, game)];
        10679310 => [mechanisms::react(Kind::HorizontalShutter, Event::Shot, game)];
        10679322 => [mechanisms::react(Kind::HorizontalShutter, Event::PowerBomb, game)];
        10715212 => [()]; 10716909 => sm_a3::Waver_Init;
        10717004 => sm_a3::Waver_Main; 10717536 => sm_a3::Metalee_Init;
        10717561 => sm_a3::Metalee_Main; 10717967 => sm_a3::Metalee_Shot;
        10718827 => [{
                    sm_a3::Fireflea_Touch(word!(game.cur_enemy_index), game);
                }];
        10719413 => sm_a3::MaridiaFish_Init; 10719531 => sm_a3::MaridiaFish_Main;
        10723079 => [()];
        10723661 => sm_a3::Roach_Init;
        10724048 => sm_a3::Roach_Main;
        10726351 => sm_a3::Sidehopper_Main;
        10733230 => sm_a3::Skree_Init; 10733255 => sm_a3::Skree_Main;
        10738715 => [()];
        10743874 => [{
                    sm_a3::Metroid_Powerbomb(word!(game.cur_enemy_index), game);
                }];
        10780748 => [()];
        10795344 => [()];
        10811067 => [sm_a4::CrocomireTongue_Main()];
        10846284 => [()];
        10851824 => sm_a5::Draygon_Shot;
        10864042 => [()];
        10864068 => [()];
        10874354 => [()];
        10911820 => [()];
        10913525 => sm_a6::Boulder_Init; 10913683 => sm_a6::Boulder_Main;
        10914607 => sm_a6::SpikeyPlatform_Init;
        10914733 => sm_a6::SpikeyPlatform_Main;
        10920181 => sm_a6::CeresRidley_Init; 10920584 => sm_a6::CeresRidley_Main;
        10924583 => sm_a6::Ridley_Main;
        10924682 => sm_a6::Ridley_Func_2; 10924695 => sm_a6::Ridley_Hurt;
        10942752 => [()];
        10977356 => [()]; 10982543 => [()];
        10982581 => [()];
        11001242 => [()];
        11001873 => [()];
        11042892 => [()];
        11048675 => sm_a8::Fune_Init;
        11048752 => sm_a8::Fune_Main;
        11051336 => sm_a8::YappingMaw_Init;
        11051537 => sm_a8::YappingMaw_Main; 11052953 => sm_a8::YappingMaw_Touch;
        11057014 => sm_a8::Beetom_Init;
        11057165 => sm_a8::Beetom_Main; 11058734 => sm_a8::Beetom_Touch;
        11062374 => [()];
        11065545 => sm_a8::MaridiaPuffer_Init;
        11065611 => sm_a8::MaridiaPuffer_Main; 11066132 => sm_a8::MaridiaPuffer_Shot;
        11068296 => sm_a8::WreckedShipOrbs_Init;
        11071880 => sm_a8::KiHunter_Init;
        11072020 => sm_a8::KiHunterWings_Init; 11072092 => sm_a8::KiHunter_Main;
        11072098 => sm_a8::KiHunterWings_Main;
        11108428 => [()];
        11111301 => [{
                    sm_a9::MotherBrainsTubesFalling_Main(word!(game.cur_enemy_index), game);
                }];
        11121927 => sm_a9::MotherBrainsBrain_Shot;
        11122117 => [()];
        11173964 => [()];
        11196359 => [()];
        11200475 => [()];
        11200476 => [()];
        11698252 => [()];
        11763788 => [()];
        11765518 => [sm_b3::UnusedSpinningTurtleEye_Main()];
        11767628 => sm_b3::BrinstarYellowPipeBug_Init;
        11767726 => sm_b3::BrinstarYellowPipeBug_Main; 11769219 => sm_b3::Botwoon_Init;
        11771926 => sm_b3::Botwoon_Shot;
        11791130 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyAi(ea, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyAi(ea, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyAi(ea, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyAi(ea, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyAi(ea, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyAi(ea, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyAi(ea, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyAi(ea, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyAi(ea, game)];
        0xb20000..=0xb2ffff => [sm_b2::DispatchEnemyAi(ea, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyAi(ea, game)];
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallEnemyPreInstr(ea: u32, game: &mut Game) => ea;
        setup { let k: u16 = word!(game.cur_enemy_index); }
        call (k, game); default { Unreachable(game); };
    {
        10518651 => [()]; 10649723 => [()];
        10652277 => [()];
        10657819 => [()]; 10659799 => [()];
        10676097 => [()]; 10730689 => [sm_a3::Bang_Func_7(k)];
        10735455 => [()];
        10739850 => [()]; 10741441 => [()];
        10998952 => [()];
        11047388 => [()];
        11055761 => [sm_a8::sub_A8B291(k)]; 11055765 => [sm_a8::NorfairLavaMan_Func_13(game)];
        11060584 => [()];
        11192670 => [()]; 11197610 => [()]; 11200422 => [()]; 11174011 => [()]; 11763835 => [()];
        11192146 => sm_aa::Torizo_Func_5;
        11192364 => sm_aa::Torizo_Func_7;
        11195889 => sm_aa::Torizo_D5F1;
        11200474 => [()];
        11197611 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyPreInstr(ea, k, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyPreInstr(ea, k, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyPreInstr(ea, k, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyPreInstr(ea, k, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyPreInstr(ea, k, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyPreInstr(ea, k, game)];
    }
}
pub fn DecrementSamusTimers(game: &mut Game) {
    {
        if word!(game.samus_invincibility_timer) != 0 {
            word!(game.samus_invincibility_timer -= 1);
        }
        if word!(game.samus_knockback_timer) != 0 {
            word!(game.samus_knockback_timer -= 1);
        }
        if word!(game.projectile_invincibility_timer) != 0 {
            word!(game.projectile_invincibility_timer -= 1);
        }
        word!(game.interactive_enemy_indexes[0] = -1i32 as u16);
        word!(game.active_enemy_indexes[0] = -1i32 as u16);
    }
}

crate::game::address_dispatch! {
    pub unsafe fn CallHitboxTouch(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10518563 => [actors::contact_reaction(&mut Runtime::new(game), actors::DeathHandling::Finalize)]; 10670984 => sm_a2::MaridiaLargeSnail_Func_12;
        10670988 => sm_a2::MaridiaLargeSnail_Touch; 10780707 => sm_a4::Enemy_NormalTouchAI_A4;
        10795325 => sm_a4::Crocomire_Func_92; 10795344 => [()]; 10846284 => [()];
        10851818 => sm_a5::Draygon_Touch; 10874348 => sm_a5::SporeSpawn_Touch;
        10936153 => sm_a6::sub_A6DF59; 10940479 => sm_a6::CeresSteam_Touch; 10977356 => [()];
        10982539 => sm_a7::Kraid_Touch_ArmFoot; 10982544 => sm_a7::KraidsArm_Touch;
        11001237 => sm_a7::Phantoon_Touch; 11122117 => [()];
        11122118 => sm_a9::MotherBrainsBrain_Touch; 11192695 => sm_aa::GoldTorizo_Touch;
        11700076 => sm_b2::WalkingSpacePirates_Touch;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallHitboxShot(ea: u32, j: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @forward [pub unsafe fn (game: &mut Game)];
    Enemy_NormalFrozenAI => actor_scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        10518573 => [actors::shot_reaction(&mut Runtime::new(game), actors::ShotHandling::Finalize)];
        10846278 => [actors::create_dud(&mut Runtime::new(game))];
        10649676 => [()];
        10671028 => sm_a2::MaridiaLargeSnail_Shot; 10780717 => sm_a4::Enemy_NormalShotAI_A4;
        10795345 => sm_a4::Crocomire_Func_93; 10795368 => sm_a4::Crocomire_Func_94;
        10795525 => sm_a4::Crocomire_Func_95; 10795700 => sm_a4::Crocomire_Func_1;
        10846253 => sm_a5::Enemy_NormalShotAI_A5; 10846284 => [()];
        10851824 => sm_a5::Draygon_Shot; 10874202 => sm_a5::SporeSpawn_Shot; 10911820 => [()];
        10936202 => sm_a6::Ridley_Shot; 10977356 => [()]; 10982581 => [()];
        10982582 => [sm_a7::Kraid_Arm_Shot(j, game)]; 11001243 => sm_a7::Phantoon_Shot;
        11121923 => sm_a9::MotherBrainsBody_Shot; 11121927 => sm_a9::MotherBrainsBrain_Shot;
        11192700 => sm_aa::Torizo_Shot; 11192769 => [()]; 11192770 => sm_aa::Torizo_Func_8;
        11700089 => sm_b2::WalkingSpacePirates_Shot; 11700168 => sm_b2::WalkingSpacePirates_87C8;
        11700286 => sm_b2::WalkingSpacePirates_883E;
    }
}

pub fn SignExtend8(a: u16) -> u16 {
    // Preserve the input high byte when bit 7 is clear.
    if a & 0x80 != 0 { a | 0xff00 } else { a }
}
pub fn Abs16(a: u16) -> u16 {
    (a as i16).wrapping_abs() as u16
}
fn SineMult8bitInner(amplitude: u16, phase: u16, game: &mut Game) -> u32 {
    crate::game::actors::queries::sine_product(&mut Runtime::new(game), amplitude, phase)
}

pub fn CosineMult8bit(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (SineMult8bitInner(amplitude, phase.wrapping_add(64) as u8 as u16, game) >> 16) as u16
}

pub fn SineMult8bit(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (SineMult8bitInner(amplitude, phase.wrapping_add(0x80) as u8 as u16, game) >> 16) as u16
}

crate::forward_calls! { pub fn (phase: u16, amplitude: u16, game: &mut Game) -> u32;
    CosineMult8bitFull => SineMult8bitInner(amplitude, phase.wrapping_add(64) as u8 as u16, game);
    SineMult8bitFull => SineMult8bitInner(amplitude, phase.wrapping_add(0x80) as u8 as u16, game);
}
pub fn ConvertAngleToXy(phase: u16, amplitude: u16, game: &mut Game) -> Point32 {
    let x_index = usize::from(phase.wrapping_add(0x40) & 0x7f);
    let x_sample = game.read_rom_word(0xa0b7ee, x_index * 2);
    let x = u32::from(x_sample) * u32::from(amplitude);
    let y_index = usize::from(phase.wrapping_add(0x80) & 0x7f);
    let y_sample = game.read_rom_word(0xa0b7ee, y_index * 2);
    let y = u32::from(y_sample) * u32::from(amplitude);
    Point32 { x, y }
}
pub fn EnemyFunc_Divide(a: u32, b: u32) -> u32 {
    if a != 0 { b.wrapping_div(a) } else { 0 }
}
fn corrected_samus_previous_position(position: u16, previous: u16) -> Option<u16> {
    let delta = position.wrapping_sub(previous);
    let distance = Abs16(delta);
    if distance.wrapping_sub(12) & 0x8000 != 0 {
        return None;
    }
    Some(if delta & 0x8000 == 0 {
        position.wrapping_add(12)
    } else {
        position.wrapping_sub(12)
    })
}

pub fn EnemyFunc_B7A1(game: &mut Game) {
    if let Some(previous) =
        corrected_samus_previous_position(game.samus_y_pos(), game.samus_prev_y_pos())
    {
        game.set_samus_prev_y_pos(previous);
    }
    if let Some(previous) =
        corrected_samus_previous_position(game.samus_x_pos(), game.samus_prev_x_pos())
    {
        game.set_samus_prev_x_pos(previous);
    }
}
