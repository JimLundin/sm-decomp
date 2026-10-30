//! Native callers and retained hardware effects for the shared request owner.
use crate::game_state::Game;
use crate::ram_map::checked_word;

pub(crate) use sm_game::music_requests as requests;

#[forbid(unsafe_code)]
pub fn HasQueuedMusic(game: &mut Game) -> u8 {
    requests::has_queued_music(game)
}

#[forbid(unsafe_code)]
fn enqueue(track: u16, delay: u16, reject_full: bool, game: &mut Game) {
    requests::enqueue(track, delay, reject_full, game).expect("audio enqueue");
}

#[forbid(unsafe_code)]
pub fn queue_music_delayed_8(track: u16, game: &mut Game) {
    enqueue(track, 8, true, game);
}
#[forbid(unsafe_code)]
pub fn QueueMusic_DelayedY(track: u16, delay: u16, game: &mut Game) {
    enqueue(track, delay, false, game);
}

pub unsafe fn HandleMusicQueue(game: &mut Game) {
    unsafe {
        requests::advance_music(game, |effect, meaning, game| {
            match effect {
                requests::MusicEffect::UploadBank => {
                    let selector =
                        sm_data::OriginalMusicBank(checked_word!(game.music_entry) as u8);
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
pub(crate) fn clear_queue(game: &mut Game) {
    requests::clear_queue(game).expect("audio queue clear");
}

#[forbid(unsafe_code)]
fn room_track_changed(game: &Game) -> bool {
    checked_word!(game.game_state) < 40 && checked_word!(game.room_music_track_index) != 0
        // The original packed comparison deliberately lets track bits overlap data bits.
        && (checked_word!(game.room_music_data_index) << 8 | checked_word!(game.room_music_track_index))
            != (checked_word!(game.music_data_index) << 8 | checked_word!(game.music_track_index))
}

#[forbid(unsafe_code)]
pub fn LoadRoomMusic(game: &mut Game) {
    if checked_word!(game.game_state) < 40
        && checked_word!(game.room_music_data_index) != 0
        && checked_word!(game.room_music_data_index) != checked_word!(game.music_data_index)
    {
        queue_music_delayed_8(0, game);
        queue_music_delayed_8(checked_word!(game.room_music_data_index) | 0xff00, game);
    }
}

#[forbid(unsafe_code)]
pub fn UpdateMusicTrackIndex(game: &mut Game) {
    if room_track_changed(game) {
        checked_word!(game.music_track_index = checked_word!(game.room_music_track_index));
    }
}

#[forbid(unsafe_code)]
pub fn LoadNewMusicTrackIfChanged(game: &mut Game) {
    if room_track_changed(game) {
        QueueMusic_DelayedY(checked_word!(game.room_music_track_index), 6, game);
    }
}

#[forbid(unsafe_code)]
pub fn PlayRoomMusicTrackAfterAFrames(delay: u16, game: &mut Game) {
    if checked_word!(game.game_state) < 40 {
        QueueMusic_DelayedY(0, delay, game);
        queue_music_delayed_8(checked_word!(game.music_track_index), game);
    }
}

pub(crate) enum MusicWaitEffect {
    EnableNmi(bool),
    AdvanceQueue,
}

pub(crate) fn wait_for_music(
    game: &mut Game,
    mut effect: impl FnMut(MusicWaitEffect, &mut Game),
) -> u8 {
    crate::verify_probe::hit("coroutine.music");
    let resume = game.ram().read_byte(0x614);
    if resume == 0 {
        effect(MusicWaitEffect::EnableNmi(true), game);
        checked_word!(game.my_counter = 0);
    }
    if resume <= 1 {
        let mut waiting = resume == 1;
        loop {
            if !waiting {
                if checked_word!(game.my_counter) == 20 {
                    effect(MusicWaitEffect::EnableNmi(false), game);
                    break;
                }
                effect(MusicWaitEffect::AdvanceQueue, game);
            }
            let yielded: u8 = crate::game::display_refresh::wait(true, game);
            if yielded != 0 {
                game.ram().write_byte(0x614, 1);
                return yielded;
            }
            checked_word!(game.my_counter += 1);
            waiting = false;
        }
    }
    game.ram().write_byte(0x614, 0);
    0
}
// Byte-backed queue layout from ram_map: three channels, sixteen slots each.
pub(crate) fn queue_sfx(channel: usize, sound: u8, max_queued: u8, game: &mut Game) -> u16 {
    #[cfg(sm_verify_dll)]
    crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
    requests::queue_sfx(channel, sound, max_queued, game).expect("audio sound enqueue")
}

crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx1_Max15 => queue_sfx(0, sound as u8, 15, game);
    QueueSfx1_Max9 => queue_sfx(0, sound as u8, 9, game);
    QueueSfx1_Max3 => queue_sfx(0, sound as u8, 3, game);
    QueueSfx1_Max1 => queue_sfx(0, sound as u8, 1, game);
    queue_sfx1_max6 => queue_sfx(0, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx2_Max15 => queue_sfx(1, sound as u8, 15, game);
    QueueSfx2_Max9 => queue_sfx(1, sound as u8, 9, game);
    queue_sfx2_max3 => queue_sfx(1, sound as u8, 3, game);
    QueueSfx2_Max1 => queue_sfx(1, sound as u8, 1, game);
    queue_sfx2_max6 => queue_sfx(1, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx3_Max15 => queue_sfx(2, sound as u8, 15, game);
    QueueSfx3_Max9 => queue_sfx(2, sound as u8, 9, game);
    queue_sfx3_max3 => queue_sfx(2, sound as u8, 3, game);
    QueueSfx3_Max1 => queue_sfx(2, sound as u8, 1, game);
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

pub fn ResetSoundQueues(game: &mut Game) {
    requests::reset_sound_queues(game).expect("audio sound queue reset");
}
