//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
pub use crate::game::prelude::*;
use crate::game::sm_80::{
    NextRandom, QueueMusic_DelayedY, QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6,
};
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_8b::Samus_CalcPos_Mode7;
use crate::game::sm_90::{
    false_0, fnHandleAutoJumpHack, fnSamus_InputHandler_E91D, kLiquidPhysicsType_Water,
    liquid_at_height, true_0, LiquidAtHeight, Samus_GetBottom_R18, Samus_GetTop_R20,
};
use crate::game::sm_91::Samus_CancelSpeedBoost;
use crate::types::FuncXY_V;
const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
const RomPtr_91: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x91>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
}

pub unsafe fn Samus_Animate(game: &mut Game) {
    let fx_type = u16::from_le_bytes([game.g_ram[0x196e], game.g_ram[0x196f]]);
    match (fx_type & 0xf) >> 1 {
        0 => Samus_Animate_NoFx(game),
        1 => Samus_Animate_LavaFx(game),
        2 => Samus_Animate_AcidFx(game),
        3 => Samus_Animate_WaterFx(game),
        // FX kinds 4..=7 leave the animation FX state unchanged.
        _ => {}
    }
    let pose = game.samus_pose();
    let mut timer = u16::from_le_bytes([game.g_ram[0xa94], game.g_ram[0xa95]]);
    if (pose == kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun as u16
        || pose == kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun as u16)
        && u16::from_le_bytes([game.g_ram[0xb36], game.g_ram[0xb37]]) != 2
        && game.samus_anim_frame() == 1
        && timer == 1
    {
        timer = 4;
    }
    timer = timer.wrapping_sub(1);
    game.set_samus_anim_frame_timer(timer);
    if timer == 0 || timer & 0x8000 != 0 {
        game.set_samus_anim_frame(game.samus_anim_frame().wrapping_add(1));
        Samus_HandleAnimDelay(game);
    }
}
pub unsafe fn Samus_Animate_NoFx(game: &mut Game) {
    let bottom = Samus_GetBottom_R18(game);
    let speed_divisor = u16::from_le_bytes([game.g_ram[0xa66], game.g_ram[0xa67]]);
    game.g_ram[0xa9c..0xa9e].copy_from_slice(&speed_divisor.to_le_bytes());

    let liquid_physics = u16::from_le_bytes([game.g_ram[0xad2], game.g_ram[0xad3]]);
    if liquid_physics == 0 {
        return;
    }
    game.g_ram[0xad2..0xad4].copy_from_slice(&0u16.to_le_bytes());
    if liquid_physics & 1 == 0 {
        return;
    }

    QueueSfx2_Max6(0xe, game);
    let suit_palette = u16::from_le_bytes([game.g_ram[0xa74], game.g_ram[0xa75]]);
    if suit_palette & 4 == 0 && matches!(game.samus_movement_type(), 3 | 20) {
        QueueSfx1_Max6(0x30, game);
    }
    Samus_SpawnWaterSplash(bottom, game);
}
pub unsafe fn Samus_Animate_WaterFx(game: &mut Game) {
    static kSamusPhys_AnimDelayInWater: u16 = 3;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (word!(game.fx_y_pos) & 0x8000) == 0
        && word!(game.fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
        && (word!(game.fx_liquid_options) & 4) == 0
    {
        word!(game.samus_anim_frame_buffer = kSamusPhys_AnimDelayInWater);
        if word!(game.liquid_physics_type) as i32 == kLiquidPhysicsType_Water as i32 {
            Samus_SpawnAirBubbles(game);
        } else {
            word!(game.liquid_physics_type = kLiquidPhysicsType_Water as u16);
            QueueSfx2_Max6(0xd, game);
            Samus_SpawnWaterSplash(r18, game);
        }
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_SpawnWaterSplash(r18: u16, game: &mut Game) {
    static kWaterSplashTypeTable: [u8; 28] = [
        1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    if kWaterSplashTypeTable[*ram_ptr!(game; 0xa1f) as usize] != 0 {
        word! { game, 256;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
        }
        word! { game, 3; atmospheric_gfx_anim_timer[0], atmospheric_gfx_anim_timer[1] }
        word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_add(4));
        word!(game.atmospheric_gfx_x_pos[1] = (word!(game.samus_x_pos)).wrapping_sub(3));
        word!(game.atmospheric_gfx_y_pos[0] = (r18).wrapping_sub(4));
        word!(game.atmospheric_gfx_y_pos[1] = (r18).wrapping_sub(4));
    } else {
        word! {game; atmospheric_gfx_frame_and_type[0] = 768; atmospheric_gfx_anim_timer[0] = 2;
        atmospheric_gfx_x_pos[0] = word!(game.samus_x_pos);
        atmospheric_gfx_y_pos[0] = word!(game.fx_y_pos); }
    }
    Samus_SpawnAirBubbles(game);
}
pub unsafe fn Samus_SpawnAirBubbles(game: &mut Game) {
    let r20: u16 = Samus_GetTop_R20(game);
    if ((r20 as i32 - 24i32 - word!(game.fx_y_pos) as i32) as i16) >= 0
        && (word!(game.nmi_frame_counter_word) & 0x7f) == 0
        && word!(game.atmospheric_gfx_frame_and_type[2]) == 0
    {
        word!(game.atmospheric_gfx_frame_and_type[2] = 1280);
        word!(game.atmospheric_gfx_anim_timer[2] = 3);
        word!(game.atmospheric_gfx_x_pos[2] = word!(game.samus_x_pos));
        word!(
            game.atmospheric_gfx_y_pos[2] =
                (word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 + 6) as u16
        );
        QueueSfx2_Max6(
            (if NextRandom(game) & 1 != 0 { 15 } else { 17 }) as u16,
            game,
        );
    }
    if word!(game.samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
        || word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        || (word!(game.equipped_items) & 0x20) != 0
    {
        word!(game.samus_anim_frame_buffer = 0);
    }
}
pub unsafe fn Samus_Animate_LavaFx(game: &mut Game) {
    static kSamusPhys_LavaDamagePerFrame: u16 = 0;
    static kSamusPhys_LavaSubdamagePerFrame: u16 = 0x8000;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (word!(game.lava_acid_y_pos) & 0x8000) == 0
        && word!(game.lava_acid_y_pos) as i32 - r18 as i32 & 0x8000 != 0
    {
        if word!(game.speed_boost_counter) != 0 {
            Samus_CancelSpeedBoost(game);
            word! { game, 0; samus_x_extra_run_speed, samus_x_extra_run_subspeed }
        }
        if (word!(game.equipped_items) & 0x20) != 0 {
            word! {game; samus_anim_frame_buffer = 0; liquid_physics_type = 2; }
        } else {
            if (word!(game.game_time_frames) & 7) == 0
                && word!(game.samus_health) as i32 - 71i32 & 0x8000 == 0
            {
                QueueSfx3_Max3(0x2d, game);
            }
            words!(
                game.samus_periodic_damage,
                samus_periodic_subdamage += words!((
                    kSamusPhys_LavaDamagePerFrame,
                    kSamusPhys_LavaSubdamagePerFrame
                ))
            );
            Samus_Animate_SubmergedLavaAcid(game);
        }
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_Animate_AcidFx(game: &mut Game) {
    static kSamusPhys_AcidSubdamagePerFrame: u16 = 0x8000;
    static kSamusPhys_AcidDamagePerFrame: u16 = 1;
    let r18: u16 = Samus_GetBottom_R18(game);
    if (word!(game.lava_acid_y_pos) & 0x8000) == 0
        && word!(game.lava_acid_y_pos) as i32 - r18 as i32 & 0x8000 != 0
    {
        if (word!(game.game_time_frames) & 7) == 0
            && word!(game.samus_health) as i32 - 71i32 & 0x8000 == 0
        {
            QueueSfx3_Max3(0x2d, game);
        }
        words!(
            game.samus_periodic_damage,
            samus_periodic_subdamage += words!((
                kSamusPhys_AcidDamagePerFrame,
                kSamusPhys_AcidSubdamagePerFrame
            ))
        );
        Samus_Animate_SubmergedLavaAcid(game);
    } else {
        Samus_Animate_NoFx(game);
    };
}
pub unsafe fn Samus_Animate_SubmergedLavaAcid(game: &mut Game) {
    static kSamusPhys_AnimDelayInAcid: u16 = 2;
    let r20: u16 = Samus_GetTop_R20(game);
    word! {game; samus_anim_frame_buffer = kSamusPhys_AnimDelayInAcid; liquid_physics_type = 2; }
    if ((r20) as i16).wrapping_sub((word!(game.lava_acid_y_pos)) as i16) < 0
        && (word!(game.atmospheric_gfx_frame_and_type[0]) & 0x400) == 0
    {
        word! { game, 1024;
            atmospheric_gfx_frame_and_type[0], atmospheric_gfx_frame_and_type[1],
            atmospheric_gfx_frame_and_type[2], atmospheric_gfx_frame_and_type[3],
        }
        word! { game, 3; atmospheric_gfx_anim_timer[0], atmospheric_gfx_anim_timer[3] }
        word!(game.atmospheric_gfx_anim_timer[1] = -32766i32 as u16);
        word!(game.atmospheric_gfx_anim_timer[2] = -32766i32 as u16);
        word!(game.atmospheric_gfx_y_pos[0] = word!(game.lava_acid_y_pos));
        word!(game.atmospheric_gfx_y_pos[1] = word!(game.lava_acid_y_pos));
        word!(game.atmospheric_gfx_y_pos[2] = word!(game.lava_acid_y_pos));
        word!(game.atmospheric_gfx_y_pos[3] = word!(game.lava_acid_y_pos));
        word!(game.atmospheric_gfx_x_pos[0] = (word!(game.samus_x_pos)).wrapping_add(6));
        word!(game.atmospheric_gfx_x_pos[1] = word!(game.samus_x_pos));
        word!(game.atmospheric_gfx_x_pos[2] = word!(game.samus_x_pos));
        word!(game.atmospheric_gfx_x_pos[3] = (word!(game.samus_x_pos)).wrapping_sub(6));
        if (word!(game.game_time_frames) & 1) == 0 {
            QueueSfx2_Max6(0x10, game);
        }
    }
    if word!(game.samus_pose) as i32 == kPose_00_FaceF_Powersuit as i32
        || word!(game.samus_pose) as i32 == kPose_9B_FaceF_VariaGravitySuit as i32
        || (word!(game.equipped_items) & 0x20) != 0
    {
        word!(game.samus_anim_frame_buffer = 0);
    }
}
fn Samus_AnimDelayFunc_6_GotoStartIfLittleHealth(game: &mut Game) -> bool {
    let frame = if game.samus_health().wrapping_sub(30) & 0x8000 != 0 {
        game.samus_anim_frame().wrapping_add(1)
    } else {
        0
    };
    game.set_samus_anim_frame(frame);
    true
}
unsafe fn Samus_AnimDelayFunc_7(jp: *const u8, game: &mut Game) -> bool {
    word!(game.samus_movement_handler = ((0x9094cbi32) as u16));
    word!(game.samus_anim_frame += 1);
    true_0 != 0
}
fn Samus_AnimDelayFunc_13_TransToPose(pose: u8, game: &mut Game) -> bool {
    game.set_samus_new_pose_transitional(u16::from(pose));
    game.set_samus_hurt_switch_index(3);
    false
}
unsafe fn Samus_AnimDelayFunc_8_AutoJumpHack(instruction: *const u8, game: &mut Game) -> bool {
    let input_handler = u16::from_le_bytes([game.g_ram[0xa60], game.g_ram[0xa61]]);
    if input_handler == fnSamus_InputHandler_E91D as u16 {
        return Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game);
    }
    let new_pose = u16::from_le_bytes([game.g_ram[0xa28], game.g_ram[0xa29]]);
    if new_pose != kPose_4B_FaceR_Jumptrans as u16
        && new_pose != kPose_4C_FaceL_Jumptrans as u16
        && new_pose != kPose_19_FaceR_SpinJump as u16
        && new_pose != kPose_1A_FaceL_SpinJump as u16
    {
        game.set_samus_input_handler(fnHandleAutoJumpHack as u16);
        return Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game);
    }
    false
}
unsafe fn Samus_AnimDelayFunc_9_TransToPose(instruction: *const u8, game: &mut Game) -> bool {
    let item_mask = u16::from_le_bytes([*instruction.add(1), *instruction.add(2)]);
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    let has_item = item_mask & equipped_items != 0;
    let y_speed = u16::from_le_bytes([game.g_ram[0xb2e], game.g_ram[0xb2f]]);
    let moving_vertically =
        y_speed != 0 || u16::from_le_bytes([game.g_ram[0xb2c], game.g_ram[0xb2d]]) != 0;
    let pose_offset = match (has_item, moving_vertically) {
        (false, false) => 3,
        (false, true) => 4,
        (true, false) => 5,
        (true, true) => 6,
    };
    game.set_samus_new_pose_transitional(u16::from(*instruction.add(pose_offset)));
    game.set_samus_hurt_switch_index(3);
    false
}
unsafe fn UNUSED_Samus_AnimDelayFunc_10(jp: *const u8, game: &mut Game) -> bool {
    if word!(game.samus_y_speed) != 0 || word!(game.samus_y_subspeed) != 0 {
        word!(game.samus_new_pose_transitional = *jp.offset(2) as u16);
    } else {
        word!(game.samus_new_pose_transitional = *jp.offset(1) as u16);
    }
    word!(game.samus_hurt_switch_index = 3);
    false_0 != 0
}
unsafe fn Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump(game: &mut Game) -> bool {
    let gravity_suit = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]) & 0x20 != 0;
    let liquid_limits_animation = if gravity_suit {
        false
    } else {
        let top = Samus_GetTop_R20(game);
        liquid_at_height(game, top) != LiquidAtHeight::None
    };
    let (sound, frame_advance) = if liquid_limits_animation {
        (0x31, 1)
    } else {
        let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
        if equipped_items & 8 != 0 {
            (0x33, 21)
        } else if equipped_items & 0x200 != 0 {
            (0x3e, 11)
        } else {
            (0x31, 1)
        }
    };
    QueueSfx1_Max6(sound, game);
    game.set_samus_anim_frame(game.samus_anim_frame().wrapping_add(frame_advance));
    true
}
unsafe fn Samus_AnimDelayFunc_12_TransToPose(jp: *const u8, game: &mut Game) -> bool {
    if (*(jp.offset(1) as *mut u16) & word!(game.equipped_items)) != 0 {
        word!(game.samus_new_pose_transitional = *jp.offset(4) as u16);
    } else {
        word!(game.samus_new_pose_transitional = *jp.offset(3) as u16);
    }
    word!(game.samus_hurt_switch_index = 3);
    false_0 != 0
}
fn Samus_AnimDelayFunc_14_Goto(frames_back: u8, game: &mut Game) -> bool {
    game.set_samus_anim_frame(game.samus_anim_frame().wrapping_sub(u16::from(frames_back)));
    true
}
fn Samus_AnimDelayFunc_15_GotoStart(game: &mut Game) -> bool {
    game.set_samus_anim_frame(0);
    true
}
unsafe fn Samus_HandleSpeedBoosterAnimDelay(jp: *const u8, game: &mut Game) -> u8 {
    let has_momentum = u16::from_le_bytes([game.g_ram[0xb3c], game.g_ram[0xb3d]]);
    let run_button = u16::from_le_bytes([game.g_ram[0x9b6], game.g_ram[0x9b7]]);
    if has_momentum == 0
        || run_button & game.joypad1_lastkeys() == 0
        || game.samus_movement_type() != 1
    {
        return *jp;
    }
    let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
    if equipped_items & 0x2000 == 0 {
        game.set_samus_anim_frame(0);
        let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
        let animation_list = game.read_rom_word(0x91b5d1, 0);
        let delay = *RomPtr_91(animation_list, game);
        game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
        return 0;
    }
    let counter = u16::from_le_bytes([game.g_ram[0xb3e], game.g_ram[0xb3f]]).wrapping_sub(1);
    game.g_ram[0xb3e..0xb40].copy_from_slice(&counter.to_le_bytes());
    if counter as u8 != 0 {
        return *jp;
    }
    let mut staged_counter = counter;
    if counter & 0x400 == 0 {
        staged_counter = counter.wrapping_add(0x100);
        game.g_ram[0xb3e..0xb40].copy_from_slice(&staged_counter.to_le_bytes());
        if staged_counter & 0x400 != 0 {
            game.g_ram[0xb40..0xb42].copy_from_slice(&1u16.to_le_bytes());
            QueueSfx3_Max6(3, game);
        }
    }
    // The table stage is captured before sound; the counter is read again afterward.
    let stage_offset = usize::from(staged_counter >> 8) * 2;
    let stage_duration = game.read_rom_word(0x91b61f, stage_offset);
    let live_counter = u16::from_le_bytes([game.g_ram[0xb3e], game.g_ram[0xb3f]]);
    let reloaded_counter = stage_duration | (live_counter & 0xff00);
    game.g_ram[0xb3e..0xb40].copy_from_slice(&reloaded_counter.to_le_bytes());
    game.set_samus_anim_frame(0);
    let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
    let animation_list = game.read_rom_word(0x91b5de, stage_offset);
    let delay = *RomPtr_91(animation_list, game);
    game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
    0
}
unsafe fn Samus_HandleAnimDelay(game: &mut Game) {
    let animation_list = game.read_rom_word(0x91b010, usize::from(game.samus_pose()) * 2);
    let mut delays = RomPtr_91(animation_list, game);
    if *delays.add(usize::from(game.samus_anim_frame())) & 0x80 != 0 {
        let command = Samus_HandleSpeedBoosterAnimDelay(
            delays.add(usize::from(game.samus_anim_frame())),
            game,
        );
        // Keep the captured list and reread the frame after each callback.
        let instruction = delays.add(usize::from(game.samus_anim_frame()));
        let update_timer = match command & 0xf {
            6 => Samus_AnimDelayFunc_6_GotoStartIfLittleHealth(game),
            7 => Samus_AnimDelayFunc_7(instruction, game),
            8 => Samus_AnimDelayFunc_8_AutoJumpHack(instruction, game),
            9 => Samus_AnimDelayFunc_9_TransToPose(instruction, game),
            10 => UNUSED_Samus_AnimDelayFunc_10(instruction, game),
            11 => Samus_AnimDelayFunc_11_SelectDelaySequenceWalljump(game),
            12 => Samus_AnimDelayFunc_12_TransToPose(instruction, game),
            13 => Samus_AnimDelayFunc_13_TransToPose(*instruction.add(1), game),
            14 => Samus_AnimDelayFunc_14_Goto(*instruction.add(1), game),
            15 => Samus_AnimDelayFunc_15_GotoStart(game),
            // Commands 0..=5 retain the timer selected by the speed helper.
            _ => false,
        };
        if !update_timer {
            return;
        }
    } else {
        let has_momentum = u16::from_le_bytes([game.g_ram[0xb3c], game.g_ram[0xb3d]]);
        if has_momentum != 0 && game.samus_movement_type() == 1 {
            let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
            let animation_list = if equipped_items & 0x2000 != 0 {
                let speed_boost_stage = usize::from(game.g_ram[0xb3f]);
                game.read_rom_word(0x91b5de, speed_boost_stage * 2)
            } else {
                game.read_rom_word(0x91b5d1, 0)
            };
            delays = RomPtr_91(animation_list, game);
        }
    }
    let frame_buffer = u16::from_le_bytes([game.g_ram[0xa9c], game.g_ram[0xa9d]]);
    let delay = *delays.add(usize::from(game.samus_anim_frame()));
    game.set_samus_anim_frame_timer(frame_buffer.wrapping_add(u16::from(delay)));
}

unsafe fn bottom_is_visible(game: &mut Game) -> u8 {
    let pose = game.samus_pose();
    let frame = game.samus_anim_frame();
    let visible = match game.samus_movement_type() {
        0 => {
            if pose == kPose_00_FaceF_Powersuit as u16 {
                let next = word!(game.oam_next_ptr);
                let entry = gOamEnt(word!(game.oam_next_ptr) as i32, game);
                *(&raw mut (*entry).xcoord as *mut u16) =
                    (word!(game.samus_x_pos) as i32 - 7 - word!(game.layer1_x_pos) as i32) as u16;
                *(&raw mut (*entry).ycoord as *mut u16) =
                    (word!(game.samus_y_pos) as i32 - 17 - word!(game.layer1_y_pos) as i32) as u16;
                *(&raw mut (*entry).charnum as *mut u16) = 14369;
                word!(game.oam_next_ptr = next.wrapping_add(4));
            }
            true
        }
        1 | 2 | 5 | 6 | 11 | 12 | 14 | 16 | 21 | 22 | 23 | 24 | 26 => true,
        3 => {
            matches!(
                SamusPose::from(pose),
                kPose_81_FaceR_Screwattack
                    | kPose_82_FaceL_Screwattack
                    | kPose_1B_FaceR_SpaceJump
                    | kPose_1C_FaceL_SpaceJump
            ) || frame == 0
                || frame.wrapping_sub(11) & 0x8000 == 0
        }
        4 | 7 | 8 | 9 | 17 | 18 | 19 => false,
        10 => {
            !matches!(
                SamusPose::from(pose),
                kPose_D7_FaceR_CrystalFlashEnd | kPose_D8_FaceL_CrystalFlashEnd
            ) || frame.wrapping_sub(3) & 0x8000 == 0
        }
        // The unused handler's OR-composed pose constants are 0x65 and 0x66.
        13 => !matches!(pose, 0x65 | 0x66) || frame.wrapping_sub(1) & 0x8000 != 0,
        15 => {
            if pose.wrapping_sub(kPose_F1_FaceR_CrouchTrans_AimU as u16) & 0x8000 == 0 {
                true
            } else if pose.wrapping_sub(219) & 0x8000 == 0 {
                frame
                    == if pose.wrapping_sub(221) & 0x8000 != 0 {
                        0
                    } else {
                        2
                    }
            } else {
                matches!(
                    SamusPose::from(pose),
                    kPose_35_FaceR_CrouchTrans
                        | kPose_36_FaceL_CrouchTrans
                        | kPose_3B_FaceR_StandTrans
                        | kPose_3C_FaceL_StandTrans
                )
            }
        }
        20 => !(3..13).contains(&frame),
        25 => !(2..9).contains(&frame),
        27 => {
            if pose.wrapping_sub(kPose_CF_FaceR_Ranintowall_AimUR as u16) & 0x8000 != 0 {
                !matches!(
                    SamusPose::from(pose),
                    kPose_CB_FaceR_Shinespark_Vert | kPose_CC_FaceL_Shinespark_Vert
                )
            } else {
                !matches!(
                    SamusPose::from(pose),
                    kPose_E8_FaceR_Drained_CrouchFalling | kPose_E9_FaceL_Drained_CrouchFalling
                ) || frame.wrapping_sub(2) & 0x8000 == 0
            }
        }
        _ => panic!("invalid Samus movement type"),
    };
    if !visible {
        word!(game.samus_bottom_half_spritemap_index = 0);
    }
    u8::from(visible)
}

pub unsafe fn Samus_Draw(game: &mut Game) {
    let knockback_timer = u16::from_le_bytes([game.g_ram[0x18aa], game.g_ram[0x18ab]]);
    let invincibility_timer = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
    let shine_timer = u16::from_le_bytes([game.g_ram[0xa68], game.g_ram[0xa69]]);
    let nmi_frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
    if knockback_timer != 0 || invincibility_timer == 0 || shine_timer != 0 || nmi_frame & 1 == 0 {
        let doubled_pose = game.samus_pose().wrapping_mul(2);
        let frame = game.samus_anim_frame();
        let top_base = game.read_rom_word(0x929263, usize::from(game.samus_pose()) * 2);
        let top_spritemap = frame.wrapping_add(top_base);
        game.g_ram[0xac8..0xaca].copy_from_slice(&top_spritemap.to_le_bytes());
        let position = Samus_CalcSpritemapPos(game.samus_pose().wrapping_mul(2), game);
        DrawSamusSpritemap(top_spritemap, position.k, position.j, game);
        if bottom_is_visible(game) & 1 != 0 {
            // The lower table keeps the captured, word-wrapped pose; frame and position stay live.
            let frame = game.samus_anim_frame();
            let bottom_base = game.read_rom_word(0x92945d, usize::from(doubled_pose >> 1) * 2);
            let bottom_spritemap = frame.wrapping_add(bottom_base);
            game.g_ram[0xaca..0xacc].copy_from_slice(&bottom_spritemap.to_le_bytes());
            let x_pos = u16::from_le_bytes([game.g_ram[0xb04], game.g_ram[0xb05]]);
            let y_pos = u16::from_le_bytes([game.g_ram[0xb06], game.g_ram[0xb07]]);
            DrawSamusSpritemap(bottom_spritemap, x_pos, y_pos, game);
        }
    }
    SetSamusTilesDefsForCurAnim(game);
}

pub unsafe fn Samus_DrawEchoes(game: &mut Game) {
    if (word!(game.speed_echoes_index) & 0x8000) == 0 {
        if (word!(game.speed_boost_counter) & 0xff00) == 1024 {
            if word!(game.speed_echo_xpos[1]) != 0 {
                Samus_DrawEcho(2, game);
            }
            if word!(game.speed_echo_xpos[0]) != 0 {
                Samus_DrawEcho(0, game);
            }
        }
        return;
    }
    for i in (0i32..=2).rev().step_by(2) {
        let v1: i32 = i >> 1;
        if (word!(game.speed_echo_xpos[v1]) != 0) {
            let v2: u16 = word!(game.speed_echo_ypos[v1]);
            let mut v3: u16 = 0;
            if v2 != word!(game.samus_y_pos) {
                if ((v2) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
                    v3 = (v2).wrapping_add(2);
                } else {
                    v3 = (v2).wrapping_sub(2);
                }
                word!(game.speed_echo_ypos[v1] = v3);
            }
            let mut skip_echo: bool = if (word!(game.speed_echo_xspeed[v1]) & 0x8000) != 0 {
                let v5: u16 = (word!(game.speed_echo_xspeed[v1]))
                    .wrapping_add(word!(game.speed_echo_xpos[v1]));
                word!(game.speed_echo_xpos[v1] = v5);
                (((v5) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0)
                    && ({
                        word!(game.speed_echo_xpos[v1] = 0);
                        true
                    })
            } else {
                let v4: u16 = (word!(game.speed_echo_xspeed[v1]))
                    .wrapping_add(word!(game.speed_echo_xpos[v1]));
                word!(game.speed_echo_xpos[v1] = v4);
                (((v4) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0)
                    && ({
                        word!(game.speed_echo_xpos[v1] = 0);
                        true
                    })
            };
            if !(skip_echo) {
                Samus_DrawEcho(i as u16, game);
            }
        }
    }
    if word!(game.speed_echo_xpos[1]) == 0 && word!(game.speed_echo_xpos[0]) == 0 {
        word!(game.speed_echoes_index = 0);
    }
}
pub unsafe fn Samus_DrawEcho(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    let v2: i16 = (word!(game.speed_echo_ypos[v1]) as i32
        - (*(rom_ptr!(game, SamusPoseParams; 0x91b629)).offset(word!(game.samus_pose) as isize))
            .y_offset_to_gfx as i32
        - word!(game.layer1_y_pos) as i32) as i16;
    if v2 >= 0 && v2 as i32 - 248i32 & 0x8000 != 0 {
        DrawSamusSpritemap(
            word!(game.samus_top_half_spritemap_index),
            (word!(game.speed_echo_xpos[v1])).wrapping_sub(word!(game.layer1_x_pos)),
            v2 as u16,
            game,
        );
        if word!(game.samus_bottom_half_spritemap_index) != 0 {
            DrawSamusSpritemap(
                word!(game.samus_bottom_half_spritemap_index),
                (word!(game.speed_echo_xpos[v1])).wrapping_sub(word!(game.layer1_x_pos)),
                v2 as u16,
                game,
            );
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    if (word!(game.nmi_frame_counter_word) & 1) != 0 {
        let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0x929263,
            word!(game.samus_pose) as isize
        ));
        let v2: i16 = (word!(game.speed_echo_ypos[(k >> 1)]) as i32
            - (*(rom_ptr!(game, SamusPoseParams; 0x91b629)).offset(word!(game.samus_pose) as isize))
                .y_offset_to_gfx as i32
            - word!(game.layer1_y_pos) as i32) as i16;
        if v2 >= 0 && v2 as i32 - 248i32 & 0x8000 != 0 {
            DrawSamusSpritemap(
                a,
                (word!(game.speed_echo_xpos[(k >> 1)])).wrapping_sub(word!(game.layer1_x_pos)),
                v2 as u16,
                game,
            );
            if bottom_is_visible(game) & 1 != 0 {
                let v5: u16 = (word!(game.samus_anim_frame)).wrapping_add(
                    crate::rtl::rom_table_word!(game, 0x92945d, word!(game.samus_pose) as isize),
                );
                DrawSamusSpritemap(
                    v5,
                    (word!(game.speed_echo_xpos[(k >> 1)])).wrapping_sub(word!(game.layer1_x_pos)),
                    v2 as u16,
                    game,
                );
            }
        }
    }
}
pub unsafe fn Samus_DrawShinesparkCrashEchoProjectiles(game: &mut Game) {
    if (word!(game.nmi_frame_counter_word) & 1) != 0 {
        if word!(game.speed_echo_xspeed[3]) != 0 {
            Samus_DrawEcho(6, game);
        }
        if word!(game.speed_echo_xspeed[2]) != 0 {
            Samus_DrawEcho(4, game);
        }
    }
}
pub unsafe fn Samus_DrawStartingDeathAnim(game: &mut Game) {
    let v0: bool = (word!(game.samus_anim_frame_timer -= 1, reread) & 0x8000) != 0;
    if word!(game.samus_anim_frame_timer) == 0 || v0 {
        word!(game.samus_anim_frame += 1);
        Samus_HandleAnimDelay(game);
    }
    Samus_DrawDuringDeathAnim(game);
}
pub unsafe fn Samus_DrawDuringDeathAnim(game: &mut Game) {
    let v1: u16 = (2u16).wrapping_mul(word!(game.samus_pose));
    let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0x929263,
        word!(game.samus_pose) as isize
    ));
    Samus_CalcSpritemapPos((2u16).wrapping_mul(word!(game.samus_pose)), game);
    DrawSamusSpritemap(
        a,
        (word!(game.layer1_x_pos)).wrapping_add(word!(game.samus_spritemap_x_pos)),
        (word!(game.layer1_y_pos)).wrapping_add(word!(game.samus_spritemap_y_pos)),
        game,
    );
    let R36: u16 = v1;
    if bottom_is_visible(game) & 1 != 0 {
        DrawSamusSpritemap(
            (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0x92945d,
                (R36 >> 1) as isize
            )),
            (word!(game.layer1_x_pos)).wrapping_add(word!(game.samus_spritemap_x_pos)),
            (word!(game.layer1_y_pos)).wrapping_add(word!(game.samus_spritemap_y_pos)),
            game,
        );
    }
    SetSamusTilesDefsForCurAnim(game);
}
pub unsafe fn Samus_DrawWhenNotAnimatingOrDying(game: &mut Game) {
    let mut v0: PairU16 = PairU16 { k: 0, j: 0 };
    let v2: u16 = (2u16).wrapping_mul(word!(game.samus_pose));
    let a: u16 = (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0x929263,
        word!(game.samus_pose) as isize
    ));
    v0 = Samus_CalcSpritemapPos((2u16).wrapping_mul(word!(game.samus_pose)), game);
    DrawSamusSpritemap(a, v0.k, v0.j, game);
    let R36: u16 = v2;
    if bottom_is_visible(game) & 1 != 0 {
        DrawSamusSpritemap(
            (word!(game.samus_anim_frame)).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0x92945d,
                (R36 >> 1) as isize
            )),
            word!(game.samus_spritemap_x_pos),
            word!(game.samus_spritemap_y_pos),
            game,
        );
    }
    SetSamusTilesDefsForCurAnim(game);
}
static kAtmosphericTypeFuncs: [Option<FuncXY_V>; 8] = callback_table! { unsafe FuncXY_V;
    None, AtmosphericTypeFunc_1_FootstepSplash, AtmosphericTypeFunc_1_FootstepSplash,
    AtmosphericTypeFunc_3_DivingSplash, AtmosphericTypeFunc_4_LavaSurfaceDmg,
    AtmosphericTypeFunc_5_Bubbles, AtmosphericTypeFunc_67_Dust, AtmosphericTypeFunc_67_Dust,
};
pub unsafe fn HandleAtmosphericEffects(game: &mut Game) {
    for i in (0i32..=6).rev().step_by(2) {
        let v1: i32 = i >> 1;
        let mut v2: u16 = word!(game.atmospheric_gfx_frame_and_type[v1]);
        if (v2 != 0) {
            let r18: u16 = (2u16).wrapping_mul((v2 as u8) as u16);
            let v3: u16 = (2u16).wrapping_mul((byte!(v2; 1)) as u16);
            let mut v4: i16 = ((word!(game.atmospheric_gfx_anim_timer[v1])) as i16).wrapping_sub(1);
            word!(game.atmospheric_gfx_anim_timer[v1] = v4 as u16);
            let mut skip_effect: bool = if v4 != 0 {
                (v4 < 0)
                    && ((v4 != (0x8000i32 as i16))
                        || ({
                            word!(
                                game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                                    (r18).wrapping_add(crate::rtl::rom_table_word!(
                                        game,
                                        0x908b93,
                                        (v3 >> 1) as isize,
                                    ),),
                                    game,
                                )
                                    as *mut u16)
                            );
                            false
                        }))
            } else {
                word!(
                    game.atmospheric_gfx_anim_timer[v1] = *(RomPtr_90(
                        (r18).wrapping_add(crate::rtl::rom_table_word!(
                            game,
                            0x908b93,
                            (v3 >> 1) as isize
                        ),),
                        game,
                    ) as *mut u16)
                );
                let v5: u16 = (word!(game.atmospheric_gfx_frame_and_type[v1])).wrapping_add(1);
                word!(game.atmospheric_gfx_frame_and_type[v1] = v5);
                (((v5 as u8) as i16).wrapping_sub(
                    (crate::rtl::rom_table_word!(game, 0x908bef, (v3 >> 1) as isize)) as i16,
                ) >= 0)
                    && ({
                        word!(game.atmospheric_gfx_frame_and_type[v1] = 0);
                        true
                    })
            };
            if !(skip_effect) {
                kAtmosphericTypeFuncs[(v3 >> 1) as usize].expect("non-null function pointer")(
                    v3, i as u16, game,
                );
            }
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_1_FootstepSplash(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    let r18: u16 = (2u16)
        .wrapping_mul((byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset(v2 as isize); 0)) as u16);
    let v3: u16 = word!(game.oam_next_ptr);
    let mut v4: i16 = (word!(game.atmospheric_gfx_x_pos[v2]) as i32
        - word!(game.layer1_x_pos) as i32
        - 4i32) as i16;
    if v4 >= 0
        && word!(game.atmospheric_gfx_x_pos[v2]) as i32 - word!(game.layer1_x_pos) as i32 - 260i32
            & 0x8000
            != 0
    {
        let mut v5: *mut OamEnt = gOamEnt(word!(game.oam_next_ptr) as i32, game);
        (*v5).xcoord = v4 as u8;
        let mut v6: i16 = (word!(game.atmospheric_gfx_y_pos[v2]) as i32
            - word!(game.layer1_y_pos) as i32
            - 4i32) as i16;
        if v6 >= 0
            && word!(game.atmospheric_gfx_y_pos[v2]) as i32
                - word!(game.layer1_y_pos) as i32
                - 260i32
                & 0x8000
                != 0
        {
            (*v5).ycoord = v6 as u8;
            *(&raw mut (*v5).charnum as *mut u16) = *(RomPtr_90(
                (r18).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0x908bff,
                    (k >> 1) as isize
                )),
                game,
            ) as *mut u16);
            word!(game.oam_next_ptr = (v3).wrapping_add(4));
        }
    }
}
pub unsafe fn AtmosphericTypeFunc_Common(j: u16, a: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    if (word!(game.atmospheric_gfx_y_pos[v1]) as i32 - word!(game.layer1_y_pos) as i32 & 0xff00)
        == 0
    {
        DrawSamusSpritemap(
            a,
            (word!(game.atmospheric_gfx_x_pos[v1])).wrapping_sub(word!(game.layer1_x_pos)),
            (word!(game.atmospheric_gfx_y_pos[v1])).wrapping_sub(word!(game.layer1_y_pos)),
            game,
        );
    }
}
pub unsafe fn AtmosphericTypeFunc_3_DivingSplash(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    let r18: u16 = byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset(v2 as isize); 0) as u16;
    word!(game.atmospheric_gfx_y_pos[v2] = word!(game.fx_y_pos));
    AtmosphericTypeFunc_Common(j, (r18).wrapping_add(399), game);
}
pub unsafe fn AtmosphericTypeFunc_4_LavaSurfaceDmg(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    let mut v3: u16 = if (j & 4) != 0 {
        (word!(game.atmospheric_gfx_x_pos[v2])).wrapping_sub(1)
    } else {
        (word!(game.atmospheric_gfx_x_pos[v2])).wrapping_add(1)
    };
    word!(game.atmospheric_gfx_x_pos[v2] = v3);
    word!(game.atmospheric_gfx_y_pos[(j >> 1)] -= 1);
    AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
}
pub unsafe fn AtmosphericTypeFunc_67_Dust(k: u16, j: u16, game: &mut Game) {
    word!(game.atmospheric_gfx_y_pos[(j >> 1)] -= 1);
    AtmosphericTypeFunc_1_FootstepSplash(k, j, game);
}
pub unsafe fn AtmosphericTypeFunc_5_Bubbles(k: u16, j: u16, game: &mut Game) {
    let r18: u16 = byte!(@pointer (ram_ptr!(game, u16; 0xaec)).offset((j >> 1) as isize); 0) as u16;
    AtmosphericTypeFunc_Common(j, (r18).wrapping_add(390), game);
}

pub unsafe fn Samus_CalcSpritemapPos(k: u16, game: &mut Game) -> PairU16 {
    let mut v1: PairU16 = PairU16 { k: 0, j: 0 };
    if (word!(game.ceres_status) & 0x8000) == 0 {
        v1 = calculate_sprite_position(k, game);
    } else {
        let old_x: u16 = word!(game.samus_x_pos);
        let old_y: u16 = word!(game.samus_y_pos);
        Samus_CalcPos_Mode7(game);
        v1 = calculate_sprite_position(k, game);
        word! {game; samus_y_pos = old_y; samus_x_pos = old_x; }
    }
    MakePairU16(v1.k, v1.j)
}

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct SamusTileAnimationDefs {
        pub top_half_idx, top_half_pos, bottom_half_idx, bottom_half_pos: u8;
    }
}
#[inline]
unsafe fn get_SamusTileAnimationDefs(a: u16, game: &mut Game) -> *mut SamusTileAnimationDefs {
    RomPtr((0x920000u32) | ((a) as u32), game) as *mut SamusTileAnimationDefs
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    let v0: u16 = (4 * word!(game.samus_anim_frame) as i32
        + crate::rtl::rom_table_word!(game, 0x92d94e, word!(game.samus_pose) as isize) as i32)
        as u16;
    let AD = get_SamusTileAnimationDefs(v0, game);
    word!(
        game.nmi_copy_samus_top_half_src = (7 * (*AD).top_half_pos as i32
            + crate::rtl::rom_table_word!(game, 0x92d91e, (*AD).top_half_idx as isize) as i32)
            as u16
    );
    byte!(@pointer ram_ptr!(game, u16; 0x71d); 0) = 1;
    if (*AD).bottom_half_idx != 255 {
        word!(
            game.nmi_copy_samus_bottom_half_src = (7 * (*AD).bottom_half_pos as i32
                + crate::rtl::rom_table_word!(game, 0x92d938, (*AD).bottom_half_idx as isize)
                    as i32) as u16
        );
        byte!(@pointer ram_ptr!(game, u16; 0x71d); 1) = 1;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> u8 {
    if word!(game.substate) != 0 {
        if word!(game.substate) == 5 {
            PlayRoomMusicTrackAfterAFrames(0x168, game);
        }
    } else {
        QueueMusic_DelayedY(1, 0xe, game);
    }
    if word!(game.substate) as i32 - 359i32 & 0x8000 != 0 {
        word!(game.substate += 1);
        0
    } else {
        crate::game::samus_pose::record_pose_history(game);
        word!(game.substate = 0);
        1
    }
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let r18: u16 = (if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.g_word_7E0DE4) as i32 + 2085
    } else {
        word!(game.g_word_7E0DE4) as i32 + 2076
    }) as u16;
    DrawSamusSpritemap(r18, word!(game.samus_x_pos), word!(game.samus_y_pos), game);
}

unsafe fn sprite_y_offset(doubled_pose: u16, game: &mut Game) -> i32 {
    static LANDING: [u8; 16] = [3, 6, 0, 0, 3, 6, 0, 0, 3, 3, 6, 0, 3, 3, 6, 0];
    static CROUCHING: [i8; 24] = [
        -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
    ];
    static DRAINED: [i8; 32] = [
        7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0, 0, 4,
        0, 0, 4, 0, 0,
    ];
    let pose = doubled_pose >> 1;
    let frame = game.samus_anim_frame();
    match game.samus_movement_type() {
        0 => {
            if matches!(
                SamusPose::from(pose),
                kPose_00_FaceF_Powersuit | kPose_9B_FaceF_VariaGravitySuit
            ) {
                if frame.wrapping_sub(2) & 0x8000 == 0 {
                    return -1;
                }
            } else if (0xa4..0xa8).contains(&pose) {
                // The legacy landing table deliberately reads a word from byte data.
                let offset = frame.wrapping_add(4 * (pose - 0xa4));
                return -(*(LANDING.as_ptr().offset(offset as isize) as *const u16) as i32);
            }
        }
        15 if (0x35..0x41).contains(&pose) => {
            return CROUCHING[frame.wrapping_add(2 * (pose - 0x35)) as usize] as i32;
        }
        27 => {
            if matches!(
                SamusPose::from(pose),
                kPose_E8_FaceR_Drained_CrouchFalling | kPose_E9_FaceL_Drained_CrouchFalling
            ) {
                return DRAINED[frame as usize] as i32;
            }
            if matches!(
                SamusPose::from(pose),
                kPose_EA_FaceR_Drained_Stand | kPose_EB_FaceL_Drained_Stand
            ) && (frame as i16).wrapping_sub(5) >= 0
            {
                return -3;
            }
        }
        1..=26 => {}
        _ => panic!("invalid Samus movement type"),
    }
    -(*(&raw mut (*(rom_ptr!(game, SamusPoseParams; 0x91b629))).y_offset_to_gfx)
        .offset((4 * doubled_pose as i32) as isize) as i8 as i32)
}

unsafe fn calculate_sprite_position(pose: u16, game: &mut Game) -> PairU16 {
    let offset = sprite_y_offset(pose, game);
    word!(
        game.samus_spritemap_y_pos =
            (word!(game.samus_y_pos) as i32 + offset - word!(game.layer1_y_pos) as i32) as u16
    );
    word!(
        game.samus_spritemap_x_pos = word!(game.samus_x_pos).wrapping_sub(word!(game.layer1_x_pos))
    );
    MakePairU16(
        word!(game.samus_spritemap_x_pos),
        word!(game.samus_spritemap_y_pos),
    )
}
