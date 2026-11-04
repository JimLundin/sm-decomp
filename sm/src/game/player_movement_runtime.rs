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
            self.report_native_diagnostic(0x900000 | u32::from(cursor));
            // Preserve the actual optional native-binding capture and release.
            drop(self.samus_game().capture_native_rom_binding());
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
        self.owned_player_call(|runtime| {
            sm_game::player_projectiles::animation::initialize(runtime, slot, Initialization::Echo)
        });
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

/// Samus's Ceres push on the lease, then the escape countdown: its owner, the
/// timer status and the objects are the lease's. The owned call keeps the
/// former foreign entry's observation refresh.
pub(crate) fn ceres_movement(runtime: &mut Runtime<'_>, action: fn(&mut Runtime<'_>)) {
    action(runtime);
    runtime.owned_player_call(|runtime| {
        sm_game::escape_timer::advance(runtime);
        if sm_game::session_words::current(runtime).timer_status != 0 {
            sm_game::escape_timer::draw(runtime);
        }
    });
}

use sm_game::explosion_window::spawn_crystal_flash;
// Raw movement identities remain at the compatibility boundary; the handlers run on the
// lease Samus's frame holds.
use {
    crate::game::{
        player_collision::operations as player_collision,
        player_movement::{self, forced, speed},
        prelude::{K_MOVEMENT_TYPE_0E_TURNING_AROUND_ON_GROUND, abs16, word},
        visual_runtime::set_hdma_enabled,
    },
    crate::game_state::Game,
    sm_game::effects::SoundEffects,
    sm_game::projectile_initialization::Initialization,
    sm_game::ram::CompatibilityRam,
};
fn samus_end_super_jump(runtime: &mut Runtime<'_>) -> u8 {
    if word!(runtime.SAMUS_HEALTH).wrapping_sub(30) & 0x8000 == 0
        && word!(runtime.SAMUS_COLLISION_FLAG) == 0
    {
        return 0;
    }
    forced::end_shinespark(runtime)
}
static KNOCKBACK_MOTION: [Option<fn(&mut Runtime<'_>)>; 6] = [
    None,
    Some(|h| forced::knockback_motion(h, true, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, true, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, false, speed::move_y_with_speed)),
    Some(|h| forced::knockback_motion(h, true, speed::no_speed_y)),
    Some(|h| forced::knockback_motion(h, true, speed::no_speed_y)),
];

/// Samus's special movement handler at `ea` ($90 bank), on the lease her frame holds.
pub(crate) fn advance_special_movement(ea: u32, runtime: &mut Runtime<'_>) {
    match ea {
        0x90946e => release_from_grapple(runtime),
        0x9094cb => drained_crouching(runtime),
        0x90a337 => player_movement::advance_normal(runtime),
        0x90d678 => crystal_flash_start(runtime),
        0x90df38 => match KNOCKBACK_MOTION[usize::from(word!(runtime.KNOCKBACK_DIR))] {
            Some(motion) => {
                motion(runtime);
                word!(runtime.INPUT_TO_POSE_CALC = 0);
            }
            None => crate::rtl::invalid_interrupt_failure(),
        },
        0x90d75b => crystal_flash_finish(runtime),
        0x90e94f => xray(runtime),
        0x90f04b => shinespark_projectile_response(runtime),
        0x90f072 => {
            if word!(runtime.SAMUS_SHINE_TIMER) == 0 {
                word!(runtime.FRAME_HANDLER_ALFA = 0xe695);
                word!(runtime.SAMUS_MOVEMENT_HANDLER = 0xa337);
            }
        }
        0x90d068 => forced::shinespark_windup(runtime),
        0x90d346 => forced::shinespark_crash(runtime),
        0x90d383 => forced::expand_shinespark_crash(runtime.session_mut()),
        0x90d396 => forced::rotate_shinespark_crash(runtime.session_mut()),
        0x90d3cc => forced::contract_shinespark_crash(runtime.session_mut()),
        0x90d3f3 => forced::wait_shinespark_crash(runtime.session_mut()),
        0x90d40d => forced::finish_shinespark_crash(runtime),
        0x90d0ab => advance_shinespark(&[player_collision::Axis::Y], runtime),
        0x90d0d7 => advance_shinespark(
            &[player_collision::Axis::X, player_collision::Axis::Y],
            runtime,
        ),
        0x90d106 => advance_shinespark(&[player_collision::Axis::X], runtime),
        0x90df50 => crate::rtl::invalid_interrupt_failure(),
        0x90df53 => forced::knockback_motion(runtime, true, speed::move_y_with_speed),
        0x90df5d => forced::knockback_motion(runtime, false, speed::move_y_with_speed),
        0x90df64 => forced::knockback_motion(runtime, true, speed::no_speed_y),
        0x90d6ce => {
            let frame = runtime.inputs().frame;
            forced::consume_crystal_flash(runtime.session_mut(), frame);
        }
        0x90e025 => forced::begin_bomb_jump(runtime),
        0x90e032 => forced::advance_bomb_jump(runtime, |_| crate::rtl::invalid_interrupt_failure()),
        0x90e07d => forced::finish_bomb_jump(runtime.session_mut()),
        0x90e90e => {}
        _ => crate::rtl::unreachable(),
    }
}

fn release_from_grapple(runtime: &mut Runtime<'_>) {
    if word!(runtime.SAMUS_Y_DIR) == 1 && (word!(runtime.SAMUS_Y_SPEED) & 0x8000) != 0 {
        word! { runtime, 0; SAMUS_Y_SUBSPEED, SAMUS_Y_SPEED }
        word! {runtime; SAMUS_Y_DIR = 2; SAMUS_MOVEMENT_HANDLER = 0xa337; }
    }
    word!(runtime.SAMUS_X_ACCEL_MODE = 2);
    let profile = player_movement::speed::grapple_speed_profile(runtime);
    let amount = player_movement::speed::base_speed(runtime, profile);
    let request = player_movement::speed::horizontal_air::<false>(runtime, amount);
    player_collision::horizontal_control(runtime, request);
    player_movement::speed::move_y_with_speed(runtime);
    if word!(runtime.INPUT_TO_POSE_CALC) != 0 {
        word!(runtime.SAMUS_MOVEMENT_HANDLER = 0xa337);
    }
}

fn drained_crouching(runtime: &mut Runtime<'_>) {
    player_movement::speed::move_y_with_speed(runtime);
    if word!(runtime.SAMUS_COLLISION_FLAG) != 0 {
        word! {runtime; SAMUS_MOVEMENT_HANDLER = 0xa337;
        SAMUS_ANIM_FRAME_TIMER = 8; SAMUS_ANIM_FRAME = 7; }
        word! { runtime, 0; SAMUS_Y_SUBSPEED, SAMUS_Y_SPEED }
    }
}

fn crystal_flash_start(runtime: &mut Runtime<'_>) {
    word!(runtime.SAMUS_Y_POS -= 2);
    if (word!(runtime.TIMER_FOR_SHINESPARKS_STARTSTOP -= 1, reread) & 0x8000) != 0 {
        word! {runtime; SAMUS_ANIM_FRAME_TIMER = 3; SAMUS_ANIM_FRAME = 6; }
        let start = word!(runtime.SAMUS_Y_POS);
        runtime.write_slice_native(0xdf0, start);
        word!(runtime.SAMUS_MOVEMENT_HANDLER = 0xd6ce);
        word! { runtime, 0; SAMUS_INVINCIBILITY_TIMER, SAMUS_KNOCKBACK_TIMER }
        runtime.queue_sound(2, 1, 15);
        word!(runtime.POWER_BOMB_FLAG = 0);
        word!(runtime.POWER_BOMB_EXPLOSION_X_POS = word!(runtime.SAMUS_X_POS));
        word!(runtime.POWER_BOMB_EXPLOSION_Y_POS = word!(runtime.SAMUS_Y_POS));
        set_hdma_enabled(true, runtime);
        // The explosion words and the HDMA objects are the lease's; the owned
        // call keeps the former foreign entry's observation refresh.
        runtime.owned_player_call(spawn_crystal_flash);
    }
}

fn crystal_flash_finish(runtime: &mut Runtime<'_>) {
    if word!(runtime.SAMUS_Y_POS) != runtime.native_word(0xdf0) {
        word!(runtime.SAMUS_Y_POS += 1);
    }
    if runtime.byte(0xa1f) == 0 {
        word! {runtime; POWER_BOMB_FLAG = 0; SAMUS_SHINE_TIMER = 0xffff;
        SAMUS_MOVEMENT_HANDLER = 0xa337; }
        if word!(runtime.SAMUS_INPUT_HANDLER) != 0xe91d {
            word!(runtime.SAMUS_INPUT_HANDLER = 0xe913);
            word! { runtime, 0; SAMUS_INVINCIBILITY_TIMER, SAMUS_KNOCKBACK_TIMER }
        }
    }
}

fn xray(runtime: &mut Runtime<'_>) {
    if u32::from(runtime.byte(0xa1f)) != K_MOVEMENT_TYPE_0E_TURNING_AROUND_ON_GROUND {
        word!(runtime.SAMUS_ANIM_FRAME_TIMER = 15);
        let facing = runtime.byte(0xa1e);
        let frame = player_movement::xray_animation_frame(facing, || word!(runtime.XRAY_ANGLE));
        word!(runtime.SAMUS_ANIM_FRAME = frame);
    }
}

fn shinespark_projectile_response(runtime: &mut Runtime<'_>) {
    if word!(runtime.SAMUS_SHINE_TIMER) == 0 {
        word!(runtime.SPECIAL_SAMUS_PALETTE_FRAME = 6);
        word! { runtime, 1; SPECIAL_SAMUS_PALETTE_TIMER, DEMO_TIMER_COUNTER }
        word! {runtime; TIMER_FOR_SHINE_TIMER = 10; SAMUS_SHINE_TIMER = 120;
        SAMUS_MOVEMENT_HANDLER = 0xf072; }
    }
}

fn advance_shinespark(axes: &[player_collision::Axis], runtime: &mut Runtime<'_>) {
    word! {runtime; SAMUS_CONTACT_DAMAGE_INDEX = 2; SAMUS_HURT_FLASH_COUNTER = 8; }
    player_movement::record_echo(runtime);
    for &axis in axes {
        player_collision::shinespark(runtime, axis);
    }
    samus_end_super_jump(runtime);
    if word!(runtime.SAMUS_HEALTH).wrapping_sub(30) & 0x8000 == 0
        && (word!(runtime.SAMUS_HEALTH -= 1, reread) & 0x8000) != 0
    {
        word!(runtime.SAMUS_HEALTH = 0);
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
