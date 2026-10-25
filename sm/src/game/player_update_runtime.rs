//! Compatibility sources and live aliases for the owned beta interaction.
use crate::{
    game::{
        player_transitions::{
            self as transitions,
            state::{FrameBetaMode, FrameGammaMode},
        },
        player_update::{self, MovementStep},
        player_update::{
            content::Content,
            effects::{Effects, GammaAction},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    game_state::Game,
    ram_map::checked_word as word,
    rom::RomRecord,
};

fn native_record(runtime: &mut Runtime<'_>, address: u32) -> RomRecord {
    if address & 0x8000 == 0 {
        runtime.foreign(|game| unsafe {
            crate::rtl::RomPtr(address, game);
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
impl Effects for Runtime<'_> {
    fn imported_exploration_byte(&self, offset: isize) -> u8 {
        self.byte(exploration_address(offset))
    }
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8) {
        self.set_byte(exploration_address(offset), value);
    }
    fn area_map_available(&self) -> bool {
        self.word(0x789) != 0
    }
    fn minimap_blink(&self) -> u8 {
        self.byte(0x5b5)
    }
    fn minimap_boss_id(&self) -> u16 {
        self.word(0x179c)
    }
    fn gamma_action(&self) -> GammaAction {
        match self.session().player.transitions.frame_gamma {
            FrameGammaMode::Empty | FrameGammaMode::Disabled => GammaAction::Inactive,
            mode => GammaAction::Legacy(mode.to_raw()),
        }
    }
    fn run_gamma(&mut self, selector: u16) {
        self.foreign(|game| unsafe {
            crate::game::sm_90::CallFrameHandlerGamma(0x900000 | u32::from(selector), game);
        });
    }
}

// Before Session admission, preserve contact clearing and the captured special
// callback even when the subsequent import rejects an already-held owner.
fn advance_beta(
    mode: FrameBetaMode,
    game: &mut Game,
) -> Result<(), crate::game::samus_periodic_damage::InvalidDamage> {
    word!(game.samus_contact_damage_index = 0);
    let handler = word!(game.samus_movement_handler);
    let movement = if handler == 0xa337 {
        MovementStep::Normal
    } else {
        unsafe {
            crate::game::sm_90::CallSamusMovementHandler(0x900000 | u32::from(handler), game);
        }
        MovementStep::AlreadyAdvanced
    };
    player_update::advance(&mut Runtime::new(game), mode, movement)
}

pub(crate) fn advance_frame(game: &mut Game) {
    use FrameBetaMode::*;
    let mode = FrameBetaMode::from_raw(word!(game.frame_handler_beta));
    match mode {
        Gameplay | Escape => {
            if advance_beta(Gameplay, game).is_err() {
                unsafe {
                    crate::game::sm_80::InvalidInterrupt_Crash(game);
                }
            }
            if word!(game.power_bomb_flag) == 0
                && word!(game.time_is_frozen_flag) == 0
                && word!(game.door_transition_flag_enemies) == 0
                && word!(game.area_index) != 6
                && word!(game.game_state) == 8
                && word!(game.joypad1_newkeys) & 0x1000 != 0
            {
                word!(game.screen_fade_delay = 1);
                word!(game.screen_fade_counter = 1);
                word!(game.game_state = 12);
            }
            low_health(game);
            if mode == Escape
                && FrameGammaMode::from_raw(word!(game.frame_handler_gamma))
                    == FrameGammaMode::DrawEscapeTimer
                && word!(game.game_state) != 35
            {
                word!(game.game_state = 35);
            }
        }
        DemoMapped | DemoUnmapped => {
            let _ = advance_beta(mode, game);
            // Read each backup after callbacks and after the preceding publication.
            word!(game.joypad1_lastkeys = word!(game.controller1_input_for_demo));
            word!(game.joypad1_newkeys = word!(game.controller1_new_input_for_demo));
            word!(game.joypad1_input_samusfilter = word!(game.demo_backup_prev_controller_input));
            word!(
                game.joypad1_newinput_samusfilter =
                    word!(game.demo_backup_prev_controller_input_new)
            );
        }
        ResumeRecorder if word!(game.joypad2_new_keys) & 0x80 != 0 => {
            word!(game.frame_handler_alfa = 0xe695);
            game.set_frame_handler_beta(Gameplay.to_raw());
            crate::game::enemy_projectiles::EnableEprojs(game);
            word!(game.time_is_frozen_flag = 0);
        }
        Fanfare => {
            crate::game::sm_90::Samus_SetRadius(game);
            {
                let h = &mut Runtime::new(game);
                player_update::minimap::update(h);
                transitions::advance_animation(h, transitions::state::AnimationStep::Gameplay);
            }
            word!(game.elevator_status = 0);
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
            match word!(game.substate) {
                0 => crate::game::music::QueueMusic_DelayedY(1, 0xe, game),
                5 => crate::game::music::PlayRoomMusicTrackAfterAFrames(0x168, game),
                _ => {}
            }
            if (word!(game.substate).wrapping_sub(359) as i16) < 0 {
                word!(game.substate += 1);
                return;
            }
            transitions::record_pose_history(Runtime::new(game).session_mut());
            word!(game.substate = 0);
            if (word!(game.debug_invincibility).wrapping_sub(7) as i16) < 0
                || word!(game.joypad2_last) & 0x8000 == 0
            {
                word!(game.debug_invincibility = 0);
            }
            word!(game.frame_handler_alfa = 0xe695);
            game.set_frame_handler_beta(Gameplay.to_raw());
        }
        Minimap => {
            word!(game.samus_contact_damage_index = 0);
            player_update::minimap::update(&mut Runtime::new(game));
        }
        AnimationOnly => {
            let _ = advance_beta(AnimationOnly, game);
        }
        LowHealth => low_health(game),
        Frozen | EquipmentFrozen | Drained | ResumeRecorder => {}
        Unknown(_) => unsafe {
            crate::rtl::Unreachable(game);
        },
    }
}

pub(crate) fn low_health(game: &mut Game) {
    if (game.samus_health().wrapping_sub(31) as i16) < 0 {
        if game.samus_health_warning() == 0 {
            crate::game::sm_80::QueueSfx3_Max6(2, game);
            game.set_samus_health_warning(1);
        }
    } else if game.samus_health_warning() != 0 {
        game.set_samus_health_warning(0);
        crate::game::sm_80::QueueSfx3_Max6(1, game);
    }
}
