//! Native callers and retained hardware effects for the shared request owner.
pub(crate) use sm_game::music_requests as requests;

// The verification checks queue music on Game's image through
// `oracle_host::OracleHost`; the engine queues it on its lease (below).
pub(crate) use requests::queue_delayed_8 as queue_music_delayed_8;

/// The requests a lease queues on its own Session: the routines the foreign calls ran on the
/// exported game, over the lease's audio host. The Session's control owns the request bytes
/// in Legacy too, so the stores reach its owner, in the same order as before, and RAM takes
/// them when the lease exports, before any APU-side read. Each refreshes the observations on
/// entry and on return.
impl crate::game::session_runtime::Runtime<'_> {
    fn audio_request<R>(&mut self, request: impl FnOnce(&mut Self) -> R) -> R {
        self.owned_player_call(|runtime| {
            #[cfg(sm_verify_dll)]
            runtime.assert_audio_requests_owned();
            request(runtime)
        })
    }
    /// QueueMusic_Delayed8.
    pub(crate) fn queue_music_request(&mut self, track: u16) {
        self.audio_request(|runtime| requests::queue_delayed_8(track, runtime));
    }
    /// QueueMusic_DelayedY: queue `track` after `delay` frames.
    pub(crate) fn queue_music_delayed(&mut self, track: u16, delay: u16) {
        self.audio_request(|runtime| requests::queue_delayed(track, delay, runtime));
    }
    /// QueueSound on `channel`, with the original's result.
    pub(crate) fn queue_sound_request(&mut self, channel: usize, sound: u8, max_queued: u8) -> u16 {
        self.audio_request(|runtime| {
            #[cfg(sm_verify_dll)]
            crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
            requests::queue_sfx(channel, sound, max_queued, runtime).expect("audio sound enqueue")
        })
    }
    /// Reset the three sound effect queues' positions and states.
    pub(crate) fn reset_sound_requests(&mut self) {
        self.audio_request(|runtime| {
            requests::reset_sound_queues(runtime).expect("audio sound queue reset")
        });
    }
    /// Clear the music queue.
    pub(crate) fn clear_music_requests(&mut self) {
        self.audio_request(|runtime| requests::clear_queue(runtime).expect("audio queue clear"));
    }
    /// PlayRoomMusicTrackAfterAFrames.
    pub(crate) fn queue_room_music_after(&mut self, delay: u16) {
        self.audio_request(|runtime| requests::queue_room_track_after(delay, runtime));
    }
    /// LoadRoomMusic on the lease, over the room entry's music fields.
    pub(crate) fn load_room_music(&mut self) {
        self.audio_request(requests::load_room_music);
    }
    /// UpdateMusicTrackIndex on the lease.
    pub(crate) fn update_music_track_index(&mut self) {
        self.audio_request(requests::update_music_track_index);
    }
    /// LoadNewMusicTrackIfChanged on the lease.
    pub(crate) fn load_new_music_track_if_changed(&mut self) {
        self.audio_request(requests::load_new_music_track_if_changed);
    }
    /// Whether any music request is still delayed.
    pub(crate) fn music_queued(&mut self) -> u8 {
        self.audio_request(|runtime| requests::has_queued_music(runtime))
    }
    /// Whether sound queue `channel` is empty.
    pub(crate) fn sound_queue_empty(&mut self, channel: usize) -> bool {
        self.audio_request(|runtime| requests::sound_queue_empty(channel, runtime))
    }
    /// HandleMusicQueue on the lease: the queue steps on the Session's audio
    /// owner, and a bank upload or track write goes out through the cue port, as
    /// `handle_music_queue` sends them from the game.
    pub(crate) fn advance_music_requests(&mut self) {
        #[cfg(sm_verify_dll)]
        self.assert_audio_requests_owned();
        requests::advance_music(self, |effect, meaning, runtime| {
            match effect {
                requests::MusicEffect::UploadBank => {
                    let selector = sm_data::OriginalMusicBank(
                        runtime.word(crate::ram_map::words::MUSIC_ENTRY) as u8,
                    );
                    runtime.emit_cue(
                        crate::native_audio::music_bank_cue(selector, meaning),
                        sm_data::CuePhase::MusicBank,
                    );
                }
                requests::MusicEffect::WriteTrack(track) => runtime.emit_cue(
                    crate::native_audio::music_cue(sm_data::OriginalMusicTrack(track), meaning),
                    sm_data::CuePhase::MusicTrack,
                ),
            }
            Ok(())
        })
        .expect("audio music advance");
    }
    /// The room entry's music wait on the lease, as `wait_for_music` runs it on the
    /// game: the NMI on, up to 20 frames of the music queue, each ended by an NMI
    /// wait that may yield, then the NMI off. The resume byte ($0614) and the
    /// frame counter are the lease's.
    pub(crate) fn wait_for_music(&mut self) -> u8 {
        use crate::ram_map::words::MY_COUNTER;
        crate::verify_probe::hit("coroutine.music");
        let resume = self.byte(0x614);
        if resume == 0 {
            sm_game::display_refresh::enable(true, self);
            self.write_slice_le(MY_COUNTER, 0);
        }
        if resume <= 1 {
            let mut waiting = resume == 1;
            loop {
                if !waiting {
                    if self.word(MY_COUNTER) == 20 {
                        sm_game::display_refresh::enable(false, self);
                        break;
                    }
                    self.advance_music_requests();
                }
                let yielded = sm_game::display_refresh::wait(true, self);
                if yielded != 0 {
                    self.set_byte(0x614, 1);
                    return yielded;
                }
                let counter = self.word(MY_COUNTER).wrapping_add(1);
                self.write_slice_le(MY_COUNTER, counter);
                waiting = false;
            }
        }
        self.set_byte(0x614, 0);
        0
    }
    /// The frame's sound effect advance on the lease: the queues step on the Session's
    /// audio owner, and each sound goes out through the cue port as it is sent.
    pub(crate) fn advance_sound_requests(&mut self) {
        self.emit_sound_requests(sm_data::CuePhase::GameplaySound);
    }
    /// A message box's sound effect advance on the lease, as the frame's.
    pub(crate) fn advance_dialogue_sound_requests(&mut self) {
        self.emit_sound_requests(sm_data::CuePhase::DialogueSound);
    }
    fn emit_sound_requests(&mut self, phase: sm_data::CuePhase) {
        #[cfg(sm_verify_dll)]
        self.assert_audio_requests_owned();
        requests::advance_sound_effects(self, |channel, value, meaning, runtime| {
            runtime.emit_cue(
                crate::native_audio::sound_cue(
                    channel,
                    sm_data::OriginalSoundEffect(value),
                    meaning,
                ),
                phase,
            );
            Ok(())
        })
        .expect("audio sound advance");
    }
}
