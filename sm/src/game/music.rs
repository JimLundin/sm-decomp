//! Ordered music commands and room selection; RAM remains the compatibility state.
use crate::game::sm_80::APU_UploadBank;
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
    let previous = checked_word!(game.music_timer);
    checked_word!(game.music_timer = previous.wrapping_sub(1));
    if checked_word!(game.music_timer) & 0x8000 == 0 {
        if previous != 1 {
            return;
        }
        if checked_word!(game.music_entry) & 0x8000 != 0 {
            checked_word!(game.music_data_index = u16::from(checked_word!(game.music_entry) as u8));
            game.g_ram[0x64c] = 0xff;
            APU_UploadBank(bank_address(game), game);
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
