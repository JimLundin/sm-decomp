//! Compatibility sources and live aliases for the owned beta interaction.
use crate::{
    game::{
        player_transitions::{
            self as transitions,
            state::{FrameBetaMode, FrameGammaMode},
        },
        player_update::{self, MovementStep},
        player_update::{content::Content, effects::Effects},
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    ram_map::checked_word as word,
    rom::RomRecord,
};

fn native_record(runtime: &mut Runtime<'_>, address: u32) -> RomRecord {
    if address & 0x8000 == 0 {
        runtime.report_native_diagnostic(address);
        // Acquire and release the actual binding, as the foreign call did.
        drop(runtime.samus_game().capture_native_rom_binding());
    }
    visual_host::Content::table(runtime, Source::Native, address)
}

impl Content for Runtime<'_> {
    type BossCoordinates = u16;
    fn area_exploration_mask(&mut self, area: u16) -> RomRecord {
        let table = visual_host::Content::table(self, Source::Loaded, 0x829717);
        let cursor = table.word(usize::from(area) * 2);
        native_record(self, 0x820000 | u32::from(cursor))
    }
    fn area_map_tiles(&mut self, area: u16) -> RomRecord {
        let table = visual_host::Content::table(self, Source::Native, 0x82964a);
        // Load24 reads the fourth adjacent byte before masking it away.
        let address = table.load24(usize::from(area) * 3);
        native_record(self, address)
    }
    fn boss_map_id(&self, index: usize) -> u16 {
        visual_host::Content::table(self, Source::Native, 0x90a83a).word(index * 4)
    }
    fn boss_coordinates(&self, index: usize) -> u16 {
        visual_host::Content::table(self, Source::Native, 0x90a83a).word(index * 4 + 2)
    }
    fn boss_coordinate(&mut self, coordinates: u16, index: usize) -> RomRecord {
        let cursor = coordinates.wrapping_add((index * 4) as u16);
        native_record(self, 0x900000 | u32::from(cursor))
    }
}

fn exploration_address(offset: isize) -> usize {
    0x7f7usize
        .checked_add_signed(offset)
        .expect("exploration offset overflow")
}
impl sm_game::progress::exploration::ExplorationHost for Runtime<'_> {
    fn minimap(&self) -> &sm_game::player_update::state::State {
        &self.session().minimap
    }
    fn minimap_mut(&mut self) -> &mut sm_game::player_update::state::State {
        &mut self.session_mut().minimap
    }
    fn imported_exploration_byte(&self, offset: isize) -> u8 {
        self.byte(exploration_address(offset))
    }
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8) {
        self.set_byte(exploration_address(offset), value);
    }
    fn map_origin(&self) -> (u16, u16) {
        let entry = &self.session().room.entry;
        (entry.map_x, entry.map_y)
    }
}
impl Effects for Runtime<'_> {
    fn area_map_available(&self) -> bool {
        self.word(0x789) != 0
    }
    fn minimap_blink(&self) -> u8 {
        self.byte(0x5b5)
    }
    fn minimap_boss_id(&self) -> u16 {
        self.word(0x179c)
    }
    fn run_gamma(&mut self, mode: FrameGammaMode) {
        // The gamma actions run on this lease. The observations are refreshed on entry and on
        // return, as the former foreign call left them.
        self.owned_player_call(|runtime| advance_gamma(mode, runtime));
    }
}

// Before the beta work, clear contact damage and run a special movement handler, which
// selects its own movement step.
fn advance_beta(
    mode: FrameBetaMode,
    runtime: &mut Runtime<'_>,
) -> Result<(), sm_game::samus_periodic_damage::InvalidDamage> {
    word!(runtime.SAMUS_CONTACT_DAMAGE_INDEX = 0);
    let handler = word!(runtime.SAMUS_MOVEMENT_HANDLER);
    let movement = if handler == 0xa337 {
        MovementStep::Normal
    } else {
        crate::game::session_runtime::advance_special_movement(
            0x900000 | u32::from(handler),
            runtime,
        );
        MovementStep::AlreadyAdvanced
    };
    player_update::advance(runtime, mode, movement)
}

/// Samus's frame (the frame handler's beta, $90:E725 and the others), on the lease the
/// caller holds.
pub(crate) fn advance_frame(runtime: &mut Runtime<'_>) {
    use FrameBetaMode::*;
    let mode = FrameBetaMode::from_raw(word!(runtime.FRAME_HANDLER_BETA));
    match mode {
        Gameplay | Escape => {
            if advance_beta(Gameplay, runtime).is_err() {
                crate::rtl::invalid_interrupt_failure();
            }
            if word!(runtime.POWER_BOMB_FLAG) == 0
                && word!(runtime.TIME_IS_FROZEN_FLAG) == 0
                && word!(runtime.DOOR_TRANSITION_FLAG_ENEMIES) == 0
                && word!(runtime.AREA_INDEX) != 6
                && word!(runtime.GAME_STATE) == 8
                && word!(runtime.JOYPAD1_NEWKEYS) & 0x1000 != 0
            {
                sm_game::screen_fade::store(runtime, 1, 1);
                word!(runtime.GAME_STATE = 12);
            }
            low_health_on(runtime);
            if mode == Escape
                && FrameGammaMode::from_raw(word!(runtime.FRAME_HANDLER_GAMMA))
                    == FrameGammaMode::DrawEscapeTimer
                && word!(runtime.GAME_STATE) != 35
            {
                word!(runtime.GAME_STATE = 35);
            }
        }
        DemoMapped | DemoUnmapped => {
            let _ = advance_beta(mode, runtime);
            // Read each backup after callbacks and after the preceding publication.
            word!(runtime.JOYPAD1_LASTKEYS = word!(runtime.CONTROLLER1_INPUT_FOR_DEMO));
            word!(runtime.JOYPAD1_NEWKEYS = word!(runtime.CONTROLLER1_NEW_INPUT_FOR_DEMO));
            word!(
                runtime.JOYPAD1_INPUT_SAMUSFILTER =
                    word!(runtime.DEMO_BACKUP_PREV_CONTROLLER_INPUT)
            );
            word!(
                runtime.JOYPAD1_NEWINPUT_SAMUSFILTER =
                    word!(runtime.DEMO_BACKUP_PREV_CONTROLLER_INPUT_NEW)
            );
        }
        ResumeRecorder if word!(runtime.JOYPAD2_NEW_KEYS) & 0x80 != 0 => {
            word!(runtime.FRAME_HANDLER_ALFA = 0xe695);
            word!(runtime.FRAME_HANDLER_BETA = Gameplay.to_raw());
            sm_game::enemy_projectiles::lifecycle::set_enabled(runtime, true);
            word!(runtime.TIME_IS_FROZEN_FLAG = 0);
        }
        Fanfare => {
            crate::game::session_runtime::set_player_radius(runtime);
            player_update::minimap::update(runtime);
            transitions::advance_animation(runtime, transitions::state::AnimationStep::Gameplay);
            word!(runtime.ELEVATOR_STATUS = 0);
            word!(runtime.SAMUS_PREV_Y_POS = word!(runtime.SAMUS_Y_POS));
            match word!(runtime.SUBSTATE) {
                0 => runtime.queue_music_delayed(1, 0xe),
                5 => runtime.queue_room_music_after(0x168),
                _ => {}
            }
            if (word!(runtime.SUBSTATE).wrapping_sub(359) as i16) < 0 {
                word!(runtime.SUBSTATE += 1);
                return;
            }
            transitions::record_pose_history(runtime.session_mut());
            word!(runtime.SUBSTATE = 0);
            if (word!(runtime.DEBUG_INVINCIBILITY).wrapping_sub(7) as i16) < 0
                || word!(runtime.JOYPAD2_LAST) & 0x8000 == 0
            {
                word!(runtime.DEBUG_INVINCIBILITY = 0);
            }
            word!(runtime.FRAME_HANDLER_ALFA = 0xe695);
            word!(runtime.FRAME_HANDLER_BETA = Gameplay.to_raw());
        }
        Minimap => {
            word!(runtime.SAMUS_CONTACT_DAMAGE_INDEX = 0);
            player_update::minimap::update(runtime);
        }
        AnimationOnly => {
            let _ = advance_beta(AnimationOnly, runtime);
        }
        LowHealth => low_health_on(runtime),
        Frozen | EquipmentFrozen | Drained | ResumeRecorder => {}
        Unknown(_) => crate::rtl::unreachable(),
    }
}

/// The low health warning on the lease Samus's frame holds.
pub(crate) fn low_health_on(runtime: &mut Runtime<'_>) {
    if (word!(runtime.SAMUS_HEALTH).wrapping_sub(31) as i16) < 0 {
        if word!(runtime.SAMUS_HEALTH_WARNING) == 0 {
            runtime.queue_sound(2, 2, 6);
            word!(runtime.SAMUS_HEALTH_WARNING = 1);
        }
    } else if word!(runtime.SAMUS_HEALTH_WARNING) != 0 {
        word!(runtime.SAMUS_HEALTH_WARNING = 0);
        runtime.queue_sound(2, 1, 6);
    }
}

// The gamma actions, on the lease.
use crate::game::{
    player_movement::forced,
    prelude::{K_BUTTON_UP, K_GAME_STATE_35_TIME_UP, K_POSE_E9_FACE_L_DRAINED_CROUCH_FALLING},
};
use crate::projection::ProjectedBytes;
use crate::ram_map::word as legacy_word;
use sm_game::effects::SoundEffects;
use sm_game::escape_timer;
fn advance_gamma(mode: FrameGammaMode, runtime: &mut Runtime<'_>) {
    use FrameGammaMode::*;
    match mode {
        RecoverFromDrain => {
            if u32::from(legacy_word!(runtime.SAMUS_POSE))
                == K_POSE_E9_FACE_L_DRAINED_CROUCH_FALLING
                && legacy_word!(runtime.SAMUS_ANIM_FRAME).wrapping_sub(8) & 0x8000 == 0
                && (u32::from(legacy_word!(runtime.JOYPAD1_NEWKEYS)) & K_BUTTON_UP) != 0
            {
                legacy_word! {runtime; SAMUS_ANIM_FRAME_TIMER = 1; SAMUS_ANIM_FRAME = 13;
                FRAME_HANDLER_GAMMA = Disabled.to_raw(); }
            }
        }
        StruggleDuringDrain => {
            if legacy_word!(runtime.SAMUS_ANIM_FRAME).wrapping_sub(8) & 0x8000 == 0
                && legacy_word!(runtime.SAMUS_ANIM_FRAME).wrapping_sub(12) & 0x8000 != 0
                && (u32::from(legacy_word!(runtime.JOYPAD1_NEWKEYS)) & K_BUTTON_UP) != 0
            {
                legacy_word! {runtime; SAMUS_ANIM_FRAME_TIMER = 1; SAMUS_ANIM_FRAME = 18; }
            }
        }
        // The owned call keeps the former foreign entry's observation refresh.
        EscapeTimer | DrawEscapeTimer => {
            runtime.owned_player_call(|runtime| escape_countdown(mode, runtime))
        }
        BeginCeresPush => {
            crate::game::session_runtime::ceres_movement(runtime, |h| forced::push_out_of_ceres(h))
        }
        CeresPush => {
            crate::game::session_runtime::ceres_movement(runtime, |h| forced::advance_ceres_push(h))
        }
        DraygonGrabbed => forced::struggle_from_draygon(runtime),
        ForcedFall => forced::finish_forced_fall(runtime),
        CappedFallAnimation => forced::animate_fall(runtime.session_mut()),
        Empty | Disabled => {}
        Unknown(_) => crate::rtl::unreachable(),
    }
}

/// The escape countdown's gamma actions on the lease: advance and draw it, or
/// only draw it. Time up stores the game state, the white target palettes, the
/// draw-only gamma action and the disabled palette effects, in that order.
fn escape_countdown(mode: FrameGammaMode, runtime: &mut Runtime<'_>) {
    if mode == FrameGammaMode::EscapeTimer {
        if escape_timer::advance(runtime) & 1 != 0 {
            legacy_word!(runtime.GAME_STATE = K_GAME_STATE_35_TIME_UP as u16);
            // Every target colour white, from the last colour down.
            for index in (0..256).rev() {
                sm_game::projection::Views::set_view_word(
                    runtime,
                    sm_game::projection::Field::PaletteTarget(index),
                    0x7fff,
                );
            }
            legacy_word!(runtime.FRAME_HANDLER_GAMMA = FrameGammaMode::DrawEscapeTimer.to_raw());
            sm_game::palette_effects::execution::set_enabled(runtime.session_mut(), false);
        }
        if sm_game::session_words::current(runtime).timer_status != 0 {
            escape_timer::draw(runtime);
        }
    } else {
        escape_timer::draw(runtime);
    }
}
