//! Native callers and retained hardware effects for the shared request owner.
use crate::game_state::Game;
use crate::ram_map::checked_word;

pub(crate) use sm_game::music_requests as requests;

#[forbid(unsafe_code)]
pub fn has_queued_music(game: &mut Game) -> u8 {
    requests::has_queued_music(game)
}

pub(crate) use requests::{
    queue_delayed as queue_music_delayed_y, queue_delayed_8 as queue_music_delayed_8,
};

pub unsafe fn handle_music_queue(game: &mut Game) {
    unsafe {
        requests::advance_music(game, |effect, meaning, game| {
            match effect {
                requests::MusicEffect::UploadBank => {
                    let selector =
                        sm_data::OriginalMusicBank(checked_word!(game.MUSIC_ENTRY) as u8);
                    crate::native_audio::emit(
                        crate::native_audio::music_bank_cue(selector, meaning),
                        sm_data::CuePhase::MusicBank,
                        game,
                    );
                }
                requests::MusicEffect::WriteTrack(track) => crate::native_audio::emit(
                    crate::native_audio::music_cue(sm_data::OriginalMusicTrack(track), meaning),
                    sm_data::CuePhase::MusicTrack,
                    game,
                ),
            }
            Ok(())
        })
        .expect("audio music advance");
    }
}

#[forbid(unsafe_code)]
pub(crate) fn reset_delays(game: &mut Game) {
    requests::reset_delays(game).expect("audio delay reset");
}

#[forbid(unsafe_code)]
fn room_track_changed(game: &Game) -> bool {
    checked_word!(game.GAME_STATE) < 40 && checked_word!(game.ROOM_MUSIC_TRACK_INDEX) != 0
        // The original packed comparison deliberately lets track bits overlap data bits.
        && (checked_word!(game.ROOM_MUSIC_DATA_INDEX) << 8 | checked_word!(game.ROOM_MUSIC_TRACK_INDEX))
            != (checked_word!(game.MUSIC_DATA_INDEX) << 8 | checked_word!(game.MUSIC_TRACK_INDEX))
}

#[forbid(unsafe_code)]
pub fn load_room_music(game: &mut Game) {
    if checked_word!(game.GAME_STATE) < 40
        && checked_word!(game.ROOM_MUSIC_DATA_INDEX) != 0
        && checked_word!(game.ROOM_MUSIC_DATA_INDEX) != checked_word!(game.MUSIC_DATA_INDEX)
    {
        queue_music_delayed_8(0, game);
        queue_music_delayed_8(checked_word!(game.ROOM_MUSIC_DATA_INDEX) | 0xff00, game);
    }
}

#[forbid(unsafe_code)]
pub fn update_music_track_index(game: &mut Game) {
    if room_track_changed(game) {
        checked_word!(game.MUSIC_TRACK_INDEX = checked_word!(game.ROOM_MUSIC_TRACK_INDEX));
    }
}

#[forbid(unsafe_code)]
pub fn load_new_music_track_if_changed(game: &mut Game) {
    if room_track_changed(game) {
        queue_music_delayed_y(checked_word!(game.ROOM_MUSIC_TRACK_INDEX), 6, game);
    }
}

#[forbid(unsafe_code)]
pub fn play_room_music_track_after_aframes(delay: u16, game: &mut Game) {
    if checked_word!(game.GAME_STATE) < 40 {
        queue_music_delayed_y(0, delay, game);
        queue_music_delayed_8(checked_word!(game.MUSIC_TRACK_INDEX), game);
    }
}

// Byte-backed queue layout from ram_map: three channels, sixteen slots each.
pub(crate) fn queue_sfx(channel: usize, sound: u8, max_queued: u8, game: &mut Game) -> u16 {
    #[cfg(sm_verify_dll)]
    crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
    requests::queue_sfx(channel, sound, max_queued, game).expect("audio sound enqueue")
}

crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    queue_sfx1_max15 => queue_sfx(0, sound as u8, 15, game);
    queue_sfx1_max9 => queue_sfx(0, sound as u8, 9, game);
    queue_sfx1_max3 => queue_sfx(0, sound as u8, 3, game);
    queue_sfx1_max1 => queue_sfx(0, sound as u8, 1, game);
    queue_sfx1_max6 => queue_sfx(0, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    queue_sfx2_max15 => queue_sfx(1, sound as u8, 15, game);
    queue_sfx2_max9 => queue_sfx(1, sound as u8, 9, game);
    queue_sfx2_max3 => queue_sfx(1, sound as u8, 3, game);
    queue_sfx2_max1 => queue_sfx(1, sound as u8, 1, game);
    queue_sfx2_max6 => queue_sfx(1, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    queue_sfx3_max15 => queue_sfx(2, sound as u8, 15, game);
    queue_sfx3_max9 => queue_sfx(2, sound as u8, 9, game);
    queue_sfx3_max3 => queue_sfx(2, sound as u8, 3, game);
    queue_sfx3_max1 => queue_sfx(2, sound as u8, 1, game);
    queue_sfx3_max6 => queue_sfx(2, sound as u8, 6, game);
}
pub(crate) fn advance_sound_effects(
    game: &mut Game,
    mut write: impl FnMut(
        u8,
        u8,
        Option<&crate::game_state::persistent_session::QueuedSoundMeaning>,
        &mut Game,
    ),
) {
    requests::advance_sound_effects(game, |channel, value, meaning, game| {
        write(channel, value, meaning, game);
        Ok(())
    })
    .expect("audio sound advance");
}

pub fn clear_sound_queues(game: &mut Game) {
    requests::reset_sound_queues(game).expect("audio sound queue reset");
}

/// The requests a lease queues on its own Session: the routines the foreign calls ran on the
/// exported game, over the lease's audio host. The Session's control owns the request bytes
/// in Legacy too, so the stores reach its owner, in the same order as before, and RAM takes
/// them when the lease exports, before any APU-side read. Each refreshes the observations on
/// entry and on return, as the foreign call left them.
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
        #[cfg(sm_verify_dll)]
        self.assert_audio_requests_owned();
        requests::advance_sound_effects(self, |channel, value, meaning, runtime| {
            runtime.emit_cue(
                crate::native_audio::sound_cue(
                    channel,
                    sm_data::OriginalSoundEffect(value),
                    meaning,
                ),
                sm_data::CuePhase::GameplaySound,
            );
            Ok(())
        })
        .expect("audio sound advance");
    }
}
