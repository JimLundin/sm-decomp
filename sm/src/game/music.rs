//! Ordered music and sound-effect commands; RAM remains the compatibility state.
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rtl::RtlApuWrite;
use crate::snes::registers::APUI00;

#[forbid(unsafe_code)]
pub fn HasQueuedMusic(game: &mut Game) -> u8 {
    u8::from(
        (0..8)
            .rev()
            .any(|slot| checked_word!(game.music_queue_delay[slot]) != 0),
    )
}

#[forbid(unsafe_code)]
fn enqueue(track: u16, delay: u16, reject_full: bool, game: &mut Game) {
    if checked_word!(game.game_state) >= 40 {
        return;
    }
    if reject_full
        && checked_word!(game.music_queue_write_pos).wrapping_add(2) & 14
            == checked_word!(game.music_queue_read_pos)
    {
        return;
    }
    let write = checked_word!(game.music_queue_write_pos);
    let slot = usize::from(write >> 1);
    checked_word!(game.music_queue_track[slot] = track);
    checked_word!(game.music_queue_delay[slot] = delay.max(8));
    checked_word!(game.music_queue_write_pos = write.wrapping_add(2) & 14);
}

#[forbid(unsafe_code)]
pub fn QueueMusic_Delayed8(track: u16, game: &mut Game) {
    enqueue(track, 8, true, game);
}
#[forbid(unsafe_code)]
pub fn QueueMusic_DelayedY(track: u16, delay: u16, game: &mut Game) {
    enqueue(track, delay, false, game);
}

#[forbid(unsafe_code)]
fn retire(game: &mut Game) {
    let read = checked_word!(game.music_queue_read_pos);
    let slot = usize::from(read >> 1);
    checked_word!(game.music_queue_track[slot] = 0);
    checked_word!(game.music_queue_delay[slot] = 0);
    checked_word!(game.music_queue_read_pos = read.wrapping_add(2) & 14);
}

#[forbid(unsafe_code)]
fn bank_address(game: &Game) -> u32 {
    let offset = usize::from(checked_word!(game.music_entry) as u8);
    // Load24 used a native dword, including the adjacent fourth byte.
    u32::from_ne_bytes(core::array::from_fn(|i| {
        game.read_rom_byte(0x8fe7e1, offset + i)
    })) & 0xffffff
}

pub unsafe fn HandleMusicQueue(game: &mut Game) {
    unsafe {
        let previous = checked_word!(game.music_timer);
        checked_word!(game.music_timer = previous.wrapping_sub(1));
        if checked_word!(game.music_timer) & 0x8000 == 0 {
            if previous != 1 {
                return;
            }
            if checked_word!(game.music_entry) & 0x8000 != 0 {
                checked_word!(
                    game.music_data_index = u16::from(checked_word!(game.music_entry) as u8)
                );
                game.g_ram[0x64c] = 0xff;
                crate::native_audio::upload_bank(bank_address(game), game);
                game.g_ram[0x64c] = 0;
                retire(game);
                checked_word!(game.sound_handler_downtime = 8);
                return;
            }
            let track = checked_word!(game.music_entry) as u8 & 0x7f;
            checked_word!(game.music_track_index = checked_word!(game.music_entry) & 0x7f);
            RtlApuWrite(APUI00, checked_word!(game.music_entry) as u8 & 0x7f, game);
            game.g_ram[0x64c] = track;
            checked_word!(game.sound_handler_downtime = 8);
            retire(game);
        }
        if checked_word!(game.music_queue_read_pos) == checked_word!(game.music_queue_write_pos) {
            checked_word!(game.music_timer = 0);
        } else {
            let slot = usize::from(checked_word!(game.music_queue_read_pos) >> 1);
            checked_word!(game.music_entry = checked_word!(game.music_queue_track[slot]));
            checked_word!(game.music_timer = checked_word!(game.music_queue_delay[slot]));
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn reset_delays(game: &mut Game) {
    checked_word!(game.music_timer = 0);
    word_transfers!(game, ascending, 8; le(music_queue_delay, 0) <- value(0));
}

#[forbid(unsafe_code)]
pub(crate) fn clear_queue(game: &mut Game) {
    word_transfers!(game, descending, 8;
        le(music_queue_track, 7) <- value(0), le(music_queue_delay, 7) <- value(0));
    checked_word!(game.music_queue_read_pos = checked_word!(game.music_queue_write_pos));
    checked_word!(game.music_timer = 0);
    checked_word!(game.music_entry = 0);
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
        QueueMusic_Delayed8(0, game);
        QueueMusic_Delayed8(checked_word!(game.room_music_data_index) | 0xff00, game);
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
        QueueMusic_Delayed8(checked_word!(game.music_track_index), game);
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
    let resume = game.g_ram[0x614];
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
                game.g_ram[0x614] = 1;
                return yielded;
            }
            checked_word!(game.my_counter += 1);
            waiting = false;
        }
    }
    game.g_ram[0x614] = 0;
    0
}
// Byte-backed queue layout from ram_map: three channels, sixteen slots each.
pub(crate) fn queue_sfx(channel: usize, sound: u8, max_queued: u8, game: &mut Game) -> u16 {
    #[cfg(sm_verify_dll)]
    crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
    let read_pos = 0x643 + channel;
    let write_pos = 0x646 + channel;
    let limit_pos = 0x653 + channel;
    let buffer = 0x656 + 16 * channel;

    // This write also happens when capacity or the sound gates reject the request.
    game.g_ram[limit_pos] = max_queued;
    let queued = game.g_ram[write_pos].wrapping_sub(game.g_ram[read_pos]) & 15;
    if queued >= max_queued {
        return u16::from_be_bytes([sound, queued]);
    }
    if game.debug_disable_sounds() != 0
        || game.game_state() >= 40
        || game.power_bomb_explosion_status() & 0x8000 != 0
    {
        return u16::from_be_bytes([queued, sound]);
    }

    let write = game.g_ram[write_pos];
    let mut next = write.wrapping_add(1);
    if next >= 16 {
        next = 0;
    }
    // Preserve adjacent-RAM access for an out-of-range current write position.
    let slot = buffer + usize::from(write);
    if next == game.g_ram[read_pos] {
        // Lower sound numbers take priority when the ring is full.
        if sound < game.g_ram[slot] {
            game.g_ram[slot] = sound;
        }
    } else {
        game.g_ram[slot] = sound;
        game.g_ram[write_pos] = next;
        game.g_ram[buffer + usize::from(next)] = 0;
    }
    // Callers that forward the original accumulator need both queue depth and sound.
    u16::from_be_bytes([queued, sound])
}

crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx1_Max15 => queue_sfx(0, sound as u8, 15, game);
    QueueSfx1_Max9 => queue_sfx(0, sound as u8, 9, game);
    QueueSfx1_Max3 => queue_sfx(0, sound as u8, 3, game);
    QueueSfx1_Max1 => queue_sfx(0, sound as u8, 1, game);
    QueueSfx1_Max6 => queue_sfx(0, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx2_Max15 => queue_sfx(1, sound as u8, 15, game);
    QueueSfx2_Max9 => queue_sfx(1, sound as u8, 9, game);
    QueueSfx2_Max3 => queue_sfx(1, sound as u8, 3, game);
    QueueSfx2_Max1 => queue_sfx(1, sound as u8, 1, game);
    QueueSfx2_Max6 => queue_sfx(1, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx3_Max15 => queue_sfx(2, sound as u8, 15, game);
    QueueSfx3_Max9 => queue_sfx(2, sound as u8, 9, game);
    QueueSfx3_Max3 => queue_sfx(2, sound as u8, 3, game);
    QueueSfx3_Max1 => queue_sfx(2, sound as u8, 1, game);
    QueueSfx3_Max6 => queue_sfx(2, sound as u8, 6, game);
}
pub(crate) fn advance_sound_effects(game: &mut Game, mut write: impl FnMut(u8, u8, &mut Game)) {
    if (game.g_ram[0x686] as i8).wrapping_sub(1) >= 0 {
        game.g_ram[0x686] = game.g_ram[0x686].wrapping_sub(1);
        for channel in 0..3 {
            write(channel as u8, 0, game);
            game.g_ram[0x64d + channel] = 0;
        }
        return;
    }
    for channel in 0..3 {
        // Retain the old callback table's bounds failure before any channel effects.
        let state = [0, 1, 2, 3, 4][usize::from(game.g_ram[0x649 + channel])];
        match state {
            0 | 3 => {
                if state == 3 {
                    let _last_sent = game.g_ram[0x64d + channel];
                    game.g_ram[0x649 + channel] = 0;
                }
                if game.g_ram[0x643 + channel] != game.g_ram[0x646 + channel] {
                    let slot = (usize::from(game.g_ram[0x643 + channel]) + channel * 16) as u8;
                    let sound = game.g_ram[0x656 + usize::from(slot)];
                    write(channel as u8, sound, game);
                    game.g_ram[0x64d + channel] = sound;
                    game.g_ram[0x643 + channel] = slot.wrapping_add(1) & 15;
                    game.g_ram[0x649 + channel] = game.g_ram[0x649 + channel].wrapping_add(1);
                }
            }
            1 => {
                let _last_sent = game.g_ram[0x64d + channel];
                game.g_ram[0x649 + channel] = game.g_ram[0x649 + channel].wrapping_add(1);
                game.g_ram[0x650 + channel] = 6;
            }
            2 | 4 => {
                let previous = game.g_ram[0x650 + channel];
                game.g_ram[0x650 + channel] = previous.wrapping_sub(1);
                if previous == 1 {
                    if state == 2 {
                        game.g_ram[0x64d + channel] = 0;
                        game.g_ram[0x649 + channel] = game.g_ram[0x649 + channel].wrapping_add(1);
                    } else {
                        game.g_ram[0x649 + channel] = 0;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

pub fn ResetSoundQueues(game: &mut Game) {
    for offset in 0x643..0x64c {
        game.g_ram[offset] = 0;
    }
}

pub(crate) fn cancel_effects(max_queued: u8, game: &mut Game) {
    for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
        queue_sfx(channel, sound, max_queued, game);
    }
}
