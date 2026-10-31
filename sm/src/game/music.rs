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
pub(crate) fn clear_queue(game: &mut Game) {
    requests::clear_queue(game).expect("audio queue clear");
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
        checked_word!(game.MY_COUNTER = 0);
    }
    if resume <= 1 {
        let mut waiting = resume == 1;
        loop {
            if !waiting {
                if checked_word!(game.MY_COUNTER) == 20 {
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
            checked_word!(game.MY_COUNTER += 1);
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
