//! Game's implementation of the gameplay frame phases' host.
//!
//! Each method is the original call the phase made. The callees keep their
//! unsafe backend contracts: the frame loop only advances gameplay with the
//! backend installed and exclusively accessed.
use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use sm_game::gameplay_frame::{NativeState, PauseHook, UnpauseHook};
use sm_game::room_entry::api::EntrySignal;

pub(crate) use sm_game::gameplay_frame::{TickInput, tick};

impl sm_game::gameplay_frame::Host for Game {
    fn latch_controller(&mut self, buttons: u16) {
        // The emulated joypad register reads the controller state bit-reversed.
        self.input1_mut().currentState = buttons.reverse_bits();
    }
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
        unsafe { crate::host::fatal_error(c"Incorrect coroutine_state_0".as_ptr()) }
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
            crate::host::report_warning(c"Coroutine State is broken!".as_ptr());
        }
    }
    fn update_hdma(&mut self) {
        crate::game::visual_runtime::update_hdma(self)
    }
    fn advance_random(&mut self) {
        self.next_random();
    }
    fn advance_native_state(&mut self, state: NativeState) -> u8 {
        use sm_game::cinematic_states::{ending_and_credits, intro, title_sequence};
        match state {
            NativeState::OpeningCinematic => title_sequence(self),
            NativeState::CeresGoesBoom => intro(self),
            NativeState::EndingAndCredits => ending_and_credits(self),
        }
    }
    fn resume_room_entry(&mut self, signal: EntrySignal) -> u8 {
        crate::game::room_entry::protocol::resume(&mut Runtime::new(self), signal).returned_byte()
    }
    fn debug_game_over_menu(&mut self) {
        unsafe { crate::cpu_infra::debug_game_over_menu() }
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

    fn initialize_player(&mut self) {
        crate::game::session_runtime::initialize_player(self);
    }

    fn load_title_sequence_graphics(&mut self) {
        sm_game::title_sequence::load_graphics(self);
    }
    fn disable_scanline_irq(&mut self) {
        crate::game::scanline_irq::disable(self);
    }
    fn enable_scanline_irq(&mut self, immediate: bool) {
        unsafe { crate::game::scanline_irq::enable(immediate, self) }
    }

    fn pause_hook(&mut self, hook: PauseHook) {
        match hook {
            PauseHook::Draygon => crate::game::room_state::pause_draygon(self),
            PauseHook::Kraid => sm_game::actors::kraid_pause::pause(self),
        }
    }
    fn unpause_hook(&mut self, hook: UnpauseHook) -> u8 {
        use sm_game::actors::kraid_pause::{unpause_alive, unpause_dead, unpause_sinking};
        match hook {
            UnpauseHook::Draygon => crate::game::room_state::unpause_draygon(self),
            UnpauseHook::KraidDead => unpause_dead(self),
            UnpauseHook::KraidAlive => unpause_alive(self),
            UnpauseHook::KraidSinking => unpause_sinking(self),
            UnpauseHook::MotherBrain => crate::game::sm_a9::body_unpause_hook(self),
        }
    }

    fn queue_music_delayed(&mut self, track: u16, delay: u16) {
        crate::game::music::queue_music_delayed_y(track, delay, self);
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
        crate::game::room_state::advance_room(self)
    }
}
