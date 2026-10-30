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
            self.foreign(|game| {
                crate::game::session_runtime::native_rom_diagnostic(
                    0x900000 | u32::from(cursor),
                    game,
                );
                // Preserve the actual optional native-binding capture and release.
                drop(game.capture_native_rom_binding());
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
    fn initialize_shinespark_echo(&mut self, slot: u16) {
        self.foreign(|game| initialize(slot, Initialization::Echo, game));
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
    runtime.foreign(|game| {
        sm_game::escape_timer::advance(game);
        if crate::ram_map::word!(game.TIMER_STATUS) != 0 {
            sm_game::escape_timer::draw(game);
        }
    });
}

use sm_game::explosion_window::spawn_crystal_flash;
// Raw movement identities and uncovered callbacks remain at the compatibility boundary.
use {
    crate::game::{
        music::QueueSfx3_Max15,
        player_collision::operations as player_collision,
        player_movement::{self, forced, speed},
        player_projectiles::initialize,
        prelude::*,
        session_runtime::Runtime as CollisionRuntime,
        visual_runtime::set_hdma_enabled,
    },
    sm_game::projectile_initialization::Initialization,
};
pub fn Samus_EndSuperJump(game: &mut Game) -> u8 {
    if word!(game.SAMUS_HEALTH).wrapping_sub(30) & 0x8000 == 0
        && word!(game.SAMUS_COLLISION_FLAG) == 0
    {
        return 0;
    }
    forced::end_shinespark(&mut Runtime::new(game))
}
crate::forward_calls! { pub fn (game: &mut Game);
    Samus_MoveHandlerShinesparkWindup => forced::shinespark_windup(&mut Runtime::new(game));
    Samus_MoveHandler_ShinesparkCrash => forced::shinespark_crash(&mut Runtime::new(game));
    Samus_MoveHandler_ShinesparkCrash_0 => forced::expand_shinespark_crash(Runtime::new(game).session_mut());
    Samus_MoveHandler_ShinesparkCrash_1 => forced::rotate_shinespark_crash(Runtime::new(game).session_mut());
    Samus_MoveHandler_ShinesparkCrash_2 => forced::contract_shinespark_crash(Runtime::new(game).session_mut());
    Samus_MoveHandler_ShinesparkCrashEchoCircle => forced::wait_shinespark_crash(Runtime::new(game).session_mut());
    Samus_MoveHandler_ShinesparkCrashFinish => forced::finish_shinespark_crash(&mut Runtime::new(game));
}
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
        if word!(game.SAMUS_Y_DIR) == 1 && (word!(game.SAMUS_Y_SPEED) & 0x8000) != 0 {
            word! { game, 0; SAMUS_Y_SUBSPEED, SAMUS_Y_SPEED }
            word! {game; SAMUS_Y_DIR = 2; SAMUS_MOVEMENT_HANDLER = ((0x90a337i32) as u16); }
        }
        word!(game.SAMUS_X_ACCEL_MODE = 2);
        {
            let mut runtime = Runtime::new(game);
            let profile = player_movement::speed::grapple_speed_profile(&mut runtime);
            let amount = player_movement::speed::base_speed(&mut runtime, profile);
            let request = player_movement::speed::horizontal_air::<false>(&mut runtime, amount);
            player_collision::horizontal_control(&mut runtime, request);
            player_movement::speed::move_y_with_speed(&mut runtime);
        }
        if word!(game.INPUT_TO_POSE_CALC) != 0 {
            word!(game.SAMUS_MOVEMENT_HANDLER = (0x90a337i32) as u16);
        }
    };
    [9475275] Samus_HandleMovement_DrainedCrouching [] {
        player_movement::speed::move_y_with_speed(&mut Runtime::new(game));
        if word!(game.SAMUS_COLLISION_FLAG) != 0 {
            word! {game; SAMUS_MOVEMENT_HANDLER = ((0x90a337i32) as u16);
                SAMUS_ANIM_FRAME_TIMER = 8; SAMUS_ANIM_FRAME = 7; }
            word! { game, 0; SAMUS_Y_SUBSPEED, SAMUS_Y_SPEED }
        }
    };
    [9478967] Samus_MovementHandler_Normal [] {
        player_movement::advance_normal(&mut Runtime::new(game));
    };
    @functions [pub fn (game: &mut Game)];
    [9492088] SamusMoveHandler_CrystalFlashStart [] {
        word!(game.SAMUS_Y_POS -= 2);
        if (word!(game.TIMER_FOR_SHINESPARKS_STARTSTOP -= 1, reread) & 0x8000)
            != 0
        {
            word! {game; SAMUS_ANIM_FRAME_TIMER = 3; SAMUS_ANIM_FRAME = 6; }
            word_transfers!(game, ascending, 1; native(0xdf0) <- value(word!(game.SAMUS_Y_POS)));
            word!(game.SAMUS_MOVEMENT_HANDLER = (0x90d6cei32) as u16);
            word! { game, 0; SAMUS_INVINCIBILITY_TIMER, SAMUS_KNOCKBACK_TIMER }
            QueueSfx3_Max15(1, game);
            word!(game.POWER_BOMB_FLAG = 0);
            word!(game.POWER_BOMB_EXPLOSION_X_POS = word!(game.SAMUS_X_POS));
            word!(game.POWER_BOMB_EXPLOSION_Y_POS = word!(game.SAMUS_Y_POS));
            set_hdma_enabled(true, game);
            spawn_crystal_flash(game);
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [9494328] Samus_MoveHandler_Knockback [] {
        match KNOCKBACK_MOTION[usize::from(word!(game.KNOCKBACK_DIR))] {
            Some(motion) => motion(&mut Runtime::new(game)),
            None => crate::rtl::invalid_interrupt(game),
        }
        word!(game.INPUT_TO_POSE_CALC = 0);
    };
    @functions [pub fn (game: &mut Game)];
    [9492315] kSamusMoveHandler_CrystalFlashFinish [] {
        if word!(game.SAMUS_Y_POS) != u16::from_ne_bytes([game.ram().read_byte(0xdf0), game.ram().read_byte(0xdf1)]) {
            word!(game.SAMUS_Y_POS += 1);
        }
        if game.ram().read_byte(0xa1f) == 0 {
            word! {game; POWER_BOMB_FLAG = 0; SAMUS_SHINE_TIMER = -1i32 as u16;
                SAMUS_MOVEMENT_HANDLER = ((0x90a337i32) as u16); }
            if word!(game.SAMUS_INPUT_HANDLER) as i32 != 0x90e91d & 0xffff {
                word!(game.SAMUS_INPUT_HANDLER = (0x90e913i32) as u16);
                word! { game, 0; SAMUS_INVINCIBILITY_TIMER, SAMUS_KNOCKBACK_TIMER }
            }
        }
    };
    [9496911] SamusMovementType_Xray [] {
        if game.ram().read_byte(0xa1f) as i32 != K_MOVEMENT_TYPE_0E_TURNING_AROUND_ON_GROUND as i32 {
            word!(game.SAMUS_ANIM_FRAME_TIMER = 15);
            let frame = player_movement::xray_animation_frame(game.ram().read_byte(0xa1e), || {
                word!(game.XRAY_ANGLE)
            });
            word!(game.SAMUS_ANIM_FRAME = frame);
        }
    };
    [9498699] Samus_Func25_ShineSpark [] {
        if word!(game.SAMUS_SHINE_TIMER) == 0 {
            word!(game.SPECIAL_SAMUS_PALETTE_FRAME = 6);
            word! { game, 1; SPECIAL_SAMUS_PALETTE_TIMER, DEMO_TIMER_COUNTER }
            word! {game; TIMER_FOR_SHINE_TIMER = 10; SAMUS_SHINE_TIMER = 120;
                SAMUS_MOVEMENT_HANDLER = ((0x90f072i32) as u16); }
        }
    };
    [9498738] Samus_MoveHandler_F072 [] {
        if word!(game.SAMUS_SHINE_TIMER) == 0 {
            word!(game.FRAME_HANDLER_ALFA = (0x90e695i32) as u16);
            word!(game.SAMUS_MOVEMENT_HANDLER = (0x90a337i32) as u16);
        }
    };
    @external {
        9490536 => [Samus_MoveHandlerShinesparkWindup(game)];
        9491270 => [Samus_MoveHandler_ShinesparkCrash(game)];
        9491331 => [Samus_MoveHandler_ShinesparkCrash_0(game)];
        9491350 => [Samus_MoveHandler_ShinesparkCrash_1(game)];
        9491404 => [Samus_MoveHandler_ShinesparkCrash_2(game)];
        9491443 => [Samus_MoveHandler_ShinesparkCrashEchoCircle(game)];
        9491469 => [Samus_MoveHandler_ShinesparkCrashFinish(game)];
        9490603 => [advance_shinespark(&[player_collision::Axis::Y], game)];
        9490647 => [advance_shinespark(&[player_collision::Axis::X, player_collision::Axis::Y], game)];
        9490694 => [advance_shinespark(&[player_collision::Axis::X], game)];
        9494352 => [crate::rtl::invalid_interrupt(game)];
        9494355 => [forced::knockback_motion(&mut Runtime::new(game), true, speed::move_y_with_speed)];
        9494365 => [forced::knockback_motion(&mut Runtime::new(game), false, speed::move_y_with_speed)];
        9494372 => [forced::knockback_motion(&mut Runtime::new(game), true, speed::no_speed_y)];
        9492174 => [{
            let mut runtime = Runtime::new(game);
            let frame = runtime.inputs().frame;
            forced::consume_crystal_flash(runtime.session_mut(), frame);
        }];
        9494565 => [forced::begin_bomb_jump(&mut Runtime::new(game))];
        9494578 => [forced::advance_bomb_jump(&mut Runtime::new(game), |h| h.foreign(|game| crate::rtl::invalid_interrupt(game)))];
        9494653 => [forced::finish_bomb_jump(Runtime::new(game).session_mut())];
        9496846 => [()];
    }
}

unsafe fn advance_shinespark(axes: &[player_collision::Axis], game: &mut Game) {
    word! {game; SAMUS_CONTACT_DAMAGE_INDEX = 2; SAMUS_HURT_FLASH_COUNTER = 8; }
    player_movement::record_echo(&mut Runtime::new(game));
    for &axis in axes {
        player_collision::shinespark(&mut CollisionRuntime::new(game), axis);
    }
    Samus_EndSuperJump(game);
    if word!(game.SAMUS_HEALTH) as i32 - 30i32 & 0x8000 == 0 && {
        (word!(game.SAMUS_HEALTH -= 1, reread) & 0x8000) != 0
    } {
        word!(game.SAMUS_HEALTH = 0);
    }
}

pub(crate) fn correct_previous_position(game: &mut Game) {
    use crate::ram_map::{checked_word, words as ram};
    for (current, previous) in [
        (ram::SAMUS_Y_POS, ram::SAMUS_PREV_Y_POS),
        (ram::SAMUS_X_POS, ram::SAMUS_PREV_X_POS),
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
