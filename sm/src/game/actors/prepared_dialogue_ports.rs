//! The actor message caller uses the already leased persistent dialogue owner.
#![forbid(unsafe_code)]
use super::prepared_execution::PreparedActorPorts;
use sm_data::message::{AssetId, Fault, Revision};
use sm_game::{
    message_composition::prepared::{Effect, Outcome},
    persistent_session::PersistentSession,
    scene::state::{DisplayError, DisplayPhase},
    session::Session,
    session_owners::{AudioHost, AudioRequests, AudioStamp, Control},
};

/// The Game integrator seals this receipt against the current returned owner and
/// revokes it at every selected-image/reset/load boundary before device display.
pub(crate) struct PreparedMessageFrame {
    pub(crate) manifest: [u8; 32],
    pub(crate) input_record: u64,
    pub(crate) media: AssetId,
    pub(crate) revision: Revision,
    pub(crate) compatibility_epoch: u64,
    pub(crate) frame: sm_runtime::message::MessageFrame,
}

impl PreparedActorPorts<'_> {
    pub(super) fn resume_actor_dialogue(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut sm_game::actors::owned_execution::ActorAllocations,
        presentation: &sm_game::scene::actor_presentation::Binding,
        samus_content: &sm_game::scene::prepared_samus::PreparedSamusContent,
        request: u16,
    ) -> u8 {
        // A changing live request does not replace the captured suspended message.
        owner.gameplay.messages.queued = request;
        let result = owner.resume_dialogue(
            &mut MessageEffects {
                ports: self,
                allocations,
                presentation,
                samus_content,
            },
            sm_runtime::message::render,
        );
        let output = match result {
            Ok(output) => output,
            Err(fault) => {
                owner.poison();
                std::panic::panic_any(fault);
            }
        };
        let content = owner
            .required
            .dialogue
            .as_ref()
            .expect("admitted dialogue content");
        let execution = owner
            .gameplay
            .prepared_dialogue
            .as_ref()
            .expect("current dialogue execution");
        let display = if execution.visible {
            &content.start.active
        } else if execution.restored {
            &content.start.restored
        } else {
            &content.start.saved
        };
        // The successful rendered result reaches the actual device consumer.
        if let Err(fault) =
            self.game
                .submit_prepared_message_frame(owner, display.media_id, output.frame)
        {
            owner.poison();
            std::panic::panic_any(fault);
        }
        match output.outcome {
            Outcome::Complete => 0,
            Outcome::Suspended => 1,
        }
    }
}

struct MessageEffects<'a, 'g> {
    ports: &'a mut PreparedActorPorts<'g>,
    allocations: &'a mut sm_game::actors::owned_execution::ActorAllocations,
    presentation: &'a sm_game::scene::actor_presentation::Binding,
    samus_content: &'a sm_game::scene::prepared_samus::PreparedSamusContent,
}
impl sm_game::prepared_dialogue::Effects for MessageEffects<'_, '_> {
    fn input_owner(
        &mut self,
        owner: &mut PersistentSession,
        _sample: sm_game::message_composition::prepared::InputSample,
    ) -> Result<sm_game::message_composition::prepared::InputSample, Fault> {
        let mut host = sm_game::actors::owned_execution::PreparedActorState::reborrow(
            owner,
            self.allocations,
            self.ports,
            self.presentation,
            self.samus_content,
        );
        host.sample_dialogue_input()
    }
    fn wait(&mut self, state: &mut Session) -> Result<sm_game::dialogue::Outcome, Fault> {
        sm_game::prepared_dialogue::Effects::wait(self.ports, state)
    }
    fn apply_owner(&mut self, effect: Effect, owner: &mut PersistentSession) -> Result<(), Fault> {
        if effect == Effect::UpdateHdma {
            let mut host = sm_game::actors::owned_execution::PreparedActorState::reborrow(
                owner,
                self.allocations,
                self.ports,
                self.presentation,
                self.samus_content,
            );
            return host.update_hdma(false).map_err(|_| Fault::Definition);
        }
        self.apply(effect, &mut owner.gameplay)
    }
    fn apply(&mut self, effect: Effect, state: &mut Session) -> Result<(), Fault> {
        sm_game::prepared_dialogue::Effects::apply(self.ports, effect, state)
    }
}

/// Algorithms operate on the actual typed numeric and semantic audio owner;
/// only native Cue delivery and exceptional aliases cross the physical seam.
struct DialogueAudio<'a, 'g> {
    state: &'a mut Session,
    ports: &'a mut PreparedActorPorts<'g>,
}
impl AudioHost for DialogueAudio<'_, '_> {
    fn audio_byte(&self, at: usize) -> u8 {
        sm_game::session_encoding::projected_byte(self.state, at)
            .unwrap_or_else(|| self.ports.game.ram().read_byte(at))
    }
    fn audio_store_byte(&mut self, at: usize, byte: u8) {
        if !sm_game::session_encoding::write_projected_byte(self.state, at, byte) {
            self.ports.game.ram().write_byte(at, byte);
        }
    }
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut AudioRequests,
            &mut sm_game::scene::start::InitialAudioIntent,
            AudioStamp,
        ) -> Result<R, DisplayError>,
    ) -> Result<Option<R>, DisplayError> {
        let producing = self.state.presentation.owner()?.state.phase == DisplayPhase::Producing;
        let Control::Owned(control) = &mut self.state.control else {
            return Err(DisplayError::Phase);
        };
        let result = control.audio_mutation(producing, operation);
        if result.is_err() {
            self.state.presentation.owner_mut()?.state.phase = DisplayPhase::Failed;
        }
        result.map(Some)
    }
}
impl sm_game::player_transitions::message_sound::MessageState for DialogueAudio<'_, '_> {
    fn message_state(&self) -> &Session {
        self.state
    }
}
impl DialogueAudio<'_, '_> {
    fn sound(&mut self, bank: u8, maximum: u8, sound: u16) {
        let value = u8::try_from(sound).expect("message sound byte");
        sm_game::music_requests::queue_sfx(usize::from(bank - 1), value, maximum, self)
            .expect("message audio queue");
    }
    fn advance_music(&mut self) -> Result<(), DisplayError> {
        sm_game::music_requests::advance_music(self, |effect, meaning, audio| {
            let selector = match effect {
                sm_game::music_requests::MusicEffect::UploadBank => {
                    Some(sm_data::OriginalMusicBank(
                        audio.state.control.requests()?.music.entry.to_le_bytes()[0],
                    ))
                }
                sm_game::music_requests::MusicEffect::WriteTrack(_) => None,
            };
            audio
                .ports
                .game
                .prepared_actor_physical_effect(audio.state, |game| {
                    crate::game::session_runtime::prepared_music_effect(
                        selector, effect, meaning, game,
                    )
                });
            Ok(())
        })
    }
    fn advance_sounds(&mut self) -> Result<(), DisplayError> {
        sm_game::music_requests::advance_sound_effects(self, |channel, value, meaning, audio| {
            audio
                .ports
                .game
                .prepared_actor_physical_effect(audio.state, |game| {
                    crate::game::session_runtime::prepared_dialogue_sound_effect(
                        channel, value, meaning, game,
                    )
                });
            Ok(())
        })
    }
}
impl sm_game::prepared_dialogue::Effects for PreparedActorPorts<'_> {
    fn wait(&mut self, state: &mut Session) -> Result<sm_game::dialogue::Outcome, Fault> {
        let display = state
            .presentation
            .owner_mut()
            .map_err(|_| Fault::MissingDisplay)?;
        let result = display.state.nmi.wait(false);
        let pending = display.state.nmi.acknowledgment != 0;
        state
            .prepared_dialogue
            .as_mut()
            .ok_or(Fault::MissingDisplay)?
            .wait_pending = pending;
        Ok(std::num::NonZeroU8::new(result).map_or(
            sm_game::dialogue::Outcome::Complete,
            sm_game::dialogue::Outcome::Suspended,
        ))
    }
    fn apply(&mut self, effect: Effect, state: &mut Session) -> Result<(), Fault> {
        if effect == Effect::UpdateHdma {
            // This effect requires the full current owner, supplied by MessageEffects.
            return Err(Fault::Definition);
        }
        if effect == Effect::SelectPause {
            state.game_loop.mode = 12;
            return Ok(());
        }
        let mut audio = DialogueAudio { state, ports: self };
        let result = match effect {
            Effect::CancelSounds => {
                for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
                    sm_game::music_requests::queue_sfx(channel, sound, 6, &mut audio)
                        .map_err(|_| Fault::Definition)?;
                }
                Ok(())
            }
            Effect::AdvanceAudio => audio.advance_music().and_then(|()| audio.advance_sounds()),
            Effect::AdvanceSoundEffects => audio.advance_sounds(),
            Effect::SaveSound => {
                sm_game::music_requests::queue_sfx(0, 0x2e, 6, &mut audio).map(|_| ())
            }
            Effect::SelectionSound => {
                sm_game::music_requests::queue_sfx(0, 0x37, 6, &mut audio).map(|_| ())
            }
            Effect::ResumeMovementSound => {
                sm_game::player_transitions::message_sound::resume_movement(
                    &mut audio,
                    |audio, channel, maximum, sound| audio.sound(channel, maximum, sound),
                );
                sm_game::player_transitions::message_sound::restore_control(
                    &mut audio,
                    |audio, channel, maximum, sound| audio.sound(channel, maximum, sound),
                );
                Ok(())
            }
            Effect::UpdateHdma | Effect::SelectPause => unreachable!(),
        };
        result.map_err(|_| Fault::Definition)
    }
}
