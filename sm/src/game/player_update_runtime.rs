//! Compatibility sources and live aliases for the owned beta interaction.
use crate::{
    game::{
        player_transitions::{
            self as transitions,
            state::{FrameBetaMode, FrameGammaMode},
        },
        player_update::{self, MovementStep},
        player_update::{content::Content, effects::Effects},
        session_runtime::{Runtime, native_rom_diagnostic},
        visual_host::{self, Source},
    },
    game_state::Game,
    ram_map::checked_word as word,
    rom::RomRecord,
};

fn native_record(runtime: &mut Runtime<'_>, address: u32) -> RomRecord {
    if address & 0x8000 == 0 {
        runtime.foreign(|game| {
            native_rom_diagnostic(address, game);
            // Acquire and release the actual binding before Runtime imports again.
            drop(game.capture_native_rom_binding());
        });
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
        self.foreign(|game| unsafe {
            advance_gamma(0x900000 | u32::from(mode.to_raw()), game);
        });
    }
}

// Before Session admission, preserve contact clearing and the captured special
// callback even when the subsequent import rejects an already-held owner.
fn advance_beta(
    mode: FrameBetaMode,
    game: &mut Game,
) -> Result<(), sm_game::samus_periodic_damage::InvalidDamage> {
    word!(game.SAMUS_CONTACT_DAMAGE_INDEX = 0);
    let handler = word!(game.SAMUS_MOVEMENT_HANDLER);
    let movement = if handler == 0xa337 {
        MovementStep::Normal
    } else {
        unsafe {
            crate::game::session_runtime::advance_special_movement(
                0x900000 | u32::from(handler),
                game,
            );
        }
        MovementStep::AlreadyAdvanced
    };
    player_update::advance(&mut Runtime::new(game), mode, movement)
}

pub(crate) fn advance_frame(game: &mut Game) {
    use FrameBetaMode::*;
    let mode = FrameBetaMode::from_raw(word!(game.FRAME_HANDLER_BETA));
    match mode {
        Gameplay | Escape => {
            if advance_beta(Gameplay, game).is_err() {
                {
                    crate::rtl::invalid_interrupt(game);
                }
            }
            if word!(game.POWER_BOMB_FLAG) == 0
                && word!(game.TIME_IS_FROZEN_FLAG) == 0
                && word!(game.DOOR_TRANSITION_FLAG_ENEMIES) == 0
                && word!(game.AREA_INDEX) != 6
                && word!(game.GAME_STATE) == 8
                && word!(game.JOYPAD1_NEWKEYS) & 0x1000 != 0
            {
                word!(game.SCREEN_FADE_DELAY = 1);
                word!(game.SCREEN_FADE_COUNTER = 1);
                word!(game.GAME_STATE = 12);
            }
            low_health(game);
            if mode == Escape
                && FrameGammaMode::from_raw(word!(game.FRAME_HANDLER_GAMMA))
                    == FrameGammaMode::DrawEscapeTimer
                && word!(game.GAME_STATE) != 35
            {
                word!(game.GAME_STATE = 35);
            }
        }
        DemoMapped | DemoUnmapped => {
            let _ = advance_beta(mode, game);
            // Read each backup after callbacks and after the preceding publication.
            word!(game.JOYPAD1_LASTKEYS = word!(game.CONTROLLER1_INPUT_FOR_DEMO));
            word!(game.JOYPAD1_NEWKEYS = word!(game.CONTROLLER1_NEW_INPUT_FOR_DEMO));
            word!(game.JOYPAD1_INPUT_SAMUSFILTER = word!(game.DEMO_BACKUP_PREV_CONTROLLER_INPUT));
            word!(
                game.JOYPAD1_NEWINPUT_SAMUSFILTER =
                    word!(game.DEMO_BACKUP_PREV_CONTROLLER_INPUT_NEW)
            );
        }
        ResumeRecorder if word!(game.JOYPAD2_NEW_KEYS) & 0x80 != 0 => {
            word!(game.FRAME_HANDLER_ALFA = 0xe695);
            game.set_frame_handler_beta(Gameplay.to_raw());
            crate::game::enemy_projectiles::enable_eprojs(game);
            word!(game.TIME_IS_FROZEN_FLAG = 0);
        }
        Fanfare => {
            crate::game::session_runtime::set_player_radius(game);
            {
                let h = &mut Runtime::new(game);
                player_update::minimap::update(h);
                transitions::advance_animation(h, transitions::state::AnimationStep::Gameplay);
            }
            word!(game.ELEVATOR_STATUS = 0);
            word!(game.SAMUS_PREV_Y_POS = word!(game.SAMUS_Y_POS));
            match word!(game.SUBSTATE) {
                0 => crate::game::music::queue_music_delayed_y(1, 0xe, game),
                5 => crate::game::music::play_room_music_track_after_aframes(0x168, game),
                _ => {}
            }
            if (word!(game.SUBSTATE).wrapping_sub(359) as i16) < 0 {
                word!(game.SUBSTATE += 1);
                return;
            }
            transitions::record_pose_history(Runtime::new(game).session_mut());
            word!(game.SUBSTATE = 0);
            if (word!(game.DEBUG_INVINCIBILITY).wrapping_sub(7) as i16) < 0
                || word!(game.JOYPAD2_LAST) & 0x8000 == 0
            {
                word!(game.DEBUG_INVINCIBILITY = 0);
            }
            word!(game.FRAME_HANDLER_ALFA = 0xe695);
            game.set_frame_handler_beta(Gameplay.to_raw());
        }
        Minimap => {
            word!(game.SAMUS_CONTACT_DAMAGE_INDEX = 0);
            player_update::minimap::update(&mut Runtime::new(game));
        }
        AnimationOnly => {
            let _ = advance_beta(AnimationOnly, game);
        }
        LowHealth => low_health(game),
        Frozen | EquipmentFrozen | Drained | ResumeRecorder => {}
        Unknown(_) => {
            crate::rtl::report_unreachable(game);
        }
    }
}

pub(crate) fn low_health(game: &mut Game) {
    if (game.samus_health().wrapping_sub(31) as i16) < 0 {
        if game.samus_health_warning() == 0 {
            crate::game::music::queue_sfx3_max6(2, game);
            game.set_samus_health_warning(1);
        }
    } else if game.samus_health_warning() != 0 {
        game.set_samus_health_warning(0);
        crate::game::music::queue_sfx3_max6(1, game);
    }
}

// Retained gamma effects; uncovered drain/timer bodies keep their original order.
use crate::game::{
    palette_effects,
    player_movement::forced,
    prelude::{K_BUTTON_UP, K_GAME_STATE_35_TIME_UP, K_POSE_E9_FACE_L_DRAINED_CROUCH_FALLING},
};
use crate::ram_map::word as legacy_word;
use sm_game::escape_timer;
crate::game::callback_dispatch! {
    #[allow(unused_unsafe)]
    unsafe fn advance_gamma(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::report_unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [9494683] samus_func1 [] {
        if legacy_word!(game.SAMUS_POSE) as i32 == K_POSE_E9_FACE_L_DRAINED_CROUCH_FALLING as i32
            && legacy_word!(game.SAMUS_ANIM_FRAME) as i32 - 8i32 & 0x8000 == 0
            && (legacy_word!(game.JOYPAD1_NEWKEYS) as i32 & K_BUTTON_UP as i32) != 0
        {
            legacy_word! {game; SAMUS_ANIM_FRAME_TIMER = 1; SAMUS_ANIM_FRAME = 13;
                FRAME_HANDLER_GAMMA = ((0x90e90ei32) as u16); }
        }
    };
    [9494725] samus_func2 [] {
        if legacy_word!(game.SAMUS_ANIM_FRAME) as i32 - 8i32 & 0x8000 == 0
            && legacy_word!(game.SAMUS_ANIM_FRAME) as i32 - 12i32 & 0x8000 != 0
            && (legacy_word!(game.JOYPAD1_NEWKEYS) as i32 & K_BUTTON_UP as i32) != 0
        {
            legacy_word! {game; SAMUS_ANIM_FRAME_TIMER = 1; SAMUS_ANIM_FRAME = 18; }
        }
    };
    [9494758] samus_func3 [] {
        if escape_timer::advance(game) & 1 != 0 {
            legacy_word!(game.GAME_STATE = K_GAME_STATE_35_TIME_UP as u16);
            word_transfers!(game, descending, 256; le(TARGET_PALETTES, 255) <- value(0x7fff));
            legacy_word!(game.FRAME_HANDLER_GAMMA = (0x90e114i32) as u16);
            palette_effects::set_enabled(false, game);
        }
        if legacy_word!(game.TIMER_STATUS) != 0 {
            escape_timer::draw(game);
        }
    };
    [9494804] draw_timer [] {
        escape_timer::draw(game);
    };
    @external {
        9495201 => [forced::struggle_from_draygon(&mut Runtime::new(game))];
        9494830 => [crate::game::session_runtime::ceres_movement(game, |h| forced::push_out_of_ceres(h))];
        9494984 => [crate::game::session_runtime::ceres_movement(game, |h| forced::advance_ceres_push(h))];
        9495459 => [forced::finish_forced_fall(&mut Runtime::new(game))];
        9495579 => [forced::animate_fall(Runtime::new(game).session_mut())];
        9495422 => [()];
        9496846 => [()];
    }
}
