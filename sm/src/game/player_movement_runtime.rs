//! Compatibility content and exceptional echo aliases for owned locomotion.
use crate::{
    game::{
        liquid_surface::LiquidAtHeight,
        player_movement::{
            content::{Content, FootstepRegion, FootstepRule, SpeedRecord},
            effects::{EchoAxis, Effects},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};
const CRATERIA_FOOTSTEP_FLAGS: [u8; 16] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0];
impl Content for Runtime<'_> {
    fn speed_record(&mut self, profile: SpeedRecord) -> RomRecord {
        let cursor = match profile {
            SpeedRecord::Selected(cursor) => cursor,
            SpeedRecord::Bomb => 0x9f25,
            SpeedRecord::GrappleAir => 0x9f31,
            SpeedRecord::GrappleWater => 0x9f3d,
            SpeedRecord::GrappleLava => 0x9f49,
        };
        if cursor & 0x8000 == 0 {
            self.foreign(|game| unsafe {
                crate::rtl::rom_bank_ptr::<0x90>(cursor, game);
            });
        }
        visual_host::Content::table(self, Source::Native, 0x900000 | u32::from(cursor))
    }
    fn liquid_speed_table(&self, liquid: LiquidAtHeight) -> Option<u16> {
        match liquid {
            LiquidAtHeight::None => None,
            LiquidAtHeight::Water => Some(0xa08d),
            LiquidAtHeight::LavaAcid => Some(0xa1dd),
        }
    }
    fn spin_sound_frames(&self) -> RomRecord {
        visual_host::Content::table(self, Source::Native, 0x90a514)
    }
    fn footstep_region(&self) -> FootstepRegion {
        use FootstepRegion::*;
        [
            Crateria, Default, Default, Default, Sand, Default, Default, Default,
        ][usize::from(self.session().room.area)]
    }
    fn footstep_room_rule(&self) -> FootstepRule {
        use FootstepRule::*;
        let room = self.session().room.entry.room_index;
        if (room as i16).wrapping_sub(16) >= 0 {
            return Default;
        }
        let flags = CRATERIA_FOOTSTEP_FLAGS[usize::from(room)];
        if flags & 1 != 0 {
            SandWhenFxTen
        } else if flags & 2 != 0 {
            SandAtOrBelow944
        } else if flags & 4 != 0 {
            Sand
        } else {
            Default
        }
    }
}
impl Effects for Runtime<'_> {
    fn boss_active(&self) -> bool {
        self.word(0x179c) != 0
    }
    fn write_echo_coordinate(&mut self, axis: EchoAxis, index: usize, value: u16) {
        let base = match axis {
            EchoAxis::X => 0xab0usize,
            EchoAxis::Y => 0xab8usize,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("echo coordinate extent"))
            .expect("echo coordinate address");
        self.write_word_le(value, at);
    }
}

pub(crate) fn ceres_movement(game: &mut crate::game_state::Game, action: fn(&mut Runtime<'_>)) {
    let mut runtime = Runtime::new(game);
    action(&mut runtime);
    runtime.foreign(|game| unsafe {
        crate::game::sm_80::ProcessTimer(game);
        if crate::ram_map::word!(game.timer_status) != 0 {
            crate::game::sm_80::DrawTimer(game);
        }
    });
}

// Raw movement identities and uncovered callbacks remain at the compatibility boundary.
use crate::game::{
    constants::bank_90::addr_loc_908029,
    explosion_window::spawn_crystal_flash,
    player_collision::operations as player_collision,
    player_movement::{self, forced, speed},
    player_projectiles::initialize,
    prelude::*,
    projectile_geometry::sine_offset,
    projectile_initialization::Initialization,
    session_runtime::Runtime as CollisionRuntime,
    sm_80::{
        InvalidInterrupt_Crash, QueueSfx1_Max6, QueueSfx3_Max6, QueueSfx3_Max9, QueueSfx3_Max15,
    },
    visual_runtime::set_hdma_enabled,
};
pub unsafe fn Samus_EndSuperJump(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
            && word!(game.samus_collision_flag) == 0
        {
            return 0;
        }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word! {game; speed_echo_xspeed[0] = 32; speed_echo_xspeed[1] = 160;
            speed_echo_xpos[2] = 4; }
        } else {
            word! {game; speed_echo_xspeed[0] = 224; speed_echo_xspeed[1] = 96;
            speed_echo_xpos[2] = -4i32 as u16; }
        }
        word! { game, 0;
            samus_y_subspeed, samus_y_speed, samus_x_extra_run_speed, samus_x_extra_run_subspeed,
            speed_boost_counter, UNUSED_word_7E0B1A, samus_y_dir,
        }
        word! {game; samus_movement_handler = ((0x90d346i32) as u16);
        samus_draw_handler = ((0x90ebf3i32) as u16); speed_echoes_index = 0;
        speed_echo_xpos[0] = word!(game.samus_x_pos);
        speed_echo_xpos[1] = word!(game.samus_x_pos);
        speed_echo_ypos[0] = word!(game.samus_y_pos);
        speed_echo_ypos[1] = word!(game.samus_y_pos); }
        word! { game, 0; speed_echo_xspeed[2], samus_hurt_flash_counter }
        QueueSfx1_Max6(0x35, game);
        QueueSfx3_Max6(0x10, game);
        1
    }
}
static kSamus_MoveHandler_ShinesparkCrash: [Func_V; 3] = callback_table! { nonnull Func_V;
    Samus_MoveHandler_ShinesparkCrash_0, Samus_MoveHandler_ShinesparkCrash_1,
    Samus_MoveHandler_ShinesparkCrash_2,
};
static KNOCKBACK_MOTION: [Option<fn(&mut Runtime<'_>)>; 6] = [
    None,
    Some(|h| forced::knockback_motion(h, true, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, true, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, false, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, true, speed::no_speed_y)),
    Some(|h| forced::knockback_motion(h, true, speed::no_speed_y)),
];
crate::game::callback_dispatch! {
    pub(crate) unsafe fn advance_special_movement(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9475182] Samus_MoveHandler_ReleaseFromGrapple [] {
        if word!(game.samus_y_dir) == 1 && (word!(game.samus_y_speed) & 0x8000) != 0 {
            word! { game, 0; samus_y_subspeed, samus_y_speed }
            word! {game; samus_y_dir = 2; samus_movement_handler = ((0x90a337i32) as u16); }
        }
        word!(game.samus_x_accel_mode = 2);
        {
            let mut runtime = Runtime::new(game);
            let profile = player_movement::speed::grapple_speed_profile(&mut runtime);
            let amount = player_movement::speed::base_speed(&mut runtime, profile);
            let request = player_movement::speed::horizontal_air::<false>(&mut runtime, amount);
            player_collision::horizontal_control(&mut runtime, request);
            player_movement::speed::move_y_with_speed(&mut runtime);
        }
        if word!(game.input_to_pose_calc) != 0 {
            word!(game.samus_movement_handler = (0x90a337i32) as u16);
        }
    };
    [9475275] Samus_HandleMovement_DrainedCrouching [] {
        player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
        if word!(game.samus_collision_flag) != 0 {
            word! {game; samus_movement_handler = ((0x90a337i32) as u16);
                samus_anim_frame_timer = 8; samus_anim_frame = 7; }
            word! { game, 0; samus_y_subspeed, samus_y_speed }
        }
    };
    [9478967] Samus_MovementHandler_Normal [] {
        player_movement::advance_normal(&mut Runtime::new(game));
    };
    [9490536] Samus_MoveHandlerShinesparkWindup [] {
        let v0: bool = (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0;
        if word!(game.timer_for_shinesparks_startstop) == 0 || v0 {
            if *ram_ptr!(game; 0xa1e) == 4 {
                word!(game.samus_new_pose_interrupted = 204);
            } else {
                word!(game.samus_new_pose_interrupted = 203);
            }
            word!(game.samus_movement_handler = (0x90d0abi32) as u16);
            word!(game.samus_input_handler = (0x90e90ei32) as u16);
            word! { game, 0;
                speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1], speed_echo_xpos[0],
                speed_echo_xpos[1],
            }
            QueueSfx3_Max9(0xf, game);
        }
    };
    [9491270] Samus_MoveHandler_ShinesparkCrash [] {
        word!(game.samus_shine_timer = 15);
        kSamus_MoveHandler_ShinesparkCrash
            [byte!(@pointer ram_ptr!(game, u16; 0xaae); 1isize) as usize]
            (game);
        for i in (0i32..=2).rev().step_by(2) {
            let v1: i32 = i >> 1;
            let pt = sine_offset::<true>(
                word!(game.speed_echo_xspeed[v1]),
                word!(game.speed_echoes_index) as u8 as u16,
            );
            word!(game.speed_echo_xpos[v1] = (pt.x).wrapping_add(word!(game.samus_x_pos)));
            word!(game.speed_echo_ypos[v1] = (pt.y).wrapping_add(word!(game.samus_y_pos)));
        }
    };
    [9491331] Samus_MoveHandler_ShinesparkCrash_0 [] {
        let mut v0: u16 = (word!(game.speed_echoes_index)).wrapping_add(4);
        if word!(game.speed_echoes_index) as i32 - 12i32 & 0x8000 == 0 {
            v0 = (v0) | (0x100u16);
        }
        word!(game.speed_echoes_index = v0);
    };
    [9491350] Samus_MoveHandler_ShinesparkCrash_1 [] {
        word! { game;
            speed_echo_xspeed[0] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8))
                .wrapping_add(*((ram_ptr!(game, u16; 0xac0)) as *mut u8)))
                as u16;
            speed_echo_xspeed[1] = ((*((ram_ptr!(game, u16; 0xab0)).offset(2) as *mut u8))
                .wrapping_add(*((ram_ptr!(game, u16; 0xac0)).offset(1) as *mut u8)))
                as u16;
        }
        word!(game.speed_echo_ypos[2] += 4);
        if word!(game.speed_echo_ypos[2]) as i32 - 128i32 & 0x8000 == 0 {
            word!(
                game.speed_echoes_index =
                    ((word!(game.speed_echoes_index) as u8) as u16) | (0x200u16)
            );
        }
    };
    [9491404] Samus_MoveHandler_ShinesparkCrash_2 [] {
        word!(game.speed_echoes_index -= 4);
        if word!(game.speed_echoes_index) as u8 == 0 {
            word!(game.samus_movement_handler = (0x90d3f3i32) as u16);
            word!(game.timer_for_shinesparks_startstop = 30);
            word! { game, 0;
                speed_echoes_index, speed_echo_xspeed[0], speed_echo_xspeed[1],
            }
        }
    };
    [9491443] Samus_MoveHandler_ShinesparkCrashEchoCircle [] {
        word!(game.samus_shine_timer = 15);
        let v0: bool = (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0;
        if word!(game.timer_for_shinesparks_startstop) == 0 || v0 {
            word!(game.samus_movement_handler = (0x90d40di32) as u16);
            word!(game.samus_draw_handler = (0x90eb52i32) as u16);
        }
    };
    [9491469] Samus_MoveHandler_ShinesparkCrashFinish [] {
        static kShinesparkCrashFinish_Tab0: [u8; 12] = [
            0, 0x80, 0, 0x80, 0x40, 0xc0, 0x40, 0xc0, 0xe0, 0x60, 0x20, 0xa0,
        ];
        word!(game.speed_echoes_index = 0);
        if word!(game.projectile_counter) as i32 - 5i32 & 0x8000 != 0 {
            if word!(game.projectile_counter) as i32 - 4i32 & 0x8000 != 0 {
                word!(game.projectile_counter += 1);
                word!(game.speed_echo_xspeed[2] = 64);
                word!(game.speed_echo_xpos[2] = word!(game.samus_x_pos));
                word!(game.speed_echo_ypos[2] = word!(game.samus_y_pos));
                word!(game.projectile_type[3] = addr_loc_908029 as u16);
                initialize(6, Initialization::Echo, game);
                word!(game.projectile_bomb_pre_instructions[3] = (0x90d4d2i32) as u16);
                word!(
                    game.projectile_variables[3] = kShinesparkCrashFinish_Tab0
                        [(2 * (word!(game.samus_pose) as i32 - 201i32)) as u16 as usize]
                        as u16
                );
                word!(game.projectile_bomb_x_speed[3] = 0);
            }
            word!(game.projectile_counter += 1);
            word! {game; speed_echo_xspeed[3] = 64; speed_echo_xpos[3] = word!(game.samus_x_pos);
                speed_echo_ypos[3] = word!(game.samus_y_pos);
                projectile_type[4] = addr_loc_908029 as u16; }
            initialize(8, Initialization::Echo, game);
            word!(game.projectile_bomb_pre_instructions[4] = (0x90d4d2i32) as u16);
            word!(
                game.projectile_variables[4] = kShinesparkCrashFinish_Tab0
                    [((2 * (word!(game.samus_pose) as i32 - 201i32)) as u16 as i32 + 1) as usize]
                    as u16
            );
            word!(game.projectile_bomb_x_speed[4] = 0);
        }
        word! {game; cooldown_timer = 0; samus_shine_timer = 1; }
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_new_pose_transitional = 2);
        } else {
            word!(game.samus_new_pose_transitional = 1);
        }
        word!(game.samus_hurt_switch_index = 2);
        word! { game, 0; substate, suit_pickup_light_beam_pos }
    };
    [9492088] SamusMoveHandler_CrystalFlashStart [] {
        word!(game.samus_y_pos -= 2);
        if (word!(game.timer_for_shinesparks_startstop -= 1, reread) & 0x8000)
            != 0
        {
            word! {game; samus_anim_frame_timer = 3; samus_anim_frame = 6; }
            *(ram_ptr!(game, u16; 0xdf0)) = word!(game.samus_y_pos);
            word!(game.samus_movement_handler = (0x90d6cei32) as u16);
            word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
            QueueSfx3_Max15(1, game);
            word!(game.power_bomb_flag = 0);
            word!(game.power_bomb_explosion_x_pos = word!(game.samus_x_pos));
            word!(game.power_bomb_explosion_y_pos = word!(game.samus_y_pos));
            set_hdma_enabled(true, game);
            spawn_crystal_flash(game);
        }
    };
    [9492315] kSamusMoveHandler_CrystalFlashFinish [] {
        if word!(game.samus_y_pos) != *(ram_ptr!(game, u16; 0xdf0)) {
            word!(game.samus_y_pos += 1);
        }
        if *ram_ptr!(game; 0xa1f) == 0 {
            word! {game; power_bomb_flag = 0; samus_shine_timer = -1i32 as u16;
                samus_movement_handler = ((0x90a337i32) as u16); }
            if word!(game.samus_input_handler) as i32 != 0x90e91d & 0xffff {
                word!(game.samus_input_handler = (0x90e913i32) as u16);
                word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
            }
        }
    };
    [9494328] Samus_MoveHandler_Knockback [] {
        match KNOCKBACK_MOTION[usize::from(word!(game.knockback_dir))] {
            Some(motion) => motion(&mut Runtime::new(game)),
            None => InvalidInterrupt_Crash(game),
        }
        word!(game.input_to_pose_calc = 0);
    };
    [9496911] SamusMovementType_Xray [] {
        let mut v0: u16 = 0;
        if *ram_ptr!(game; 0xa1f) as i32 != kMovementType_0E_TurningAroundOnGround as i32 {
            word!(game.samus_anim_frame_timer = 15);
            if *ram_ptr!(game; 0xa1e) == 4 {
                if word!(game.xray_angle) as i32 - 153i32 & 0x8000 != 0 {
                    v0 = 4;
                } else if word!(game.xray_angle) as i32 - 178i32 & 0x8000 != 0 {
                    v0 = 3;
                } else if word!(game.xray_angle) as i32 - 203i32 & 0x8000 != 0 {
                    v0 = 2;
                } else {
                    v0 = ((word!(game.xray_angle) as i32 - 228i32 & 0x8000) != 0) as u16;
                }
            } else if word!(game.xray_angle) as i32 - 25i32 & 0x8000 != 0 {
                v0 = 0;
            } else if word!(game.xray_angle) as i32 - 50i32 & 0x8000 != 0 {
                v0 = 1;
            } else if word!(game.xray_angle) as i32 - 75i32 & 0x8000 != 0 {
                v0 = 2;
            } else if word!(game.xray_angle) as i32 - 100i32 & 0x8000 != 0 {
                v0 = 3;
            } else {
                v0 = 4;
            }
            word!(game.samus_anim_frame = v0);
        }
    };
    [9498699] Samus_Func25_ShineSpark [] {
        if word!(game.samus_shine_timer) == 0 {
            word!(game.special_samus_palette_frame = 6);
            word! { game, 1; special_samus_palette_timer, demo_timer_counter }
            word! {game; timer_for_shine_timer = 10; samus_shine_timer = 120;
                samus_movement_handler = ((0x90f072i32) as u16); }
        }
    };
    [9498738] Samus_MoveHandler_F072 [] {
        if word!(game.samus_shine_timer) == 0 {
            word!(game.frame_handler_alfa = (0x90e695i32) as u16);
            word!(game.samus_movement_handler = (0x90a337i32) as u16);
        }
    };
    @external {
        9490603 => [advance_shinespark(&[player_collision::Axis::Y], game)];
        9490647 => [advance_shinespark(&[player_collision::Axis::X, player_collision::Axis::Y], game)];
        9490694 => [advance_shinespark(&[player_collision::Axis::X], game)];
        9494352 => [InvalidInterrupt_Crash(game)];
        9494355 => [forced::knockback_motion(&mut Runtime::new(game), true, speed::move_y_with_speed)];
        9494365 => [forced::knockback_motion(&mut Runtime::new(game), false, speed::move_y_with_speed)];
        9494372 => [forced::knockback_motion(&mut Runtime::new(game), true, speed::no_speed_y)];
        9492174 => [{
            let mut runtime = Runtime::new(game);
            let frame = runtime.inputs().frame;
            forced::consume_crystal_flash(runtime.session_mut(), frame);
        }];
        9494565 => [forced::begin_bomb_jump(&mut Runtime::new(game))];
        9494578 => [forced::advance_bomb_jump(&mut Runtime::new(game), |h| h.foreign(|game| InvalidInterrupt_Crash(game)))];
        9494653 => [forced::finish_bomb_jump(Runtime::new(game).session_mut())];
        9496846 => [()];
    }
}

unsafe fn advance_shinespark(axes: &[player_collision::Axis], game: &mut Game) {
    unsafe {
        word! {game; samus_contact_damage_index = 2; samus_hurt_flash_counter = 8; }
        player_movement::record_echo(&mut Runtime::new(game));
        for &axis in axes {
            player_collision::shinespark(&mut CollisionRuntime::new(game), axis);
        }
        Samus_EndSuperJump(game);
        if word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0 && {
            (word!(game.samus_health -= 1, reread) & 0x8000) != 0
        } {
            word!(game.samus_health = 0);
        }
    }
}

pub(crate) fn correct_previous_position(game: &mut Game) {
    use crate::ram_map::{checked_word, words as ram};
    for (current, previous) in [
        (ram::samus_y_pos, ram::samus_prev_y_pos),
        (ram::samus_x_pos, ram::samus_prev_x_pos),
    ] {
        let position = checked_word!(game, current;);
        let delta = position.wrapping_sub(checked_word!(game, previous;));
        if abs16(delta).wrapping_sub(12) & 0x8000 == 0 {
            let correction = if delta & 0x8000 == 0 {
                12
            } else {
                (-12i16) as u16
            };
            checked_word!(game, previous; = position.wrapping_add(correction));
        }
    }
}
