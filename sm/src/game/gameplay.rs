//! Game's implementation of the gameplay frame phases' host.
//!
//! Each method is the original call the phase made. The callees keep their
//! unsafe backend contracts: the frame loop only advances gameplay with the
//! backend installed and exclusively accessed.
use crate::game::display_refresh;
use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use sm_game::gameplay_frame::{NativeState, PauseHook, UnpauseHook};
use sm_game::room_entry::api::EntrySignal;

pub(crate) use sm_game::gameplay_frame::{Entry, advance};

impl sm_game::gameplay_frame::Host for Game {
    fn begin_session_interval(&mut self, fresh: bool) {
        Game::begin_session_interval(self, fresh).expect("Session interval entry");
    }
    fn resume_point(&self) -> u8 {
        self.coroutine_state_1()
    }
    fn set_resume_point(&mut self, point: u8) {
        self.set_coroutine_state_1(point);
    }

    fn resume_boot(&mut self) -> u8 {
        unsafe { crate::game::boot::resume(self) }
    }
    fn fail_frame_state(&mut self) {
        unsafe { crate::host::Die(c"Incorrect coroutine_state_0".as_ptr()) }
    }
    fn report_broken_coroutine(&mut self, state: [u8; 4]) {
        unsafe {
            printf(
                c"Coroutine state: %d, %d, %d, %d\n".as_ptr(),
                state[0] as i32,
                state[1] as i32,
                state[2] as i32,
                state[3] as i32,
            );
            crate::host::Warning(c"Coroutine State is broken!".as_ptr());
        }
    }
    fn update_hdma(&mut self) {
        unsafe { crate::game::visual_runtime::update_hdma(self) }
    }
    fn advance_random(&mut self) {
        self.next_random();
    }
    fn advance_native_state(&mut self, state: NativeState) -> u8 {
        use crate::game::samus_animation::{
            advance_death_pose, begin_death, explode_suit, flash_death,
        };
        use crate::game::sm_8b::{
            GameState_1_OpeningCinematic_, GameState_37_CeresGoesBoomWithSamus_,
            GameState_39_EndingAndCredits_,
        };
        match state {
            NativeState::OpeningCinematic => GameState_1_OpeningCinematic_(self),
            NativeState::CeresGoesBoom => GameState_37_CeresGoesBoomWithSamus_(self),
            NativeState::EndingAndCredits => GameState_39_EndingAndCredits_(self),
            NativeState::BeginDeath => begin_death(self),
            NativeState::DeathPose => advance_death_pose(self),
            NativeState::FlashDeath => flash_death(self),
            NativeState::ExplodeSuit => explode_suit(self),
        }
    }
    fn resume_room_entry(&mut self, signal: EntrySignal) -> u8 {
        crate::game::room_entry::protocol::resume(&mut Runtime::new(self), signal).returned_byte()
    }
    fn debug_game_over_menu(&mut self) {
        unsafe { crate::cpu_infra::DebugGameOverMenu() }
    }
    fn upload_options_tiles(&mut self) {
        let game = self;
        unsafe {
            crate::rtl::register_writes! { game;
                mirror(VMADDL as u16, 0, 0x72, true); byte(VMADDH as u16, 0x58);
                byte(VMAIN as u16, 0x80);
            }
            display_refresh::setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x8edc00, 0x800, game);
            WriteReg(MDMAEN as u16, 2, game);
        }
    }
    fn advance_sound_effects(&mut self) {
        crate::game::music::advance_sound_effects(self, |channel, value, meaning, game| unsafe {
            crate::native_audio::emit(
                crate::native_audio::sound_cue(
                    channel,
                    sm_data::OriginalSoundEffect(value),
                    meaning,
                ),
                sm_data::CuePhase::GameplaySound,
                game,
            )
        });
    }
    fn refresh_display(&mut self) {
        unsafe { display_refresh::run(self) }
    }

    fn initialize_gameplay_display(&mut self) {
        unsafe { display_refresh::initialize_gameplay(self) }
    }
    fn initialize_player(&mut self) {
        crate::game::session_runtime::initialize_player(self);
    }
    fn flush_tilemaps(&mut self) {
        unsafe { display_refresh::flush_tilemaps(self) }
    }

    fn load_title_sequence_graphics(&mut self) {
        crate::game::sm_8b::LoadTitleSequenceGraphics(self);
    }
    fn disable_scanline_irq(&mut self) {
        crate::game::scanline_irq::disable(self);
    }
    fn enable_scanline_irq(&mut self, immediate: bool) {
        unsafe { crate::game::scanline_irq::enable(immediate, self) }
    }
    fn reset_layer_mirrors(&mut self) {
        display_refresh::reset_layer_mirrors(self);
    }

    fn capture_pause_background(&mut self) {
        unsafe { display_refresh::capture_pause_background(self) }
    }
    fn pause_hook(&mut self, hook: PauseHook) {
        match hook {
            PauseHook::Draygon => crate::game::room_state::pause_draygon(self),
            PauseHook::Kraid => crate::game::sm_a7::PauseHook_Kraid(self),
        }
    }
    fn unpause_hook(&mut self, hook: UnpauseHook) -> u8 {
        use crate::game::sm_a7::{
            Kraid_UnpauseHook_IsSinking, UnpauseHook_Kraid_IsAlive, UnpauseHook_Kraid_IsDead,
        };
        match hook {
            UnpauseHook::Draygon => crate::game::room_state::unpause_draygon(self),
            UnpauseHook::KraidDead => UnpauseHook_Kraid_IsDead(self),
            UnpauseHook::KraidAlive => UnpauseHook_Kraid_IsAlive(self),
            UnpauseHook::KraidSinking => Kraid_UnpauseHook_IsSinking(self),
            UnpauseHook::MotherBrain => crate::game::sm_a9::MotherBrainsBody_UnpauseHook(self),
        }
    }
    fn load_pause_assets(&mut self) {
        unsafe { display_refresh::load_pause_assets(self) }
    }
    fn upload_pause_map(&mut self) {
        unsafe { display_refresh::upload_pause_map(self) }
    }
    fn pause_graphics(&mut self, paused: bool) {
        unsafe { display_refresh::pause_graphics(paused, self) }
    }

    fn finish_gameplay_display(&mut self, clear_current_irq: bool) {
        unsafe { display_refresh::finish_gameplay(clear_current_irq, self) }
    }
    fn queue_music_delayed(&mut self, track: u16, delay: u16) {
        crate::game::music::QueueMusic_DelayedY(track, delay, self);
    }
    fn advance_low_health_warning(&mut self) {
        crate::game::session_runtime::player_update_runtime::low_health(self);
    }

    fn player_projectiles_hit_samus(&mut self) {
        unsafe { crate::game::projectile_contact::player_projectiles_hit_samus(self) }
    }
    fn advance_player(&mut self) {
        crate::game::session_runtime::player_update_runtime::advance_frame(self);
    }
    fn run_room_mechanisms(&mut self) -> u8 {
        unsafe { crate::game::room_mechanisms::run(self) }
    }
    fn advance_room(&mut self) {
        unsafe { crate::game::room_state::advance_room(self) }
    }
}
